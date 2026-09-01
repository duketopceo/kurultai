#![allow(clippy::field_reassign_with_default)]
//! Acceptance tests — HTTP API surface (KHAN-251).
//!
//! Covers: health, /api/status, /api/atoms, /api/search (GET+POST), /ask
//! (GET+POST), /api/recall, /api/ontology, /api/graph, brain UI, and hub
//! API-key auth middleware.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::daemon::DaemonStatus;
use kurultai::embed::NullEmbedder;
use kurultai::http::{build_app, HubAuth, HubGate};
use kurultai::mcp::brain::BrainService;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

fn fixture_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

async fn fixture_brain() -> BrainService {
    static N: AtomicU64 = AtomicU64::new(0);
    let db_dir = std::env::temp_dir().join(format!(
        "khan251-http-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&db_dir).unwrap();
    let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
    let embedder: Arc<dyn kurultai::embed::Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));
    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert(
        "root_path".into(),
        fixture_vault().to_string_lossy().into_owned(),
    );
    connector
        .init(&SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();
    pipeline
        .index_connector("notes", &connector, true)
        .await
        .unwrap();
    BrainService::new(
        store,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    )
}

async fn empty_brain() -> BrainService {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "khan251-http-empty-{}-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap());
    let embedder: Arc<dyn kurultai::embed::Embedder> = Arc::new(NullEmbedder::new(4));
    BrainService::new(
        store,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    )
}

fn app(brain: BrainService, hub: HubGate) -> axum::Router {
    build_app(brain, Arc::new(DaemonStatus::default()), hub)
}

async fn body_str(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ── /health ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_ok() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["service"], "kurultai");
}

// ── /api/status ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn api_status_ok_with_metrics_and_memory() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap(),
        "no-store"
    );
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    assert!(v["atoms"].is_number());
    assert!(v["memory"]["hot"].is_number());
    assert!(v["memory"]["warm"].is_number());
    assert!(v["memory"]["cold"].is_number());
    assert!(v["scheduler"]["last_client_activity_unix"].is_number());
    assert!(!v["request_id"].as_str().unwrap_or("").is_empty());
}

// ── /api/atoms ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn api_atoms_lists_fixture_with_limit() {
    let app = app(fixture_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/atoms?limit=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let results: Vec<kurultai::types::SearchResult> =
        serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rank, 0);
    assert!(results[0].matched_by.iter().any(|m| m == "list"));
}

// ── /api/search GET + POST ───────────────────────────────────────────────────

