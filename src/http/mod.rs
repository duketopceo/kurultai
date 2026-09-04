//! Thin local HTTP API mirroring MCP read tools (Phase 3 / #7).
//!
//! Default bind is localhost; REST is unauthenticated on loopback.
//! Hub mode (`KURULTAI_HUB_AUTH=api_key`, `KURULTAI_HUB_BIND=all`) authenticates `/api/*`.
//! MCP HTTP/SSE (`/mcp`, `/mcp/sse`) is **opt-in** when a shared secret is set.
//! Loopback dump ingest (`POST /ingest`) is **opt-in** when `KURULTAI_INGEST_SECRET` is set.
//! Brain UI: single surface at `GET /ui` (embedded `ui/` assets — see `ui` module).

mod auth;
mod hub_listen;
mod mcp;
mod ui;

#[cfg(feature = "postgres")]
pub use auth::HubPrincipal;
pub use auth::{
    path_requires_hub_auth, resolve_admin_token, resolve_bind_all_from_env,
    resolve_hub_gate_from_env, write_route_decision, HubAuth, HubGate, MaybeHubPrincipal,
    WriteRouteDecision, ENV_ADMIN_TOKEN,
};
pub use hub_listen::resolve_listen_socket;
mod ingest;

pub use ingest::resolve_ingest_secret;
pub use mcp::resolve_mcp_http_secret;

use crate::brain::AgentAtomView;
use crate::daemon::DaemonStatus;
use crate::error::KurultaiError;
use crate::mcp::brain::BrainService;
use crate::mcp::interface::AgentRead;
use crate::metrics::{MetricOp, MetricsRegistry, TimedObserve};
use crate::synthesize::WhoKnowsEntry;
use crate::types::{Answer, Citation, SearchResult};
use auth::hub_api_auth;
use axum::extract::{Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    brain: Arc<BrainService>,
    status: Arc<DaemonStatus>,
    metrics: Arc<MetricsRegistry>,
    hub: HubGate,
    #[cfg(feature = "postgres")]
    hub_activity: Option<Arc<crate::hub::HubActivityStore>>,
}

/// Options for the localhost HTTP daemon.
#[derive(Debug, Clone, Default)]
pub struct ServeOptions {
    pub port: u16,
    /// When set, mounts authenticated MCP HTTP/SSE routes.
    pub mcp_http_secret: Option<String>,
    /// Bind `0.0.0.0` instead of loopback (hub mode).
    pub bind_all: bool,
    pub hub: HubGate,
}

fn json_error(
    status: StatusCode,
    message: impl Into<String>,
    request_id: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({
            "ok": false,
            "error": message.into(),
            "request_id": request_id,
        })),
    )
}

/// Serve search/ask/cite/who_knows on `127.0.0.1:port` until cancelled.
pub async fn serve(brain: BrainService, status: Arc<DaemonStatus>, port: u16) -> crate::Result<()> {
    serve_with(
        brain,
        status,
        ServeOptions {
            port,
            mcp_http_secret: None,
            bind_all: false,
            hub: HubGate::default(),
        },
    )
    .await
}

