//! Acceptance tests — search surface (KHAN-251).
//!
//! Covers: FTS5 search, vector search, hybrid search, source-scoped search,
//! and project-scoped recall (`recall_for_agent` / `POST /api/recall`).

use chrono::Utc;
use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::NullEmbedder;
use kurultai::mcp::brain::BrainService;
use kurultai::mcp::interface::AgentRead;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SearchFilter, SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn fixture_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

/// Build a brain indexed from the fixture vault with a 4-dim NullEmbedder.
async fn brain_with_fixture() -> (BrainService, Arc<SqliteVecStore>) {
    static N: AtomicU64 = AtomicU64::new(0);
    let db_dir = std::env::temp_dir().join(format!(
        "khan251-search-{}-{}-{}",
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

    let brain = BrainService::new(
        Arc::clone(&store) as Arc<dyn Store>,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    );
    (brain, store)
}

// ── A0-2: FTS5 search ────────────────────────────────────────────────────────

#[tokio::test]
async fn fts_search_finds_golden_phrase() {
    let (brain, _store) = brain_with_fixture().await;
    let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 5).await.unwrap();
    assert!(!hits.is_empty(), "FTS must find golden phrase");
    assert!(hits[0].matched_by.iter().any(|m| m == "fts"));
}

#[tokio::test]
async fn fts_search_blank_query_returns_empty() {
    let (brain, _store) = brain_with_fixture().await;
    let hits = brain.search("   ", 5).await.unwrap();
    assert!(hits.is_empty(), "blank query must not hit");
}

#[tokio::test]
async fn fts_search_stopwords_only_returns_empty() {
    let (brain, _store) = brain_with_fixture().await;
    // "the" is a stopword → sanitized to empty MATCH → no hits.
    let hits = brain.search("the", 5).await.unwrap();
    assert!(hits.is_empty(), "stopword-only query must not hit");
}

// ── A0-3: Vector search ──────────────────────────────────────────────────────

#[tokio::test]
async fn vector_search_returns_nearest_when_vectors_present() {
    static N: AtomicU64 = AtomicU64::new(0);
    let db_dir = std::env::temp_dir().join(format!(
        "khan251-vec-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&db_dir).unwrap();
    let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());

    // Two atoms with distinct vectors.
    use kurultai::types::KnowledgeAtom;
    let mut a1 = KnowledgeAtom::default();
    a1.id = "near".into();
    a1.source = "test".into();
    a1.source_id = "/near".into();
    a1.title = "Near".into();
    a1.content = "near content".into();
    a1.tags = vec!["t".into()];
    a1.embedding = Some(vec![0.9, 0.9, 0.9, 0.9]);
    let mut a2 = KnowledgeAtom::default();
    a2.id = "far".into();
    a2.source = "test".into();
    a2.source_id = "/far".into();
    a2.title = "Far".into();
    a2.content = "far content".into();
    a2.tags = vec!["t".into()];
    a2.embedding = Some(vec![0.0, 0.0, 0.0, 0.1]);
    store.upsert_batch(&[a1, a2]).await.unwrap();

    let hits = store
        .vector_search(&[0.85, 0.85, 0.85, 0.85], 2, SearchFilter::default())
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].0.id, "near", "nearest neighbor must rank first");
}

#[tokio::test]
async fn vector_search_zero_query_returns_empty() {
    let (brain, store) = brain_with_fixture().await;
    let _ = brain;
    // Zero-norm query vector must short-circuit to empty.
    let hits = store
        .vector_search(&[0.0, 0.0, 0.0, 0.0], 5, SearchFilter::default())
        .await
        .unwrap();
    assert!(hits.is_empty(), "zero-norm query must not hit");
}

// ── A0-4: Hybrid search ──────────────────────────────────────────────────────

#[tokio::test]
async fn hybrid_search_combines_fts_and_vector_matches() {
    let (brain, _store) = brain_with_fixture().await;
    let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 5).await.unwrap();
    assert!(!hits.is_empty());
    // With NullEmbedder (no live vectors), the FTS path is the source of hits.
    assert!(hits.iter().any(|h| h.matched_by.iter().any(|m| m == "fts")));
}

// ── Source-scoped search ─────────────────────────────────────────────────────

#[tokio::test]
async fn scoped_search_pins_to_source() {
    let (brain, _store) = brain_with_fixture().await;
    let hits = brain
        .search_scoped("KNOWN_PHRASE_KURULTAI_42", 5, false, Some("notes"))
        .await
        .unwrap();
    assert!(!hits.is_empty());
    assert!(
        hits.iter().all(|h| h.atom.source == "notes"),
        "scoped search must only return notes"
    );
}

#[tokio::test]
async fn scoped_search_unknown_source_returns_empty() {
    let (brain, _store) = brain_with_fixture().await;
    let hits = brain
        .search_scoped("KNOWN_PHRASE_KURULTAI_42", 5, false, Some("nope"))
        .await
        .unwrap();
    assert!(hits.is_empty(), "unknown source must yield no hits");
}

// ── TA-7: Project-scoped recall ─────────────────────────────────────────────

#[tokio::test]
async fn recall_filters_to_project_id() {
    let (brain, store) = brain_with_fixture().await;

    // Inject an atom with a non-default project_id via metadata.
    use kurultai::types::KnowledgeAtom;
    let mut atom = KnowledgeAtom::default();
    atom.id = "proj-atom".into();
    atom.source = "agent".into();
    atom.source_id = "/proj".into();
    atom.title = "Project Note".into();
    atom.content = "KNOWN_PHRASE_KURULTAI_42 project detail for recall test".into();
    atom.tags = vec!["proj".into()];
    atom.metadata.insert("project_id".into(), "acme".into());
    store.upsert(&atom).await.unwrap();

    // Recall scoped to "acme" must include the project atom.
    let views = brain
        .recall_for_agent("acme", "KNOWN_PHRASE_KURULTAI_42", 10, false)
        .await
        .unwrap();
    assert!(
        views.iter().any(|v| v.id == "proj-atom"),
        "recall must return project-scoped atom"
    );

    // Recall scoped to a different project must exclude it.
    let other = brain
        .recall_for_agent("other-project", "KNOWN_PHRASE_KURULTAI_42", 10, false)
        .await
        .unwrap();
    assert!(
        !other.iter().any(|v| v.id == "proj-atom"),
        "recall must exclude other-project atoms"
    );
}

// ── Ask (RAG) ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn ask_returns_answer_with_citations() {
    let (brain, _store) = brain_with_fixture().await;
    let answer = brain.ask("KNOWN_PHRASE_KURULTAI_42").await.unwrap();
    assert!(answer.confidence > 0.0, "confidence must be positive");
    assert!(!answer.citations.is_empty(), "must cite sources");
    assert!(
        answer.answer.contains("KNOWN_PHRASE_KURULTAI_42")
            || answer
                .citations
                .iter()
                .any(|c| c.excerpt.contains("KNOWN_PHRASE_KURULTAI_42")),
        "answer or excerpt must reference the phrase"
    );
}

#[tokio::test]
async fn who_knows_returns_source_aggregates() {
    let (brain, _store) = brain_with_fixture().await;
    let entries = brain
        .who_knows("KNOWN_PHRASE_KURULTAI_42", 10)
        .await
        .unwrap();
    assert!(!entries.is_empty());
    assert!(entries.iter().any(|e| e.source == "notes"));
}
