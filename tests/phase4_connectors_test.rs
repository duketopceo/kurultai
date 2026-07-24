//! Phase 4: Dayflow fixture index → FTS hit.

use kurultai::connectors::dayflow::DayflowConnector;
use kurultai::connectors::Connector;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::mcp::interface::AgentRead;
use kurultai::mcp::BrainService;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;

fn dayflow_fixture_db() -> std::path::PathBuf {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chunks.sqlite");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE timeline_cards (
            id INTEGER PRIMARY KEY,
            start TEXT NOT NULL,
            end TEXT NOT NULL,
            start_ts INTEGER,
            end_ts INTEGER,
            day DATE NOT NULL,
            title TEXT NOT NULL,
            summary TEXT,
            category TEXT NOT NULL,
            subcategory TEXT,
            detailed_summary TEXT,
            metadata TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0
        );
        INSERT INTO timeline_cards
          (id, start, end, start_ts, day, title, summary, category, detailed_summary)
        VALUES
          (7, '1:00 PM', '2:00 PM', 1700000000, '2023-11-14',
           'Dayflow CI debug', 'summary', 'Work',
           'KNOWN_DAYFLOW_PHRASE_88 fixed the pipeline');",
    )
    .unwrap();
    // Keep dir alive
    std::mem::forget(dir);
    path
}

#[tokio::test]
async fn phase4_dayflow_index_searchable() {
    let db_path = dayflow_fixture_db();
    let store_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(store_dir.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let mut connector = DayflowConnector::new();
    let mut extra = HashMap::new();
    extra.insert("db_path".into(), db_path.to_string_lossy().into_owned());
    connector
        .init(&SourceConfig {
            name: "activity".into(),
            kind: SourceKind::Dayflow,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();
    pipeline
        .index_connector("activity", &connector, true)
        .await
        .unwrap();

    let brain = BrainService::new(
        store,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    );
    let hits = brain.search("KNOWN_DAYFLOW_PHRASE_88", 5).await.unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().any(|h| h.atom.source == "activity"));
}
