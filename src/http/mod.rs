//! Thin local HTTP API mirroring MCP read tools (Phase 3 / #7).
//!
//! Bind to localhost only — no auth in this slice.

use crate::daemon::DaemonStatus;
use crate::mcp::brain::BrainService;
use crate::mcp::interface::AgentRead;
use crate::synthesize::WhoKnowsEntry;
use crate::types::{Answer, Citation, SearchResult};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse};
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
        .route("/api/search", get(search_get).post(search_post))
        .route("/ui", get(ui_dashboard))
        .route("/ui/", get(ui_dashboard))
        .route("/search", get(search_get).post(search_post))
        .route("/ask", get(ask_get).post(ask_post))
        .route("/cite", post(cite_post))
        .route("/who_knows", post(who_knows_post))
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

async fn ui_dashboard() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(DASHBOARD_HTML),
    )
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>Kurultai · local brain</title>
  <style>
    :root { font-family: ui-sans-serif, system-ui, sans-serif; color: #e8e6e3; background: #12141a; }
    body { max-width: 52rem; margin: 2rem auto; padding: 0 1rem; }
    h1 { font-weight: 600; letter-spacing: -0.02em; }
    .card { background: #1c1f28; border: 1px solid #2a2f3a; border-radius: 12px; padding: 1rem 1.25rem; margin: 1rem 0; }
    input, button { font: inherit; padding: 0.5rem 0.75rem; border-radius: 8px; border: 1px solid #3a4150; background: #0f1116; color: inherit; }
    button { cursor: pointer; background: #3d5afe; border-color: #3d5afe; }
    pre { white-space: pre-wrap; word-break: break-word; font-size: 0.85rem; }
    .muted { color: #9aa3b2; font-size: 0.9rem; }
  </style>
</head>
<body>
  <h1>Kurultai</h1>
  <p class="muted">Local dev dashboard (#76) — localhost only, no auth.</p>
  <div class="card">
    <strong>Status</strong>
    <pre id="status">loading…</pre>
  </div>
  <div class="card">
    <strong>Search</strong>
    <form id="f" style="display:flex; gap:0.5rem; margin-top:0.5rem;">
      <input id="q" name="q" placeholder="query" style="flex:1" />
      <button type="submit">Search</button>
    </form>
    <pre id="results" class="muted">results appear here</pre>
  </div>
  <script>
    async function refreshStatus() {
      try {
        const r = await fetch('/api/status');
        const j = await r.json();
        document.getElementById('status').textContent = JSON.stringify(j, null, 2);
      } catch (e) {
        document.getElementById('status').textContent = String(e);
      }
    }
    document.getElementById('f').addEventListener('submit', async (ev) => {
      ev.preventDefault();
      const q = document.getElementById('q').value;
      const r = await fetch('/api/search?q=' + encodeURIComponent(q) + '&limit=5');
      const j = await r.json();
      document.getElementById('results').textContent = JSON.stringify(j, null, 2);
    });
    refreshStatus();
    setInterval(refreshStatus, 5000);
  </script>
</body>
</html>
"#;

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
