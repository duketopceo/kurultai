//! `kurultai connect` — RFC 8628-style device authorization for agent onboarding.
//!
//! CLI: `POST /api/device/code` → prints a `user_code` + `verify_url`, then polls
//! `POST /api/device/token` until the human approves. Human: opens `/connect`,
//! which is gated by Cloudflare Access (`HubGate.cf_access`) when configured,
//! else a hub API-key bearer, else loopback-only for solo daemons. Approval
//! mints a seat-scoped key for `(codename, instance_id)` via
//! `Store::issue_agent_seat_token` — one agents row per codename, seats
//! distinguished by `instance_id` (never `codename-2`).

use super::auth::HubAuth;
use super::device_auth::human_base_url;
use super::AppState;
use crate::hashutil::sha256_hex;
use crate::store::DeviceFlow;
use axum::extract::{ConnectInfo, Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub(crate) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/device/code", post(device_code_post))
        .route("/api/device/token", post(device_token_post))
        .route("/connect", get(connect_page_get).post(connect_page_post))
        .with_state(state)
}

const DEFAULT_CLIENT_ID: &str = "kurultai-cli";
const DEFAULT_EXPIRES_IN: u64 = 600; // 10 minutes
const DEFAULT_INTERVAL: u64 = 5;
/// Codenames reserved for human/admin identities — a device flow must never
/// mint a seat key that Hey PATCH/DELETE would treat as the admin identity.
pub(crate) const RESERVED_CODENAMES: &[&str] = &["luke"];

/// `KURULTAI_ADMIN_EMAILS` — comma-separated allowlist for `/connect`
/// approvers on hub daemons (U3/KTD4). The Access application admits any
/// org member; approval mints a team-bound credential, so hub mode
/// requires a verified Access identity AND list membership.
pub(crate) const ENV_ADMIN_EMAILS: &str = "KURULTAI_ADMIN_EMAILS";

