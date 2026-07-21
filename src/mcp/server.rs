//! Minimal MCP stdio JSON-RPC server (Phase 1 #11 slice).
//!
//! Speaks newline-delimited JSON-RPC 2.0 over stdin/stdout.
//! Tools: `search`, `cite`, `remember` (ask is available but thin).

use crate::error::{KurultaiError, Result};
use crate::mcp::brain::BrainService;
use crate::mcp::interface::{AgentRead, AgentWrite};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "kurultai";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Reject pathological single-line payloads (agents should not dump megabytes).
const MAX_STDIN_LINE: usize = 1_048_576;

const TOOL_SEARCH: &str = "search";
const TOOL_CITE: &str = "cite";
const TOOL_REMEMBER: &str = "remember";
const TOOL_ASK: &str = "ask";

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
        if line.len() > MAX_STDIN_LINE {
            tracing::warn!(len = line.len(), "mcp stdin line exceeds cap; dropping");
            continue;
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

        let Some(response) = handle_message(&brain, msg).await? else {
            continue;
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

/// Handle one JSON-RPC message. Returns `None` for notifications (no response).
pub async fn handle_message(brain: &BrainService, msg: Value) -> Result<Option<Value>> {
    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

    if id.is_none() {
        tracing::debug!(method, "mcp notification");
        return Ok(None);
    }

    let mut error_code = -32000;
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
            call_tool(brain, params).await
        }
        _ => {
            error_code = -32601;
            Err(KurultaiError::Other(anyhow::anyhow!(
                "method not found: {method}"
            )))
        }
    };

    Ok(Some(match result {
        Ok(value) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": value,
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": error_code,
                "message": e.to_string(),
            }
        }),
    }))
}

fn tool_defs() -> &'static [Value] {
    static DEFS: OnceLock<Vec<Value>> = OnceLock::new();
    DEFS.get_or_init(|| {
        vec![
            json!({
                "name": TOOL_SEARCH,
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
                "name": TOOL_CITE,
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
                "name": TOOL_REMEMBER,
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
                "name": TOOL_ASK,
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
    })
    .as_slice()
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Deserialize)]
struct CiteArgs {
    source: String,
    source_id: String,
}

#[derive(Debug, Deserialize)]
struct RememberArgs {
    title: String,
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AskArgs {
    question: String,
}

async fn call_tool(brain: &BrainService, params: Value) -> Result<Value> {
    let call: ToolCallParams = serde_json::from_value(params)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad tools/call params: {e}")))?;

    let text = match call.name.as_str() {
        TOOL_SEARCH => {
            let args: SearchArgs = serde_json::from_value(call.arguments)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad search args: {e}")))?;
            let views = brain.search_views(&args.query, args.limit).await?;
            serde_json::to_string(&views)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{e}")))?
        }
        TOOL_CITE => {
            let args: CiteArgs = serde_json::from_value(call.arguments)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad cite args: {e}")))?;
            match brain.cite(&args.source, &args.source_id).await? {
                Some(c) => serde_json::to_string(&c)
                    .map_err(|e| KurultaiError::Other(anyhow::anyhow!("{e}")))?,
                None => format!("No atom for {}/{}", args.source, args.source_id),
            }
        }
        TOOL_REMEMBER => {
            let args: RememberArgs = serde_json::from_value(call.arguments)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad remember args: {e}")))?;
            let id = brain
                .remember(&args.title, &args.summary, &args.tags, &[])
                .await?;
            format!("remembered atom id={id}")
        }
        TOOL_ASK => {
            let args: AskArgs = serde_json::from_value(call.arguments)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad ask args: {e}")))?;
            let answer = brain.ask(&args.question).await?;
            serde_json::to_string(&answer)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::{Embedder, NullEmbedder};
    use crate::pipeline::IndexPipeline;
    use crate::store::{SqliteVecStore, Store};
    use crate::types::{SourceConfig, SourceKind};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    async fn brain_with_fixture() -> BrainService {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-mcp-rpc-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline =
            IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));
        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap();
        pipeline
            .index_connector("notes", &connector, true)
            .await
            .unwrap();
        BrainService::new(store, embedder)
    }

    #[test]
    fn tool_defs_expose_phase1_tools() {
        let names: Vec<&str> = tool_defs()
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"search"));
        assert!(names.contains(&"cite"));
        assert!(names.contains(&"remember"));
        assert!(names.contains(&"ask"));
    }

    #[tokio::test]
    async fn tools_list_and_search_roundtrip() {
        let brain = brain_with_fixture().await;

        let list = handle_message(
            &brain,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
        )
        .await
        .unwrap()
        .expect("response");
        assert!(list.get("result").is_some());
        assert!(list["result"]["tools"].as_array().unwrap().len() >= 3);

        let search = handle_message(
            &brain,
            json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "search",
                    "arguments": { "query": "KNOWN_PHRASE_KURULTAI_42", "limit": 3 }
                }
            }),
        )
        .await
        .unwrap()
        .expect("response");
        let text = search["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("KNOWN_PHRASE_KURULTAI_42") || text.contains("notes"));
        assert!(!text.contains(&"x".repeat(500)));
    }

    #[tokio::test]
    async fn unknown_method_returns_jsonrpc_code() {
        let brain = brain_with_fixture().await;
        let resp = handle_message(&brain, json!({"jsonrpc":"2.0","id":9,"method":"nope"}))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resp["error"]["code"], -32601);
    }
}
