//! Phase 3 integration: extractive ask + who_knows on fixture vault.

use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::mcp::interface::AgentRead;
use kurultai::mcp::BrainService;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(1);

struct FixtureBrain {
    brain: BrainService,
    /// Keep the temp DB directory alive for the brain's lifetime.
    _db_dir: tempfile::TempDir,
}

async fn brain_with_fixture() -> FixtureBrain {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let db_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(db_dir.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
    // Unique source name avoids any cross-test FTS bleed if paths ever collide.
    let source_name = format!(
        "notes-{}",
        FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    connector
        .init(&SourceConfig {
            name: source_name.clone(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();
    pipeline
        .index_connector(&source_name, &connector, true)
        .await
        .unwrap();

    FixtureBrain {
        brain: BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        ),
        _db_dir: db_dir,
    }
}

#[tokio::test]
async fn phase3_ask_extractive_fixture() {
    let fx = brain_with_fixture().await;
    let answer = fx.brain.ask("KNOWN_PHRASE_KURULTAI_42").await.unwrap();
    assert!(answer.confidence > 0.0);
    assert!(!answer.citations.is_empty());
    assert!(
        answer.answer.contains("KNOWN_PHRASE_KURULTAI_42")
            || answer
                .citations
                .iter()
                .any(|c| c.excerpt.contains("KNOWN_PHRASE_KURULTAI_42")
                    || c.title.contains("KNOWN_PHRASE"))
    );
}

#[tokio::test]
async fn phase3_ask_empty_store() {
    let db_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(db_dir.path().join("store.db"), 4).unwrap());
    let brain = BrainService::new(
        store,
        Arc::new(NullEmbedder::new(4)),
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    );
    let answer = brain.ask("nothing here").await.unwrap();
    assert_eq!(answer.confidence, 0.0);
    assert!(answer.citations.is_empty());
    assert!(answer.answer.contains("No indexed"));
}

#[tokio::test]
async fn phase3_who_knows_fixture() {
    let fx = brain_with_fixture().await;
    let entries = fx
        .brain
        .who_knows("KNOWN_PHRASE_KURULTAI_42", 10)
        .await
        .unwrap();
    assert!(!entries.is_empty());
    assert!(entries.iter().any(|e| e.source.starts_with("notes")));
}
