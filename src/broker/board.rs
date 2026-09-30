//! Board endpoints: `POST /board` mints a `sess_*` session key bound to a
//! stable `(agent, chat_id)` identity; `GET /board/whoami` echoes the resolved
//! identity (debug/verify); `POST /board/revoke` kills all keys for a chat.

use super::registry::{BoardRequest, Registry};
use crate::error::KurultaiError;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type SharedRegistry = Arc<Registry>;

pub fn routes(registry: SharedRegistry) -> Router {
    Router::new()
        .route("/board", post(board_post))
        .route("/board/whoami", get(whoami))
        .route("/board/revoke", post(revoke))
        .with_state(registry)
}

#[derive(Serialize)]
struct BoardResponse {
    session_key: String,
    agent: String,
    chat_id: String,
    chat_name: String,
    instance_id: String,
    device: String,
}

async fn board_post(
    State(reg): State<SharedRegistry>,
    Json(req): Json<BoardRequest>,
) -> Result<Json<BoardResponse>, BoardError> {
    let id = reg.board(&req).map_err(BoardError::internal)?;
    Ok(Json(BoardResponse {
        session_key: id.session_key,
        agent: id.agent,
        chat_id: id.chat_id,
        chat_name: id.chat_name,
        instance_id: id.instance_id,
        device: id.device,
    }))
}

#[derive(Serialize)]
struct WhoamiResponse {
    agent: String,
    chat_id: String,
    chat_name: String,
    instance_id: String,
    device: String,
}

/// Session key arrives as `Authorization: Bearer sess_*` or
/// `X-Kurultai-Session: sess_*`.
pub(crate) fn session_key_from(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers
        .get("x-kurultai-session")
        .and_then(|v| v.to_str().ok())
    {
        let v = v.trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").map(str::to_string))
}

async fn whoami(
    State(reg): State<SharedRegistry>,
    headers: HeaderMap,
) -> Result<Json<WhoamiResponse>, BoardError> {
    let key = session_key_from(&headers).ok_or(BoardError::unauthorized())?;
    let id = reg
        .resolve(&key)
        .map_err(BoardError::internal)?
        .ok_or(BoardError::unauthorized())?;
    Ok(Json(WhoamiResponse {
        agent: id.agent,
        chat_id: id.chat_id,
        chat_name: id.chat_name,
        instance_id: id.instance_id,
        device: id.device,
    }))
}

#[derive(Deserialize)]
struct RevokeRequest {
    agent: String,
    chat_id: String,
}

async fn revoke(
    State(reg): State<SharedRegistry>,
    Json(req): Json<RevokeRequest>,
) -> Result<Json<serde_json::Value>, BoardError> {
    let n = reg
        .revoke_chat(&req.agent, &req.chat_id)
        .map_err(BoardError::internal)?;
    Ok(Json(serde_json::json!({ "revoked": n })))
}

pub struct BoardError {
    status: StatusCode,
    message: String,
}

impl BoardError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "missing or invalid session key".into(),
        }
    }
    fn internal(e: KurultaiError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }
    }
}

impl IntoResponse for BoardError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}
