use crate::types::{Answer, SearchResult};
use anyhow::Result;

/// Full query pipeline: embed → search → fuse → rerank → synthesize.
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    /// Ask a question and get a synthesized answer with citations.
    async fn ask(&self, question: &str) -> Result<Answer>;

    /// Raw search with results ranked but not synthesized.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

/// Default query engine implementation.
pub struct DefaultQueryEngine;

impl DefaultQueryEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl QueryEngine for DefaultQueryEngine {
    async fn ask(&self, question: &str) -> Result<Answer> {
        Ok(Answer {
            question: question.to_string(),
            answer: "Not implemented yet.".to_string(),
            citations: vec![],
            sources_used: vec![],
            confidence: 0.0,
        })
    }

    async fn search(&self, _query: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        Ok(vec![])
    }
}
