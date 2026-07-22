//! Brain facade — AgentRead / AgentWrite over the SQLite store.

use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use crate::hashutil::atom_id;
use crate::mcp::interface::{AgentRead, AgentWrite};
use crate::query::{
    citation_from_atom, compose_answer, expand_markdown_context, hybrid_search, Synthesizer,
};
use crate::rerank::Reranker;
use crate::store::Store;
use crate::types::{Answer, Citation, KnowledgeAtom, SearchResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static REMEMBER_SEQ: AtomicU64 = AtomicU64::new(1);

/// MCP-facing brain bound to the app store + embedder.
pub struct BrainService {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
    reranker: Arc<dyn Reranker>,
    synthesizer: Arc<dyn Synthesizer>,
}

impl BrainService {
    pub fn new(
        store: Arc<dyn Store>,
        embedder: Arc<dyn Embedder>,
        reranker: Arc<dyn Reranker>,
        synthesizer: Arc<dyn Synthesizer>,
    ) -> Self {
        Self {
            store,
            embedder,
            reranker,
            synthesizer,
        }
    }

    /// Search returning token-capped views (primary MCP payload).
    pub async fn search_views(&self, query: &str, limit: usize) -> Result<Vec<AgentAtomView>> {
        let results = self.search(query, limit).await?;
        Ok(results
            .into_iter()
            .map(|r| AgentAtomView::from_atom(&r.atom, r.score, DEFAULT_EXCERPT_CAP))
            .collect())
    }
}

#[async_trait::async_trait]
impl AgentRead for BrainService {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let results =
            hybrid_search(&self.store, &self.embedder, &self.reranker, query, limit).await?;
        expand_markdown_context(&self.store, results).await
    }

    async fn cite(&self, source: &str, source_id: &str) -> Result<Option<Citation>> {
        let Some(atom) = self.store.get_by_source_id(source, source_id).await? else {
            return Ok(None);
        };
        Ok(Some(citation_from_atom(&atom, 1.0, true)))
    }

    async fn ask(&self, question: &str) -> Result<Answer> {
        let hits = self.search(question, 5).await?;
        compose_answer(question, &hits, &self.synthesizer).await
    }
}

#[async_trait::async_trait]
impl AgentWrite for BrainService {
    async fn remember(
        &self,
        title: &str,
        summary: &str,
        tags: &[String],
        metadata: &[(&str, &str)],
    ) -> Result<String> {
        if title.trim().is_empty() || summary.trim().is_empty() {
            return Err(KurultaiError::config(
                "remember requires non-empty title and summary",
            ));
        }

        // Clamp write payload — agents must distill, not dump chat.
        let title: String = title.chars().take(200).collect();
        let summary: String = summary.chars().take(4_000).collect();

        let mut meta = HashMap::new();
        for (k, v) in metadata {
            meta.insert((*k).to_string(), (*v).to_string());
        }

        let source = "agent";
        let source_id = format!(
            "remember/{}_{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            REMEMBER_SEQ.fetch_add(1, Ordering::Relaxed)
        );
        let content = summary.clone();
        let id = atom_id(source, &source_id, &content);

        let mut atom = KnowledgeAtom {
            id: id.clone(),
            source: source.into(),
            source_id,
            title,
            summary: summary.chars().take(280).collect(),
            content,
            question: None,
            resolution: None,
            tags: tags.to_vec(),
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: None,
            metadata: meta,
        };

        if self.embedder.is_live() {
            let text = format!("{}\n{}", atom.title, atom.content);
            if let Ok(emb) = self.embedder.embed(&text).await {
                atom.embedding = Some(emb);
            }
        }

        self.store.upsert(&atom).await?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::NullEmbedder;
    use crate::pipeline::IndexPipeline;
    use crate::rerank::NullReranker;
    use crate::store::SqliteVecStore;
    use crate::types::{SourceConfig, SourceKind};
    use std::path::PathBuf;

    async fn brain_with_fixture() -> BrainService {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-mcp-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline =
            IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

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

        BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(crate::query::NullSynthesizer::new()),
        )
    }

