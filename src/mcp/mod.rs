use crate::app::Runtime;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

/// Serve MCP over stdio (newline-delimited JSON-RPC).
pub async fn serve_stdio(rt: Runtime) -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    loop {
        let line = {
            let mut reader = stdin.lock();
            let mut line = String::new();
            let n = reader.read_line(&mut line).context("read stdin")?;
            if n == 0 {
                break;
            }
            line
        };
        if line.trim().is_empty() {
            continue;
        }
        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                write_response(
                    &mut stdout,
                    RpcResponse {
                        jsonrpc: "2.0",
                        id: Value::Null,
                        result: None,
                        error: Some(RpcError {
                            code: -32700,
                            message: format!("parse error: {e}"),
                        }),
                    },
                )?;
                continue;
            }
        };
        if req.id.is_none() {
            continue;
        }
        let id = req.id.clone().unwrap_or(Value::Null);
        let result = handle(&rt, &req.method, req.params).await;
        let resp = match result {
            Ok(v) => RpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(v),
                error: None,
            },
            Err(e) => RpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(RpcError {
                    code: -32000,
                    message: e.to_string(),
                }),
            },
        };
        write_response(&mut stdout, resp)?;
    }
    Ok(())
}

fn write_response(stdout: &mut std::io::Stdout, resp: RpcResponse) -> Result<()> {
    serde_json::to_writer(&mut *stdout, &resp)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

async fn handle(rt: &Runtime, method: &str, params: Value) -> Result<Value> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "kurultai", "version": env!("CARGO_PKG_VERSION") }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_defs() })),
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("missing tool name"))?;
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            call_tool(rt, name, args).await
        }
        other => anyhow::bail!("method not found: {other}"),
    }
}

fn tool_defs() -> Vec<Value> {
    vec![
        json!({
            "name": "search",
            "description": "Search the Kurultai knowledge store (FTS-first)",
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
            "name": "read_atom",
            "description": "Read a knowledge atom by id",
            "inputSchema": {
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            }
        }),
        json!({
            "name": "status",
            "description": "Kurultai store and source status",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "reindex",
            "description": "Re-index configured sources",
            "inputSchema": {
                "type": "object",
                "properties": { "full": { "type": "boolean", "default": true } }
            }
        }),
    ]
}

async fn call_tool(rt: &Runtime, name: &str, args: Value) -> Result<Value> {
    let payload = match name {
        "search" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("query required"))?;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let results = rt.query.search(query, limit).await?;
            let citations: Vec<Value> = results
                .iter()
                .map(|r| {
                    json!({
                        "id": r.atom.id,
                        "source": r.atom.source,
                        "source_id": r.atom.source_id,
                        "title": r.atom.title,
                        "url": r.atom.source_uri,
                        "score": r.score,
                        "matched_by": r.matched_by,
                        "excerpt": r.atom.content.chars().take(240).collect::<String>(),
                    })
                })
                .collect();
            json!({ "results": citations })
        }
        "read_atom" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("id required"))?;
            match rt.store.get(id)? {
                Some(atom) => json!({ "atom": atom }),
                None => json!({ "atom": null }),
            }
        }
        "status" => {
            json!({
                "env": rt.config.env.as_str(),
                "atoms": rt.store.count()?,
                "store": rt.store.path().display().to_string(),
                "embed_mode": rt.embedder.name(),
                "sources": rt.config.sources.iter().map(|s| json!({
                    "name": s.name,
                    "kind": s.kind.as_str(),
                    "enabled": s.enabled,
                    "implemented": s.kind.is_implemented(),
                })).collect::<Vec<_>>(),
            })
        }
        "reindex" => {
            let full = args.get("full").and_then(|v| v.as_bool()).unwrap_or(true);
            let report = rt.index(full).await?;
            json!({
                "upserted": report.upserted,
                "skipped_unchanged": report.skipped_unchanged,
                "orphans_removed": report.orphans_removed,
                "sources": report.sources,
            })
        }
        other => anyhow::bail!("unknown tool: {other}"),
    };
    Ok(json!({
        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&payload)? }],
        "structuredContent": payload,
    }))
}

/// Tool names exposed by this server (for tests).
pub fn tool_names() -> Vec<&'static str> {
    vec!["search", "read_atom", "status", "reindex"]
}
