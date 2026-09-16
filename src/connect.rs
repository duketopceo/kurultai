//! `kurultai connect <instance-url>` — RFC 8628 device-authorization onboarding.
//!
//! One browser login replaces manual `agent add` key minting: the CLI requests a
//! device code, the human approves at `/connect`, the daemon mints a seat key
//! for `(codename, instance_id)`, and the CLI stores it via `omaseal` (or a 0600
//! key file under `~/.config/kurultai/agent-keys/`) and wires MCP — the same
//! `wire_agent` path `init` uses. Keys never land in `config.toml` or stdout.

use crate::error::KurultaiError;
use crate::mcp::AgentTarget;
use crate::Result;
use serde::Deserialize;
use std::io::{IsTerminal, Write};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ConnectOptions {
    /// Instance base URL, e.g. `https://knowledge.shippedit.dev` or `http://127.0.0.1:8421`.
    pub url: String,
    /// Codename for the agent row (product family). Server default: "anonymous".
    pub codename: Option<String>,
    /// Seat id distinguishing concurrent machines under one codename.
    pub instance_id: Option<String>,
    /// MCP clients to wire after the key lands (`wire_agent`, same as `init`).
    pub agent: AgentTarget,
    /// Never open a browser.
    pub no_open: bool,
    /// Lane (`dev`/`staging`/`prod`) namespaced into the credential name.
    pub lane: String,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    code: String,
    user_code: String,
    verify_url: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenResponse {
    agent_key: String,
    codename: String,
    instance_id: String,
}

/// Default seat id: `KURULTAI_INSTANCE_ID` → `$HOSTNAME` → `hostname` → "default".
fn default_instance_id() -> String {
    if let Ok(v) = std::env::var("KURULTAI_INSTANCE_ID") {
        let v = v.trim();
        if !v.is_empty() {
            return v.to_string();
        }
    }
    if let Ok(v) = std::env::var("HOSTNAME") {
        let v = v.trim();
        if !v.is_empty() {
            return v.to_string();
        }
    }
    if let Ok(out) = Command::new("hostname").output() {
        let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !v.is_empty() {
            return v;
        }
    }
    "default".to_string()
}

fn binary_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}

/// Store the minted agent key: `omaseal` when on PATH, else a 0600 key file.
/// Returns a one-line, secret-free description of where it landed.
fn store_agent_key(name: &str, key: &str) -> Result<String> {
    if binary_available("omaseal") {
        let mut child = Command::new("omaseal")
            .args(["set", "kurultai", name])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("omaseal spawn: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(key.as_bytes());
        }
        let status = child
            .wait()
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("omaseal wait: {e}")))?;
        if status.success() {
            return Ok(format!("omaseal `kurultai/{name}`"));
        }
        eprintln!("warning: omaseal set failed — falling back to key file");
    }
    let path = crate::security::write_agent_key_file(name, key)?;
    Ok(format!("{} (0600)", path.display()))
}

pub async fn run(opts: ConnectOptions) -> Result<()> {
    let client = reqwest::Client::new();
    let base = opts.url.trim_end_matches('/').to_string();
    let instance_id = opts.instance_id.clone().unwrap_or_else(default_instance_id);

    let code_resp = client
        .post(format!("{base}/api/device/code"))
        .json(&serde_json::json!({
            "client_id": "kurultai-cli",
            "codename": opts.codename.as_deref().unwrap_or(""),
            "instance_id": &instance_id,
        }))
        .send()
        .await
        .map_err(|e| {
            KurultaiError::Other(anyhow::anyhow!("device code request to {base} failed: {e}"))
        })?;
    if !code_resp.status().is_success() {
        let status = code_resp.status();
        let text = code_resp.text().await.unwrap_or_default();
        return Err(KurultaiError::Other(anyhow::anyhow!(
            "device code request failed: {status} {text}"
        )));
    }
    let code_resp = code_resp.json::<DeviceCodeResponse>().await.map_err(|e| {
        KurultaiError::Other(anyhow::anyhow!("device code response parse failed: {e}"))
    })?;

    println!("Code:     {}", code_resp.user_code);
    println!("Approve:  {}", code_resp.verify_url);

    // Browser only on an interactive TTY — SSH/piped sessions just poll.
    if !opts.no_open && std::io::stdout().is_terminal() {
        crate::webui::open_browser(&code_resp.verify_url);
    }

    let interval = std::cmp::max(code_resp.interval, 2);
    let deadline = Instant::now() + Duration::from_secs(code_resp.expires_in);

    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(interval)).await;

        let res = client
            .post(format!("{base}/api/device/token"))
            .json(&serde_json::json!({ "code": &code_resp.code }))
            .send()
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("token poll failed: {e}")))?;

        match res.status().as_u16() {
            200 => {
                let token = res.json::<DeviceTokenResponse>().await.map_err(|e| {
                    KurultaiError::Other(anyhow::anyhow!("token response parse failed: {e}"))
                })?;
                let name = format!("{}-{}-agent-token", opts.lane, token.codename);
                let where_stored = store_agent_key(&name, &token.agent_key)?;
                println!();
                println!(
                    "Connected as '{}' (seat '{}').",
                    token.codename, token.instance_id
                );
                println!("Agent key stored in {where_stored}.");
                let wired = crate::mcp::wire_agent(opts.agent)?;
                for p in &wired {
                    println!("MCP wired: {}", p.display());
                }
                println!("Done. If this key is ever revoked, run `kurultai connect` again.");
                return Ok(());
            }
            // 428 PRECONDITION_REQUIRED — still waiting on the human.
            428 => {
                eprint!(".");
                continue;
            }
            410 => {
                return Err(KurultaiError::Other(anyhow::anyhow!(
                    "code expired — run `kurultai connect` again"
                )));
            }
            403 => {
                return Err(KurultaiError::Other(anyhow::anyhow!(
                    "connection request was denied"
                )));
            }
            status => {
                let text = res.text().await.unwrap_or_default();
                return Err(KurultaiError::Other(anyhow::anyhow!(
                    "token poll returned {status}: {text}"
                )));
            }
        }
    }

    Err(KurultaiError::Other(anyhow::anyhow!(
        "code expired before approval — run `kurultai connect` again"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_id_prefers_env() {
        unsafe {
            std::env::set_var("KURULTAI_INSTANCE_ID", "ci-seat-7");
        }
        assert_eq!(default_instance_id(), "ci-seat-7");
        unsafe {
            std::env::remove_var("KURULTAI_INSTANCE_ID");
        }
    }

    #[test]
    fn instance_id_never_empty() {
        assert!(!default_instance_id().is_empty());
    }

    #[test]
    fn key_name_is_lane_scoped() {
        // Mirrors the store_agent_key naming used on success.
        let name = format!("{}-{}-agent-token", "dev", "cursor");
        assert_eq!(name, "dev-cursor-agent-token");
    }
}
