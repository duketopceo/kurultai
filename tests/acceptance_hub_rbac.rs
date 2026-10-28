#![cfg(feature = "postgres")]
#![allow(clippy::field_reassign_with_default)]
//! Acceptance tests — hub RBAC on search/ask (#115, slice 1).
//!
//! Drives the real axum router over a SQLite-backed brain and injects a
//! [`HubPrincipal`] the way `hub_api_auth` does for an issued key. Team
//! isolation is therefore proven by the Rust-side lane enforcement, not by
//! Postgres, so these tests need no database.
//!
//! Corpus: an `eng` team atom, a `sales` team atom, a `company` atom and a
//! `personal` atom. The sales atom is reachable from the eng atom only through
//! the shared tag `runbook` (an ask multi-hop), never by the query text.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::Router;
use chrono::Utc;
use kurultai::daemon::DaemonStatus;
use kurultai::embed::NullEmbedder;
use kurultai::http::{build_app, HubGate};
use kurultai::hub::{HubPrincipal, HubRole};
use kurultai::mcp::brain::BrainService;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{KnowledgeAtom, VisibilityScope};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

const ENG_ID: &str = "eng-1";
const SALES_ID: &str = "sales-1";
const CO_ID: &str = "co-1";
const PERSONAL_ID: &str = "personal-1";

fn atom(
    id: &str,
    vis: VisibilityScope,
    team: Option<&str>,
    tags: &[&str],
    body: &str,
) -> KnowledgeAtom {
    let mut a = KnowledgeAtom {
        id: id.into(),
        source: "hubtest".into(),
        source_id: format!("/{id}"),
        title: format!("Title {id}"),
        summary: body.into(),
        content: body.into(),
        tags: tags.iter().map(|t| t.to_string()).collect(),
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        visibility: vis,
        ..Default::default()
    };
    if let Some(t) = team {
        a.metadata.insert("team_id".into(), t.into());
    }
    a
}

async fn brain() -> BrainService {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "hub-rbac-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap());
    for a in [
        atom(
            ENG_ID,
            VisibilityScope::Team,
            Some("eng"),
            &["runbook"],
            "zebrafish rollout procedure for the engineering team",
        ),
        atom(
            SALES_ID,
            VisibilityScope::Team,
            Some("sales"),
            &["runbook"],
            "SALESSECRET confidential runbook with the discount ladder for enterprise deals",
        ),
        atom(
            CO_ID,
            VisibilityScope::Company,
            None,
            &["handbook"],
            "zebrafish company handbook visible to every team",
        ),
        atom(
            PERSONAL_ID,
            VisibilityScope::Personal,
            None,
            &["private"],
            "zebrafish private diary entry that must never reach a hub caller",
        ),
    ] {
        store.upsert(&a).await.unwrap();
    }
    BrainService::new(
        store,
        Arc::new(NullEmbedder::new(4)),
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    )
}

/// Router that behaves as if `hub_api_auth` had resolved `principal`.
async fn app_as(principal: Option<HubPrincipal>) -> Router {
    let app = build_app(
        brain().await,
        Arc::new(DaemonStatus::default()),
        HubGate::default(),
    );
    app.layer(axum::middleware::from_fn(
        move |mut req: Request, next: Next| {
            let principal = principal.clone();
            async move {
                if let Some(p) = principal {
                    req.extensions_mut().insert(p);
                }
                next.run(req).await
            }
        },
    ))
}

fn key(team: &str, role: HubRole) -> Option<HubPrincipal> {
    Some(HubPrincipal {
        key_id: 1,
        agent_id: format!("agent-{team}"),
        team_id: team.into(),
        key_prefix: "krt_test".into(),
        role,
    })
}

