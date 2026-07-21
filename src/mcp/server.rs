//! Minimal MCP stdio JSON-RPC server (Phase 1 #11 slice).
//!
//! Speaks newline-delimited JSON-RPC 2.0 over stdin/stdout.
//! Tools: `search`, `cite`, `remember` (ask is available but thin).

use crate::error::{KurultaiError, Result};
use crate::mcp::brain::BrainService;
use crate::mcp::interface::{AgentRead, AgentWrite};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "kurultai";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the MCP server until stdin closes.
pub async fn run_stdio(brain: BrainService) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp stdin: {e}")))?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "mcp invalid json");
                continue;
            }
        };

        // Notifications have no id — ignore after handling initialized.
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

        if id.is_none() {
            tracing::debug!(method, "mcp notification");
            continue;
        }

        let result = match method {
            "initialize" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION,
                }
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tool_defs() })),
            "tools/call" => {
                let params = msg.get("params").cloned().unwrap_or(json!({}));
                call_tool(&brain, params).await
            }
            _ => Err(KurultaiError::Other(anyhow::anyhow!(
                "method not found: {method}"
            ))),
        };

        let response = match result {
            Ok(value) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": value,
            }),
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32000,
                    "message": e.to_string(),
                }
            }),
        };

        let out = serde_json::to_string(&response)
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp encode: {e}")))?;
        stdout
            .write_all(out.as_bytes())
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp stdout: {e}")))?;
        stdout
            .write_all(b"\n")
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp stdout: {e}")))?;
        stdout
            .flush()
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp flush: {e}")))?;
    }

    Ok(())
}

fn tool_defs() -> Vec<Value> {
    vec![
        json!({
            "name": "search",
            "description": "Search the Kurultai knowledge brain. Returns token-capped excerpts, not full documents.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 10 }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "cite",
            "description": "Fetch one citation-sized excerpt by source + source_id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": { "type": "string" },
                    "source_id": { "type": "string" }
                },
                "required": ["source", "source_id"]
            }
        }),
        json!({
            "name": "remember",
            "description": "Store a distilled fact (title + summary + tags). Do not dump raw chat.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "summary": { "type": "string" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "default": []
                    }
                },
                "required": ["title", "summary"]
            }
        }),
        json!({
            "name": "ask",
            "description": "Thin retrieval answer with citations (full synthesis is Phase 3).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": { "type": "string" }
                },
                "required": ["question"]
            }
        }),
    ]
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

async fn call_tool(brain: &BrainService, params: Value) -> Result<Value> {
    let call: ToolCallParams = serde_json::from_value(params)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad tools/call params: {e}")))?;

    let text = match call.name.as_str() {
        "search" => {
            let query = call
                .arguments
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let limit = call
                .arguments
                .get("limit")
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as usize;
            let views = brain.search_views(query, limit).await?;
            serde_json::to_string_pretty(&views)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{e}")))?
        }
        "cite" => {
            let source = call
                .arguments
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let source_id = call
                .arguments
                .get("source_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match brain.cite(source, source_id).await? {
                Some(c) => serde_json::to_string_pretty(&c)
                    .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{e}")))?,
                None => format!("No atom for {source}/{source_id}"),
            }
        }
        "remember" => {
            let title = call
                .arguments
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let summary = call
                .arguments
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tags: Vec<String> = call
                .arguments
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let id = brain.remember(title, summary, &tags, &[]).await?;
            format!("remembered atom id={id}")
        }
        "ask" => {
            let question = call
                .arguments
                .get("question")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let answer = brain.ask(question).await?;
            serde_json::to_string_pretty(&answer)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{e}")))?
        }
        other => {
            return Err(KurultaiError::Other(anyhow::anyhow!(
                "unknown tool: {other}"
            )));
        }
    };

    Ok(json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    }))
}
