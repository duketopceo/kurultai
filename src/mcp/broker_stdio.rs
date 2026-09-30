//! Broker stdio client (`kurultai mcp --broker`).
//!
//! Speaks the same newline-delimited JSON-RPC 2.0 as [`super::server`], but
//! instead of serving a local `BrainService` it forwards each frame to the
//! device broker's `POST /mcp` with the caller's `sess_*` key. The broker
//! attaches the upstream seat credential and `X-Kurultai-*` identity stamps —
//! this process never holds upstream credentials (plan 2026-09-28-001 D4/U3).

use super::server::{read_stdin_frame, rpc_error, write_response, StdinFrame};
use crate::error::{KurultaiError, Result};
use serde_json::{json, Value};
use tokio::io::BufReader;

const MAX_STDIN_LINE: usize = 1_048_576;

/// Session key resolution: explicit flag wins, then env. Agent configs written
/// by `init --agent <x> --broker` export `KURULTAI_SESSION_KEY`.
pub fn resolve_session_key(flag: Option<&str>) -> Result<String> {
    if let Some(k) = flag.map(str::trim).filter(|k| !k.is_empty()) {
        return Ok(k.to_string());
    }
    std::env::var("KURULTAI_SESSION_KEY")
        .ok()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .ok_or_else(|| {
            KurultaiError::config(
                "no broker session key — pass `--session-key` or set KURULTAI_SESSION_KEY \
                 (mint one via `POST /board` on the device broker)",
            )
        })
}

/// Run the stdio ↔ broker relay until stdin closes.
pub async fn run_broker_stdio(broker_url: &str, session_key: &str) -> Result<()> {
    let broker_url = broker_url.trim_end_matches('/');
    let client = reqwest::Client::new();
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    loop {
        let line = match read_stdin_frame(&mut reader, MAX_STDIN_LINE).await? {
            StdinFrame::Eof => return Ok(()),
            StdinFrame::TooLarge => {
                write_response(
                    &mut stdout,
                    &rpc_error(Value::Null, -32600, "frame too large"),
                )
                .await?;
                continue;
            }
            StdinFrame::Line(l) => l,
        };
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => {
                write_response(&mut stdout, &rpc_error(Value::Null, -32700, "parse error")).await?;
                continue;
            }
        };
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let is_notification = msg.get("id").is_none();

        match relay_once(&client, broker_url, session_key, &msg).await {
            Ok(Some(response)) => write_response(&mut stdout, &response).await?,
            Ok(None) => {
                // Upstream accepted without a body (notification path).
                if !is_notification {
                    write_response(
                        &mut stdout,
                        &rpc_error(id, -32000, "upstream returned no response"),
                    )
                    .await?;
                }
            }
            Err(e) => {
                if !is_notification {
                    write_response(&mut stdout, &rpc_error(id, -32000, e.to_string())).await?;
                }
            }
        }
    }
}

async fn relay_once(
    client: &reqwest::Client,
    broker_url: &str,
    session_key: &str,
    msg: &Value,
) -> Result<Option<Value>> {
    let res = client
        .post(format!("{broker_url}/mcp"))
        .header("x-kurultai-session", session_key)
        .json(msg)
        .send()
        .await
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("broker unreachable: {e}")))?;
    let status = res.status();
    if status == reqwest::StatusCode::ACCEPTED || status == reqwest::StatusCode::NO_CONTENT {
        return Ok(None);
    }
    let body: Value = res.json().await.unwrap_or_else(|_| {
        json!({ "jsonrpc": "2.0", "error": { "code": -32000, "message": "broker returned non-JSON" } })
    });
    if !status.is_success() {
        let msg = body
            .get("error")
            .and_then(|e| e.get("message").or(Some(e)))
            .and_then(Value::as_str)
            .unwrap_or("broker error");
        return Err(KurultaiError::Other(anyhow::anyhow!(
            "broker {status}: {msg}"
        )));
    }
    Ok(if body.is_null() { None } else { Some(body) })
}
