use crate::types::{Answer, Citation, SearchResult};
use async_trait::async_trait;

/// Read path: token-efficient retrieval from the indexed brain.
///
/// Agents should never receive full source files — only ranked excerpts
/// and citations pointing back to `source_id` / `source_uri`.
#[async_trait]
pub trait AgentRead: Send + Sync {
    /// Semantic + keyword search. Returns short excerpts, not full documents.
    async fn search(&self, query: &str, limit: usize) -> crate::Result<Vec<SearchResult>>;

    /// One citation-sized slice for grounding a response.
    async fn cite(&self, source: &str, source_id: &str) -> crate::Result<Option<Citation>>;

    /// Synthesized answer with citations (higher token cost — use sparingly).
    async fn ask(&self, question: &str) -> crate::Result<Answer>;
}

/// Write path: agents contribute durable knowledge without bloating the index.
///
/// Prefer distilled fields (`summary`, `resolution`, `tags`) over raw dumps.
#[async_trait]
pub trait AgentWrite: Send + Sync {
    /// Store a distilled fact the agent wants future sessions to recall.
    async fn remember(
        &self,
        title: &str,
        summary: &str,
        tags: &[String],
        metadata: &[(&str, &str)],
    ) -> crate::Result<String>;
}
