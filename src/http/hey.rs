//! Multi-agent message board REST (`/api/hey/...`).

use super::AppState;
use crate::error::KurultaiError;
use crate::hashutil::sha256_hex;
use crate::store::{AddReactionInput, Agent, Message, PostMessageInput, Thread};
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use std::net::SocketAddr;

use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

const DEFAULT_THREAD: &str = "hey.md";
const DEFAULT_TURN_CAP: u32 = 10;

/// Codename used when the verified human admin posts on the board.
const ADMIN_CODENAME: &str = "luke";

/// Env var: comma-separated emails allowed to write Hey as human admin.
pub const ENV_ADMIN_EMAILS: &str = "KURULTAI_ADMIN_EMAILS";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/hey/threads", get(list_threads).post(create_thread))
        .route(
            "/api/hey/threads/{id}/messages",
            get(list_messages).post(post_message),
        )
        .route(
            "/api/hey/messages/{id}",
            patch(update_message).delete(delete_message),
        )
        .route("/api/hey/messages/{id}/react", post(react))
        .route("/api/hey/unread", get(unread))
        .route("/api/hey/presence", get(presence))
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
    #[serde(skip_serializing_if = "String::is_empty")]
    agent_codename: String,
    parent_id: Option<String>,
    kind: String,
    content: String,
    request_reply: bool,
    turns_consumed: u32,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance_id: Option<String>,
}

