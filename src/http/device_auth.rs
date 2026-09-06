//! Device authorization sign-in (RFC 8628 style) for the Kurultai CLI.
//!
//! Agents run `kurultai login`, receive a `user_code`, and poll `/auth/device/token`.
//! The human opens the Cloudflare-Access-protected `/auth/device` page, enters the
//! `user_code`, and clicks approve. Once approved, the CLI receives a long-lived
//! agent token that is stored in the local platform keyring / `omaseal`.

use crate::hashutil::sha256_hex;
use crate::http::AppState;
use crate::store::DeviceFlow;
use axum::extract::{Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub(crate) fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/device", get(device_page_get).post(device_page_post))
        .route("/auth/device/code", post(device_code_post))
        .route("/auth/device/token", post(device_token_post))
        .with_state(state)
}

const DEFAULT_CLIENT_ID: &str = "kurultai-cli";
const DEFAULT_EXPIRES_IN: u64 = 600; // 10 minutes
const DEFAULT_INTERVAL: u64 = 5;

#[derive(Debug, Deserialize)]
struct DeviceCodeRequest {
    #[serde(default)]
    client_id: String,
    #[serde(default)]
    codename: String,
}

#[derive(Debug, Serialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenRequest {
    grant_type: String,
    device_code: String,
    #[serde(default)]
    #[allow(dead_code)]
    client_id: String,
}

#[derive(Debug, Serialize)]
struct DeviceTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct DevicePageForm {
    user_code: String,
}

#[derive(Debug, Deserialize)]
struct DevicePageQuery {
    user_code: Option<String>,
}

fn access_user_email(headers: &axum::http::HeaderMap) -> Option<String> {
    for name in [
        "cf-access-user-email",
        "cf-access-user",
        "CF-Access-User-Email",
        "CF-Access-User",
    ] {
        if let Some(value) = headers.get(name).and_then(|v| v.to_str().ok()) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn is_expired(flow: &DeviceFlow) -> bool {
    match DateTime::parse_from_rfc3339(&flow.expires_at) {
        Ok(expires) => Utc::now() > expires.with_timezone(&Utc),
        Err(_) => true,
    }
}

fn request_scheme(headers: &axum::http::HeaderMap) -> &'static str {
    if let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    {
        if proto.eq_ignore_ascii_case("https") {
            return "https";
        } else if proto.eq_ignore_ascii_case("http") {
            return "http";
        }
    }
    if let Some(proto) = headers
        .get("cf-visitor")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok())
        .and_then(|v| v.get("scheme").and_then(|s| s.as_str()).map(String::from))
    {
        if proto.eq_ignore_ascii_case("https") {
            return "https";
        } else if proto.eq_ignore_ascii_case("http") {
            return "http";
        }
    }
    if let Ok(scheme) = std::env::var("KURULTAI_AUTH_SCHEME") {
        if scheme.eq_ignore_ascii_case("http") {
            return "http";
        } else if scheme.eq_ignore_ascii_case("https") {
            return "https";
        }
    }

    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    if host.starts_with("127.0.0.1") || host.starts_with("localhost") || host.starts_with("::1") {
        "http"
    } else {
        "https"
    }
}

