use crate::embed::{EmbedMode, Embedder};
use crate::error::Result;
use crate::store::Store;
use crate::types::{Answer, SearchResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Full query pipeline: embed → search → fuse → rerank → synthesize.
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    /// Ask a question and get a synthesized answer with citations.
    async fn ask(&self, question: &str) -> Result<Answer>;

    /// Raw search with results ranked but not synthesized.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

/// Hybrid query engine: FTS-first, augmented by brute-force vector search when
/// an embedding key is configured. Phase 1 uses a simple reciprocal-rank fuse.
pub struct HybridQueryEngine {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
}

impl HybridQueryEngine {
    pub fn new(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self { store, embedder }
    }

    fn reciprocal_rank(rank: usize) -> f64 {
        1.0 / (1.0 + rank as f64)
    }

    async fn search_vector(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if self.embedder.mode() != EmbedMode::Full {
            return Ok(vec![]);
        }

        let query_embed = match self.embedder.embed(query).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "query embedding failed; falling back to FTS");
                return Ok(vec![]);
            }
        };

        let rows = self.store.vector_search(&query_embed, limit).await?;
        let mut results = Vec::with_capacity(rows.len());
        for (rank, (atom, score)) in rows.into_iter().enumerate() {
            results.push(SearchResult {
                atom,
                score,
                rank: rank + 1,
                matched_by: vec!["vector".into()],
            });
        }
        Ok(results)
    }

    async fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let rows = self.store.fts_search(query, limit).await?;
        let mut results = Vec::with_capacity(rows.len());
        for (rank, (atom, score)) in rows.into_iter().enumerate() {
            results.push(SearchResult {
                atom,
                score,
                rank: rank + 1,
                matched_by: vec!["fts".into()],
            });
        }
        Ok(results)
    }

    fn fuse(
        vector_results: Vec<SearchResult>,
        fts_results: Vec<SearchResult>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let mut scores: HashMap<String, (SearchResult, f64)> = HashMap::new();

        for (rank, mut result) in vector_results.into_iter().enumerate() {
            result.rank = rank + 1;
            let id = result.atom.id.clone();
            let rrf = Self::reciprocal_rank(result.rank);
            scores
                .entry(id)
                .and_modify(|(existing, score)| {
                    existing.matched_by.push("vector".into());
                    *score += rrf;
                })
                .or_insert((result, rrf));
        }

        for (rank, mut result) in fts_results.into_iter().enumerate() {
            result.rank = rank + 1;
            let id = result.atom.id.clone();
            let rrf = Self::reciprocal_rank(result.rank);
            scores
                .entry(id)
                .and_modify(|(existing, score)| {
                    existing.matched_by.push("fts".into());
                    *score += rrf;
                })
                .or_insert((result, rrf));
        }

        let mut fused: Vec<SearchResult> = scores
            .into_values()
            .map(|(mut result, rrf)| {
                result.score = rrf;
                result.matched_by.sort();
                result.matched_by.dedup();
                result
            })
            .collect();

        fused.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        fused.truncate(limit);

        for (rank, result) in fused.iter_mut().enumerate() {
            result.rank = rank + 1;
        }
        fused
    }
}

#[async_trait::async_trait]
impl QueryEngine for HybridQueryEngine {
    async fn ask(&self, question: &str) -> Result<Answer> {
        // Phase 1: ask is intentionally thin. We return a stub because full
        // synthesis with citations is a Phase 3 concern (#7).
        Ok(Answer {
            question: question.to_string(),
            answer: "Not implemented yet. See issue #7.".to_string(),
            citations: vec![],
            sources_used: vec![],
            confidence: 0.0,
        })
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let vector = self.search_vector(query, limit * 2).await?;
        let fts = self.search_fts(query, limit * 2).await?;
        Ok(Self::fuse(vector, fts, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::FixedEmbedder;
    use crate::store::SqliteVecStore;
    use crate::types::KnowledgeAtom;
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_atom(id: &str, content: &str, embedding: Option<Vec<f32>>) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "test".into(),
            source_id: id.into(),
            title: "title".into(),
            summary: "summary".into(),
            content: content.into(),
            question: None,
            resolution: None,
            tags: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            embedding,
            content_hash: format!("hash-{content}"),
            source_uri: None,
            provenance: None,
        }
    }

    fn engine() -> (HybridQueryEngine, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(tmp.path().join("store.db"), "fixed", 8).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(FixedEmbedder::new(8));
        let engine = HybridQueryEngine::new(store, embedder);
        (engine, tmp)
    }

    #[tokio::test]
    async fn search_hits_fts_without_key() {
        let (engine, _tmp) = engine();
        engine
            .store
            .upsert(&make_atom("1", "the quick brown fox", None))
            .await
            .unwrap();

        let results = engine.search("fox", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].atom.id, "1");
    }

    #[tokio::test]
    async fn search_fuses_vector_and_fts() {
        let (engine, _tmp) = engine();
        // Same text -> vector and FTS should both match.
        engine
            .store
            .upsert(&make_atom(
                "1",
                "unique kurultai phrase",
                Some(vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ))
            .await
            .unwrap();

        let results = engine.search("unique kurultai phrase", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].matched_by.contains(&"fts".to_string()));
        assert!(results[0].matched_by.contains(&"vector".to_string()));
    }

    #[tokio::test]
    async fn search_respects_limit() {
        let (engine, _tmp) = engine();
        for i in 0..5 {
            engine
                .store
                .upsert(&make_atom(&format!("{i}"), &format!("word {i}"), None))
                .await
                .unwrap();
        }
        let results = engine.search("word", 3).await.unwrap();
        assert_eq!(results.len(), 3);
    }
}