async fn call(app: &Router, method: Method, uri: &str, body: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let resp = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

// ── Roles ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn viewer_can_read_but_not_write_or_promote() {
    let app = app_as(key("eng", HubRole::Viewer)).await;
    let (s, _) = call(&app, Method::GET, "/api/search?q=zebrafish", None).await;
    assert_eq!(s, StatusCode::OK, "viewer may search");
    let (s, _) = call(
        &app,
        Method::POST,
        "/ask",
        Some(r#"{"question":"zebrafish"}"#),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "viewer may ask (read-only POST)");
    let (s, _) = call(&app, Method::GET, "/api/atoms", None).await;
    assert_eq!(s, StatusCode::OK, "viewer may list");

    let (s, body) = call(
        &app,
        Method::POST,
        "/api/hey/threads",
        Some(r#"{"name":"t","turn_cap":10}"#),
    )
    .await;
    assert_eq!(
        s,
        StatusCode::FORBIDDEN,
        "viewer cannot write board: {body}"
    );
    assert!(
        body.contains("contributor"),
        "names the needed role: {body}"
    );
    let (s, _) = call(
        &app,
        Method::POST,
        "/api/promote",
        Some(&format!(r#"{{"atom_id":"{ENG_ID}"}}"#)),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "viewer cannot promote");
}

#[tokio::test]
async fn contributor_writes_but_cannot_promote_or_read_activity() {
    let app = app_as(key("eng", HubRole::Contributor)).await;
    let (s, body) = call(
        &app,
        Method::POST,
        "/api/hey/threads",
        Some(r#"{"name":"t","turn_cap":10}"#),
    )
    .await;
    assert_ne!(
        s,
        StatusCode::FORBIDDEN,
        "contributor passes role guard: {body}"
    );
    let (s, _) = call(
        &app,
        Method::POST,
        "/api/promote",
        Some(&format!(r#"{{"atom_id":"{ENG_ID}"}}"#)),
    )
    .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "promote is admin-only");
    let (s, _) = call(&app, Method::GET, "/api/hub/activity", None).await;
    assert_eq!(s, StatusCode::FORBIDDEN, "activity log is admin-only");
}

#[tokio::test]
async fn admin_passes_role_guard_on_promote() {
    let app = app_as(key("eng", HubRole::Admin)).await;
    let (s, body) = call(
        &app,
        Method::POST,
        "/api/promote",
        Some(&format!(r#"{{"atom_id":"{ENG_ID}"}}"#)),
    )
    .await;
    assert_ne!(s, StatusCode::FORBIDDEN, "admin passes role guard: {body}");
}

#[tokio::test]
async fn admin_cannot_promote_or_touch_another_teams_atom() {
    let app = app_as(key("eng", HubRole::Admin)).await;
    for uri in ["/api/promote", "/api/touch"] {
        let (s, body) = call(
            &app,
            Method::POST,
            uri,
            Some(&format!(r#"{{"atom_id":"{SALES_ID}"}}"#)),
        )
        .await;
        assert_eq!(
            s,
            StatusCode::NOT_FOUND,
            "{uri} out-of-lane id looks missing: {body}"
        );
        assert!(
            !body.contains("SALESSECRET"),
            "{uri} leaked content: {body}"
        );
    }
    let (s, _) = call(
        &app,
        Method::POST,
        "/api/touch",
        Some(&format!(r#"{{"atom_id":"{ENG_ID}"}}"#)),
    )
    .await;
    assert_eq!(s, StatusCode::OK, "own-team atom still touchable");
}

// ── Lane scoping on every read path ───────────────────────────────────────────

#[tokio::test]
async fn search_respects_lane() {
    let app = app_as(key("eng", HubRole::Viewer)).await;
    let (s, body) = call(&app, Method::GET, "/api/search?q=zebrafish&limit=10", None).await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains(ENG_ID), "own team visible: {body}");
    assert!(body.contains(CO_ID), "company visible: {body}");
    assert!(!body.contains(PERSONAL_ID), "personal hidden: {body}");
    assert!(!body.contains(SALES_ID), "other team hidden: {body}");
}

#[tokio::test]
async fn ask_multi_hop_does_not_pull_other_teams_atoms() {
    // Control: with no hub principal the hop reaches the sales atom via the
    // shared `runbook` tag — proving the fixture exercises the second hop.
    let solo = app_as(None).await;
    let (_, body) = call(
        &solo,
        Method::POST,
        "/ask",
        Some(r#"{"question":"zebrafish rollout"}"#),
    )
    .await;
    assert!(
        body.contains(SALES_ID),
        "fixture must reach sales atom via hop when unscoped: {body}"
    );

    let app = app_as(key("eng", HubRole::Viewer)).await;
    let (s, body) = call(
        &app,
        Method::POST,
        "/ask",
        Some(r#"{"question":"zebrafish rollout"}"#),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        !body.contains("SALESSECRET"),
        "hop leaked sales content: {body}"
    );
    assert!(!body.contains(SALES_ID), "hop leaked sales atom: {body}");
    assert!(!body.contains(PERSONAL_ID), "personal leaked: {body}");
}

#[tokio::test]
async fn list_atoms_respects_lane() {
    let app = app_as(key("eng", HubRole::Viewer)).await;
    let (s, body) = call(&app, Method::GET, "/api/atoms?limit=50", None).await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains(ENG_ID) && body.contains(CO_ID), "{body}");
    assert!(
        !body.contains(SALES_ID) && !body.contains(PERSONAL_ID),
        "{body}"
    );
}

#[tokio::test]
async fn recall_respects_lane() {
    let app = app_as(key("eng", HubRole::Viewer)).await;
    let (s, body) = call(
        &app,
        Method::POST,
        "/api/recall",
        Some(r#"{"query":"zebrafish","limit":10}"#),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        !body.contains("SALESSECRET") && !body.contains("diary"),
        "{body}"
    );
}

#[tokio::test]
async fn who_knows_respects_lane() {
    let app = app_as(key("eng", HubRole::Viewer)).await;
    let (s, body) = call(
        &app,
        Method::POST,
        "/who_knows",
        Some(r#"{"topic":"confidential discount","limit":5}"#),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(
        !body.contains("SALESSECRET") && !body.contains(SALES_ID),
        "{body}"
    );
}

#[tokio::test]
async fn cite_respects_lane() {
    let app = app_as(key("eng", HubRole::Viewer)).await;
    let cite = |id: &str| format!(r#"{{"source":"hubtest","source_id":"/{id}"}}"#);
    let (_, own) = call(&app, Method::POST, "/cite", Some(&cite(ENG_ID))).await;
    assert!(own.contains(ENG_ID), "own atom citable: {own}");
    for hidden in [SALES_ID, PERSONAL_ID] {
        let (s, body) = call(&app, Method::POST, "/cite", Some(&cite(hidden))).await;
        assert_eq!(s, StatusCode::OK);
        assert_eq!(body.trim(), "null", "{hidden} must read as a miss: {body}");
    }
}

#[tokio::test]
async fn graph_and_db_browse_fail_closed_for_hub_keys() {
    let app = app_as(key("eng", HubRole::Admin)).await;
    for uri in ["/api/graph", "/api/db/atoms"] {
        let (s, body) = call(&app, Method::GET, uri, None).await;
        assert_eq!(s, StatusCode::FORBIDDEN, "{uri}: {body}");
        assert!(!body.contains("SALESSECRET"), "{uri}: {body}");
    }
}

// ── Solo path unchanged (AE1) ─────────────────────────────────────────────────

#[tokio::test]
async fn no_principal_sees_everything_and_is_never_role_checked() {
    let app = app_as(None).await;
    let (s, body) = call(&app, Method::GET, "/api/atoms?limit=50", None).await;
    assert_eq!(s, StatusCode::OK);
    for id in [ENG_ID, SALES_ID, CO_ID, PERSONAL_ID] {
        assert!(body.contains(id), "solo sees {id}: {body}");
    }
    let (s, _) = call(&app, Method::GET, "/api/graph", None).await;
    assert_eq!(s, StatusCode::OK, "solo graph unchanged");
    let (s, body) = call(
        &app,
        Method::POST,
        "/api/hey/threads",
        Some(r#"{"name":"t","turn_cap":10}"#),
    )
    .await;
    assert_ne!(
        s,
        StatusCode::FORBIDDEN,
        "no role check without a key: {body}"
    );
    let (s, body) = call(
        &app,
        Method::POST,
        "/cite",
        Some(&format!(
            r#"{{"source":"hubtest","source_id":"/{SALES_ID}"}}"#
        )),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert!(body.contains(SALES_ID), "solo cite unscoped: {body}");
}