fn human_auth_url(headers: &axum::http::HeaderMap) -> String {
    let scheme = request_scheme(headers);
    let host = if let Ok(host) = std::env::var("KURULTAI_AUTH_HOST") {
        if !host.is_empty() {
            host
        } else {
            headers
                .get("host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("localhost")
                .to_string()
        }
    } else {
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        if let Some(stripped) = host.strip_prefix("api-") {
            stripped.to_string()
        } else if let Some(stripped) = host.strip_prefix("api.") {
            stripped.to_string()
        } else {
            host.to_string()
        }
    };
    format!("{scheme}://{host}/auth/device")
}

async fn device_code_post(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<DeviceCodeRequest>,
) -> Response {
    let client_id = if body.client_id.is_empty() {
        DEFAULT_CLIENT_ID.to_string()
    } else {
        body.client_id
    };
    let codename = if body.codename.is_empty() {
        "anonymous".to_string()
    } else {
        body.codename
    };

    let flow = match state
        .brain
        .store()
        .create_device_flow(&codename, &client_id, DEFAULT_EXPIRES_IN)
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

    let base = human_auth_url(&headers);
    let verification_uri = format!("{}?user_code={}", base, flow.user_code);

    Json(DeviceCodeResponse {
        device_code: flow.device_code,
        user_code: flow.user_code,
        verification_uri,
        expires_in: DEFAULT_EXPIRES_IN,
        interval: DEFAULT_INTERVAL,
    })
    .into_response()
}

async fn device_token_post(
    State(state): State<AppState>,
    Json(body): Json<DeviceTokenRequest>,
) -> Response {
    if body.grant_type != "urn:ietf:params:oauth:grant-type:device_code" {
        return json_error(StatusCode::BAD_REQUEST, "unsupported grant_type");
    }

    let flow = match state
        .brain
        .store()
        .get_device_flow_by_device_code(&body.device_code)
        .await
    {
        Ok(Some(f)) => f,
        Ok(None) => return json_error(StatusCode::BAD_REQUEST, "expired or invalid device_code"),
        Err(e) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if is_expired(&flow) {
        return json_error(StatusCode::BAD_REQUEST, "expired_token");
    }

    if flow.status == "pending" {
        return json_error(StatusCode::BAD_REQUEST, "authorization_pending");
    }

    if flow.status != "approved" {
        return json_error(StatusCode::BAD_REQUEST, "access_denied");
    }

    let (agent, token) = match state.brain.store().issue_agent_token(&flow.codename).await {
        Ok(pair) => pair,
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("agent token issue failed: {e}"),
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
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: 0,
    })
    .into_response()
}

async fn device_page_get(
    State(_state): State<AppState>,
    Query(query): Query<DevicePageQuery>,
) -> Response {
    let page = render_approve_page(query.user_code.as_deref().unwrap_or_default(), None, "");
    Html(page).into_response()
}

async fn device_page_post(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<DevicePageForm>,
) -> Response {
    let Some(approver) = access_user_email(&headers) else {
        let page = render_approve_page(
            &form.user_code,
            Some("Cloudflare Access identity not found."),
            "",
        );
        return (StatusCode::UNAUTHORIZED, Html(page)).into_response();
    };

    let user_code = form.user_code.trim().to_uppercase();
    if user_code.is_empty() {
        let page = render_approve_page(&user_code, Some("Enter a user code."), &approver);
        return (StatusCode::BAD_REQUEST, Html(page)).into_response();
    }

    match state
        .brain
        .store()
        .approve_device_flow(&user_code, &approver)
        .await
    {
        Ok(_) => {
            let page = success_page(&user_code, &approver);
            Html(page).into_response()
        }
        Err(e) => {
            let page = render_approve_page(&user_code, Some(&e.to_string()), &approver);
            (StatusCode::BAD_REQUEST, Html(page)).into_response()
        }
    }
}

fn render_approve_page(user_code: &str, error: Option<&str>, approver: &str) -> String {
    let error_block =
        error.map_or_else(String::new, |e| format!(r#"<div class="error">{e}</div>"#));
    let user_field = if user_code.is_empty() {
        r#"<input type="text" name="user_code" placeholder="ABCD-1234" value="" autofocus />"#
            .to_string()
    } else {
        format!(r#"<input type="text" name="user_code" value="{user_code}" readonly />"#)
    };
    let approver_block = if approver.is_empty() {
        String::new()
    } else {
        format!(r#"<p>Signed in as <strong>{approver}</strong>.</p>"#)
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Kurultai agent sign-in</title>
<style>
body {{ font-family: system-ui, -apple-system, sans-serif; background: #0a0a0a; color: #e5e5e5; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; }}
.card {{ background: #141414; border: 1px solid #2a2a2a; border-radius: 12px; padding: 2rem; max-width: 420px; width: 100%; box-shadow: 0 10px 40px rgba(0,0,0,0.5); }}
h1 {{ margin-top: 0; font-size: 1.25rem; color: #fff; }}
input {{ width: 100%; box-sizing: border-box; padding: 0.75rem; background: #0a0a0a; border: 1px solid #333; color: #fff; border-radius: 6px; font-family: monospace; font-size: 1.25rem; text-align: center; letter-spacing: 0.1em; }}
button {{ width: 100%; margin-top: 1rem; padding: 0.75rem; background: #7c3aed; color: #fff; border: none; border-radius: 6px; font-weight: 600; cursor: pointer; }}
button:hover {{ background: #6d28d9; }}
.error {{ background: #450a0a; color: #fecaca; padding: 0.75rem; border-radius: 6px; margin-bottom: 1rem; }}
.success {{ color: #86efac; }}
p {{ color: #a3a3a3; line-height: 1.5; }}
</style>
</head>
<body>
<div class="card">
<h1>Approve Kurultai agent sign-in</h1>
{approver_block}
<p>Enter the user code shown by the agent you are signing in.</p>
{error_block}
<form method="post" action="/auth/device">
{user_field}
<button type="submit">Approve sign-in</button>
</form>
</div>
</body>
</html>"#
    )
}

fn success_page(user_code: &str, approver: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Signed in</title>
<style>
body {{ font-family: system-ui, -apple-system, sans-serif; background: #0a0a0a; color: #e5e5e5; display: flex; align-items: center; justify-content: center; min-height: 100vh; margin: 0; }}
.card {{ background: #141414; border: 1px solid #2a2a2a; border-radius: 12px; padding: 2rem; max-width: 420px; width: 100%; text-align: center; }}
h1 {{ color: #86efac; }}
p {{ color: #a3a3a3; }}
</style>
</head>
<body>
<div class="card">
<h1>Agent approved</h1>
<p>The agent using code <strong>{user_code}</strong> has been approved by <strong>{approver}</strong>.</p>
<p>You can close this tab.</p>
</div>
</body>
</html>"#
    )
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
