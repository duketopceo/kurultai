//! First-run onboarding for new self-hosted instances.
//!
//! * `GET /ui/` redirects to `/ui/welcome` while the brain is empty (add
//!   `?welcome=0` to skip). Populated brains, including every hosted instance,
//!   are untouched.
//! * `GET /ui/welcome` is a static page (no build step) that walks through
//!   creating a first note, importing a folder, and choosing embeddings.
//! * `GET /api/onboarding` reports what the page needs (atom count, active
//!   embedder, whether this build can embed locally, whether an OpenRouter key
//!   exists). It never returns key material.
//! * `POST /api/onboarding/note` stores the first note through the same
//!   `remember` path agents use, so quality gates and write containment apply.
//!
//! Nothing here writes config or touches the filesystem: folder import and
//! embedding choice are made with mounts and environment variables (see
//! `docs/self-host.md`), so a remote caller cannot repoint the instance.

use super::{ui, AppState};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

const WELCOME_HTML: &str = include_str!("welcome.html");

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ui/", get(ui_index))
        .route("/ui/welcome", get(welcome_page))
        .route("/api/onboarding", get(api_onboarding))
        .route("/api/onboarding/note", post(api_onboarding_note))
}

#[derive(Deserialize)]
struct IndexQuery {
    welcome: Option<String>,
}

/// `/ui/`: the Brain UI, or the welcome page for an empty brain.
async fn ui_index(State(state): State<AppState>, Query(q): Query<IndexQuery>) -> Response {
    let skip = q.welcome.as_deref() == Some("0");
    if !skip && matches!(state.brain.atom_count().await, Ok(0)) {
        return Redirect::temporary("/ui/welcome").into_response();
    }
    ui::index_response()
}

async fn welcome_page() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        WELCOME_HTML,
    )
        .into_response()
}

fn openrouter_key_present() -> bool {
    use crate::security::{api_key_from_env_optional, api_key_from_keyfile};
    api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"))
        .or_else(api_key_from_keyfile)
        .is_some()
}

async fn api_onboarding(State(state): State<AppState>) -> Json<serde_json::Value> {
    let atoms = state.brain.atom_count().await.unwrap_or(0);
    let embedder = state.brain.embedder();
    Json(serde_json::json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "atoms": atoms,
        "needs_onboarding": atoms == 0,
        "embedder": { "name": embedder.name(), "live": embedder.is_live(), "dim": embedder.dim() },
        "local_embed_available": cfg!(feature = "local-embed"),
        "openrouter_key_present": openrouter_key_present(),
    }))
}

#[derive(Deserialize)]
struct NoteBody {
    title: String,
    body: String,
    #[serde(default)]
    tags: Vec<String>,
}

async fn api_onboarding_note(
    State(state): State<AppState>,
    Json(note): Json<NoteBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut tags: Vec<String> = note
        .tags
        .iter()
        .map(|t| t.trim().trim_start_matches('#').to_ascii_lowercase())
        .filter(|t| !t.is_empty())
        .take(8)
        .collect();
    if tags.is_empty() {
        tags.push("welcome".into());
    }
    let ctx =
        crate::write_policy::WriteContext::from_env(crate::write_policy::WriteTransport::Http);
    match state
        .brain
        .remember_with(&note.title, &note.body, &tags, &[], &ctx)
        .await
    {
        Ok(id) => Ok(Json(
            serde_json::json!({ "ok": true, "id": id, "tags": tags }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{Embedder, NullEmbedder};
    use crate::http::auth::HubGate;
    use crate::mcp::brain::BrainService;
    use crate::metrics::MetricsRegistry;
    use crate::rerank::NullReranker;
    use crate::store::SqliteVecStore;
    use crate::synthesize::{ExtractiveSynthesizer, Synthesizer};
    use axum::body::Body;
    use axum::http::Request;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn app() -> (Router, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let synth: Arc<dyn Synthesizer> = Arc::new(ExtractiveSynthesizer::new());
        let brain = BrainService::new(store, embedder, Arc::new(NullReranker::new()), synth);
        let state = AppState {
            brain: Arc::new(brain),
            status: Arc::new(crate::daemon::DaemonStatus::default()),
            metrics: MetricsRegistry::shared(),
            #[cfg(feature = "postgres")]
            hub_activity: None,
            graph_cache: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            hub: HubGate::default(),
        };
        (routes().with_state(state), dir)
    }

    async fn get(app: &Router, uri: &str) -> Response {
        app.clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn empty_brain_redirects_to_welcome_unless_skipped() {
        let (app, _d) = app();
        let r = get(&app, "/ui/").await;
        assert_eq!(r.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(r.headers()[header::LOCATION], "/ui/welcome");
        let r = get(&app, "/ui/?welcome=0").await;
        assert_eq!(r.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn welcome_page_serves_html() {
        let (app, _d) = app();
        let r = get(&app, "/ui/welcome").await;
        assert_eq!(r.status(), StatusCode::OK);
        let body = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Welcome to Kurultai"));
    }

    #[tokio::test]
    async fn first_note_flows_through_remember_and_ends_onboarding() {
        let (app, _d) = app();
        let r = get(&app, "/api/onboarding").await;
        let body = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["needs_onboarding"], true);
        assert_eq!(v["embedder"]["live"], false);

        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/onboarding/note")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r##"{"title":"How we deploy","body":"Docker compose behind a tunnel.","tags":["#Ops"]}"##,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let body = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["tags"][0], "ops");

        let r = get(&app, "/api/onboarding").await;
        let body = axum::body::to_bytes(r.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["needs_onboarding"], false);
        assert_eq!(get(&app, "/ui/").await.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn empty_note_is_rejected() {
        let (app, _d) = app();
        let r = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/onboarding/note")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":" ","body":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    }
}
