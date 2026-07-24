//! Thin local HTTP API mirroring MCP read tools (Phase 3 / #7).
//!
//! Bind to localhost only — no auth in this slice.

use crate::mcp::brain::BrainService;
use crate::mcp::interface::AgentRead;
use crate::synthesize::WhoKnowsEntry;
use crate::types::{Answer, Citation, SearchResult};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    brain: Arc<BrainService>,
}

/// Serve search/ask/cite/who_knows on `127.0.0.1:port` until cancelled.
pub async fn serve(brain: BrainService, port: u16) -> crate::Result<()> {
    let state = AppState {
        brain: Arc::new(brain),
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
        .route("/search", post(search_post))
        .route("/ask", post(ask_post))
        .route("/cite", post(cite_post))
        .route("/who_knows", post(who_knows_post))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "service": "kurultai" }))
}

fn default_limit() -> usize {
    10
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
    state
        .brain
        .search(&body.query, body.limit)
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
    state
        .brain
        .ask(&body.question)
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
    use crate::embed::{Embedder, NullEmbedder};
    use crate::rerank::NullReranker;
    use crate::store::SqliteVecStore;
    use crate::synthesize::ExtractiveSynthesizer;
    use crate::synthesize::Synthesizer;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

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
}
