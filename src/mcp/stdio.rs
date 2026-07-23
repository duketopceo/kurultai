//! Minimal MCP stdio server exposing the Phase 1 `search` read tool.
//!
//! Implements just enough of the Model Context Protocol for an agent to call
//! `tools/list` and `tools/call` for `search`. Everything is JSON-RPC 2.0 over
//! stdin/stdout, one message per line.

use crate::query::QueryEngine;
use crate::types::SearchResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn err(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// Run the MCP stdio server loop until stdin closes.
pub async fn run(query_engine: Arc<dyn QueryEngine>) -> crate::Result<()> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await.map_err(|e| {
        crate::KurultaiError::Other(anyhow::Error::msg(format!("stdin read failed: {e}")))
    })? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        tracing::trace!(line, "mcp request");
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse::err(None, -32700, format!("parse error: {e}"));
                send(&mut stdout, &response).await?;
                continue;
            }
        };

        let response = handle_request(&query_engine, request).await;
        send(&mut stdout, &response).await?;
    }

    Ok(())
}

async fn send(stdout: &mut tokio::io::Stdout, response: &JsonRpcResponse) -> crate::Result<()> {
    let mut text = serde_json::to_string(response).map_err(|e| {
        crate::KurultaiError::Other(anyhow::Error::msg(format!("json serialize failed: {e}")))
    })?;
    text.push('\n');
    stdout.write_all(text.as_bytes()).await.map_err(|e| {
        crate::KurultaiError::Other(anyhow::Error::msg(format!("stdout write failed: {e}")))
    })?;
    stdout.flush().await.map_err(|e| {
        crate::KurultaiError::Other(anyhow::Error::msg(format!("stdout flush failed: {e}")))
    })?;
    Ok(())
}

async fn handle_request(
    query_engine: &Arc<dyn QueryEngine>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(request.id),
        "initialized" => JsonRpcResponse::ok(request.id, Value::Null),
        "tools/list" => handle_tools_list(request.id),
        "tools/call" => handle_tools_call(query_engine, request.id, request.params).await,
        _ => JsonRpcResponse::err(
            request.id,
            -32601,
            format!("method not found: {}", request.method),
        ),
    }
}

fn handle_initialize(id: Option<Value>) -> JsonRpcResponse {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "kurultai",
            "version": env!("CARGO_PKG_VERSION")
        }
    });
    JsonRpcResponse::ok(id, result)
}

fn handle_tools_list(id: Option<Value>) -> JsonRpcResponse {
    let result = serde_json::json!({
        "tools": [
            {
                "name": "search",
                "description": "Search the indexed knowledge base. Returns short excerpts with citations — never full source files.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "limit": { "type": "integer", "description": "Maximum number of results", "default": 10 }
                    },
                    "required": ["query"]
                }
            }
        ]
    });
    JsonRpcResponse::ok(id, result)
}

async fn handle_tools_call(
    query_engine: &Arc<dyn QueryEngine>,
    id: Option<Value>,
    params: Value,
) -> JsonRpcResponse {
    let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

    match name {
        "search" => handle_search(query_engine, id, params).await,
        _ => JsonRpcResponse::err(id, -32602, format!("tool not found: {name}")),
    }
}

async fn handle_search(
    query_engine: &Arc<dyn QueryEngine>,
    id: Option<Value>,
    params: Value,
) -> JsonRpcResponse {
    let args = match params.get("arguments").cloned() {
        Some(Value::Object(map)) => map,
        _ => {
            return JsonRpcResponse::err(id, -32602, "missing arguments".into());
        }
    };

    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

    if query.is_empty() {
        return JsonRpcResponse::err(id, -32602, "query is required".into());
    }

    match query_engine.search(query, limit).await {
        Ok(results) => {
            let citations: Vec<Value> = results.into_iter().map(search_result_to_json).collect();
            let result = serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string(&citations).unwrap_or_else(|_| "[]".into())
                    }
                ],
                "isError": false
            });
            JsonRpcResponse::ok(id, result)
        }
        Err(e) => JsonRpcResponse::err(id, -32603, format!("search failed: {e}")),
    }
}

fn search_result_to_json(result: SearchResult) -> Value {
    serde_json::json!({
        "id": result.atom.id,
        "source": result.atom.source,
        "source_id": result.atom.source_id,
        "title": result.atom.title,
        "excerpt": excerpt(&result.atom.content, 240),
        "source_uri": result.atom.source_uri,
        "score": result.score,
        "rank": result.rank,
        "matched_by": result.matched_by,
    })
}

fn excerpt(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        let mut end = max_len;
        while !content.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…", &content[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{KnowledgeAtom, SearchResult};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn search_result_to_json_is_citation_shape() {
        let atom = KnowledgeAtom {
            id: "md:notes.md".into(),
            source: "notes".into(),
            source_id: "notes.md".into(),
            title: "Notes".into(),
            summary: String::new(),
            content: "unique kurultai phrase for testing".into(),
            question: None,
            resolution: None,
            tags: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            embedding: None,
            content_hash: "hash1".into(),
            source_uri: Some("file:///tmp/notes.md".into()),
            provenance: None,
        };
        let result = SearchResult {
            atom,
            score: 0.95,
            rank: 1,
            matched_by: vec!["fts".into(), "vector".into()],
        };
        let json = search_result_to_json(result);
        assert_eq!(json.get("id").unwrap().as_str().unwrap(), "md:notes.md");
        assert_eq!(json.get("source_id").unwrap().as_str().unwrap(), "notes.md");
        assert!(json
            .get("excerpt")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("unique kurultai"));
        assert_eq!(
            json.get("source_uri").unwrap().as_str().unwrap(),
            "file:///tmp/notes.md"
        );
    }
}