fn admin_emails() -> Vec<String> {
    std::env::var(ENV_ADMIN_EMAILS)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn admin_email_allowed(email: &str) -> bool {
    let list = admin_emails();
    !list.is_empty() && list.iter().any(|e| e == &email.trim().to_ascii_lowercase())
}

/// The hub key store when this daemon runs the Postgres hub lane (U3) —
/// gates the team picker + mint-at-approve path. `None` on the solo lane.
#[cfg(feature = "postgres")]
fn hub_keys(state: &AppState) -> Option<std::sync::Arc<crate::hub::HubKeyStore>> {
    state.hub.key_store.clone()
}

/// Distinct existing teams for the `/connect` picker (server-rendered —
/// the roster must never reach the unauthenticated `/api/device/code`
/// response, KTD4). Empty on the solo lane.
async fn hub_teams(state: &AppState) -> Vec<String> {
    #[cfg(feature = "postgres")]
    {
        if let Some(keys) = hub_keys(state) {
            if let Ok(teams) = keys.list_teams().await {
                return teams;
            }
        }
    }
    let _ = state;
    Vec::new()
}

/// Whether this daemon mints team-bound hub keys — the `/connect` page
/// renders the team picker and approve mints at decision time. `false`
/// keeps the solo seat-token path byte-for-byte identical.
#[cfg(feature = "postgres")]
fn hub_lane(state: &AppState) -> bool {
    state.hub.key_store.is_some()
}

/// Solo lane — no team picker, no mint-at-approve.
#[cfg(not(feature = "postgres"))]
fn hub_lane(_state: &AppState) -> bool {
    false
}

/// `hub:` agent id minted into `hub_api_keys.agent_id` — `codename@seat`
/// when a seat is requested, else the bare codename (KTD4: "codename+seat").
fn hub_agent_label(codename: &str, instance_id: &str) -> String {
    if instance_id.is_empty() {
        codename.to_string()
    } else {
        format!("{codename}@{instance_id}")
    }
}

#[derive(Debug, Deserialize)]
struct DeviceCodeRequest {
    #[serde(default)]
    codename: String,
    #[serde(default)]
    instance_id: String,
    #[serde(default)]
    client_id: String,
    /// `--team` prefill hint for the hub picker — never authoritative;
    /// the approver's choice wins (KTD4).
    #[serde(default)]
    team: String,
}

#[derive(Debug, Serialize)]
struct DeviceCodeResponse {
    /// Secret the CLI polls with (`device_code`).
    code: String,
    /// Short human code shown on the `/connect` page.
    user_code: String,
    verify_url: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenRequest {
    code: String,
}

#[derive(Debug, Serialize)]
struct DeviceTokenResponse {
    agent_key: String,
    codename: String,
    instance_id: String,
    /// Team scope the approver bound the minted hub key to (hub lane only).
    #[serde(skip_serializing_if = "Option::is_none")]
    team_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConnectQuery {
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConnectForm {
    code: String,
    #[serde(default)]
    action: String,
    /// Approver's team pick (hub lane). Required on hub approve; ignored
    /// on the solo lane where no picker renders.
    #[serde(default)]
    team: String,
}

/// What a token poll should do next, derived from flow status + expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PollDecision {
    Pending,
    Denied,
    Expired,
    Approved,
}

fn poll_decision(flow: &DeviceFlow, now: DateTime<Utc>) -> PollDecision {
    let expired = match DateTime::parse_from_rfc3339(&flow.expires_at) {
        Ok(expires) => now > expires.with_timezone(&Utc),
        Err(_) => true,
    };
    if expired {
        return PollDecision::Expired;
    }
    match flow.status.as_str() {
        "approved" => PollDecision::Approved,
        "pending" => PollDecision::Pending,
        _ => PollDecision::Denied,
    }
}

/// Who is allowed to approve a `/connect` request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApproverGate {
    /// Cloudflare Access configured — require a verified Access JWT.
    CfAccess,
    /// Hub API-key auth configured — require a valid hub/admin bearer.
    Hub,
    /// Nothing configured (default solo daemon) — loopback peers only.
    Loopback,
}

fn approver_gate(cf_configured: bool, auth: HubAuth) -> ApproverGate {
    if cf_configured {
        ApproverGate::CfAccess
    } else if auth == HubAuth::ApiKey {
        ApproverGate::Hub
    } else {
        ApproverGate::Loopback
    }
}

/// Resolve the approving human identity for a `/connect` request, or a 401.
async fn approver_identity(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    peer: Option<SocketAddr>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    match approver_gate(state.hub.cf_access.is_some(), state.hub.auth) {
        ApproverGate::CfAccess => {
            let cf = state.hub.cf_access.as_ref().expect("gate checked");
            if let Some(identity) = cf.verify_headers(headers).await {
                let email = identity
                    .email
                    .unwrap_or_else(|| format!("cf:{}", identity.sub));
                // U3/KTD4: on a hub daemon the approver must also be on the
                // `KURULTAI_ADMIN_EMAILS` allowlist — the Access app admits
                // every org member, and approving mints a team-bound
                // credential.
                if crate::features::enabled("hub") && !admin_email_allowed(&email) {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(
                            serde_json::json!({"ok": false, "error": "not on the /connect admin allowlist (KURULTAI_ADMIN_EMAILS)"}),
                        ),
                    ));
                }
                Ok(email)
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(
                        serde_json::json!({"ok": false, "error": "sign in via Cloudflare Access to approve agent connections"}),
                    ),
                ))
            }
        }
        ApproverGate::Hub => {
            if let Some(token) = super::auth::extract_bearer(headers) {
                if super::auth::token_accepted(&token, &state.hub.api_keys) {
                    return Ok("hub-api-key".to_string());
                }
                if let Some(admin) = super::auth::resolve_admin_token() {
                    if super::auth::token_accepted(&token, &[admin]) {
                        return Ok("admin-token".to_string());
                    }
                }
            }
            Err((
                StatusCode::UNAUTHORIZED,
                Json(
                    serde_json::json!({"ok": false, "error": "hub API key required to approve agent connections"}),
                ),
            ))
        }
        ApproverGate::Loopback => {
            // A reverse proxy / tunnel sidecar makes remote clients appear
            // loopback — refuse the loopback approver when forwarding headers
            // show the request came through a proxy.
            let proxied = [
                "x-forwarded-for",
                "x-real-ip",
                "cf-connecting-ip",
                "forwarded",
            ]
            .iter()
            .any(|h| headers.contains_key(*h));
            if !proxied && peer.map(|p| p.ip().is_loopback()).unwrap_or(false) {
                Ok("loopback".to_string())
            } else {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(
                        serde_json::json!({"ok": false, "error": "no human auth configured — /connect approvals are loopback-only"}),
                    ),
                ))
            }
        }
    }
}

