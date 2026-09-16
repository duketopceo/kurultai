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

#[derive(Debug, Deserialize)]
struct DeviceCodeRequest {
    #[serde(default)]
    codename: String,
    #[serde(default)]
    instance_id: String,
    #[serde(default)]
    client_id: String,
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
                Ok(identity
                    .email
                    .unwrap_or_else(|| format!("cf:{}", identity.sub)))
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

    let flow = match state
        .brain
        .store()
        .create_device_flow(&codename, &client_id, &instance_id, DEFAULT_EXPIRES_IN)
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
    render_connect_page(&state, &code, None, &approver).await
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
        return render_connect_page(&state, &code, Some("Enter a code."), &approver).await;
    }
    let result = if form.action == "deny" {
        state.brain.store().deny_device_flow(&code, &approver).await
    } else {
        state
            .brain
            .store()
            .approve_device_flow(&code, &approver)
            .await
    };
    match result {
        Ok(()) => Html(decision_page(&code, &approver, form.action != "deny")).into_response(),
        Err(e) => render_connect_page(&state, &code, Some(&e.to_string()), &approver).await,
    }
}

async fn render_connect_page(
    state: &AppState,
    code: &str,
    error: Option<&str>,
    approver: &str,
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

fn decision_page(code: &str, approver: &str, approved: bool) -> String {
    let code = html_escape(code);
    let approver = html_escape(approver);
    let (title, body) = if approved {
        (
            "Agent approved",
            format!("Code <strong>{code}</strong> was approved by <strong>{approver}</strong> — the agent now holds a key."),
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
}