/// Serve HTTP (+ optional MCP HTTP/SSE + optional loopback `/ingest`) on `127.0.0.1`.
pub async fn serve_with(
    brain: BrainService,
    status: Arc<DaemonStatus>,
    opts: ServeOptions,
) -> crate::Result<()> {
    let brain = Arc::new(brain);
    let mut hub = opts.hub;
    if hub.auth == HubAuth::None && hub.api_keys.is_empty() {
        hub = auth::resolve_hub_gate_from_env();
    }
    #[cfg(feature = "postgres")]
    let hub_activity = if crate::features::enabled("hub") {
        if let Some(url) = crate::store::database_url_from_env() {
            match crate::hub::HubKeyStore::connect(&url).await {
                Ok(key_store) => {
                    let pool = key_store.pool().clone();
                    hub.key_store = Some(Arc::new(key_store));
                    Some(Arc::new(crate::hub::HubActivityStore::new(pool)))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "hub key store unavailable");
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };
    let bind_all = opts.bind_all || auth::resolve_bind_all_from_env();
    let addr = hub_listen::resolve_listen_socket(opts.port, bind_all, &hub)?;
    let state = app_state(
        Arc::clone(&brain),
        status,
        MetricsRegistry::shared(),
        hub.clone(),
    );
    #[cfg(feature = "postgres")]
    let state = {
        let mut state = state;
        state.hub_activity = hub_activity;
        state
    };
    let mut app = router(state);
    if let Some(secret) = mcp::resolve_mcp_http_secret(opts.mcp_http_secret.as_deref()) {
        tracing::info!("mcp HTTP/SSE enabled at POST /mcp and GET /mcp/sse (bearer auth)");
        app = app.merge(mcp::routes(mcp::McpHttpState::new(
            Arc::clone(&brain),
            secret,
        )));
    } else {
        tracing::info!(
            "mcp HTTP/SSE disabled (set KURULTAI_MCP_HTTP_SECRET or [runtime].mcp_http_secret)"
        );
    }
    if let Some(secret) = resolve_ingest_secret() {
        tracing::info!("loopback ingest enabled at POST /ingest (shared secret required)");
        app = app.merge(ingest::routes(ingest::IngestState {
            store: brain.store(),
            embedder: brain.embedder(),
            secret,
            mode: crate::write_policy::WriteMode::from_env(),
        }));
    } else {
        tracing::info!("loopback ingest disabled (set KURULTAI_INGEST_SECRET to enable)");
    }
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::KurultaiError::Other(anyhow::anyhow!("bind {addr}: {e}")))?;
    tracing::info!(
        %addr,
        bind_all,
        auth = ?hub.auth,
        "http daemon listening"
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|e| crate::KurultaiError::Other(anyhow::anyhow!("http serve: {e}")))?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/status", get(api_status))
        .route("/api/metrics", get(api_metrics))
        .route("/api/atoms", get(api_atoms))
        .route("/api/graph", get(api_graph))
        .route("/api/ontology", get(api_ontology))
        .route("/api/touch", post(api_touch))
        .route("/api/activity", get(api_activity))
        .route("/api/hub/activity", get(api_hub_activity))
        .route("/api/promote", post(api_promote))
        .route("/api/search", get(search_get).post(search_post))
        .route("/api/recall", post(recall_post))
        .route("/api/ask", get(ask_get).post(ask_post))
        .route("/api/open", get(api_open))
        .route("/search", get(search_get).post(search_post))
        .route("/ask", get(ask_get).post(ask_post))
        .route("/cite", post(cite_post))
        .route("/who_knows", post(who_knows_post))
        .merge(ui::routes())
        .layer(middleware::from_fn_with_state(
            state.hub.clone(),
            hub_api_auth,
        ))
        // Outermost of the two: runs before hub auth, and fails closed on write
        // routes regardless of `HubAuth` (which is `None` by default).
        .layer(middleware::from_fn(auth::write_route_guard))
        .layer(middleware::from_fn(no_store_api))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Build the loopback `POST /ingest` router for integration tests / external embedding.
///
/// Mirrors the route mounted by [`serve_with`] without binding a socket, with the
/// write containment `mode` injected rather than read from the environment.
pub fn build_ingest_app(
    store: std::sync::Arc<dyn crate::store::Store>,
    embedder: std::sync::Arc<dyn crate::embed::Embedder>,
    secret: String,
    mode: crate::write_policy::WriteMode,
) -> Router {
    ingest::routes(ingest::IngestState {
        store,
        embedder,
        secret,
        mode,
    })
}

/// Build the application router for integration tests / external embedding.
///
/// Mirrors the routes mounted by [`serve_with`] without binding a socket.
pub fn build_app(brain: BrainService, status: Arc<DaemonStatus>, hub: HubGate) -> Router {
    router(app_state(
        Arc::new(brain),
        status,
        MetricsRegistry::shared(),
        hub,
    ))
}

fn app_state(
    brain: Arc<BrainService>,
    status: Arc<DaemonStatus>,
    metrics: Arc<MetricsRegistry>,
    mut hub: HubGate,
) -> AppState {
    hub.agent_store = Some(brain.store());
    AppState {
        brain,
        status,
        metrics,
        hub,
        #[cfg(feature = "postgres")]
        hub_activity: None,
    }
}

/// Browsers must not reuse `/api/*` JSON (graph/status used to boot from a stale cache).
async fn no_store_api(req: Request, next: Next) -> Response {
    let is_api = req.uri().path().starts_with("/api/");
    let mut res = next.run(req).await;
    if is_api {
        res.headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    res
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "service": "kurultai" }))
}

/// Prometheus text exposition of in-process query histograms (#102 thin).
async fn api_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.render_prometheus();
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
        )],
        body,
    )
}

async fn api_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_status", request_id=%request_id);
    match state.brain.atom_count().await {
        Ok(atoms) => match state.brain.lane_counts().await {
            Ok((trusted, quarantine, merge_pending)) => {
                let memory = match state.brain.tier_counts().await {
                    Ok((hot, warm, cold)) => serde_json::json!({
                        "hot": hot,
                        "warm": warm,
                        "cold": cold,
                    }),
                    Err(_) => serde_json::Value::Null,
                };
                let inbox = state.status.inbox_summary();
                Ok(Json(serde_json::json!({
                    "ok": true,
                    "service": "kurultai",
                    "version": env!("CARGO_PKG_VERSION"),
                    "atoms": atoms,
                    "request_id": &request_id,
                    "brain": {
                        "trusted_count": trusted,
                        "quarantine_count": quarantine,
                        "merge_candidates_pending": merge_pending,
                    },
                    "inbox": inbox,
                    "memory": memory,
                    "scheduler": state.status.snapshot(),
                    "metrics": state.metrics.summary_json(),
                })))
            }
            Err(e) => Err(json_error(
                StatusCode::SERVICE_UNAVAILABLE,
                e.to_string(),
                &request_id,
            )),
        },
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "ok": false,
                "service": "kurultai",
                "version": env!("CARGO_PKG_VERSION"),
                "atoms": null,
                "error": e.to_string(),
                "request_id": &request_id,
                "scheduler": state.status.snapshot(),
                "metrics": state.metrics.summary_json(),
            })),
        )),
    }
}

/// Full-atom list ceiling — large Brain loads must use `/api/graph` tiers instead.
const ATOMS_LIST_CEILING: usize = 20_000;
/// Lean graph node ceiling for max-mode progressive loads.
const GRAPH_LIST_CEILING: usize = 20_000;

fn parse_atoms_limit(raw: Option<&str>) -> usize {
    match raw {
        Some(s) => s
            .parse::<usize>()
            .unwrap_or(500)
            .clamp(1, ATOMS_LIST_CEILING),
        None => 500,
    }
}

