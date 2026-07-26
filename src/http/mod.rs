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

#[derive(Clone)]
struct AppState {
    brain: Arc<BrainService>,
    status: Arc<DaemonStatus>,
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
        .route("/api/activity", get(api_activity))
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
    match state.brain.atom_count().await {
        Ok(atoms) => Ok(Json(serde_json::json!({
            "ok": true,
            "service": "kurultai",
            "atoms": atoms,
            "scheduler": state.status.snapshot(),
        }))),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "ok": false,
                "service": "kurultai",
                "atoms": null,
                "error": e.to_string(),
                "scheduler": state.status.snapshot(),
            })),
        )),
    }
}

async fn api_atoms(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<crate::types::SearchResult>>, (StatusCode, String)> {
    state.status.touch_client_activity();
    let limit: usize = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(500)
        .min(500);
    state
        .brain
        .list_atoms(limit)
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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
}

async fn search_post(
    State(state): State<AppState>,
    Json(body): Json<SearchBody>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .search(&body.query, body.limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn search_get(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .search(&query.q, query.limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
struct AskBody {
    question: String,
}

async fn ask_post(
    State(state): State<AppState>,
    Json(body): Json<AskBody>,
) -> Result<Json<Answer>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .ask(&body.question)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn ask_get(
    State(state): State<AppState>,
    Query(query): Query<AskQuery>,
) -> Result<Json<Answer>, (StatusCode, String)> {
    state.status.touch_client_activity();
    state
        .brain
        .ask(&query.question)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[derive(Debug, Deserialize)]
struct CiteBody {
    source: String,
    source_id: String,
}

async fn cite_post(
    State(state): State<AppState>,
    Json(body): Json<CiteBody>,
) -> Result<Json<Option<Citation>>, (StatusCode, String)> {
    state
        .brain
        .cite(&body.source, &body.source_id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
) -> Result<Json<Vec<WhoKnowsEntry>>, (StatusCode, String)> {
    state
        .brain
        .who_knows(&body.topic, body.limit)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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
        ) -> crate::Result<Vec<(crate::types::KnowledgeAtom, f64)>> {
            Ok(vec![])
        }
        async fn fts_search(
            &self,
            _query: &str,
            _limit: usize,
        ) -> crate::Result<Vec<(crate::types::KnowledgeAtom, f64)>> {
            Ok(vec![])
        }
        async fn fts_search_ids(
            &self,
            _query: &str,
            _limit: usize,
        ) -> crate::Result<Vec<(String, f64)>> {
            Ok(vec![])
        }
        async fn vector_search_ids(
            &self,
            _query_embed: &[f32],
            _limit: usize,
        ) -> crate::Result<Vec<(String, f64)>> {
            Ok(vec![])
        }
        async fn get_many(
            &self,
            _ids: &[String],
        ) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
            Ok(vec![])
        }
        async fn delete_source(&self, _source: &str) -> crate::Result<()> {
            Ok(())
        }
        async fn count(&self) -> crate::Result<u64> {
            Err(crate::KurultaiError::Store("count failed".into()))
        }
        async fn list_atoms(&self, _limit: usize) -> crate::Result<Vec<crate::types::KnowledgeAtom>> {
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
        assert!(v["scheduler"]["last_client_activity_unix"].is_number());
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
}
