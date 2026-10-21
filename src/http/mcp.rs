//! MCP over HTTP + SSE (Phase 6 / #104).
//!
//! Opt-in when a shared secret is configured (`KURULTAI_MCP_HTTP_SECRET` or
//! `[runtime] mcp_http_secret`). Exposes a **read-only** tool surface
//! (`search` / `cite` / `ask` / `who_knows`) — no `remember` / `promote`.
//!
//! Transports:
//! - `POST /mcp` — JSON-RPC request/response (primary remote path)
//! - `GET /mcp/sse` — SSE bootstrap: `endpoint` event points at `POST /mcp`

use crate::mcp::{handle_message_scoped, BrainService, ToolSurface};
use crate::write_policy::{WriteContext, WriteTransport};
use axum::extract::{FromRequest, Request, State};
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
    /// Lazily-connected hub key store (U5). `serve_with` mounts `/mcp` with
    /// just the shared secret; hub-key auth resolves `KURULTAI_DATABASE_URL`
    /// on first use — the same env seam `serve_with` uses for the REST gate's
    /// `hub.key_store`. `None` forever on solo lanes.
    #[cfg(feature = "postgres")]
    hub_keys: Arc<tokio::sync::OnceCell<Option<Arc<crate::hub::HubKeyStore>>>>,
}

impl McpHttpState {
    pub(crate) fn new(brain: Arc<BrainService>, secret: String) -> Self {
        Self {
            brain,
            secret: Arc::from(secret),
            #[cfg(feature = "postgres")]
            hub_keys: Arc::new(tokio::sync::OnceCell::new()),
        }
    }

    /// Hub key store for `/mcp` member auth — only when `KURULTAI_FEATURE_HUB`
    /// is on AND a Postgres DSN is set; connect failures are logged and
    /// treated as "no hub" so a bad DSN degrades to solo behavior (which
    /// then fails seat lookups as before) rather than retrying per request.
    #[cfg(feature = "postgres")]
    async fn hub_keys(&self) -> Option<Arc<crate::hub::HubKeyStore>> {
        self.hub_keys
            .get_or_init(|| async {
                if !crate::features::enabled("hub") {
                    return None;
                }
                let url = crate::store::database_url_from_env()?;
                match crate::hub::HubKeyStore::connect(&url).await {
                    Ok(store) => Some(Arc::new(store)),
                    Err(e) => {
                        tracing::warn!(error = %e, "mcp: hub key store unavailable");
                        None
                    }
                }
            })
            .await
            .clone()
    }