async fn device_code_post(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<DeviceCodeRequest>,
) -> Response {
    let codename = if body.codename.trim().is_empty() {
        "anonymous".to_string()
    } else {
        body.codename.trim().to_string()
    };
    if RESERVED_CODENAMES
        .iter()
        .any(|r| codename.eq_ignore_ascii_case(r))
    {
        return json_error(
            StatusCode::BAD_REQUEST,
            "codename is reserved for a human admin identity",
        );
    }
    let client_id = if body.client_id.trim().is_empty() {
        DEFAULT_CLIENT_ID.to_string()
    } else {
        body.client_id.trim().to_string()
    };
    let instance_id = body.instance_id.trim().to_string();
    // `--team` is a picker prefill hint only — validated to the same shape
    // mint enforces, but never authoritative (KTD4).
    let team_hint = body.team.trim().to_string();
    if !team_hint.is_empty() {
        if let Err(e) = crate::store::scope::validate_team_id(&team_hint) {
            return json_error(StatusCode::BAD_REQUEST, &e);
        }
    }

    let flow = match state
        .brain
        .store()
        .create_device_flow_with_hint(
            &codename,
            &client_id,
            &instance_id,
            if team_hint.is_empty() {
                None
            } else {
                Some(team_hint.as_str())
            },
            DEFAULT_EXPIRES_IN,
        )
        .await
    {
        Ok(f) => f,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("device flow creation failed: {e}"),
            )
        }
    };

    let verify_url = format!(
        "{}/connect?code={}",
        human_base_url(&headers),
        flow.user_code
    );

    Json(DeviceCodeResponse {
        code: flow.device_code,
        user_code: flow.user_code,
        verify_url,
        expires_in: DEFAULT_EXPIRES_IN,
        interval: DEFAULT_INTERVAL,
    })
    .into_response()
}

async fn device_token_post(
    State(state): State<AppState>,
    Json(body): Json<DeviceTokenRequest>,
) -> Response {
    if body.code.trim().is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "missing code");
    }
    let flow = match state
        .brain
        .store()
        .get_device_flow_by_device_code(body.code.trim())
        .await
    {
        Ok(Some(f)) => f,
        Ok(None) => return json_error(StatusCode::GONE, "unknown or expired code"),
        Err(e) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    match poll_decision(&flow, Utc::now()) {
        PollDecision::Expired => json_error(
            StatusCode::GONE,
            "code expired — run `kurultai connect` again",
        ),
        PollDecision::Pending => {
            json_error(StatusCode::PRECONDITION_REQUIRED, "authorization_pending")
        }
        PollDecision::Denied => json_error(StatusCode::FORBIDDEN, "access_denied"),
        PollDecision::Approved => {
            // Hub lane (U3/KTD4): the key was minted at approve and sits on
            // the flow — hand it out exactly once via the atomic exchange.
            if let Some(key) = flow.issued_key.clone() {
                let agent_label = hub_agent_label(&flow.codename, &flow.instance_id);
                let token_hash = sha256_hex(&key);
                if let Err(e) = state
                    .brain
                    .store()
                    .exchange_device_flow(&flow.device_code, &token_hash, &agent_label)
                    .await
                {
                    tracing::warn!(error = %e, "hub device exchange failed");
                    return json_error(StatusCode::FORBIDDEN, "access_denied");
                }
                return Json(DeviceTokenResponse {
                    agent_key: key,
                    codename: flow.codename.clone(),
                    instance_id: flow.instance_id.clone(),
                    team_id: flow.team_id.clone(),
                })
                .into_response();
            }

            let approver = flow.approved_by.as_deref().unwrap_or("unknown");
            let (agent, token) = match state
                .brain
                .store()
                .issue_agent_seat_token(&flow.codename, &flow.instance_id, approver)
                .await
            {
                Ok(pair) => pair,
                Err(e) => {
                    return json_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("agent seat token issue failed: {e}"),
                    )
                }
            };
            let token_hash = sha256_hex(&token);
            if let Err(e) = state
                .brain
                .store()
                .exchange_device_flow(&flow.device_code, &token_hash, &agent.id)
                .await
            {
                return json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("device flow exchange failed: {e}"),
                );
            }
            Json(DeviceTokenResponse {
                agent_key: token,
                codename: agent.codename,
                instance_id: flow.instance_id.clone(),
                team_id: None,
            })
            .into_response()
        }
    }
}

