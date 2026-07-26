//! Thin local HTTP API mirroring MCP read tools (Phase 3 / #7).
//!
//! Bind to localhost only — no auth in this slice.
//! Brain UI: single surface at `GET /ui` (embedded `ui/` assets — see `ui` module).

mod ui;

use crate::daemon::DaemonStatus;
use crate::mcp::brain::BrainService;
use crate::mcp::interface::AgentRead;
use crate::synthesize::WhoKnowsEntry;
use crate::types::{Answer, Citation, SearchResult};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
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
    let state = AppState {
        brain: Arc::new(brain),
        status,
    };
    let app = router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| crate::KurultaiError::Other(anyhow::anyhow!("bind {addr}: {e}")))?;
    tracing::info!(%addr, "http daemon listening (localhost only)");
    axum::serve(listener, app)
        .await
        .map_err(|e| crate::KurultaiError::Other(anyhow::anyhow!("http serve: {e}")))?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/status", get(api_status))
        .route("/api/atoms", get(api_atoms))
        .route("/api/graph", get(api_graph))
        .route("/api/touch", post(api_touch))
        .route("/api/activity", get(api_activity))
        .route("/api/promote", post(api_promote))
        .route("/api/search", get(search_get).post(search_post))
        .route("/api/ask", get(ask_get).post(ask_post))
        .route("/api/open", get(api_open))
        .route("/search", get(search_get).post(search_post))
        .route("/ask", get(ask_get).post(ask_post))
        .route("/cite", post(cite_post))
        .route("/who_knows", post(who_knows_post))
        .merge(ui::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "service": "kurultai" }))
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
                Ok(Json(serde_json::json!({
                    "ok": true,
                    "service": "kurultai",
                    "atoms": atoms,
                    "request_id": &request_id,
                    "brain": {
                        "trusted_count": trusted,
                        "quarantine_count": quarantine,
                        "merge_candidates_pending": merge_pending,
                    },
                    "memory": memory,
                    "scheduler": state.status.snapshot(),
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
                "atoms": null,
                "error": e.to_string(),
                "request_id": &request_id,
                "scheduler": state.status.snapshot(),
            })),
        )),
    }
}

async fn api_atoms(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::types::SearchResult>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_atoms", request_id=%request_id);
    state.status.touch_client_activity();
    let limit: usize = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(500)
        .min(500);
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

async fn api_graph(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_graph", request_id=%request_id);
    state.status.touch_client_activity();
    let limit: usize = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000)
        .min(50_000);
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
    let include_quarantine = params
        .get("include_quarantine")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let nodes = state
        .brain
        .list_graph_nodes(tier, limit, include_quarantine)
        .await
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "request_id": &request_id,
        "tier": tier.map(|t| t.as_str()),
        "count": nodes.len(),
        "nodes": nodes,
    })))
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
        Ok(Some(atom)) => Ok(Json(serde_json::json!({
            "ok": true,
            "request_id": &request_id,
            "atom": atom,
        }))),
        Ok(None) => Err(json_error(
            StatusCode::NOT_FOUND,
            "atom not found",
            &request_id,
        )),
        Err(e) => Err(json_error(
            StatusCode::BAD_REQUEST,
            e.to_string(),
            &request_id,
        )),
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
    Json(body): Json<PromoteBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("api_promote", request_id=%request_id);
    state.status.touch_client_activity();
    match state
        .brain
        .promote(&body.atom_id, "http", body.reason.as_deref())
        .await
    {
        Ok(res) => Ok(Json(serde_json::json!({
            "ok": true,
            "request_id": &request_id,
            "atom_id": res.atom_id,
            "actor": res.actor,
        }))),
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
}

async fn search_post(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("search_post", request_id=%request_id);
    state.status.touch_client_activity();
    state
        .brain
        .search_filtered(&body.query, body.limit, body.include_quarantine)
        .await
        .map(Json)
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
}

async fn search_get(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("search_get", request_id=%request_id);
    state.status.touch_client_activity();
    state
        .brain
        .search_filtered(&query.q, query.limit, query.include_quarantine)
        .await
        .map(Json)
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
}

#[derive(Debug, Deserialize)]
struct AskBody {
    question: String,
}

async fn ask_post(
    State(state): State<AppState>,
    Json(body): Json<AskBody>,
) -> Result<Json<Answer>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("ask_post", request_id=%request_id);
    state.status.touch_client_activity();
    state
        .brain
        .ask(&body.question)
        .await
        .map(Json)
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
}

async fn ask_get(
    State(state): State<AppState>,
    Query(query): Query<AskQuery>,
) -> Result<Json<Answer>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("ask_get", request_id=%request_id);
    state.status.touch_client_activity();
    state
        .brain
        .ask(&query.question)
        .await
        .map(Json)
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
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
    state
        .brain
        .cite(&body.source, &body.source_id)
        .await
        .map(Json)
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
}

#[derive(Debug, Deserialize)]
struct WhoKnowsBody {
    topic: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

async fn who_knows_post(
    State(state): State<AppState>,
    Json(body): Json<WhoKnowsBody>,
) -> Result<Json<Vec<WhoKnowsEntry>>, (StatusCode, Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4().to_string();
    let _span = tracing::info_span!("who_knows_post", request_id=%request_id);
    state
        .brain
        .who_knows(&body.topic, body.limit)
        .await
        .map(Json)
        .map_err(|e| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
                &request_id,
            )
        })
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
    async fn api_activity_empty_then_after_search() {
        let brain = Arc::new(test_brain());
        let app = router(AppState {
            brain: Arc::clone(&brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
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
    }

    #[tokio::test]
    async fn api_status_ok_includes_scheduler() {
        let status = Arc::new(crate::daemon::DaemonStatus::default());
        let app = router(AppState {
            brain: Arc::new(test_brain()),
            status: Arc::clone(&status),
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
        assert_eq!(v["ok"], true);
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
            content: "HTTPTRUSTTOKEN quarantine content body".into(),
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
}
