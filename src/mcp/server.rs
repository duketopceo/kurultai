//! Minimal MCP stdio JSON-RPC server (Phase 1 #11 + Phase 3 #7).
//!
//! Speaks newline-delimited JSON-RPC 2.0 over stdin/stdout.
//! Tools: `search`, `cite`, `remember`, `ask`, `who_knows`.

use crate::error::{KurultaiError, Result};
use crate::mcp::brain::BrainService;
use crate::mcp::interface::{AgentRead, AgentWrite};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "kurultai";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Reject pathological single-line payloads (agents should not dump megabytes).
const MAX_STDIN_LINE: usize = 1_048_576;

const TOOL_SEARCH: &str = "search";
const TOOL_CITE: &str = "cite";
const TOOL_REMEMBER: &str = "remember";
const TOOL_ASK: &str = "ask";
const TOOL_WHO_KNOWS: &str = "who_knows";

enum StdinFrame {
    Eof,
    Line(String),
    TooLarge,
}

/// Cap accumulation before newline so oversized frames never fully allocate.
async fn read_stdin_frame<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> Result<StdinFrame> {
    let mut collected = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .await
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp stdin: {e}")))?;
        if available.is_empty() {
            return if collected.is_empty() {
                Ok(StdinFrame::Eof)
            } else {
                Ok(StdinFrame::Line(
                    String::from_utf8_lossy(&collected).into_owned(),
                ))
            };
        }

        if let Some(pos) = available.iter().position(|&b| b == b'\n') {
            let end = collected.len() + pos + 1;
            if end > max {
                reader.consume(pos + 1);
                return Ok(StdinFrame::TooLarge);
            }
            collected.extend_from_slice(&available[..=pos]);
            reader.consume(pos + 1);
            // Strip trailing newline (and optional CR).
            if collected.last() == Some(&b'\n') {
                collected.pop();
            }
            if collected.last() == Some(&b'\r') {
                collected.pop();
            }
            return Ok(StdinFrame::Line(
                String::from_utf8_lossy(&collected).into_owned(),
            ));
        }

        if collected.len() + available.len() > max {
            let n = available.len();
            reader.consume(n);
            // Drain until newline or EOF so the next frame can resync.
            loop {
                let buf = reader
                    .fill_buf()
                    .await
                    .map_err(|e| KurultaiError::Other(anyhow::anyhow!("mcp stdin drain: {e}")))?;
                if buf.is_empty() {
                    return Ok(StdinFrame::TooLarge);
                }
                if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    reader.consume(pos + 1);
                    return Ok(StdinFrame::TooLarge);
                }
                let n = buf.len();
                reader.consume(n);
            }
        }

        let n = available.len();
        collected.extend_from_slice(available);
        reader.consume(n);
    }
}

fn rpc_error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.into() }
    })
}

async fn write_response(stdout: &mut (impl AsyncWriteExt + Unpin), response: &Value) -> Result<()> {
    let out = serde_json::to_string(response)
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
    Ok(())
}

/// Run the MCP server until stdin closes.
pub async fn run_stdio(brain: BrainService) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    loop {
        match read_stdin_frame(&mut reader, MAX_STDIN_LINE).await? {
            StdinFrame::Eof => break,
            StdinFrame::TooLarge => {
                tracing::warn!(max = MAX_STDIN_LINE, "mcp stdin frame exceeds cap");
                write_response(
                    &mut stdout,
                    &rpc_error(Value::Null, -32600, "request too large"),
                )
                .await?;
            }
            StdinFrame::Line(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let msg: Value = match serde_json::from_str(trimmed) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(error = %e, "mcp invalid json");
                        write_response(
                            &mut stdout,
                            &rpc_error(Value::Null, -32700, format!("parse error: {e}")),
                        )
                        .await?;
                        continue;
                    }
                };

                let Some(response) = handle_message(&brain, msg).await? else {
                    continue;
                };
                write_response(&mut stdout, &response).await?;
            }
        }
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
                "description": "Synthesize an answer with citations and confidence from the indexed brain.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "question": { "type": "string" }
                    },
                    "required": ["question"]
                }
            }),
            json!({
                "name": TOOL_WHO_KNOWS,
                "description": "Discover which sources know about a topic (source aggregates, not full synthesis).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "topic": { "type": "string" },
                        "limit": { "type": "integer", "default": 10 }
                    },
                    "required": ["topic"]
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

#[derive(Debug, Deserialize)]
struct WhoKnowsArgs {
    topic: String,
    #[serde(default = "default_limit")]
    limit: usize,
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
        TOOL_WHO_KNOWS => {
            let args: WhoKnowsArgs = serde_json::from_value(call.arguments)
                .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bad who_knows args: {e}")))?;
            let entries = brain.who_knows(&args.topic, args.limit).await?;
            serde_json::to_string(&entries)
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
        BrainService::new(
            store,
            embedder,
            Arc::new(crate::rerank::NullReranker::new()),
            Arc::new(crate::synthesize::ExtractiveSynthesizer::new()),
        )
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
        assert!(names.contains(&"who_knows"));
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

    #[tokio::test]
    async fn cite_and_remember_tool_calls() {
        let brain = brain_with_fixture().await;

        let remember = handle_message(
            &brain,
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "remember",
                    "arguments": {
                        "title": "MCP note",
                        "summary": "remember via tools/call works",
                        "tags": ["mcp"]
                    }
                }
            }),
        )
        .await
        .unwrap()
        .unwrap();
        let text = remember["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("remembered atom id="));
        assert_eq!(remember["result"]["isError"], false);

        let cite_miss = handle_message(
            &brain,
            json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "cite",
                    "arguments": { "source": "agent", "source_id": "missing" }
                }
            }),
        )
        .await
        .unwrap()
        .unwrap();
        let miss = cite_miss["result"]["content"][0]["text"].as_str().unwrap();
        assert!(miss.contains("No atom"));

        // Cite a known fixture atom via search → source/source_id.
        let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 1).await.unwrap();
        let atom = &hits[0].atom;
        let cite_hit = handle_message(
            &brain,
            json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "cite",
                    "arguments": {
                        "source": atom.source,
                        "source_id": atom.source_id
                    }
                }
            }),
        )
        .await
        .unwrap()
        .unwrap();
        let cite_text = cite_hit["result"]["content"][0]["text"].as_str().unwrap();
        assert!(cite_text.contains(&atom.source));
        assert!(!cite_text.contains("No atom"));
    }

    #[tokio::test]
    async fn ask_and_who_knows_tool_calls() {
        let brain = brain_with_fixture().await;

        let ask = handle_message(
            &brain,
            json!({
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "ask",
                    "arguments": { "question": "KNOWN_PHRASE_KURULTAI_42" }
                }
            }),
        )
        .await
        .unwrap()
        .unwrap();
        let ask_text = ask["result"]["content"][0]["text"].as_str().unwrap();
        let answer: crate::types::Answer = serde_json::from_str(ask_text).unwrap();
        assert!(answer.confidence > 0.0);
        assert!(!answer.citations.is_empty());

        let who = handle_message(
            &brain,
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {
                    "name": "who_knows",
                    "arguments": { "topic": "KNOWN_PHRASE_KURULTAI_42", "limit": 5 }
                }
            }),
        )
        .await
        .unwrap()
        .unwrap();
        let who_text = who["result"]["content"][0]["text"].as_str().unwrap();
        assert!(who_text.contains("notes"));
    }
}
