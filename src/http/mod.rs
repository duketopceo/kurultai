//! Local HTTP daemon — Phase 3 / #7 WO2.
//!
//! Mirrors MCP read tools for curl/scripts. Default bind is loopback.

use crate::error::{KurultaiError, Result};
use crate::mcp::interface::{AgentRead, AgentWrite};
use crate::mcp::BrainService;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

/// Build the HTTP router (also used by tests).
pub fn router(brain: Arc<BrainService>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/search", get(search_get).post(search_post))
        .route("/v1/cite", get(cite_get))
        .route("/v1/ask", post(ask_post))
        .route("/v1/remember", post(remember_post))
        .with_state(brain)
}

/// Bind and serve until the process is stopped.
pub async fn serve(brain: Arc<BrainService>, addr: SocketAddr) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("bind {addr}: {e}")))?;
    serve_listener(brain, listener).await
}

/// Serve on an already-bound listener (tests use ephemeral ports).
pub async fn serve_listener(
    brain: Arc<BrainService>,
    listener: tokio::net::TcpListener,
) -> Result<()> {
    let addr = listener
        .local_addr()
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("local_addr: {e}")))?;
    let app = router(brain);
    tracing::info!(%addr, "HTTP daemon listening");
    axum::serve(listener, app)
        .await
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("serve: {e}")))?;
    Ok(())
}

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

async fn health() -> Json<HealthBody> {
    Json(HealthBody { status: "ok" })
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    query: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Deserialize)]
struct SearchBody {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

async fn search_get(
    State(brain): State<Arc<BrainService>>,
    Query(params): Query<SearchQuery>,
) -> ApiResult<impl IntoResponse> {
    let q = params.q.or(params.query).unwrap_or_default();
    let views = brain.search_views(&q, params.limit).await?;
    Ok(Json(views))
}

async fn search_post(
    State(brain): State<Arc<BrainService>>,
    Json(body): Json<SearchBody>,
) -> ApiResult<impl IntoResponse> {
    let views = brain.search_views(&body.query, body.limit).await?;
    Ok(Json(views))
}

#[derive(Deserialize)]
struct CiteQuery {
    source: String,
    source_id: String,
}

async fn cite_get(
    State(brain): State<Arc<BrainService>>,
    Query(params): Query<CiteQuery>,
) -> ApiResult<impl IntoResponse> {
    match brain.cite(&params.source, &params.source_id).await? {
        Some(c) => Ok((StatusCode::OK, Json(c)).into_response()),
        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("No atom for {}/{}", params.source, params.source_id),
        }),
    }
}

#[derive(Deserialize)]
struct AskBody {
    question: String,
}

async fn ask_post(
    State(brain): State<Arc<BrainService>>,
    Json(body): Json<AskBody>,
) -> ApiResult<impl IntoResponse> {
    let answer = brain.ask(&body.question).await?;
    Ok(Json(answer))
}

#[derive(Deserialize)]
struct RememberBody {
    title: String,
    summary: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Serialize)]
struct RememberResponse {
    id: String,
}

async fn remember_post(
    State(brain): State<Arc<BrainService>>,
    Json(body): Json<RememberBody>,
) -> ApiResult<impl IntoResponse> {
    let id = brain
        .remember(&body.title, &body.summary, &body.tags, &[])
        .await?;
    Ok(Json(RememberResponse { id }))
}

type ApiResult<T> = std::result::Result<T, ApiError>;

struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<KurultaiError> for ApiError {
    fn from(err: KurultaiError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::NullEmbedder;
    use crate::rerank::NullReranker;
    use crate::store::{SqliteVecStore, Store};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn empty_brain() -> Arc<BrainService> {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        // Leak tempdir for unit test process lifetime.
        std::mem::forget(dir);
        Arc::new(BrainService::new(
            store as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
        ))
    }

    #[tokio::test]
    async fn health_ok() {
        let app = router(empty_brain());
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn cite_miss_404() {
        let app = router(empty_brain());
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/v1/cite?source=notes&source_id=missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