    /// Scope for the solo credential classes (shared secret, seat token)
    /// when a request reaches them on a hub daemon: fail closed to
    /// `company` visibility — the same "authenticated without a member
    /// principal" ruling [`crate::http::auth::request_pg_scope`] makes for
    /// REST reads. Without a hub key store this is the ambient
    /// `PgScope::INTERNAL`, so solo behavior is literally unchanged.
    async fn non_member_scope(&self) -> crate::store::scope::PgScope {
        #[cfg(feature = "postgres")]
        if self.hub_keys().await.is_some() {
            return crate::store::scope::PgScope::closed();
        }
        crate::store::scope::PgScope::INTERNAL
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
    /// Minted `hub_api_keys` bearer (U5/R13) — member keys carry a `team_id`
    /// that becomes the request's [`crate::store::scope::PgScope`];
    /// `is_admin` keys (U4/R17) get the admin scope + full tool surface.
    #[cfg(feature = "postgres")]
    Hub(crate::hub::HubPrincipal),
}

/// Solo seat-token lane — the registered-agent key hash lookup on the brain
/// store. Reached only when no hub key store resolved the bearer first.
async fn seat_auth(state: &McpHttpState, token: &str) -> Result<McpAuth, StatusCode> {
    let hash = crate::hashutil::sha256_hex(token);
    match state.brain.store().resolve_agent_by_key_hash(&hash).await {
        Ok(Some(agent)) => Ok(McpAuth::Seat(agent.codename)),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn authorize(headers: &HeaderMap, state: &McpHttpState) -> Result<McpAuth, StatusCode> {
    match extract_bearer(headers) {
        Some(token) if secrets_equal(&token, &state.secret) => Ok(McpAuth::Secret),
        Some(token) => {
            // U5/R13: on a hub daemon the minted `hub_api_keys` table is the
            // authority for member credentials — resolve it before the seat
            // path (which a Postgres brain store can't serve anyway). A miss
            // while active keys exist denies at the edge (revoked/unknown →
            // 401); with zero active keys we fall through so a seat token on
            // a half-migrated hybrid still resolves — same ruling
            // `hub_api_auth`'s `authorize_bearer` makes for REST.
            #[cfg(feature = "postgres")]
            if let Some(keys) = state.hub_keys().await {
                return match keys.resolve_token(&token).await {
                    Ok(Some(principal)) => Ok(McpAuth::Hub(principal)),
                    Ok(None) => match keys.has_active_keys().await {
                        Ok(true) => Err(StatusCode::UNAUTHORIZED),
                        Ok(false) => seat_auth(state, &token).await,
                        Err(e) => {
                            tracing::warn!(error = %e, "hub key count failed");
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "hub key resolve failed");
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                };
            }
            seat_auth(state, &token).await
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn mcp_post(
    State(state): State<McpHttpState>,
    // `mut` only mutates under `postgres` — the HubPrincipal extension insert.
    #[allow(unused_mut)] mut req: Request,
) -> Result<Response, StatusCode> {
    let headers = req.headers().clone();
    let auth = authorize(&headers, &state).await?;
    // Stamp the resolved principal into request extensions — the same seam
    // `hub_api_auth` uses for REST member paths (`MaybeHubPrincipal` and
    // `request_pg_scope` read it there).
    #[cfg(feature = "postgres")]
    if let McpAuth::Hub(principal) = &auth {
        req.extensions_mut().insert(principal.clone());
    }
    let Json(msg) = Json::<Value>::from_request(req, &state)
        .await
        .map_err(|e| e.status())?;
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    // X-Kurultai-* stamps from the broker override the credential defaults.
    let h = |n: &str| {
        headers
            .get(n)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    };
    let (surface, ctx, scope) = match auth {
        McpAuth::Secret => (
            ToolSurface::ReadOnly,
            WriteContext::from_env(WriteTransport::Mcp),
            state.non_member_scope().await,
        ),
        McpAuth::Seat(codename) => {
            let ctx = WriteContext {
                agent_id: Some(h("x-kurultai-agent").unwrap_or(codename)),
                namespace: h("x-kurultai-chat"),
                transport: WriteTransport::Mcp,
                mode: crate::write_policy::WriteMode::from_env(),
            };
            (ToolSurface::Full, ctx, state.non_member_scope().await)
        }
        #[cfg(feature = "postgres")]
        McpAuth::Hub(principal) => {
            let ctx = WriteContext {
                agent_id: Some(h("x-kurultai-agent").unwrap_or_else(|| principal.agent_id.clone())),
                namespace: h("x-kurultai-chat"),
                transport: WriteTransport::Mcp,
                mode: crate::write_policy::WriteMode::from_env(),
            };
            // The resolved principal is the only source of team/admin truth —
            // the same ruling `request_pg_scope` makes from the extension.
            let scope = if principal.is_admin {
                crate::store::scope::PgScope::admin()
            } else {
                crate::store::scope::PgScope::member(principal.team_id.clone())
            };
            let surface = if principal.is_admin {
                ToolSurface::Full
            } else {
                ToolSurface::HubMember
            };
            (surface, ctx, scope)
        }
    };
    match handle_message_scoped(&state.brain, msg, surface, &ctx, scope).await {
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
