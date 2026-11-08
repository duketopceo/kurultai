//! OpenRouter "Connect" for hosted/solo daemons.
//!
//! - `POST /api/openrouter/connect` — starts a flow, returns `{auth_url, expires_in}`.
//!   A WRITE route (`auth::WRITE_ROUTES`): shared-write mode needs the admin token.
//! - `POST /api/openrouter/key` `{key}` — the browser (first-party `@openrouter/sdk`
//!   PKCE flow) hands over the exchanged key. WRITE route. Keys are read once at daemon
//!   start (embedder/synthesizer), so the response carries `restart_required: true`.
//! - `GET /api/openrouter/callback?code&state` — OpenRouter's redirect target. The
//!   browser carries no bearer, so it is exempt from hub auth and self-gated by
//!   the single-use, short-TTL `state` minted by `connect` (PKCE verifier lives
//!   only in memory). The key is written to the 0600 key file and is never
//!   logged or returned. Restart the daemon to pick the key up.

use crate::openrouter_connect::{
    build_auth_url, complete_flow, failure_html, FileKeyStore, KeyStore, PendingFlows,
    OPENROUTER_ORIGIN, STATE_TTL, SUCCESS_HTML,
};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct OrState {
    flows: PendingFlows,
    store: Arc<dyn KeyStore>,
    origin: String,
    client: reqwest::Client,
}

pub(crate) fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    let store: Arc<dyn KeyStore> = match FileKeyStore::default_location() {
        Ok(s) => Arc::new(s),
        Err(_) => Arc::new(Unavailable),
    };
    router_with(OrState {
        flows: PendingFlows::default(),
        store,
        origin: OPENROUTER_ORIGIN.to_string(),
        client: reqwest::Client::new(),
    })
}

fn router_with<S: Clone + Send + Sync + 'static>(st: OrState) -> Router<S> {
    Router::new()
        .route("/api/openrouter/connect", post(connect))
        .route("/api/openrouter/key", post(set_key))
        .route("/api/openrouter/callback", get(callback))
        .with_state(st)
}

struct Unavailable;
impl KeyStore for Unavailable {
    fn store(&self, _key: &str) -> crate::Result<String> {
        Err(crate::error::KurultaiError::config(
            "no writable key file location",
        ))
    }
}

/// `KURULTAI_PUBLIC_URL` wins; else the human base URL derived from the request.
fn public_base(headers: &HeaderMap) -> String {
    match std::env::var("KURULTAI_PUBLIC_URL") {
        Ok(v) if !v.trim().is_empty() => v.trim().trim_end_matches('/').to_string(),
        _ => crate::http::device_auth::human_base_url(headers),
    }
}

async fn connect(State(st): State<OrState>, headers: HeaderMap) -> Response {
    let callback = format!("{}/api/openrouter/callback", public_base(&headers));
    let (state, challenge) = st.flows.begin();
    match build_auth_url(&st.origin, &callback, &challenge, &state) {
        Ok(auth_url) => Json(serde_json::json!({
            "auth_url": auth_url,
            "expires_in": STATE_TTL.as_secs(),
        }))
        .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "bad OpenRouter origin").into_response(),
    }
}

#[derive(serde::Deserialize)]
struct KeyBody {
    key: String,
}