fn parse_graph_limit(raw: Option<&str>) -> usize {
    match raw {
        Some("max") | Some("all") => GRAPH_LIST_CEILING,
        Some(s) => s
            .parse::<usize>()
            .unwrap_or(GRAPH_LIST_CEILING)
            .clamp(1, 50_000),
        None => GRAPH_LIST_CEILING,
    }
}

async fn api_atoms(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::types::SearchResult>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_atoms", request_id=%request_id);
    state.status.touch_client_activity();
    let limit = parse_atoms_limit(params.get("limit").map(String::as_str));
    let include_quarantine = params
        .get("include_quarantine")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    state
        .brain
        .list_atoms_filtered(limit, include_quarantine)
        .await
        .map(|atoms| {
            Json(
                atoms
                    .into_iter()
                    .enumerate()
                    .map(|(i, atom)| crate::types::SearchResult {
                        atom,
                        score: 1.0,
                        rank: i,
                        matched_by: vec!["list".to_string()],
                    })
                    .collect(),
            )
        })
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
}

async fn api_ontology(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_ontology", request_id=%request_id);
    state.status.touch_client_activity();
    let store = state.brain.store();
    let entities = match store.list_ontology_entities(500).await {
        Ok(v) => v,
        Err(e) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ));
        }
    };
    let links = match store.list_ontology_links(None).await {
        Ok(v) => v,
        Err(e) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ));
        }
    };
    Ok(Json(serde_json::json!({
        "ok": true,
        "request_id": &request_id,
        "entities": entities,
        "links": links,
    })))
}

async fn api_graph(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_graph", request_id=%request_id);
    state.status.touch_client_activity();
    let limit = parse_graph_limit(params.get("limit").map(String::as_str));
    let tier = params
        .get("tier")
        .and_then(|s| crate::memory::MemoryTier::parse(s));
    if let Some(raw) = params.get("tier") {
        if tier.is_none() && !raw.is_empty() {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                format!("invalid tier '{raw}' (want hot|warm|cold)"),
                &request_id,
            ));
        }
    }
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::Graph);
    let include_quarantine = params
        .get("include_quarantine")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    match state
        .brain
        .list_graph_nodes(tier, limit, include_quarantine)
        .await
    {
        Ok(nodes) => {
            let count = nodes.len();
            timer.success(count as u64);
            Ok(Json(serde_json::json!({
                "ok": true,
                "request_id": &request_id,
                "tier": tier.map(|t| t.as_str()),
                "count": count,
                "nodes": nodes,
            })))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct TouchBody {
    atom_id: String,
}

async fn api_touch(
    State(state): State<AppState>,
    Json(body): Json<TouchBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_touch", request_id=%request_id);
    state.status.touch_client_activity();
    match state.brain.touch_access(&body.atom_id).await {
        Ok(Some(atom)) => {
            let file_path = atom
                .metadata
                .get("file_path")
                .cloned()
                .filter(|s| !s.is_empty());
            Ok(Json(serde_json::json!({
                "ok": true,
                "request_id": &request_id,
                "atom_id": atom.id,
                "title": atom.title,
                "source": atom.source,
                "summary": atom.summary,
                "tags": atom.tags,
                "file_path": file_path,
                "indexed_at": atom.indexed_at,
                "last_accessed_at": atom.last_accessed_at,
            })))
        }
        Ok(None) => Err(json_error(
            StatusCode::NOT_FOUND,
            "atom not found",
            &request_id,
        )),
        Err(e) => {
            let status = match &e {
                KurultaiError::Store(msg) if msg.contains("atom not found") => {
                    StatusCode::NOT_FOUND
                }
                _ => StatusCode::BAD_REQUEST,
            };
            Err(json_error(status, e.to_string(), &request_id))
        }
    }
}

#[derive(Debug, Deserialize)]
struct PromoteBody {
    atom_id: String,
    #[serde(default)]
    reason: Option<String>,
}

async fn api_promote(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    Json(body): Json<PromoteBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_promote", request_id=%request_id);
    state.status.touch_client_activity();
    if let Some(reason) = &body.reason {
        if reason.chars().count() > 200 {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                "reason must be at most 200 characters",
                &request_id,
            ));
        }
    }
    let actor = http_actor(&principal);
    match state
        .brain
        .promote(&body.atom_id, &actor, body.reason.as_deref())
        .await
    {
        Ok(res) => {
            #[cfg(feature = "postgres")]
            log_hub_write(
                &state,
                &principal,
                "promote",
                "http",
                body.reason.as_deref(),
                Some(&body.atom_id),
            )
            .await;
            Ok(Json(serde_json::json!({
                "ok": true,
                "request_id": &request_id,
                "atom_id": res.atom_id,
                "actor": res.actor,
            })))
        }
        Err(e) => Err(json_error(
            StatusCode::BAD_REQUEST,
            e.to_string(),
            &request_id,
        )),
    }
}

async fn api_activity(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let since: u64 = params
        .get("since")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let (next_seq, events) = state.brain.activity().since(since);
    Json(serde_json::json!({
        "next_seq": next_seq,
        "events": events,
    }))
}

#[cfg(feature = "postgres")]
async fn api_hub_activity(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50)
        .clamp(1, 500);
    let Some(store) = state.hub_activity.as_ref() else {
        return Err(json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "hub activity log requires KURULTAI_FEATURE_HUB=1 with Postgres",
            &request_id,
        ));
    };
    match store.list(limit).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "ok": true, "entries": entries }))),
        Err(e) => Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
            &request_id,
        )),
    }
}

