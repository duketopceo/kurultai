//! MCP over HTTP + SSE (Phase 6 / #104).
//!
//! Opt-in when a shared secret is configured (`KURULTAI_MCP_HTTP_SECRET` or
//! `[runtime] mcp_http_secret`). Exposes a **read-only** tool surface
//! (`search` / `cite` / `ask` / `who_knows`) — no `remember` / `promote`.
//!
//! Transports:
//! - `POST /mcp` — JSON-RPC request/response (primary remote path)
//! - `GET /mcp/sse` — SSE bootstrap: `endpoint` event points at `POST /mcp`

use crate::mcp::{handle_message_with, BrainService, ToolSurface};
use crate::write_policy::{WriteContext, WriteTransport};
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

/// How the caller authenticated on `/mcp`.
pub(crate) enum McpAuth {
    /// Shared-secret auth (existing deployments) — read-only surface.
    Secret,
    /// Registered agent seat token — full surface; identity carries through
    /// to stamped writes (broker path, plan 2026-09-28-001 U4).
    Seat(String),
}

async fn authorize(headers: &HeaderMap, state: &McpHttpState) -> Result<McpAuth, StatusCode> {
    match extract_bearer(headers) {
        Some(token) if secrets_equal(&token, &state.secret) => Ok(McpAuth::Secret),
        Some(token) => {
            let hash = crate::hashutil::sha256_hex(&token);
            match state.brain.store().resolve_agent_by_key_hash(&hash).await {
                Ok(Some(agent)) => Ok(McpAuth::Seat(agent.codename)),
                Ok(None) => Err(StatusCode::UNAUTHORIZED),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn mcp_post(
    State(state): State<McpHttpState>,
    headers: HeaderMap,
    Json(msg): Json<Value>,
) -> Result<Response, StatusCode> {
    let auth = authorize(&headers, &state).await?;
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let (surface, ctx) = match auth {
        McpAuth::Secret => (
            ToolSurface::ReadOnly,
            WriteContext::from_env(WriteTransport::Mcp),
        ),
        McpAuth::Seat(codename) => {
            // X-Kurultai-* stamps from the broker override the seat defaults.
            let h = |n: &str| {
                headers
                    .get(n)
                    .and_then(|v| v.to_str().ok())
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
            };
            let ctx = WriteContext {
                agent_id: Some(h("x-kurultai-agent").unwrap_or(codename)),
                namespace: h("x-kurultai-chat"),
                transport: WriteTransport::Mcp,
                mode: crate::write_policy::WriteMode::from_env(),
            };
            (ToolSurface::Full, ctx)
        }
    };
    match handle_message_with(&state.brain, msg, surface, &ctx).await {
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
    authorize(&headers, &state).await?;
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
    use crate::embed::NullEmbedder;
    use crate::rerank::NullReranker;
    use crate::store::{SqliteVecStore, Store};
    use crate::synthesize::{ExtractiveSynthesizer, Synthesizer};
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_app() -> (Router, Arc<SqliteVecStore>) {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "k-mcp-seat-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap());
        let brain = Arc::new(BrainService::new(
            store.clone() as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer) as Arc<dyn Synthesizer>,
        ));
        (
            routes(McpHttpState::new(brain, "test-secret".into())),
            store,
        )
    }

    fn tool_names(body: &Value) -> Vec<String> {
        body["result"]["tools"]
            .as_array()
            .map(|t| {
                t.iter()
                    .filter_map(|t| t["name"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn tools_list(app: &Router, bearer: &str) -> (StatusCode, Value) {
        let res = app
            .clone()
            .oneshot(
                Request::post("/mcp")
                    .header("authorization", format!("Bearer {bearer}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), 1 << 20)
            .await
            .unwrap();
        (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
    }

    #[tokio::test]
    async fn secret_auth_gets_readonly_surface() {
        let (app, _store) = test_app();
        let (status, body) = tools_list(&app, "test-secret").await;
        assert_eq!(status, StatusCode::OK);
        let names = tool_names(&body);
        assert!(names.iter().any(|n| n == "search"));
        assert!(!names.iter().any(|n| n == "remember"));
    }

    #[tokio::test]
    async fn seat_token_gets_full_surface_and_write_ctx() {
        let (app, store) = test_app();
        let (_agent, seat) = store
            .issue_agent_seat_token("devin", "laptop", "test")
            .await
            .unwrap();
        let (status, body) = tools_list(&app, &seat).await;
        assert_eq!(status, StatusCode::OK);
        let names = tool_names(&body);
        assert!(names.iter().any(|n| n == "remember"));
    }

    #[tokio::test]
    async fn unknown_token_is_401() {
        let (app, _store) = test_app();
        let (status, _) = tools_list(&app, "not-a-real-token").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn revoked_seat_is_401() {
        let (app, store) = test_app();
        let (_agent, seat) = store
            .issue_agent_seat_token("devin", "revoked", "test")
            .await
            .unwrap();
        store.revoke_agent("devin", Some("revoked")).await.unwrap();
        let (status, _) = tools_list(&app, &seat).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn secrets_equal_rejects_length_mismatch() {
        assert!(!secrets_equal("abc", "ab"));
        assert!(secrets_equal("secret", "secret"));
        assert!(!secrets_equal("secret", "Secret"));
    }

    #[test]
    fn resolve_prefers_env() {
        let _guard = crate::testutil::EnvGuard::set("KURULTAI_MCP_HTTP_SECRET", "from-env");
        assert_eq!(
            resolve_mcp_http_secret(Some("from-config")).as_deref(),
            Some("from-env")
        );
        drop(_guard);
        let _unset = crate::testutil::EnvGuard::remove("KURULTAI_MCP_HTTP_SECRET");
        assert_eq!(
            resolve_mcp_http_secret(Some("from-config")).as_deref(),
            Some("from-config")
        );
        assert_eq!(resolve_mcp_http_secret(Some("  ")), None);
    }
}