/// OpenRouter keys look like `sk-or-v1-<hex>`; accept a sane shape only.
fn valid_key_shape(k: &str) -> bool {
    (12..=256).contains(&k.len())
        && k.starts_with("sk-or-")
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

async fn set_key(State(st): State<OrState>, Json(body): Json<KeyBody>) -> Response {
    let key = body.key.trim();
    if !valid_key_shape(key) {
        return (StatusCode::BAD_REQUEST, "not an OpenRouter key").into_response();
    }
    match st.store.store(key) {
        Ok(where_) => {
            tracing::info!(location = %where_, "OpenRouter key stored from browser");
            Json(serde_json::json!({ "ok": true, "restart_required": true })).into_response()
        }
        Err(e) => {
            tracing::warn!(error = %e, "OpenRouter key store failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "could not store key").into_response()
        }
    }
}

async fn callback(State(st): State<OrState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let (Some(code), Some(state)) = (q.get("code"), q.get("state")) else {
        return (
            StatusCode::BAD_REQUEST,
            Html(failure_html("Authorization was denied or incomplete.")),
        )
            .into_response();
    };
    match complete_flow(
        &st.flows,
        &st.client,
        &st.origin,
        st.store.as_ref(),
        state,
        code,
    )
    .await
    {
        Ok(where_) => {
            tracing::info!(location = %where_, "OpenRouter key connected");
            Html(SUCCESS_HTML).into_response()
        }
        Err(e) => {
            // Messages are secret-free by construction (see ConnectError).
            tracing::warn!(error = %e, "OpenRouter connect failed");
            (StatusCode::BAD_REQUEST, Html(failure_html(&e.to_string()))).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openrouter_connect::FileKeyStore;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn fake_or() -> String {
        let app = Router::new().route(
            "/api/v1/auth/keys",
            post(|| async { Json(serde_json::json!({"key": "sk-or-hosted"})) }),
        );
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let a = l.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
        format!("http://{a}")
    }

    async fn get_status(app: &Router, uri: &str) -> StatusCode {
        app.clone()
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn hosted_flow_single_use_and_state_checked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("openrouter.key");
        let st = OrState {
            flows: PendingFlows::default(),
            store: Arc::new(FileKeyStore::new(path.clone())),
            origin: fake_or().await,
            client: reqwest::Client::new(),
        };
        let app: Router = router_with::<()>(st.clone());

        let resp = app
            .clone()
            .oneshot(
                Request::post("/api/openrouter/connect")
                    .header("host", "example.test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 16)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let url = reqwest::Url::parse(v["auth_url"].as_str().unwrap()).unwrap();
        let q: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert!(q["callback_url"].ends_with("/api/openrouter/callback"));
        let state = q["state"].clone();

        assert_eq!(
            get_status(&app, "/api/openrouter/callback?code=c&state=wrong").await,
            StatusCode::BAD_REQUEST
        );
        assert!(!path.exists());
        assert_eq!(
            get_status(&app, "/api/openrouter/callback?code=c").await,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            get_status(
                &app,
                &format!("/api/openrouter/callback?code=c&state={state}")
            )
            .await,
            StatusCode::OK
        );
        assert_eq!(
            std::fs::read_to_string(&path).unwrap().trim(),
            "sk-or-hosted"
        );
        // single use
        assert_eq!(
            get_status(
                &app,
                &format!("/api/openrouter/callback?code=c&state={state}")
            )
            .await,
            StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    async fn key_route_validates_and_stores() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("openrouter.key");
        let st = OrState {
            flows: PendingFlows::default(),
            store: Arc::new(FileKeyStore::new(path.clone())),
            origin: "http://127.0.0.1:1".into(),
            client: reqwest::Client::new(),
        };
        let app: Router = router_with::<()>(st);
        let post_key = |k: &str| {
            Request::post("/api/openrouter/key")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({ "key": k }).to_string()))
                .unwrap()
        };
        let r = app.clone().oneshot(post_key("nope")).await.unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        assert!(!path.exists());
        let r = app
            .clone()
            .oneshot(post_key("sk-or-v1-abc123def456"))
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(r.into_body(), 1 << 16).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["restart_required"], true);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap().trim(),
            "sk-or-v1-abc123def456"
        );
    }

    #[tokio::test]
    async fn expired_state_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let st = OrState {
            flows: PendingFlows::with_ttl(std::time::Duration::from_millis(1)),
            store: Arc::new(FileKeyStore::new(dir.path().join("k"))),
            origin: "http://127.0.0.1:1".into(),
            client: reqwest::Client::new(),
        };
        let (state, _) = st.flows.begin();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let app: Router = router_with::<()>(st);
        assert_eq!(
            get_status(
                &app,
                &format!("/api/openrouter/callback?code=c&state={state}")
            )
            .await,
            StatusCode::BAD_REQUEST
        );
    }
}
