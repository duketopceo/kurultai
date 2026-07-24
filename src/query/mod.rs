//! Query pipeline: hybrid retrieval (Phase 2) and synthesis (Phase 3).

mod context;
mod hybrid;
mod rrf;

pub use context::expand_markdown_context;
pub use hybrid::hybrid_search;
pub use rrf::{candidate_limit, fuse_rrf, fuse_rrf_ids, FusedId, RRF_K};

use crate::embed::Embedder;
use crate::error::Result;
use crate::rerank::Reranker;
use crate::store::Store;
use crate::synthesize::Synthesizer;
use crate::types::{Answer, SearchResult};
use std::sync::Arc;

/// Full query pipeline: retrieve → synthesize.
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    /// Ask a question and get a synthesized answer with citations.
    async fn ask(&self, question: &str) -> Result<Answer>;

    /// Raw search with results ranked but not synthesized.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

/// Hybrid FTS ∥ vector → RRF → optional rerank → synthesize.
pub struct HybridQueryEngine {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
    reranker: Arc<dyn Reranker>,
    synthesizer: Arc<dyn Synthesizer>,
}

impl HybridQueryEngine {
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
}

#[async_trait::async_trait]
impl QueryEngine for HybridQueryEngine {
    async fn ask(&self, question: &str) -> Result<Answer> {
        let hits = self.search(question, 8).await?;
        self.synthesizer.synthesize(question, &hits).await
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let results =
            hybrid_search(&self.store, &self.embedder, &self.reranker, query, limit).await?;
        expand_markdown_context(&self.store, results).await
    }
}