    #[tokio::test]
    async fn search_returns_capped_views() {
        let brain = brain_with_fixture().await;
        let views = brain
            .search_views("KNOWN_PHRASE_KURULTAI_42", 5)
            .await
            .unwrap();
        assert!(!views.is_empty());
        assert!(views[0].excerpt.chars().count() <= DEFAULT_EXCERPT_CAP);
        // Full vault content must not appear as unbounded dump
        assert!(!views[0].excerpt.contains(&"x".repeat(500)));
    }

    #[tokio::test]
    async fn blank_query_returns_empty() {
        let brain = brain_with_fixture().await;
        let hits = brain.search("   ", 5).await.unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn fts_only_marks_matched_by_fts() {
        let brain = brain_with_fixture().await;
        let hits = brain.search("KNOWN_PHRASE_KURULTAI_42", 5).await.unwrap();
        assert!(!hits.is_empty());
        assert!(hits[0].matched_by.iter().any(|m| m == "fts"));
        assert!(!hits[0].matched_by.iter().any(|m| m == "vector"));
    }

    #[tokio::test]
    async fn remember_creates_agent_atom() {
        let brain = brain_with_fixture().await;
        let id = brain
            .remember(
                "Decision",
                "Use FTS-first boot without API keys",
                &["architecture".into()],
                &[("via", "test")],
            )
            .await
            .unwrap();
        assert!(!id.is_empty());
        let hits = brain.search("FTS-first boot", 5).await.unwrap();
        let hit = hits
            .iter()
            .find(|h| h.atom.source == "agent" && h.atom.id == id)
            .expect("remembered atom searchable");
        assert_eq!(hit.atom.title, "Decision");
        assert!(hit.atom.tags.iter().any(|t| t == "architecture"));
        assert_eq!(
            hit.atom.metadata.get("via").map(String::as_str),
            Some("test")
        );
        assert!(hit.atom.source_id.starts_with("remember/"));

        let err = brain.remember(" ", "ok", &[], &[]).await.unwrap_err();
        assert!(err.to_string().contains("non-empty"));
    }

    #[tokio::test]
    async fn ask_extractive_with_citations() {
        let brain = brain_with_fixture().await;
        let answer = brain.ask("KNOWN_PHRASE_KURULTAI_42").await.unwrap();
        assert!(!answer.citations.is_empty());
        assert!(!answer.answer.contains("deferred to #7"));
        assert!(answer.answer.contains("Based on indexed atoms") || !answer.answer.is_empty());
        assert!((answer.confidence - 0.45).abs() < 1e-9);
        for c in &answer.citations {
            assert!(c.excerpt.chars().count() <= DEFAULT_EXCERPT_CAP);
        }
    }

    #[tokio::test]
    async fn ask_no_hits_zero_confidence() {
        let brain = brain_with_fixture().await;
        let answer = brain.ask("ZZZ_NO_MATCH_TOKEN_XYZ").await.unwrap();
        assert!(answer.citations.is_empty());
        assert_eq!(answer.confidence, 0.0);
        assert!(answer.answer.contains("index"));
    }

    struct FailSynthesizer;

    #[async_trait::async_trait]
    impl crate::query::Synthesizer for FailSynthesizer {
        fn name(&self) -> &str {
            "fail"
        }
        async fn synthesize(&self, _question: &str, _hits: &[SearchResult]) -> Result<String> {
            Err(KurultaiError::Query("forced synth fail".into()))
        }
    }

    #[tokio::test]
    async fn ask_live_synth_fail_falls_back_extractive() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-mcp-fail-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline =
            IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));
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
        let brain = BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(FailSynthesizer),
        );
        let answer = brain.ask("KNOWN_PHRASE_KURULTAI_42").await.unwrap();
        assert!(!answer.citations.is_empty());
        assert!(answer.answer.contains("Based on indexed atoms"));
        assert!((answer.confidence - 0.45).abs() < 1e-9);
    }
}
