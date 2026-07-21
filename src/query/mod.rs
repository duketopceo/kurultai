//! Query pipeline: hybrid retrieval (Phase 2) and thin ask stub (#7 later).

mod hybrid;
mod rrf;

pub use hybrid::hybrid_search;
pub use rrf::{candidate_limit, fuse_rrf, fuse_rrf_ids, FusedId, RRF_K};

use crate::embed::Embedder;
use crate::error::Result;
use crate::store::Store;
use crate::types::{Answer, SearchResult};
use std::sync::Arc;

/// Full query pipeline: retrieve → (later) synthesize.
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    /// Ask a question and get a synthesized answer with citations.
    async fn ask(&self, question: &str) -> Result<Answer>;

    /// Raw search with results ranked but not synthesized.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

/// Hybrid FTS ∥ vector → RRF engine used by CLI/MCP brain.
pub struct HybridQueryEngine {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
}

impl HybridQueryEngine {
    pub fn new(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self { store, embedder }
    }
}

#[async_trait::async_trait]
impl QueryEngine for HybridQueryEngine {
    async fn ask(&self, question: &str) -> Result<Answer> {
        // Thin stub — full planner/synthesis is #7.
        let hits = self.search(question, 5).await?;
        let answer = if hits.is_empty() {
            "No indexed atoms matched. Run `kurultai index` first.".into()
        } else {
            format!(
                "Top matches (synthesis deferred to #7):\n{}",
                hits.iter()
                    .take(3)
                    .map(|r| format!(
                        "- {} ({}/{}): {}",
                        r.atom.title,
                        r.atom.source,
                        r.atom.source_id,
                        r.atom.summary.chars().take(120).collect::<String>()
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        Ok(Answer {
            question: question.into(),
            answer,
            citations: vec![],
            sources_used: hits.iter().map(|h| h.atom.source.clone()).collect(),
            confidence: if hits.is_empty() { 0.0 } else { 0.4 },
        })
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        hybrid_search(&self.store, &self.embedder, query, limit).await
    }
}
