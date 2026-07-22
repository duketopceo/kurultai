//! Query pipeline: hybrid retrieval (Phase 2) and thin ask stub (#7 later).

mod context;
mod hybrid;
mod rrf;
mod synthesize;

pub use context::expand_markdown_context;
pub use hybrid::hybrid_search;
pub use rrf::{candidate_limit, fuse_rrf, fuse_rrf_ids, FusedId, RRF_K};
pub use synthesize::{
    confidence_for, extractive_answer, NullSynthesizer, OpenRouterSynthesizer, Synthesizer,
};

use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::Result;
use crate::rerank::Reranker;
use crate::store::Store;
use crate::types::{Answer, Citation, KnowledgeAtom, SearchResult};
use std::sync::Arc;

/// Full query pipeline: retrieve → (later) synthesize.
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    /// Ask a question and get a synthesized answer with citations.
    async fn ask(&self, question: &str) -> Result<Answer>;

    /// Raw search with results ranked but not synthesized.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

/// Hybrid FTS ∥ vector → RRF → optional rerank → synthesize engine.
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
        let hits = self.search(question, 5).await?;
        compose_answer(question, &hits, &self.synthesizer).await
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let results =
            hybrid_search(&self.store, &self.embedder, &self.reranker, query, limit).await?;
        expand_markdown_context(&self.store, results).await
    }
}

/// Shared ask composition for brain + query engine (citations from hits).
pub async fn compose_answer(
    question: &str,
    hits: &[SearchResult],
    synthesizer: &Arc<dyn Synthesizer>,
) -> Result<Answer> {
    let citations: Vec<Citation> = hits
        .iter()
        .map(|r| citation_from_atom(&r.atom, r.score, false))
        .collect();
    let sources_used: Vec<String> = citations.iter().map(|c| c.source.clone()).collect();

    let (answer, used_live) = if hits.is_empty() {
        (extractive_answer(hits), false)
    } else if synthesizer.is_live() {
        match synthesizer.synthesize(question, hits).await {
            Ok(text) => (text, true),
            Err(err) => {
                tracing::warn!(error = %err, "synthesize failed; using extractive fallback");
                (extractive_answer(hits), false)
            }
        }
    } else {
        (extractive_answer(hits), false)
    };

    Ok(Answer {
        question: question.into(),
        answer,
        citations,
        sources_used,
        confidence: confidence_for(hits, used_live),
    })
}

fn citation_from_atom(atom: &KnowledgeAtom, score: f64, include_url: bool) -> Citation {
    let view = AgentAtomView::from_atom(atom, score, DEFAULT_EXCERPT_CAP);
    Citation {
        source: view.source,
        source_id: view.source_id,
        title: view.title,
        url: if include_url {
            atom.metadata.get("source_uri").cloned()
        } else {
            None
        },
        excerpt: view.excerpt,
    }
}
