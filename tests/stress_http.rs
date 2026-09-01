//! HTTP stress test — mixed read/write workload against the full Axum stack.
//!
//! Builds a small fixture brain and hammers the router with 1,000 concurrent
//! requests over 100 client sessions. Passes if every request returns 2xx and
//! the run stays under a generous ceiling.

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request};
use chrono::Utc;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::http::{build_app, HubAuth, HubGate};
use kurultai::mcp::BrainService;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{KnowledgeAtom, TrustLane, VisibilityScope};
use std::sync::Arc;
use std::time::Instant;
use tower::util::ServiceExt;

const CONCURRENCY: usize = 100;
const OPS_PER_CLIENT: usize = 10;
const TOTAL_OPS: usize = CONCURRENCY * OPS_PER_CLIENT;
const MAX_TOTAL_SECONDS: u64 = 30;

fn stress_atom(id: &str, content: &str) -> KnowledgeAtom {
    KnowledgeAtom {
        id: id.into(),
        source: "stress".into(),
        source_id: format!("/stress/{id}"),
        title: format!("Stress {id}"),
        summary: "stress summary".into(),
        content: content.into(),
        tags: vec!["stress".into()],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        visibility: VisibilityScope::Company,
        ..Default::default()
    }
}

async fn fixture_brain_app() -> (axum::Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));

    // Seed the store with quarantined atoms for /api/promote and trusted atoms
    // for reads, so every request has a fresh target and something to find.
    for i in 0..10 {
        let content = format!(
            "KURULTAI_STRESS_TOKEN search target {i}: the brain must handle concurrent search requests across many client sessions without locking or corrupting the shared store."
        );
        let atom = stress_atom(&format!("stress-{i}"), &content);
        store.upsert(&atom).await.unwrap();
    }
    for i in 0..TOTAL_OPS {
        let content = format!(
            "KURULTAI_STRESS_TOKEN promote target {i}: the system must run under heavy concurrency with many clients promoting atoms from quarantine to trusted via the http api."
        );
        let mut atom = stress_atom(&format!("promote-{i}"), &content);
        atom.trust_lane = TrustLane::Quarantine;
        atom.quarantine_reason = Some("stress seed".into());
        store.upsert(&atom).await.unwrap();
    }

    let brain = BrainService::new(
        store,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    );
    let app = build_app(
        brain,
        Arc::new(kurultai::daemon::DaemonStatus::default()),
        HubGate {
            auth: HubAuth::None,
            api_keys: vec![],
            #[cfg(feature = "postgres")]
            key_store: None,
        },
    );
    (app, dir)
}

fn request_for(idx: usize) -> Request<Body> {
    match idx % 6 {
        0 => Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap(),
        1 => Request::builder()
            .method(Method::GET)
            .uri("/api/status")
            .body(Body::empty())
            .unwrap(),
        2 => Request::builder()
            .method(Method::GET)
            .uri("/api/atoms?limit=5")
            .body(Body::empty())
            .unwrap(),
        3 => Request::builder()
            .method(Method::GET)
            .uri("/search?q=KURULTAI_STRESS_TOKEN&limit=5")
            .body(Body::empty())
            .unwrap(),
        4 => Request::builder()
            .method(Method::POST)
            .uri("/api/touch")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"atom_id":"stress-{}","reason":"stress"}}"#,
                idx % 10
            )))
            .unwrap(),
        _ => Request::builder()
            .method(Method::POST)
            .uri("/api/promote")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"atom_id":"promote-{}","reason":"stress"}}"#,
                idx
            )))
            .unwrap(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stress_http_mixed_workload() {
    let (app, _dir) = fixture_brain_app().await;
    let start = Instant::now();

    let mut handles = Vec::with_capacity(CONCURRENCY);
    for client in 0..CONCURRENCY {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            let mut ok = 0;
            for round in 0..OPS_PER_CLIENT {
                let idx = client * OPS_PER_CLIENT + round;
                let resp = app
                    .clone()
                    .oneshot(request_for(idx))
                    .await
                    .expect("request must not panic");
                if resp.status().is_success() {
                    ok += 1;
                } else {
                    let status = resp.status();
                    let bytes = to_bytes(resp.into_body(), 1024 * 1024)
                        .await
                        .unwrap_or_default();
                    let text = String::from_utf8_lossy(&bytes);
                    panic!("client {client} idx {idx} got {status}: {text}");
                }
            }
            ok
        }));
    }

    let mut total = 0;
    for h in handles {
        total += h.await.expect("client task panicked");
    }

    let elapsed = start.elapsed();
    let rps = total as f64 / elapsed.as_secs_f64();
    eprintln!("stress_http: {total} ok in {elapsed:?} ({rps:.1} rps)");

    assert_eq!(total, TOTAL_OPS, "all requests must succeed");
    assert!(
        elapsed.as_secs() <= MAX_TOTAL_SECONDS,
        "stress took {elapsed:?}, must finish in {MAX_TOTAL_SECONDS}s"
    );
}