async fn connect_page_get(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Query(query): Query<ConnectQuery>,
) -> Response {
    let approver = match approver_identity(&state, &headers, Some(peer)).await {
        Ok(a) => a,
        Err((st, body)) => return (st, body).into_response(),
    };
    let code = query.code.unwrap_or_default().trim().to_uppercase();
    render_connect_page(&state, &code, None, &approver, None).await
}

/// CSRF guard for the cookie-authenticated approve form: browsers always send
/// `Origin` on cross-site form POSTs, so a mismatched Origin/Referer means a
/// forged request. Non-browser clients (curl) send neither and pass through.
pub(crate) fn same_origin_form_post(headers: &axum::http::HeaderMap) -> bool {
    let source = headers
        .get(axum::http::header::ORIGIN)
        .or_else(|| headers.get(axum::http::header::REFERER));
    let Some(source) = source else {
        return true;
    };
    let Ok(source) = source.to_str() else {
        return false;
    };
    let Some(host) = headers
        .get(axum::http::header::HOST)
        .and_then(|h| h.to_str().ok())
    else {
        return false;
    };
    // source looks like "https://host[:port][/path]" — compare the authority.
    let authority = source
        .split("://")
        .nth(1)
        .unwrap_or(source)
        .split('/')
        .next()
        .unwrap_or_default();
    authority == host
}

async fn connect_page_post(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Form(form): Form<ConnectForm>,
) -> Response {
    if !same_origin_form_post(&headers) {
        return json_error(StatusCode::FORBIDDEN, "cross-origin form post rejected");
    }
    let approver = match approver_identity(&state, &headers, Some(peer)).await {
        Ok(a) => a,
        Err((st, body)) => return (st, body).into_response(),
    };
    let code = form.code.trim().to_uppercase();
    if code.is_empty() {
        return render_connect_page(&state, &code, Some("Enter a code."), &approver, None).await;
    }
    let result = if form.action == "deny" {
        state
            .brain
            .store()
            .deny_device_flow(&code, &approver)
            .await
            .map(|_| None)
    } else {
        approve_for_team(&state, &code, &approver, &form.team).await
    };
    match result {
        Ok(team) => Html(decision_page(
            &code,
            &approver,
            form.action != "deny",
            team.as_deref(),
        ))
        .into_response(),
        Err(e) => {
            render_connect_page(
                &state,
                &code,
                Some(&e.to_string()),
                &approver,
                Some(&form.team),
            )
            .await
        }
    }
}

/// Approve a pending flow. Solo lane: mark approved — the seat key mints
/// at token poll (unchanged). Hub lane: the approver's team pick is
/// authoritative, the hub key mints NOW and rides on the flow until the
/// single token-poll handout (KTD4 mint-at-approve — concurrent polls can
/// never double-mint). Returns the team the key was bound to.
async fn approve_for_team(
    state: &AppState,
    user_code: &str,
    approver: &str,
    form_team: &str,
) -> crate::Result<Option<String>> {
    #[cfg(feature = "postgres")]
    {
        if let Some(keys) = hub_keys(state) {
            return approve_hub(state, &keys, user_code, approver, form_team).await;
        }
    }
    #[cfg(not(feature = "postgres"))]
    let _ = form_team;
    state
        .brain
        .store()
        .approve_device_flow(user_code, approver)
        .await?;
    Ok(None)
}

