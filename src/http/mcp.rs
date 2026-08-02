//! MCP over HTTP + SSE (Phase 6 / #104).
//!
//! Opt-in when a shared secret is configured (`KURULTAI_MCP_HTTP_SECRET` or
//! `[runtime] mcp_http_secret`). Exposes a **read-only** tool surface
//! (`search` / `cite` / `ask` / `who_knows`) — no `remember` / `promote`.
//!
//! Transports:
//! - `POST /mcp` — JSON-RPC request/response (primary remote path)
//! - `GET /mcp/sse` — SSE bootstrap: `endpoint` event points at `POST /mcp`

use crate::mcp::{handle_message, BrainService, ToolSurface};
use axum::extract::State;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct McpHttpState {
    pub brain: Arc<BrainService>,
    pub secret: Arc<str>,
}

impl McpHttpState {
    pub(crate) fn new(brain: Arc<BrainService>, secret: String) -> Self {
        Self {
            brain,
            secret: Arc::from(secret),
        }
    }
}

pub(crate) fn routes(state: McpHttpState) -> Router {
    Router::new()
        .route("/mcp", post(mcp_post))
        .route("/mcp/sse", get(mcp_sse))
        .with_state(state)
}

fn secrets_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let lower = value.to_ascii_lowercase();
    if !lower.starts_with("bearer ") {
        return None;
    }
    let token = value["bearer ".len()..].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn authorize(headers: &HeaderMap, secret: &str) -> Result<(), StatusCode> {
    match extract_bearer(headers) {
        Some(token) if secrets_equal(&token, secret) => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn mcp_post(
    State(state): State<McpHttpState>,
    headers: HeaderMap,
    Json(msg): Json<Value>,
) -> Result<Response, StatusCode> {
    authorize(&headers, &state.secret)?;
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    match handle_message(&state.brain, msg, ToolSurface::ReadOnly).await {
        Ok(Some(response)) => Ok(Json(response).into_response()),
        Ok(None) => Ok(StatusCode::ACCEPTED.into_response()),
        Err(e) => Ok(Json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32000, "message": e.to_string() }
        }))
        .into_response()),
    }
}

/// Classic MCP SSE bootstrap: one `endpoint` event advertising `POST /mcp`.
async fn mcp_sse(
    State(state): State<McpHttpState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    authorize(&headers, &state.secret)?;
    // Keep-alive comment + endpoint event. Clients then POST JSON-RPC to /mcp.
    let body = "event: endpoint\ndata: /mcp\n\n: ping\n\n";
    let mut response = Response::new(body.to_string().into());
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    Ok(response)
}

/// Resolve MCP HTTP shared secret from env (preferred) then config.
pub fn resolve_mcp_http_secret(config_secret: Option<&str>) -> Option<String> {
    if let Ok(env) = std::env::var("KURULTAI_MCP_HTTP_SECRET") {
        let trimmed = env.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    config_secret
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_equal_rejects_length_mismatch() {
        assert!(!secrets_equal("abc", "ab"));
        assert!(secrets_equal("secret", "secret"));
        assert!(!secrets_equal("secret", "Secret"));
    }

    #[test]
    fn resolve_prefers_env() {
        std::env::set_var("KURULTAI_MCP_HTTP_SECRET", "from-env");
        assert_eq!(
            resolve_mcp_http_secret(Some("from-config")).as_deref(),
            Some("from-env")
        );
        std::env::remove_var("KURULTAI_MCP_HTTP_SECRET");
        assert_eq!(
            resolve_mcp_http_secret(Some("from-config")).as_deref(),
            Some("from-config")
        );
        assert_eq!(resolve_mcp_http_secret(Some("  ")), None);
    }
}