#[tokio::test]
async fn http_search_get_finds_phrase() {
    let app = app(fixture_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/search?q=KNOWN_PHRASE_KURULTAI_42&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let results: Vec<kurultai::types::SearchResult> =
        serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn http_search_post_finds_phrase() {
    let app = app(fixture_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/search")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"query":"KNOWN_PHRASE_KURULTAI_42","limit":5}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let results: Vec<kurultai::types::SearchResult> =
        serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(!results.is_empty());
}

// ── /ask GET + POST ──────────────────────────────────────────────────────────

#[tokio::test]
async fn http_ask_post_returns_answer() {
    let app = app(fixture_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ask")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"question":"what is KNOWN_PHRASE_KURULTAI_42"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let answer: kurultai::types::Answer = serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(answer.confidence > 0.0);
    assert!(!answer.citations.is_empty());
}

#[tokio::test]
async fn http_ask_empty_store_returns_zero_confidence() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/ask")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"question":"anything?"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let answer: kurultai::types::Answer = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(answer.confidence, 0.0);
    assert!(answer.citations.is_empty());
}

// ── /api/recall (project-scoped) ─────────────────────────────────────────────

#[tokio::test]
async fn http_recall_returns_agent_views() {
    let brain = fixture_brain().await;
    use kurultai::types::KnowledgeAtom;
    let mut atom = KnowledgeAtom::default();
    atom.id = "recall-http".into();
    atom.source = "agent".into();
    atom.source_id = "/recall-http".into();
    atom.title = "Recall HTTP".into();
    atom.content = "KNOWN_PHRASE_KURULTAI_42 recall project detail for http test".into();
    atom.tags = vec!["proj".into()];
    atom.metadata.insert("project_id".into(), "acme".into());
    brain.store().upsert(&atom).await.unwrap();

    let app = app(brain, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/recall")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"project":"acme","query":"KNOWN_PHRASE_KURULTAI_42","limit":10}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let views: Vec<kurultai::brain::AgentAtomView> =
        serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(views.iter().any(|v| v.id == "recall-http"));
}

/// Negative case: recall scoped to a namespace with no matching atoms returns
/// an empty list rather than leaking another session's hits.
#[tokio::test]
async fn http_recall_does_not_leak_other_projects() {
    let brain = fixture_brain().await;
    use kurultai::types::KnowledgeAtom;
    let mut atom = KnowledgeAtom::default();
    atom.id = "recall-acme-only".into();
    atom.source = "agent".into();
    atom.source_id = "/recall-acme-only".into();
    atom.title = "Acme Only".into();
    atom.content = "KNOWN_PHRASE_KURULTAI_42 recall project detail for http leak test".into();
    atom.metadata.insert("project_id".into(), "acme".into());
    brain.store().upsert(&atom).await.unwrap();

    let app = app(brain, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/recall")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"project":"crew-yam","query":"KNOWN_PHRASE_KURULTAI_42","limit":10}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let views: Vec<kurultai::brain::AgentAtomView> =
        serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(
        !views.iter().any(|v| v.id == "recall-acme-only"),
        "acme atom must not appear in crew-yam recall"
    );
    assert!(
        views.iter().all(|v| v.project == "crew-yam"),
        "every hit must belong to the requested project: {views:?}"
    );
}

/// `project` is optional; omitting it falls back to the "default" namespace
/// rather than 400-ing or spanning every project.
#[tokio::test]
async fn http_recall_without_project_uses_default_namespace() {
    let brain = fixture_brain().await;
    let app = app(brain, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/recall")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"query":"KNOWN_PHRASE_KURULTAI_42","limit":5}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let views: Vec<kurultai::brain::AgentAtomView> =
        serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(views.iter().all(|v| v.project == "default"));
}

// ── /api/ontology ────────────────────────────────────────────────────────────

#[tokio::test]
async fn http_api_ontology_returns_classes_and_links() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/ontology")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(v["ok"], true);
    let entities = v["entities"].as_array().unwrap();
    assert!(entities.len() >= 6);
    let ids: Vec<&str> = entities.iter().filter_map(|e| e["id"].as_str()).collect();
    assert!(ids.contains(&"class:memory"));
    assert!(ids.contains(&"class:note"));
    assert!(v["links"].as_array().unwrap().len() >= 5);
}

// ── /api/graph ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn http_api_graph_invalid_tier_is_400() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/graph?tier=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert!(!v["request_id"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn http_api_graph_returns_nodes() {
    let app = app(fixture_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/graph?limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_str(&body_str(resp).await).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["count"].as_u64().unwrap_or(0) > 0);
}

// ── Brain UI ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn brain_ui_route_redirects_to_slash() {
    // /ui is a redirect to /ui/ (the embedded SPA shell).
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(Request::builder().uri("/ui").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(
        resp.status().is_redirection(),
        "/ui must redirect to /ui/: got {}",
        resp.status()
    );
}

#[tokio::test]
async fn brain_ui_index_route_responds_ok() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── TA-8: Hub API-key auth middleware ────────────────────────────────────────

fn gate_with_key(key: &str) -> HubGate {
    HubGate {
        auth: HubAuth::ApiKey,
        api_keys: vec![key.into()],
        #[cfg(feature = "postgres")]
        key_store: None,
    }
}

#[tokio::test]
async fn hub_auth_blocks_api_without_bearer() {
    let app = app(empty_brain().await, gate_with_key("secret-123"));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn hub_auth_allows_api_with_valid_bearer() {
    let app = app(empty_brain().await, gate_with_key("secret-123"));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .header("authorization", "Bearer secret-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn hub_auth_allows_health_without_token() {
    let app = app(empty_brain().await, gate_with_key("secret-123"));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn hub_auth_blocks_unprefixed_query_routes_without_token() {
    let brain = empty_brain().await;
    let app = app(brain, gate_with_key("secret-123"));

    let routes = [
        ("GET", "/search?q=test"),
        ("POST", "/search"),
        ("GET", "/ask?question=test"),
        ("POST", "/ask"),
        ("POST", "/cite"),
        ("POST", "/who_knows"),
    ];

    for (method, uri) in routes {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "route {method} {uri} must require hub authentication"
        );
    }
}

#[tokio::test]
async fn hub_auth_allows_unprefixed_query_routes_with_token() {
    let app = app(empty_brain().await, gate_with_key("secret-123"));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/search?q=test")
                .header("authorization", "Bearer secret-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn hub_auth_rejects_wrong_token() {
    let app = app(empty_brain().await, gate_with_key("secret-123"));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .header("authorization", "Bearer wrong")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn hub_auth_accepts_sha256_hashed_key() {
    // Keys may be stored as sha256 hex; the middleware accepts either form.
    use kurultai::hashutil::sha256_hex;
    let plain = "plain-token";
    let hashed = sha256_hex(plain);
    let app = app(
        empty_brain().await,
        HubGate {
            auth: HubAuth::ApiKey,
            api_keys: vec![hashed],
            #[cfg(feature = "postgres")]
            key_store: None,
        },
    );
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .header("authorization", &format!("Bearer {plain}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── /api/promote gate ────────────────────────────────────────────────────────

#[tokio::test]
async fn http_promote_missing_atom_is_400() {
    let app = app(empty_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/promote")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"atom_id":"nope"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── /api/touch ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn http_touch_missing_atom_is_404() {
    let app = app(fixture_brain().await, HubGate::default());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/touch")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"atom_id":"does-not-exist"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