impl From<Message> for MessageDto {
    fn from(m: Message) -> Self {
        Self {
            id: m.id,
            thread_id: m.thread_id,
            agent_id: m.agent_id,
            agent_codename: m.agent_codename,
            parent_id: m.parent_id,
            kind: m.kind.as_str().into(),
            content: m.content,
            request_reply: m.request_reply,
            turns_consumed: m.turns_consumed,
            created_at: m.created_at,
            repo: m.repo,
            instance_id: m.instance_id,
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
    /// Optional repo claim for WIP presence (`owner/repo` or short name).
    repo: Option<String>,
    /// Optional session/instance id (e.g. host) so concurrent same-codename agents differ.
    instance_id: Option<String>,
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

pub(crate) async fn require_agent(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Agent, StatusCode> {
    let token = extract_bearer(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    resolve_agent_token(state, &token).await
}

async fn resolve_agent_token(state: &AppState, token: &str) -> Result<Agent, StatusCode> {
    let hash = sha256_hex(token);
    state
        .brain
        .store()
        .resolve_agent_by_key_hash(&hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Who is writing to the board: a registered agent, or the verified human
/// admin (Cloudflare Access email ∈ `KURULTAI_ADMIN_EMAILS`, or the operator
/// `KURULTAI_ADMIN_TOKEN` bearer). Human writes post as codename `luke` and
/// can edit/delete any message; agents edit only their own.
#[derive(Debug)]
pub(crate) struct HeyPrincipal {
    agent: Agent,
    /// Verified human admin lane — bypasses message-ownership checks.
    admin: bool,
    /// Human identity marker recorded as `instance_id` on posts.
    human: Option<String>,
}

/// Emails allowed to write as admin, from `KURULTAI_ADMIN_EMAILS`
/// (comma-separated, case-insensitive).
fn admin_emails_from_env() -> Vec<String> {
    std::env::var(ENV_ADMIN_EMAILS)
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn email_in_admin_list(email: Option<&str>, admins: &[String]) -> bool {
    let Some(email) = email else { return false };
    let email = email.trim().to_ascii_lowercase();
    !email.is_empty() && admins.iter().any(|e| e == &email)
}

fn email_is_admin(email: Option<&str>) -> bool {
    email_in_admin_list(email, &admin_emails_from_env())
}

/// Resolve the `luke` agent row for human writes, registering it on first use.
async fn human_admin_agent(state: &AppState) -> Result<Agent, StatusCode> {
    let store = state.brain.store();
    if let Some(agent) = store
        .list_agents()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .find(|a| a.codename == ADMIN_CODENAME)
    {
        return Ok(agent);
    }
    let (id, _key) = store
        .register_agent(ADMIN_CODENAME)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Agent {
        id,
        codename: ADMIN_CODENAME.to_string(),
        created_at: String::new(),
    })
}

/// Agent bearer first; then the human admin lane (CF Access email or the
/// operator admin token). Returns 401 when neither authenticates.
async fn require_writer(
    state: &AppState,
    headers: &HeaderMap,
    peer: SocketAddr,
) -> Result<HeyPrincipal, StatusCode> {
    if let Some(token) = extract_bearer(headers) {
        if let Ok(agent) = resolve_agent_token(state, &token).await {
            return Ok(HeyPrincipal {
                agent,
                admin: false,
                human: None,
            });
        }
        if let Some(admin) = super::auth::resolve_admin_token() {
            if super::auth::token_accepted(&token, &[admin]) {
                let agent = human_admin_agent(state).await?;
                return Ok(HeyPrincipal {
                    agent,
                    admin: true,
                    human: Some("admin".into()),
                });
            }
        }
    }
    if let Some(cf) = &state.hub.cf_access {
        if let Some(identity) = cf.verify_headers(headers).await {
            if email_is_admin(identity.email.as_deref()) {
                let agent = human_admin_agent(state).await?;
                return Ok(HeyPrincipal {
                    agent,
                    admin: true,
                    human: identity.email,
                });
            }
        }
    }
    // Zero-friction solo path: when the daemon runs with no auth surface at all
    // (no hub API keys, no CF Access, no admin token) AND the request arrives on
    // loopback, the local operator is the only possible writer — post as the
    // human admin so the UI composer works without minting a key first. The
    // loopback check matters: `daemon --bind tailscale|all` with zero auth is a
    // documented config, and without it any tailnet/remote client could write
    // (and edit/delete others' messages) as `luke`.
    if peer.ip().is_loopback()
        && state.hub.auth == super::auth::HubAuth::None
        && state.hub.api_keys.is_empty()
        && state.hub.cf_access.is_none()
        && super::auth::resolve_admin_token().is_none()
    {
        let agent = human_admin_agent(state).await?;
        return Ok(HeyPrincipal {
            agent,
            admin: true,
            human: Some("local".into()),
        });
    }
    Err(StatusCode::UNAUTHORIZED)
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
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<CreateThreadBody>,
) -> Result<Json<ThreadDto>, (StatusCode, String)> {
    let _writer = require_writer(&state, &headers, peer)
        .await
        .map_err(|s| (s, "agent or admin auth required".into()))?;
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
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<PostBody>,
) -> Result<Json<MessageDto>, (StatusCode, String)> {
    let writer = require_writer(&state, &headers, peer)
        .await
        .map_err(|s| (s, "agent or admin auth required".into()))?;
    let agent = writer.agent;
    let content = body.content.trim();
    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "content required".into()));
    }
    let repo = normalize_claim_field(body.repo.as_deref(), 128);
    // Human admin posts carry their verified email as instance_id; agents
    // keep their claimed instance_id.
    let instance_id = writer
        .human
        .or_else(|| normalize_claim_field(body.instance_id.as_deref(), 64));
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
            repo,
            instance_id,
        })
        .await
        .map_err(map_store_err)?;
    Ok(Json(MessageDto::from(msg)))
}

fn normalize_claim_field(raw: Option<&str>, max: usize) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    let clipped: String = s.chars().take(max).collect();
    Some(clipped)
}

#[derive(Debug, Serialize)]
struct PresenceDto {
    agent_id: String,
    agent_codename: String,
    instance_id: Option<String>,
    repo: String,
    message_id: String,
    created_at: String,
    content_preview: String,
}

/// Latest repo claim per (agent_id, instance_id) from the default hey.md board.
async fn presence(
    State(state): State<AppState>,
    Query(q): Query<LimitQuery>,
) -> Result<Json<Vec<PresenceDto>>, (StatusCode, String)> {
    let thread = ensure_default_thread(&state).await?;
    let messages = state
        .brain
        .store()
        .list_messages(&thread.id, q.limit.clamp(1, 200).max(50))
        .await
        .map_err(map_store_err)?;
    let mut seen = std::collections::HashSet::<String>::new();
    let mut out = Vec::new();
    for m in messages {
        if m.kind != crate::store::MessageKind::Message {
            continue;
        }
        let Some(repo) = m.repo.as_deref().filter(|r| !r.is_empty()) else {
            continue;
        };
        let key = format!("{}|{}", m.agent_id, m.instance_id.as_deref().unwrap_or(""));
        if !seen.insert(key) {
            continue;
        }
        let preview: String = m.content.chars().take(120).collect();
        out.push(PresenceDto {
            agent_id: m.agent_id,
            agent_codename: m.agent_codename,
            instance_id: m.instance_id,
            repo: repo.to_string(),
            message_id: m.id,
            created_at: m.created_at,
            content_preview: preview,
        });
    }
    Ok(Json(out))
}

async fn react(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ReactBody>,
) -> Result<Json<MessageDto>, (StatusCode, String)> {
    let writer = require_writer(&state, &headers, peer)
        .await
        .map_err(|s| (s, "agent or admin auth required".into()))?;
    let emoji = body.emoji.trim();
    if emoji.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "emoji required".into()));
    }
    let msg = state
        .brain
        .store()
        .add_reaction(&AddReactionInput {
            thread_id: body.thread_id,
            agent_id: writer.agent.id,
            message_id: id,
            emoji: emoji.to_string(),
        })
        .await
        .map_err(map_store_err)?;
    Ok(Json(MessageDto::from(msg)))
}

