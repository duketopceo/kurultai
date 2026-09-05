//! Multi-agent message board REST (`/api/hey/...`).

use super::AppState;
use crate::error::KurultaiError;
use crate::hashutil::sha256_hex;
use crate::store::{AddReactionInput, Agent, Message, PostMessageInput, Thread};
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};

use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

const DEFAULT_THREAD: &str = "hey.md";
const DEFAULT_TURN_CAP: u32 = 10;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/hey/threads", get(list_threads).post(create_thread))
        .route(
            "/api/hey/threads/{id}/messages",
            get(list_messages).post(post_message),
        )
        .route("/api/hey/messages/{id}/react", post(react))
        .route("/api/hey/unread", get(unread))
}

#[derive(Debug, Serialize)]
struct ThreadDto {
    id: String,
    name: String,
    parent_thread_id: Option<String>,
    turn_cap: u32,
    turns_used: u32,
    created_at: String,
    updated_at: String,
}

impl From<Thread> for ThreadDto {
    fn from(t: Thread) -> Self {
        Self {
            id: t.id,
            name: t.name,
            parent_thread_id: t.parent_thread_id,
            turn_cap: t.turn_cap,
            turns_used: t.turns_used,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct MessageDto {
    id: String,
    thread_id: String,
    agent_id: String,
    parent_id: Option<String>,
    kind: String,
    content: String,
    request_reply: bool,
    turns_consumed: u32,
    created_at: String,
}

impl From<Message> for MessageDto {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            thread_id: m.thread_id,
            agent_id: m.agent_id,
            parent_id: m.parent_id,
            kind: m.kind.as_str().into(),
            content: m.content,
            request_reply: m.request_reply,
            turns_consumed: m.turns_consumed,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct LimitQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Deserialize)]
struct CreateThreadBody {
    #[serde(default = "default_thread_name")]
    name: String,
    parent_thread_id: Option<String>,
    turn_cap: Option<u32>,
}

fn default_thread_name() -> String {
    DEFAULT_THREAD.into()
}

#[derive(Debug, Deserialize)]
struct PostBody {
    content: String,
    parent_id: Option<String>,
    #[serde(default)]
    request_reply: bool,
    /// When omitted, posts to the path thread id; when set, may retarget by name.
    thread_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReactBody {
    emoji: String,
    thread_id: String,
}

#[derive(Debug, Deserialize)]
struct UnreadQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    /// ISO timestamp; when set, only messages with `created_at` > since.
    since: Option<String>,
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    if !value.to_ascii_lowercase().starts_with("bearer ") {
        return None;
    }
    let token = value["bearer ".len()..].trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

async fn require_agent(state: &AppState, headers: &HeaderMap) -> Result<Agent, StatusCode> {
    let token = extract_bearer(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let hash = sha256_hex(&token);
    state
        .brain
        .store()
        .resolve_agent_by_key_hash(&hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

fn map_store_err(e: KurultaiError) -> (StatusCode, String) {
    let msg = e.to_string();
    if msg.contains("turn cap") || msg.contains("turn_cap") {
        (StatusCode::CONFLICT, msg)
    } else if msg.contains("not found") {
        (StatusCode::NOT_FOUND, msg)
    } else if msg.contains("not implemented") {
        (StatusCode::NOT_IMPLEMENTED, msg)
    } else {
        (StatusCode::BAD_REQUEST, msg)
    }
}

async fn list_threads(
    State(state): State<AppState>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<ThreadDto>>, (StatusCode, String)> {
    let threads = state
        .brain
        .store()
        .list_threads(q.limit.clamp(1, 200))
        .await
        .map_err(map_store_err)?;
    Ok(Json(threads.into_iter().map(ThreadDto::from).collect()))
}

async fn create_thread(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateThreadBody>,
) -> Result<Json<ThreadDto>, (StatusCode, String)> {
    let _agent = require_agent(&state, &headers)
        .await
        .map_err(|s| (s, "agent bearer required".into()))?;
    let name = body.name.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name required".into()));
    }
    if let Some(existing) = state
        .brain
        .store()
        .get_thread_by_name(name)
        .await
        .map_err(map_store_err)?
    {
        return Ok(Json(ThreadDto::from(existing)));
    }
    let thread = state
        .brain
        .store()
        .create_thread(name, body.parent_thread_id.as_deref(), body.turn_cap)
        .await
        .map_err(map_store_err)?;
    Ok(Json(ThreadDto::from(thread)))
}

async fn ensure_default_thread(state: &AppState) -> Result<Thread, (StatusCode, String)> {
    if let Some(t) = state
        .brain
        .store()
        .get_thread_by_name(DEFAULT_THREAD)
        .await
        .map_err(map_store_err)?
    {
        return Ok(t);
    }
    state
        .brain
        .store()
        .create_thread(DEFAULT_THREAD, None, Some(DEFAULT_TURN_CAP))
        .await
        .map_err(map_store_err)
}

async fn list_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<MessageDto>>, (StatusCode, String)> {
    let thread_id = resolve_thread_id(&state, &id).await?;
    let messages = state
        .brain
        .store()
        .list_messages(&thread_id, q.limit.clamp(1, 500))
        .await
        .map_err(map_store_err)?;
    Ok(Json(messages.into_iter().map(MessageDto::from).collect()))
}

async fn resolve_thread_id(
    state: &AppState,
    id_or_name: &str,
) -> Result<String, (StatusCode, String)> {
    if let Some(t) = state
        .brain
        .store()
        .get_thread_by_name(id_or_name)
        .await
        .map_err(map_store_err)?
    {
        return Ok(t.id);
    }
    // Treat as raw id (list_messages will fail if missing).
    Ok(id_or_name.to_string())
}

async fn post_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<PostBody>,
) -> Result<Json<MessageDto>, (StatusCode, String)> {
    let agent = require_agent(&state, &headers)
        .await
        .map_err(|s| (s, "agent bearer required".into()))?;
    let content = body.content.trim();
    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "content required".into()));
    }
    let thread_id = if let Some(name) = body.thread_name.as_deref() {
        resolve_thread_id(&state, name).await?
    } else if id == DEFAULT_THREAD || id == "default" {
        ensure_default_thread(&state).await?.id
    } else {
        resolve_thread_id(&state, &id).await?
    };
    let msg = state
        .brain
        .store()
        .post_message(&PostMessageInput {
            thread_id,
            agent_id: agent.id,
            parent_id: body.parent_id,
            content: content.to_string(),
            request_reply: body.request_reply,
        })
        .await
        .map_err(map_store_err)?;
    Ok(Json(MessageDto::from(msg)))
}

async fn react(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ReactBody>,
) -> Result<Json<MessageDto>, (StatusCode, String)> {
    let agent = require_agent(&state, &headers)
        .await
        .map_err(|s| (s, "agent bearer required".into()))?;
    let emoji = body.emoji.trim();
    if emoji.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "emoji required".into()));
    }
    let msg = state
        .brain
        .store()
        .add_reaction(&AddReactionInput {
            thread_id: body.thread_id,
            agent_id: agent.id,
            message_id: id,
            emoji: emoji.to_string(),
        })
        .await
        .map_err(map_store_err)?;
    Ok(Json(MessageDto::from(msg)))
}

async fn unread(
    State(state): State<AppState>,
    Query(q): Query<UnreadQuery>,
) -> Result<Json<Vec<MessageDto>>, (StatusCode, String)> {
    let thread = ensure_default_thread(&state).await?;
    let mut messages = state
        .brain
        .store()
        .list_messages(&thread.id, q.limit.clamp(1, 200))
        .await
        .map_err(map_store_err)?;
    if let Some(since) = q.since.as_deref() {
        messages.retain(|m| m.created_at.as_str() > since);
    }
    Ok(Json(messages.into_iter().map(MessageDto::from).collect()))
}