#[cfg(not(feature = "postgres"))]
async fn api_hub_activity(
    State(_state): State<AppState>,
    Query(_params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    Err(json_error(
        StatusCode::SERVICE_UNAVAILABLE,
        "hub activity log requires postgres feature",
        &request_id,
    ))
}

fn http_actor(principal: &MaybeHubPrincipal) -> String {
    if let Some(agent) = principal.agent_id() {
        format!("hub:{agent}")
    } else {
        crate::write_policy::WriteContext::from_env(crate::write_policy::WriteTransport::Http)
            .actor()
    }
}

#[cfg(feature = "postgres")]
async fn log_hub_write(
    state: &AppState,
    principal: &MaybeHubPrincipal,
    namespace: &str,
    transport: &str,
    reason: Option<&str>,
    atom_id: Option<&str>,
) {
    let (Some(store), Some(p)) = (state.hub_activity.as_ref(), principal.0.as_ref()) else {
        return;
    };
    if let Err(e) = store
        .append(
            &p.agent_id,
            &p.team_id,
            namespace,
            transport,
            reason,
            atom_id,
        )
        .await
    {
        tracing::warn!(error = %e, "hub activity append failed");
    }
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    include_quarantine: bool,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AskQuery {
    question: String,
}

#[derive(Debug, Deserialize)]
struct SearchBody {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    include_quarantine: bool,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecallBody {
    /// Project namespace. Omit to fall back to `$KURULTAI_PROJECT`, then `"default"`.
    #[serde(default)]
    project: Option<String>,
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    include_quarantine: bool,
}

async fn search_post(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    Json(body): Json<SearchBody>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("search_post", request_id=%request_id);
    state.status.touch_client_activity();
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::Search);
    match state
        .brain
        .search_scoped_hub(
            &body.query,
            body.limit,
            body.include_quarantine,
            body.source.as_deref(),
            principal.team_id(),
        )
        .await
    {
        Ok(results) => {
            timer.success(results.len() as u64);
            Ok(Json(results))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

async fn recall_post(
    State(state): State<AppState>,
    Json(body): Json<RecallBody>,
) -> Result<Json<Vec<AgentAtomView>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("recall_post", request_id=%request_id);
    state.status.touch_client_activity();
    match state
        .brain
        .recall_for_agent(
            &crate::project::resolve_project(body.project.as_deref()),
            &body.query,
            body.limit,
            body.include_quarantine,
        )
        .await
    {
        Ok(views) => Ok(Json(views)),
        Err(e) => Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
            &request_id,
        )),
    }
}

async fn search_get(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("search_get", request_id=%request_id);
    state.status.touch_client_activity();
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::Search);
    match state
        .brain
        .search_scoped_hub(
            &query.q,
            query.limit,
            query.include_quarantine,
            query.source.as_deref(),
            principal.team_id(),
        )
        .await
    {
        Ok(results) => {
            timer.success(results.len() as u64);
            Ok(Json(results))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct AskBody {
    question: String,
}

async fn ask_post(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    Json(body): Json<AskBody>,
) -> Result<Json<Answer>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("ask_post", request_id=%request_id);
    state.status.touch_client_activity();
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::Ask);
    match state
        .brain
        .ask_with_team(&body.question, principal.team_id())
        .await
    {
        Ok(answer) => {
            timer.success(answer.citations.len() as u64);
            Ok(Json(answer))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

async fn ask_get(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    Query(query): Query<AskQuery>,
) -> Result<Json<Answer>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("ask_get", request_id=%request_id);
    state.status.touch_client_activity();
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::Ask);
    match state
        .brain
        .ask_with_team(&query.question, principal.team_id())
        .await
    {
        Ok(answer) => {
            timer.success(answer.citations.len() as u64);
            Ok(Json(answer))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct CiteBody {
    source: String,
    source_id: String,
}

async fn cite_post(
    State(state): State<AppState>,
    Json(body): Json<CiteBody>,
) -> Result<Json<Option<Citation>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("cite_post", request_id=%request_id);
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::Cite);
    match state.brain.cite(&body.source, &body.source_id).await {
        Ok(citation) => {
            timer.success(u64::from(citation.is_some()));
            Ok(Json(citation))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct WhoKnowsBody {
    topic: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

async fn who_knows_post(
    State(state): State<AppState>,
    principal: MaybeHubPrincipal,
    Json(body): Json<WhoKnowsBody>,
) -> Result<Json<Vec<WhoKnowsEntry>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("who_knows_post", request_id=%request_id);
    let timer = TimedObserve::start(Arc::clone(&state.metrics), MetricOp::WhoKnows);
    match state
        .brain
        .who_knows_with_team(&body.topic, body.limit, principal.team_id())
        .await
    {
        Ok(entries) => {
            timer.success(entries.len() as u64);
            Ok(Json(entries))
        }
        Err(e) => {
            timer.failure();
            Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            ))
        }
    }
}

async fn api_open(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(file) = params.get("file") {
        if let Ok(path) = crate::security::resolve_allowed_path(file) {
            let _ = std::process::Command::new("open").arg(&path).status();
        }
    }
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::{Embedder, NullEmbedder};
    use crate::pipeline::IndexPipeline;
    use crate::rerank::NullReranker;
    use crate::store::{SqliteVecStore, Store};
    use crate::synthesize::ExtractiveSynthesizer;
    use crate::synthesize::Synthesizer;
    use crate::types::{SourceConfig, SourceKind};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tower::ServiceExt;

    static HTTP_FIXTURE_SEQ: AtomicU64 = AtomicU64::new(1);

    fn test_brain() -> BrainService {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-http-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let synth: Arc<dyn Synthesizer> = Arc::new(ExtractiveSynthesizer::new());
        BrainService::new(store, embedder, Arc::new(NullReranker::new()), synth)
    }

    #[tokio::test]
    async fn health_ok() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_ontology_returns_seeded_classes() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/ontology")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
        let entities = v["entities"].as_array().expect("entities");
        assert!(entities.len() >= 6);
        let ids: Vec<&str> = entities.iter().filter_map(|e| e["id"].as_str()).collect();
        assert!(ids.contains(&"class:memory"));
        assert!(ids.contains(&"class:note"));
        let links = v["links"].as_array().expect("links");
        assert!(links.len() >= 5);
    }

    #[tokio::test]
    async fn api_activity_empty_then_after_search() {
        let brain = Arc::new(test_brain());
        let app = router(AppState {
            brain: Arc::clone(&brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/activity?since=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["next_seq"], 0);
        assert_eq!(v["events"].as_array().unwrap().len(), 0);

        let _ = brain.search("anything", 3).await;

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/activity?since=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["next_seq"].as_u64().unwrap() >= 1);
        assert_eq!(v["events"][0]["tool"], "search");
        assert_eq!(v["events"][0]["query"], "anything");
    }

    #[tokio::test]
    async fn ask_empty_store_json() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ask")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"question":"anything?"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let answer: Answer = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(answer.confidence, 0.0);
        assert!(answer.citations.is_empty());
    }

    async fn fixture_brain_app() -> (Router, tempfile::TempDir) {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.path().join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline =
            IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        let source_name = format!(
            "notes-http-{}",
            HTTP_FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed)
        );
        connector
            .init(&SourceConfig {
                name: source_name.clone(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap();
        pipeline
            .index_connector(&source_name, &connector, true)
            .await
            .unwrap();

        let brain = BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let app = router(AppState {
            brain: Arc::new(brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        (app, db_dir)
    }

    #[tokio::test]
    async fn fixture_vault_search_ask_who_knows() {
        let (app, _db_dir) = fixture_brain_app().await;

        // POST /search
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/search")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"query":"KNOWN_PHRASE_KURULTAI_42","limit":5}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());

        // GET /search
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/search?q=KNOWN_PHRASE_KURULTAI_42&limit=5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());

        // POST /ask
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ask")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"question":"what is KNOWN_PHRASE_KURULTAI_42"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let answer: Answer = serde_json::from_slice(&bytes).unwrap();
        assert!(answer.confidence > 0.0);
        assert!(!answer.citations.is_empty());

        // GET /ask
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/ask?question=what%20is%20KNOWN_PHRASE_KURULTAI_42")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let answer: Answer = serde_json::from_slice(&bytes).unwrap();
        assert!(answer.confidence > 0.0);
        assert!(!answer.citations.is_empty());

        // POST /who_knows
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/who_knows")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"topic":"KNOWN_PHRASE_KURULTAI_42","limit":10}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let entries: Vec<WhoKnowsEntry> = serde_json::from_slice(&bytes).unwrap();
        assert!(!entries.is_empty());
    }

    #[tokio::test]
    async fn api_atoms_lists_and_limits_atoms() {
        let (app, _db_dir) = fixture_brain_app().await;

        // No limit: returns all fixture atoms with expected shape
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].rank, 0);
        assert!(results[0].matched_by.iter().any(|m| m == "list"));

        // Limit respected
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms?limit=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].rank, 0);

        // Invalid limit falls back to default (fixture has fewer than 500 atoms)
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms?limit=bad")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(!results.is_empty());

        // Invalid / symbolic limits fall back to the full-atom ceiling (500),
        // not a bulk dump — max-mode clients must use /api/graph instead.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms?limit=max")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let max_results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(max_results.len(), results.len());
    }

    #[test]
    fn parse_graph_limit_max_uses_ceiling() {
        assert_eq!(parse_graph_limit(Some("max")), 20_000);
        assert_eq!(parse_graph_limit(Some("all")), 20_000);
        assert_eq!(parse_graph_limit(None), 20_000);
        assert_eq!(parse_graph_limit(Some("2500")), 2500);
        assert_eq!(parse_graph_limit(Some("999999")), 50_000);
        assert_eq!(parse_atoms_limit(Some("max")), 500);
        assert_eq!(parse_atoms_limit(Some("9000")), 9000);
        assert_eq!(parse_atoms_limit(Some("12")), 12);
    }

    #[tokio::test]
    async fn api_atoms_refreshes_client_activity() {
        let status = Arc::new(crate::daemon::DaemonStatus::default());
        assert_eq!(
            status
                .last_client_activity_unix
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::clone(&status),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/atoms")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            status
                .last_client_activity_unix
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        );
    }

    /// Store stub: only `count` matters (always Err) for `/api/status` failure path.
    struct FailCountStore;

    #[async_trait::async_trait]
    impl Store for FailCountStore {
        async fn upsert(&self, _atom: &crate::types::KnowledgeAtom) -> crate::Result<()> {
            Ok(())
        }
        async fn upsert_batch(&self, _atoms: &[crate::types::KnowledgeAtom]) -> crate::Result<()> {
            Ok(())
        }
        async fn vector_search(
            &self,
            _query_embed: &[f32],
            _limit: usize,
            _filter: crate::store::SearchFilter,
        ) -> crate::Result<Vec<(crate::types::KnowledgeAtom, f64)>> {
            Ok(vec![])
        }
        async fn fts_search(
            &self,
            _query: &str,
            _limit: usize,
            _filter: crate::store::SearchFilter,
        ) -> crate::Result<Vec<(crate::types::KnowledgeAtom, f64)>> {
            Ok(vec![])
        }
        async fn fts_search_ids(
            &self,
            _query: &str,
            _limit: usize,
            _filter: crate::store::SearchFilter,
        ) -> crate::Result<Vec<(String, f64)>> {
            Ok(vec![])
        }
        async fn vector_search_ids(
            &self,
            _query_embed: &[f32],
            _limit: usize,
            _filter: crate::store::SearchFilter,
        ) -> crate::Result<Vec<(String, f64)>> {
            Ok(vec![])
        }
        async fn get_many(
            &self,
            _ids: &[String],
        ) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
            Ok(vec![])
        }
        async fn get(&self, _id: &str) -> crate::Result<Option<crate::types::KnowledgeAtom>> {
            Ok(None)
        }
        async fn delete_atom(&self, _id: &str) -> crate::Result<()> {
            Ok(())
        }
        async fn apply_auto_merge(
            &self,
            _survivor: &crate::types::KnowledgeAtom,
            _loser_id: &str,
            _audit_detail: &serde_json::Value,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn delete_source(&self, _source: &str) -> crate::Result<()> {
            Ok(())
        }
        async fn count(&self) -> crate::Result<u64> {
            Err(crate::KurultaiError::Store("count failed".into()))
        }
        async fn count_by_lane(&self, _lane: crate::types::TrustLane) -> crate::Result<u64> {
            Ok(0)
        }
        async fn list_atoms(
            &self,
            _limit: usize,
            _filter: crate::store::SearchFilter,
        ) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
            Ok(vec![])
        }
        async fn get_by_source_id(
            &self,
            _source: &str,
            _source_id: &str,
        ) -> crate::Result<Option<crate::types::KnowledgeAtom>> {
            Ok(None)
        }
        async fn get_by_chunk_meta(
            &self,
            _source: &str,
            _rel_path: &str,
            _chunk_index: u32,
        ) -> crate::Result<Option<crate::types::KnowledgeAtom>> {
            Ok(None)
        }
        async fn has_fresh_embedding(&self, _id: &str, _content_hash: &str) -> crate::Result<bool> {
            Ok(false)
        }
        async fn find_trusted_by_content_hash(
            &self,
            _content_hash: &str,
        ) -> crate::Result<Option<String>> {
            Ok(None)
        }
        async fn set_trust_lane(
            &self,
            _id: &str,
            _lane: crate::types::TrustLane,
            _quarantine_reason: Option<&str>,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn insert_quality_audit(
            &self,
            _action: &str,
            _atom_id: &str,
            _actor: &str,
            _detail: &serde_json::Value,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn insert_merge_candidate(
            &self,
            _atom_a: &str,
            _atom_b: &str,
            _reason: &str,
        ) -> crate::Result<bool> {
            Ok(false)
        }
        async fn count_merge_candidates_pending(&self) -> crate::Result<u64> {
            Ok(0)
        }
        async fn list_near_dupe_candidates(
            &self,
            _limit: usize,
        ) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
            Ok(vec![])
        }
        async fn touch_access(&self, _id: &str) -> crate::Result<()> {
            Ok(())
        }
        async fn count_by_tier(
            &self,
            _policy: crate::memory::TierPolicy,
        ) -> crate::Result<(u64, u64, u64)> {
            Ok((0, 0, 0))
        }
        async fn list_graph_nodes(
            &self,
            _tier: Option<crate::memory::MemoryTier>,
            _limit: usize,
            _filter: crate::store::SearchFilter,
            _policy: crate::memory::TierPolicy,
        ) -> crate::Result<Vec<crate::memory::GraphNode>> {
            Ok(vec![])
        }
        async fn record_ingestion_start(
            &self,
            _batch_id: &str,
            _source: &str,
            _file_path: &str,
        ) -> crate::Result<i64> {
            Ok(0)
        }
        async fn record_ingestion_finish(
            &self,
            _job_id: i64,
            _atoms_count: Option<i64>,
            _error_message: Option<&str>,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn list_pending_ingestion_jobs(
            &self,
        ) -> crate::Result<Vec<crate::store::IngestionJob>> {
            Ok(vec![])
        }
        async fn find_atoms_by_source_id_patterns(
            &self,
            _patterns: &[&str],
        ) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
            Ok(vec![])
        }
        async fn upsert_ontology_entity(
            &self,
            _e: &crate::types::OntologyEntity,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn get_ontology_entity(
            &self,
            _id: &str,
        ) -> crate::Result<Option<crate::types::OntologyEntity>> {
            Ok(None)
        }
        async fn list_ontology_entities(
            &self,
            _limit: usize,
        ) -> crate::Result<Vec<crate::types::OntologyEntity>> {
            Ok(vec![])
        }
        async fn upsert_ontology_link(&self, _l: &crate::types::OntologyLink) -> crate::Result<()> {
            Ok(())
        }
        async fn list_ontology_links(
            &self,
            _endpoint: Option<&str>,
        ) -> crate::Result<Vec<crate::types::OntologyLink>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn api_status_ok_includes_scheduler() {
        let status = Arc::new(crate::daemon::DaemonStatus::default());
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::clone(&status),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let cache = resp
            .headers()
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(cache, "no-store");
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
        assert!(v["atoms"].is_number());
        assert!(v["memory"]["hot"].is_number());
        assert!(v["memory"]["warm"].is_number());
        assert!(v["memory"]["cold"].is_number());
        assert!(v["scheduler"]["last_client_activity_unix"].is_number());
        let rid = v["request_id"].as_str().unwrap_or("");
        assert!(
            !rid.is_empty(),
            "success request_id must be present and non-empty"
        );
    }

    #[tokio::test]
    async fn api_status_store_failure_is_503() {
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let synth: Arc<dyn Synthesizer> = Arc::new(ExtractiveSynthesizer::new());
        let brain = BrainService::new(
            Arc::new(FailCountStore),
            embedder,
            Arc::new(NullReranker::new()),
            synth,
        );
        let app = router(AppState {
            brain: Arc::new(brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], false);
        assert!(v["atoms"].is_null());
        assert!(v["error"].as_str().unwrap_or("").contains("count failed"));
        let rid = v["request_id"].as_str().unwrap_or("");
        assert!(
            !rid.is_empty(),
            "error request_id must be present and non-empty"
        );
    }

    #[tokio::test]
    async fn search_and_ask_refresh_client_activity() {
        let status = Arc::new(crate::daemon::DaemonStatus::default());
        assert_eq!(
            status
                .last_client_activity_unix
                .load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::clone(&status),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=hello&limit=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let after_get = status
            .last_client_activity_unix
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_get > 0);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/search")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"query":"hello","limit":1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let after_post = status
            .last_client_activity_unix
            .load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_post > 0);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/ask")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"question":"anything?"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            status
                .last_client_activity_unix
                .load(std::sync::atomic::Ordering::Relaxed)
                >= after_post
        );
    }

    #[tokio::test]
    async fn promote_gate_audit_and_quarantine_exclusion() {
        use crate::quality::gate::{apply_gate, GateOutcome};
        use crate::types::{KnowledgeAtom, TrustLane};
        use chrono::Utc;

        let dir = std::env::temp_dir().join(format!(
            "kurultai-http-lanes-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let sqlite = Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap());
        let store: Arc<dyn Store> = Arc::clone(&sqlite) as Arc<dyn Store>;
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let synth: Arc<dyn Synthesizer> = Arc::new(ExtractiveSynthesizer::new());
        let brain = Arc::new(BrainService::new(
            Arc::clone(&store),
            embedder,
            Arc::new(NullReranker::new()),
            synth,
        ));

        let mut trusted = KnowledgeAtom {
            id: "http-t1".into(),
            source: "agent".into(),
            source_id: "/http-t1".into(),
            title: "Trusted Hit".into(),
            summary: "HTTPTRUSTTOKEN trusted summary".into(),
            content: "HTTPTRUSTTOKEN trusted content body".into(),
            tags: vec!["ops".into()],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            ..Default::default()
        };
        apply_gate(&mut trusted, GateOutcome::Trusted);

        let mut quarantine = KnowledgeAtom {
            id: "http-q1".into(),
            source: "agent".into(),
            source_id: "/http-q1".into(),
            title: "Quarantine Hit".into(),
            summary: "HTTPTRUSTTOKEN quarantine summary".into(),
            content: "HTTPTRUSTTOKEN quarantine content body with enough operational detail for promote after tags".into(),
            tags: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            ..Default::default()
        };
        apply_gate(
            &mut quarantine,
            GateOutcome::Quarantine {
                reason: "untagged".into(),
            },
        );

        store.upsert(&trusted).await.unwrap();
        store.upsert(&quarantine).await.unwrap();

        let app = router(AppState {
            brain: Arc::clone(&brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });

        // Default list excludes quarantine.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let listed: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(listed.iter().any(|r| r.atom.id == "http-t1"));
        assert!(!listed.iter().any(|r| r.atom.id == "http-q1"));

        // include_quarantine=true includes quarantine.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms?include_quarantine=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let listed: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(listed.iter().any(|r| r.atom.id == "http-q1"));

        // Default search excludes quarantine.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=HTTPTRUSTTOKEN&limit=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let hits: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(hits.iter().any(|r| r.atom.id == "http-t1"));
        assert!(!hits.iter().any(|r| r.atom.id == "http-q1"));

        // include_quarantine on search.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=HTTPTRUSTTOKEN&limit=10&include_quarantine=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let hits: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        assert!(hits.iter().any(|r| r.atom.id == "http-q1"));

        // Promote gate refuses untagged quarantine.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/promote")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"atom_id":"http-q1"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Fix tags, then promote succeeds + audit row.
        quarantine.tags = vec!["ops".into()];
        store.upsert(&quarantine).await.unwrap();
        store
            .set_trust_lane("http-q1", TrustLane::Quarantine, Some("untagged"))
            .await
            .unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/promote")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"atom_id":"http-q1","reason":"added tags via http test"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["atom_id"], "http-q1");

        let promoted = store.get("http-q1").await.unwrap().unwrap();
        assert_eq!(promoted.trust_lane, TrustLane::Trusted);

        let conn = rusqlite::Connection::open(sqlite.path()).unwrap();
        let audit_n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM quality_audit WHERE action = 'promote' AND atom_id = 'http-q1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(audit_n >= 1);
    }
    #[tokio::test]
    async fn api_graph_invalid_tier_includes_request_id() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/graph?tier=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let rid = v["request_id"].as_str().unwrap_or("");
        assert!(!rid.is_empty(), "request_id must be present and non-empty");
    }

    #[tokio::test]
    async fn api_metrics_prometheus_after_search() {
        let metrics = MetricsRegistry::shared();
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: Arc::clone(&metrics),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=hello&limit=3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("text/plain"), "content-type={ct}");
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(
            body.contains("kurultai_query_requests_total{op=\"search\"} 1"),
            "body={body}"
        );
        assert!(body.contains("kurultai_query_latency_ms_bucket"));
    }

    #[tokio::test]
    async fn api_status_includes_metrics_summary() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v["metrics"]["search"]["requests"].is_number());
        assert!(v["metrics"]["ask"]["requests"].is_number());
    }

    #[tokio::test]
    async fn api_touch_missing_atom_is_404() {
        let (app, _db_dir) = fixture_brain_app().await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/touch")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"atom_id":"does-not-exist"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], false);
        assert!(v["error"].as_str().unwrap_or("").contains("atom not found"));
        assert!(!v["request_id"].as_str().unwrap_or("").is_empty());
    }

    #[tokio::test]
    async fn api_touch_ok_is_lean_without_full_content() {
        let (app, _db_dir) = fixture_brain_app().await;
        let list = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/atoms?limit=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(list.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&bytes).unwrap();
        let atom_id = results[0].atom.id.clone();

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/touch")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"atom_id":"{atom_id}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["atom_id"], atom_id);
        assert!(v.get("atom").is_none(), "must not dump full atom payload");
        assert!(v.get("content").is_none());
        assert!(!v["request_id"].as_str().unwrap_or("").is_empty());
    }

    fn mcp_http_app(brain: BrainService, secret: &str) -> Router {
        let brain = Arc::new(brain);
        let state = AppState {
            brain: Arc::clone(&brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate::default(),
        };
        router(state).merge(mcp::routes(mcp::McpHttpState::new(
            brain,
            secret.to_string(),
        )))
    }

    #[tokio::test]
    async fn mcp_http_rejects_missing_bearer() {
        let app = mcp_http_app(test_brain(), "test-secret");
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn mcp_http_search_roundtrip_with_bearer() {
        let brain = {
            let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
            let db_dir = tempfile::tempdir().unwrap();
            let store = Arc::new(SqliteVecStore::open(db_dir.path().join("store.db"), 4).unwrap());
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
            let brain = BrainService::new(
                store,
                embedder,
                Arc::new(NullReranker::new()),
                Arc::new(ExtractiveSynthesizer::new()),
            );
            std::mem::forget(db_dir);
            brain
        };
        let app = mcp_http_app(brain, "s3cr3t");
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer s3cr3t")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search","arguments":{"query":"KNOWN_PHRASE_KURULTAI_42","limit":3}}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(v.get("result").is_some(), "{v}");
        let text = v["result"]["content"][0]["text"].as_str().unwrap_or("");
        assert!(
            text.contains("KNOWN_PHRASE_KURULTAI_42") || text.contains("notes"),
            "{text}"
        );
    }

    #[tokio::test]
    async fn mcp_http_rejects_remember_on_readonly_surface() {
        let app = mcp_http_app(test_brain(), "s3cr3t");
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer s3cr3t")
                    .body(Body::from(
                        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"remember","arguments":{"title":"x","summary":"y","tags":["t"]}}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let msg = v["error"]["message"].as_str().unwrap_or("");
        assert!(
            msg.contains("read-only") || msg.contains("not available"),
            "{v}"
        );
    }

    #[tokio::test]
    async fn mcp_sse_requires_auth() {
        let app = mcp_http_app(test_brain(), "s3cr3t");
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/mcp/sse")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn hub_api_key_rejects_missing_and_wrong_bearer() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate {
                auth: HubAuth::ApiKey,
                api_keys: vec!["hub-secret".into()],
                #[cfg(feature = "postgres")]
                key_store: None,
            },
        });
        let missing = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        let wrong = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .header("authorization", "Bearer nope")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
        let ok = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .header("authorization", "Bearer hub-secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn hub_api_key_leaves_health_open() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate {
                auth: HubAuth::ApiKey,
                api_keys: vec!["hub-secret".into()],
                #[cfg(feature = "postgres")]
                key_store: None,
            },
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn hub_api_key_blocks_unprefixed_query_aliases_without_bearer() {
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            hub: HubGate {
                auth: HubAuth::ApiKey,
                api_keys: vec!["hub-secret".into()],
                #[cfg(feature = "postgres")]
                key_store: None,
            },
        });
        for path in &["/search?q=test", "/ask?question=test"] {
            let resp = app
                .clone()
                .oneshot(Request::builder().uri(*path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                resp.status(),
                StatusCode::UNAUTHORIZED,
                "path {path} should require auth under HubAuth::ApiKey"
            );
        }

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/search?q=test")
                    .header("authorization", "Bearer hub-secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