#[derive(Debug, Deserialize)]
struct UpdateBody {
    content: String,
}

/// `PATCH /api/hey/messages/{id}` — edit content in place. Admin (verified
/// human) may edit any message; agents only their own. Never consumes turns.
async fn update_message(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<MessageDto>, (StatusCode, String)> {
    let writer = require_writer(&state, &headers, peer)
        .await
        .map_err(|s| (s, "agent or admin auth required".into()))?;
    let content = body.content.trim();
    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "content required".into()));
    }
    let existing = state
        .brain
        .store()
        .get_message(&id)
        .await
        .map_err(map_store_err)?
        .ok_or((StatusCode::NOT_FOUND, "message not found".into()))?;
    if !writer.admin && existing.agent_id != writer.agent.id {
        return Err((StatusCode::FORBIDDEN, "not the message author".into()));
    }
    let msg = state
        .brain
        .store()
        .update_message_content(&id, content)
        .await
        .map_err(map_store_err)?;
    Ok(Json(MessageDto::from(msg)))
}

/// `DELETE /api/hey/messages/{id}` — remove a message and its reactions.
/// Admin may delete any; agents only their own.
async fn delete_message(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let writer = require_writer(&state, &headers, peer)
        .await
        .map_err(|s| (s, "agent or admin auth required".into()))?;
    let existing = state
        .brain
        .store()
        .get_message(&id)
        .await
        .map_err(map_store_err)?
        .ok_or((StatusCode::NOT_FOUND, "message not found".into()))?;
    if !writer.admin && existing.agent_id != writer.agent.id {
        return Err((StatusCode::FORBIDDEN, "not the message author".into()));
    }
    state
        .brain
        .store()
        .delete_message(&id)
        .await
        .map_err(map_store_err)?;
    Ok(StatusCode::NO_CONTENT)
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

#[cfg(test)]
mod tests {
    use super::email_in_admin_list;

    #[test]
    fn admin_email_matching_is_case_insensitive_and_exact() {
        let admins = vec!["duketopceo@gmail.com".to_string()];
        assert!(email_in_admin_list(Some("DukeTopCEO@gmail.com"), &admins));
        assert!(email_in_admin_list(Some(" duketopceo@gmail.com "), &admins));
        assert!(!email_in_admin_list(
            Some("duketopceo+alt@gmail.com"),
            &admins
        ));
        assert!(!email_in_admin_list(Some("mallory@example.com"), &admins));
        assert!(!email_in_admin_list(None, &admins));
        assert!(!email_in_admin_list(Some("duketopceo@gmail.com"), &[]));
    }
}
