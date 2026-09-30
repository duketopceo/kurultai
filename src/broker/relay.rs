//! `POST /mcp` relay: agent JSON-RPC → upstream `/mcp` with the device's seat
//! token plus `X-Kurultai-{agent,chat,session,device}` stamps resolved from the
//! caller's `sess_*` key. The upstream token never leaves the broker.

use super::board::{session_key_from, SharedRegistry};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct RelayState {
    pub registry: SharedRegistry,
    pub upstream_url: Arc<String>,
    pub upstream_token: crate::security::SecretString,
    pub client: reqwest::Client,
}

pub fn routes(state: RelayState) -> Router {
    Router::new()
        .route("/mcp", post(mcp_relay))
        .with_state(state)
}

async fn mcp_relay(
    State(st): State<RelayState>,
    headers: HeaderMap,
    Json(msg): Json<Value>,
) -> Result<Response, RelayError> {
    let key = session_key_from(&headers).ok_or(RelayError::unauthorized())?;
    let id = st
        .registry
        .resolve(&key)
        .map_err(|e| RelayError::internal(e.to_string()))?
        .ok_or(RelayError::unauthorized())?;

    let res = st
        .client
        .post(format!("{}/mcp", st.upstream_url))
        .bearer_auth(st.upstream_token.expose())
        .header("x-kurultai-agent", &id.agent)
        .header("x-kurultai-chat", &id.chat_id)
        .header("x-kurultai-session", &key)
        .header("x-kurultai-device", &id.device)
        .json(&msg)
        .send()
        .await
        .map_err(|e| RelayError::upstream(format!("upstream unreachable: {e}")))?;

    let status = res.status();
    let body: Value = res.json().await.unwrap_or_else(|_| Value::Null);
    if body.is_null() {
        return Ok(status.into_response());
    }
    Ok((status, Json(body)).into_response())
}

pub struct RelayError {
    status: StatusCode,
    message: String,
}

impl RelayError {
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "missing or invalid session key".into(),
        }
    }
    fn upstream(m: String) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message: m,
        }
    }
    fn internal(m: String) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: m,
        }
    }
}

impl IntoResponse for RelayError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}
