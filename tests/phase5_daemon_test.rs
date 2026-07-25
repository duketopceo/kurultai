//! Phase 5: daemon poll_once indexes without full sync.

use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::{Connector, ConnectorRegistry};
use kurultai::daemon::poll_once;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::mcp::interface::AgentRead;
use kurultai::mcp::BrainService;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn phase5_poll_once_makes_fixture_searchable() {
    let root = format!("{}/tests/fixtures/vault", env!("CARGO_MANIFEST_DIR"));
    let mut connector = MarkdownConnector::new();
    connector
        .init(&SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([("root_path".into(), root)]),
        })
        .await
        .unwrap();

    let mut registry = ConnectorRegistry::new();
    registry.register("notes".into(), Box::new(connector));

    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let n = poll_once(&pipeline, &registry).await.unwrap();
    assert_eq!(n, 1);

    let brain = BrainService::new(
        store,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    );
    let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 10).await.unwrap();
    assert!(!hits.is_empty(), "expected FTS hits after daemon poll_once");
}
