//! Phase 3 WO2 — HTTP daemon against fixture vault.

use assert_cmd::Command;
use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::http;
use kurultai::mcp::BrainService;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

async fn brain_with_fixture() -> Arc<BrainService> {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    std::mem::forget(dir);
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));
    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
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
    Arc::new(BrainService::new(
        store,
        embedder,
        Arc::new(NullReranker::new()),
    ))
}

#[tokio::test]
async fn http_search_ask_against_fixture() {
    let brain = brain_with_fixture().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        http::serve_listener(brain, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let client = reqwest::Client::new();
    let health: serde_json::Value = client
        .get(format!("http://{addr}/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(health["status"], "ok");

    let views: serde_json::Value = client
        .get(format!(
            "http://{addr}/v1/search?q=KNOWN_PHRASE_KURULTAI_42&limit=5"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!views.as_array().unwrap().is_empty());
    assert!(views[0]["excerpt"].as_str().unwrap().chars().count() <= 400);

    let answer: serde_json::Value = client
        .post(format!("http://{addr}/v1/ask"))
        .json(&serde_json::json!({"question": "KNOWN_PHRASE_KURULTAI_42"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(answer.get("answer").is_some());
    assert!(answer.get("citations").is_some());

    let miss = client
        .get(format!("http://{addr}/v1/cite?source=notes&source_id=nope"))
        .send()
        .await
        .unwrap();
    assert_eq!(miss.status(), 404);
}

#[test]
fn daemon_cli_help_lists_bind() {
    Command::cargo_bin("kurultai")
        .unwrap()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("--bind"))
        .stdout(predicates::str::contains("8421"));
}