/// Hub-lane approve: validate the approver's team pick, mint the
/// team-bound key once, then atomically stamp it onto the flow. If a
/// racing approve wins the conditional update, the key we minted is
/// revoked so exactly one key ever exists per flow.
#[cfg(feature = "postgres")]
async fn approve_hub(
    state: &AppState,
    keys: &std::sync::Arc<crate::hub::HubKeyStore>,
    user_code: &str,
    approver: &str,
    form_team: &str,
) -> crate::Result<Option<String>> {
    let team = form_team.trim();
    if team.is_empty() {
        return Err(crate::KurultaiError::Store(
            "pick a team for the new key".into(),
        ));
    }
    crate::store::scope::validate_team_id(team).map_err(crate::KurultaiError::Store)?;

    // Minting for a dead/expired flow would orphan a key — check first,
    // though the conditional UPDATE below remains the authoritative gate.
    let flow = state
        .brain
        .store()
        .get_device_flow_by_user_code(user_code)
        .await?
        .ok_or_else(|| crate::KurultaiError::Store("device flow not found".into()))?;
    if flow.status != "pending" {
        return Err(crate::KurultaiError::Store(
            "device flow already decided".into(),
        ));
    }

    let agent_label = hub_agent_label(&flow.codename, &flow.instance_id);
    let (key, key_id) = keys.issue(&agent_label, team).await?;
    match state
        .brain
        .store()
        .approve_device_flow_hub(user_code, approver, team, &key)
        .await
    {
        Ok(()) => Ok(Some(team.to_string())),
        Err(e) => {
            // A racing approve won the flow — revoke the orphan we minted.
            if let Err(rev) = keys.revoke_by_id(key_id).await {
                tracing::warn!(error = %rev, "orphan hub key revoke failed");
            }
            Err(e)
        }
    }
}

