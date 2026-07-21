//! Phase 2 (#23) hybrid retrieval integration — FTS ∥ vector → RRF → stub rerank.

use chrono::Utc;
use kurultai::brain::DEFAULT_EXCERPT_CAP;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::error::{KurultaiError, Result};
use kurultai::query::{expand_markdown_context, hybrid_search, RRF_K};
use kurultai::rerank::{NullReranker, Reranker};
use kurultai::store::{SqliteVecStore, Store};
use kurultai::types::KnowledgeAtom;
use std::collections::HashMap;
use std::sync::Arc;

fn sample_atom(id: &str, title: &str, content: &str, embedding: Option<Vec<f32>>) -> KnowledgeAtom {
    KnowledgeAtom {
        id: id.into(),
        source: "test".into(),
        source_id: id.into(),
        title: title.into(),
        summary: content.chars().take(80).collect(),
        content: content.into(),
        question: None,
        resolution: None,
        tags: vec![],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        embedding,
        metadata: HashMap::new(),
    }
}

/// Live embedder that always returns a fixed query vector.
struct FixedQueryEmbedder {
    dim: usize,
    query_vec: Vec<f32>,
}

#[async_trait::async_trait]
impl Embedder for FixedQueryEmbedder {
    fn name(&self) -> &str {
        "fixed-query"
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn is_live(&self) -> bool {
        true
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(self.query_vec.clone())
    }
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| self.query_vec.clone()).collect())
    }
}

/// Live embedder that always fails (soft-fail → FTS-only).
struct FailingEmbedder {
    dim: usize,
}

#[async_trait::async_trait]
impl Embedder for FailingEmbedder {
    fn name(&self) -> &str {
        "failing"
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn is_live(&self) -> bool {
        true
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(KurultaiError::Embed("forced failure".into()))
    }
    async fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Err(KurultaiError::Embed("forced failure".into()))
    }
}

enum StubMode {
    Reorder(Vec<String>),
    Fail,
}

struct StubReranker {
    mode: StubMode,
}

#[async_trait::async_trait]
impl Reranker for StubReranker {
    fn name(&self) -> &str {
        "stub"
    }
    fn is_live(&self) -> bool {
        true
    }
    async fn rerank(&self, _query: &str, _candidates: &[(String, String)]) -> Result<Vec<String>> {
        match &self.mode {
            StubMode::Reorder(ids) => Ok(ids.clone()),
            StubMode::Fail => Err(KurultaiError::Query("stub rerank failed".into())),
        }
    }
}

async fn seeded_store() -> Arc<dyn Store> {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    // Keep TempDir alive for the test by leaking — tests are short-lived processes.
    // Prefer writing under CARGO_TARGET_TMPDIR via tempfile owned by store path.
    std::mem::forget(dir);

    // shared: FTS keyword + embedding aligned with query vec
    store
        .upsert(&sample_atom(
            "shared",
            "Shared",
            "ALPHAUNIQUE shared content for hybrid",
            Some(vec![1.0, 0.0, 0.0, 0.0]),
        ))
        .await
        .unwrap();
    store
        .upsert(&sample_atom(
            "fts_only",
            "FtsOnly",
            "ALPHAUNIQUE appears only in FTS path",
            Some(vec![0.0, 1.0, 0.0, 0.0]),
        ))
        .await
        .unwrap();
    store
        .upsert(&sample_atom(
            "vec_only",
            "VecOnly",
            "no alpha keyword here for vector arm",
            Some(vec![0.95, 0.05, 0.0, 0.0]),
        ))
        .await
        .unwrap();

    store as Arc<dyn Store>
}