async fn render_connect_page(
    state: &AppState,
    code: &str,
    error: Option<&str>,
    approver: &str,
    team_override: Option<&str>,
) -> Response {
    let flow = if code.is_empty() {
        None
    } else {
        state
            .brain
            .store()
            .get_device_flow_by_user_code(code)
            .await
            .ok()
            .flatten()
    };
    let error_block = error.map_or_else(String::new, |e| {
        format!(r#"<div class="error">{}</div>"#, html_escape(e))
    });
    let detail = match &flow {
        Some(f) => format!(
            "<p>Agent <strong>{}</strong> (seat <strong>{}</strong>) is requesting a key.</p>",
            html_escape(&f.codename),
            html_escape(if f.instance_id.is_empty() {
                "default"
            } else {
                &f.instance_id
            }),
        ),
        None if code.is_empty() => "<p>Enter the code shown by `kurultai connect`.</p>".to_string(),
        None => "<p class=\"error\">Unknown code — check the agent's output.</p>".to_string(),
    };
    let code_field = if code.is_empty() {
        r#"<input type="text" name="code" placeholder="ABCD1234" autofocus />"#.to_string()
    } else {
        format!(
            r#"<input type="text" name="code" value="{}" readonly />"#,
            html_escape(code)
        )
    };
    // Hub lane (U3/KTD4): the approver picks the team — existing teams from
    // `hub_api_keys` plus a free-text "new team". The CLI's `--team` only
    // prefills; this choice is what the minted key is bound to. The roster
    // lives only on this authenticated page, never in /api/device/code.
    let team_block = if hub_lane(state) {
        let teams = hub_teams(state).await;
        let prefill = team_override
            .map(str::to_string)
            .or_else(|| flow.as_ref().and_then(|f| f.team_hint.clone()))
            .unwrap_or_default();
        let options = teams
            .iter()
            .map(|t| format!(r#"<option value="{}"></option>"#, html_escape(t)))
            .collect::<String>();
        format!(
            r#"<label class="team-label" for="team">Team <span class="muted">pick one or type a new team</span></label>
<input type="text" id="team" name="team" list="hub-teams" value="{}" placeholder="eng" autocomplete="off" required />
<datalist id="hub-teams">{}</datalist>"#,
            html_escape(&prefill),
            options,
        )
    } else {
        String::new()
    };
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Kurultai connect</title>
<style>
body {{ font-family: system-ui, -apple-system, sans-serif; background: #0a0a0a; color: #e5e5e5; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; }}
.card {{ background: #141414; border: 1px solid #2a2a2a; border-radius: 12px; padding: 2rem; max-width: 420px; width: 100%; box-shadow: 0 10px 40px rgba(0,0,0,0.5); }}
h1 {{ margin-top: 0; font-size: 1.25rem; color: #fff; }}
input {{ width: 100%; box-sizing: border-box; padding: 0.75rem; background: #0a0a0a; border: 1px solid #333; color: #fff; border-radius: 6px; font-family: monospace; font-size: 1.25rem; text-align: center; letter-spacing: 0.1em; }}
.team-label {{ display: block; margin-top: 1rem; font-size: 0.85rem; color: #a3a3a3; }}
.team-label .muted {{ color: #666; }}
#team {{ margin-top: 0.35rem; font-size: 1rem; text-align: left; letter-spacing: 0; }}
.row {{ display: flex; gap: 0.75rem; margin-top: 1rem; }}
button {{ flex: 1; padding: 0.75rem; border: none; border-radius: 6px; font-weight: 600; cursor: pointer; }}
.approve {{ background: #7c3aed; color: #fff; }}
.approve:hover {{ background: #6d28d9; }}
.deny {{ background: #262626; color: #a3a3a3; }}
.deny:hover {{ background: #333; }}
.error {{ background: #450a0a; color: #fecaca; padding: 0.75rem; border-radius: 6px; margin-bottom: 1rem; }}
p {{ color: #a3a3a3; line-height: 1.5; }}
</style>
</head>
<body>
<div class="card">
<h1>Approve agent connection</h1>
<p>Signed in as <strong>{}</strong>.</p>
{detail}
{error_block}
<form method="post" action="/connect">
{code_field}
{team_block}
<div class="row">
<button class="approve" type="submit" name="action" value="approve">Approve</button>
<button class="deny" type="submit" name="action" value="deny">Deny</button>
</div>
</form>
</div>
</body>
</html>"#,
        html_escape(approver),
    ))
    .into_response()
}

fn decision_page(code: &str, approver: &str, approved: bool, team: Option<&str>) -> String {
    let code = html_escape(code);
    let approver = html_escape(approver);
    let (title, body) = if approved {
        let team_txt = team
            .map(|t| format!(" for team <strong>{}</strong>", html_escape(t)))
            .unwrap_or_default();
        (
            "Agent approved",
            format!("Code <strong>{code}</strong> was approved by <strong>{approver}</strong>{team_txt} — the agent now holds a key."),
        )
    } else {
        (
            "Request denied",
            format!("Code <strong>{code}</strong> was denied by <strong>{approver}</strong>."),
        )
    };
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
body {{ font-family: system-ui, sans-serif; background: #0a0a0a; color: #e5e5e5; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; }}
.card {{ background: #141414; border: 1px solid #2a2a2a; border-radius: 12px; padding: 2rem; max-width: 420px; text-align: center; }}
h1 {{ color: #86efac; }}
p {{ color: #a3a3a3; }}
</style>
</head>
<body><div class="card"><h1>{title}</h1><p>{body}</p><p>You can close this tab.</p></div></body>
</html>"#
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn json_error(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "ok": false,
            "error": message,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow(status: &str, expires_at: &str) -> DeviceFlow {
        DeviceFlow {
            id: "f1".into(),
            device_code: "dc".into(),
            user_code: "ABCD1234".into(),
            codename: "cursor".into(),
            instance_id: "laptop".into(),
            client_id: "kurultai-cli".into(),
            status: status.into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            expires_at: expires_at.into(),
            approved_at: None,
            approved_by: None,
            agent_id: None,
            token_hash: None,
            team_hint: None,
            team_id: None,
            issued_key: None,
        }
    }

    #[test]
    fn poll_decision_transitions() {
        let now = Utc::now();
        let future = (now + chrono::Duration::minutes(5)).to_rfc3339();
        let past = (now - chrono::Duration::minutes(1)).to_rfc3339();

        assert_eq!(
            poll_decision(&flow("pending", &future), now),
            PollDecision::Pending
        );
        assert_eq!(
            poll_decision(&flow("approved", &future), now),
            PollDecision::Approved
        );
        assert_eq!(
            poll_decision(&flow("denied", &future), now),
            PollDecision::Denied
        );
        // Exchanged flows (token already handed out) must not re-issue.
        assert_eq!(
            poll_decision(&flow("exchanged", &future), now),
            PollDecision::Denied
        );
        // Expiry wins over every status.
        assert_eq!(
            poll_decision(&flow("pending", &past), now),
            PollDecision::Expired
        );
        assert_eq!(
            poll_decision(&flow("approved", &past), now),
            PollDecision::Expired
        );
        // Unparseable expiry fails closed.
        assert_eq!(
            poll_decision(&flow("pending", "garbage"), now),
            PollDecision::Expired
        );
    }

    #[test]
    fn gate_prefers_cf_then_hub_then_loopback() {
        assert_eq!(approver_gate(false, HubAuth::None), ApproverGate::Loopback);
        assert_eq!(approver_gate(false, HubAuth::ApiKey), ApproverGate::Hub);
        assert_eq!(approver_gate(true, HubAuth::ApiKey), ApproverGate::CfAccess);
        assert_eq!(approver_gate(true, HubAuth::None), ApproverGate::CfAccess);
    }

    #[test]
    fn html_escape_blocks_markup() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b\"c"), "a&amp;b&quot;c");
    }

    #[test]
    fn reserved_codename_rejected() {
        for c in ["luke", "Luke", "LUKE"] {
            assert!(
                RESERVED_CODENAMES.iter().any(|r| c.eq_ignore_ascii_case(r)),
                "{c} must be reserved"
            );
        }
        assert!(!RESERVED_CODENAMES
            .iter()
            .any(|r| "cursor".eq_ignore_ascii_case(r)));
    }

    #[test]
    fn csrf_origin_check() {
        use axum::http::{HeaderMap, HeaderValue};
        let mut h = HeaderMap::new();
        // No Origin/Referer (non-browser client) passes.
        assert!(same_origin_form_post(&h));
        // Same-origin passes.
        h.insert("host", HeaderValue::from_static("knowledge.shippedit.dev"));
        h.insert(
            "origin",
            HeaderValue::from_static("https://knowledge.shippedit.dev"),
        );
        assert!(same_origin_form_post(&h));
        // Cross-origin fails.
        h.insert("origin", HeaderValue::from_static("https://evil.example"));
        assert!(!same_origin_form_post(&h));
        // Origin host-prefix tricks fail (exact authority match).
        h.insert(
            "origin",
            HeaderValue::from_static("https://knowledge.shippedit.dev.evil.example"),
        );
        assert!(!same_origin_form_post(&h));
        // Referer fallback: same-origin referer passes.
        h.remove("origin");
        h.insert(
            "referer",
            HeaderValue::from_static("https://knowledge.shippedit.dev/connect?code=ABCD"),
        );
        assert!(same_origin_form_post(&h));
        // Missing Host fails closed when an Origin is present.
        h.remove("host");
        assert!(!same_origin_form_post(&h));
    }

    #[test]
    fn admin_email_allowlist_matches_case_and_csv() {
        let _guard = crate::testutil::EnvGuard::set(ENV_ADMIN_EMAILS, "Luke@x.dev, second@x.dev ,");
        assert!(admin_email_allowed("luke@x.dev"));
        assert!(admin_email_allowed("SECOND@x.dev"));
        assert!(!admin_email_allowed("intruder@x.dev"));
        assert!(!admin_email_allowed(""));
    }

    #[test]
    fn admin_email_allowlist_unset_fails_closed() {
        let _guard = crate::testutil::EnvGuard::remove(ENV_ADMIN_EMAILS);
        assert!(!admin_email_allowed("luke@x.dev"));
    }

    #[test]
    fn hub_agent_label_seat_suffix() {
        assert_eq!(hub_agent_label("devin", "seat-1"), "devin@seat-1");
        assert_eq!(hub_agent_label("devin", ""), "devin");
    }

    /// U3/KTD4: under `KURULTAI_FEATURE_HUB=1` a verified Access identity
    /// must ALSO appear in `KURULTAI_ADMIN_EMAILS` — the Access app admits
    /// every org member, but approval mints a team-bound credential. The
    /// seeded verifier mirrors `cf_access` tests (ephemeral RSA, no JWKS fetch).
    #[cfg(feature = "postgres")]
    #[tokio::test]
    async fn cf_access_allowlist_gates_hub_approvers() {
        use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header as JwtHeader};

        const TEAM: &str = "https://duketopceo.cloudflareaccess.com";
        const AUD: &str = "u3-test-aud";

        let dir = std::env::temp_dir().join(format!(
            "kurultai-devcf-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let priv_path = dir.join("k.pem");
        let pub_path = dir.join("k.pub.pem");
        let run = |args: &[&str]| {
            let out = std::process::Command::new("openssl")
                .args(args)
                .output()
                .expect("openssl not found");
            assert!(out.status.success(), "openssl {args:?} failed");
        };
        run(&["genrsa", "-out", priv_path.to_str().unwrap(), "2048"]);
        run(&[
            "rsa",
            "-in",
            priv_path.to_str().unwrap(),
            "-pubout",
            "-out",
            pub_path.to_str().unwrap(),
        ]);
        let enc = EncodingKey::from_rsa_pem(&std::fs::read(&priv_path).unwrap()).unwrap();
        let dec = DecodingKey::from_rsa_pem(&std::fs::read(&pub_path).unwrap()).unwrap();
        std::fs::remove_dir_all(&dir).ok();

        let cf = std::sync::Arc::new(crate::http::cf_access::CfAccess::seeded_for_test(
            TEAM,
            vec![AUD.to_string()],
            vec![(Some("test-kid".to_string()), dec)],
        ));

        let sign = |email: &str| {
            #[derive(serde::Serialize)]
            struct Claims<'a> {
                sub: &'a str,
                email: &'a str,
                aud: &'a str,
                iss: &'a str,
                exp: usize,
            }
            let mut header = JwtHeader::new(Algorithm::RS256);
            header.kid = Some("test-kid".to_string());
            encode(
                &header,
                &Claims {
                    sub: "u1",
                    email,
                    aud: AUD,
                    iss: TEAM,
                    exp: (chrono::Utc::now().timestamp() + 3600) as usize,
                },
                &enc,
            )
            .unwrap()
        };

        // Minimal AppState — the store is never touched by the gate.
        let dir =
            std::env::temp_dir().join(format!("kurultai-devcf-store-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let store =
            std::sync::Arc::new(crate::store::SqliteVecStore::open(dir.join("s.db"), 4).unwrap());
        let brain = std::sync::Arc::new(crate::mcp::brain::BrainService::new(
            store,
            std::sync::Arc::new(crate::embed::NullEmbedder::new(4)),
            std::sync::Arc::new(crate::rerank::NullReranker::new()),
            std::sync::Arc::new(crate::synthesize::ExtractiveSynthesizer::new()),
        ));
        let gate = crate::http::auth::HubGate {
            auth: HubAuth::ApiKey,
            api_keys: vec![],
            agent_store: None,
            cf_access: Some(cf),
            key_store: Some(std::sync::Arc::new(crate::hub::HubKeyStore::lazy_for_test())),
        };
        let state = crate::http::app_state(
            brain,
            std::sync::Arc::new(crate::daemon::DaemonStatus::default()),
            crate::metrics::MetricsRegistry::shared(),
            gate,
        );

        let _env = crate::testutil::EnvGuard::apply(&[
            ("KURULTAI_FEATURE_HUB", Some("1")),
            (ENV_ADMIN_EMAILS, Some("admin@x.dev")),
        ]);

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "cf-access-jwt-assertion",
            sign("admin@x.dev").parse().unwrap(),
        );
        assert_eq!(
            approver_identity(&state, &headers, None).await.unwrap(),
            "admin@x.dev"
        );

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "cf-access-jwt-assertion",
            sign("intruder@x.dev").parse().unwrap(),
        );
        let err = approver_identity(&state, &headers, None).await.unwrap_err();
        assert_eq!(err.0, StatusCode::FORBIDDEN);

        // No assertion at all → 401 (sign-in prompt, not an allowlist error).
        let err = approver_identity(&state, &axum::http::HeaderMap::new(), None)
            .await
            .unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    }
}