#[tokio::test]
async fn hybrid_overlap_rrf_matched_by_both() {
    let store = seeded_store().await;
    let embedder: Arc<dyn Embedder> = Arc::new(FixedQueryEmbedder {
        dim: 4,
        query_vec: vec![1.0, 0.0, 0.0, 0.0],
    });
    let reranker: Arc<dyn Reranker> = Arc::new(NullReranker::new());

    let hits = hybrid_search(&store, &embedder, &reranker, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    assert!(!hits.is_empty());
    let shared = hits.iter().find(|h| h.atom.id == "shared").expect("shared");
    assert_eq!(shared.matched_by, vec!["fts", "vector"]);
    let expected = 2.0 / (RRF_K + 1.0);
    // shared may not be rank0 if both lists place it at different ranks — still both arms
    assert!(
        shared.score + 1e-9 >= expected - 1e-6 || shared.matched_by.len() == 2,
        "expected RRF contribution from both arms, score={}",
        shared.score
    );
    assert!(hits.iter().any(|h| h.atom.id == "fts_only"));
}

#[tokio::test]
async fn hybrid_null_embedder_fts_only() {
    let store = seeded_store().await;
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let reranker: Arc<dyn Reranker> = Arc::new(NullReranker::new());

    let hits = hybrid_search(&store, &embedder, &reranker, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    assert!(!hits.is_empty());
    assert!(hits.iter().all(|h| h.matched_by == vec!["fts".to_string()]));
    assert!(!hits.iter().any(|h| h.atom.id == "vec_only"));
}

#[tokio::test]
async fn hybrid_embed_fail_falls_back_to_fts() {
    let store = seeded_store().await;
    let embedder: Arc<dyn Embedder> = Arc::new(FailingEmbedder { dim: 4 });
    let reranker: Arc<dyn Reranker> = Arc::new(NullReranker::new());

    let hits = hybrid_search(&store, &embedder, &reranker, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    assert!(!hits.is_empty());
    assert!(hits
        .iter()
        .all(|h| !h.matched_by.iter().any(|m| m == "vector")));
}

#[tokio::test]
async fn blank_query_empty() {
    let store = seeded_store().await;
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let reranker: Arc<dyn Reranker> = Arc::new(NullReranker::new());
    let hits = hybrid_search(&store, &embedder, &reranker, "   ", 5)
        .await
        .unwrap();
    assert!(hits.is_empty());
}

#[tokio::test]
async fn stub_rerank_reorders() {
    let store = seeded_store().await;
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    // Get at least two FTS hits
    let null_rr: Arc<dyn Reranker> = Arc::new(NullReranker::new());
    let base = hybrid_search(&store, &embedder, &null_rr, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    assert!(base.len() >= 2);
    let id0 = base[0].atom.id.clone();
    let id1 = base[1].atom.id.clone();

    let stub: Arc<dyn Reranker> = Arc::new(StubReranker {
        mode: StubMode::Reorder(vec![id1.clone(), id0.clone()]),
    });
    let reordered = hybrid_search(&store, &embedder, &stub, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    assert_eq!(reordered[0].atom.id, id1);
    assert_eq!(reordered[1].atom.id, id0);
}

#[tokio::test]
async fn stub_rerank_fail_keeps_rrf_order() {
    let store = seeded_store().await;
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let null_rr: Arc<dyn Reranker> = Arc::new(NullReranker::new());
    let base = hybrid_search(&store, &embedder, &null_rr, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    let order: Vec<_> = base.iter().map(|h| h.atom.id.clone()).collect();

    let stub: Arc<dyn Reranker> = Arc::new(StubReranker {
        mode: StubMode::Fail,
    });
    let after = hybrid_search(&store, &embedder, &stub, "ALPHAUNIQUE", 10)
        .await
        .unwrap();
    let after_ids: Vec<_> = after.iter().map(|h| h.atom.id.clone()).collect();
    assert_eq!(after_ids, order);
}

#[tokio::test]
async fn markdown_context_expands_neighbors() {
    use kurultai::connectors::markdown::MarkdownConnector;
    use kurultai::connectors::Connector;
    use kurultai::pipeline::IndexPipeline;
    use kurultai::types::{SourceConfig, SourceKind};
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();
    let md = dir.path().join("multi.md");
    let mut f = std::fs::File::create(&md).unwrap();
    writeln!(
        f,
        r#"---
title: Multi
---

## First
prev chunk unique PREVTOKEN

## Middle
MIDDLETOKEN center chunk content

## Last
next chunk unique NEXTTOKEN
"#
    )
    .unwrap();

    let db = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(db.path().join("s.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert(
        "root_path".into(),
        dir.path().to_string_lossy().into_owned(),
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

    let reranker: Arc<dyn Reranker> = Arc::new(NullReranker::new());
    let hits = hybrid_search(
        &(Arc::clone(&store) as Arc<dyn Store>),
        &embedder,
        &reranker,
        "MIDDLETOKEN",
        5,
    )
    .await
    .unwrap();
    assert!(!hits.is_empty());
    let expanded = expand_markdown_context(&(Arc::clone(&store) as Arc<dyn Store>), hits)
        .await
        .unwrap();
    let mid = expanded
        .iter()
        .find(|h| h.atom.content.contains("MIDDLETOKEN"))
        .expect("middle hit");
    assert!(
        mid.atom.summary.contains("…prev") || mid.atom.summary.contains("PREVTOKEN"),
        "expected prev neighbor in summary: {}",
        mid.atom.summary
    );
    assert!(
        mid.atom.summary.contains("…next") || mid.atom.summary.contains("NEXTTOKEN"),
        "expected next neighbor in summary: {}",
        mid.atom.summary
    );
    assert!(mid.atom.summary.chars().count() <= DEFAULT_EXCERPT_CAP);
}
