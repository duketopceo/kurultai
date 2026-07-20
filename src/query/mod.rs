use crate::embed::Embedder;
use crate::store::Store;
use crate::types::{Answer, Citation, SearchResult};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Full query pipeline: FTS-first, optional vector fuse.
#[async_trait::async_trait]
pub trait QueryEngine: Send + Sync {
    async fn ask(&self, question: &str) -> Result<Answer>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
}

pub struct DefaultQueryEngine {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
}

impl DefaultQueryEngine {
    pub fn new(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self { store, embedder }
    }

    fn fuse(
        fts: Vec<(crate::types::KnowledgeAtom, f64)>,
        vec: Vec<(crate::types::KnowledgeAtom, f64)>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let mut by_id: HashMap<String, SearchResult> = HashMap::new();
        for (atom, score) in fts {
            by_id.insert(
                atom.id.clone(),
                SearchResult {
                    atom,
                    score,
                    rank: 0,
                    matched_by: vec!["fts".into()],
                },
            );
        }
        for (atom, score) in vec {
            by_id
                .entry(atom.id.clone())
                .and_modify(|r| {
                    r.score = r.score.max(score);
                    if !r.matched_by.iter().any(|m| m == "vector") {
                        r.matched_by.push("vector".into());
                    }
                })
                .or_insert(SearchResult {
                    atom,
                    score,
                    rank: 0,
                    matched_by: vec!["vector".into()],
                });
        }
        let mut results: Vec<_> = by_id.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        for (i, r) in results.iter_mut().enumerate() {
            r.rank = i + 1;
        }
        results
    }
}

#[async_trait::async_trait]
impl QueryEngine for DefaultQueryEngine {
    async fn ask(&self, question: &str) -> Result<Answer> {
        let results = self.search(question, 5).await?;
        let citations: Vec<Citation> = results
            .iter()
            .map(|r| Citation {
                source: r.atom.source.clone(),
                source_id: r.atom.source_id.clone(),
                title: r.atom.title.clone(),
                url: r.atom.source_uri.clone(),
                excerpt: excerpt(&r.atom.content, 240),
            })
            .collect();
        let sources_used: Vec<String> = citations.iter().map(|c| c.source.clone()).collect();
        let answer = if results.is_empty() {
            "No matching atoms found.".into()
        } else {
            format!(
                "Top match: {} ({})",
                results[0].atom.title, results[0].atom.source_id
            )
        };
        Ok(Answer {
            question: question.into(),
            answer,
            citations,
            sources_used,
            confidence: results.first().map(|r| r.score).unwrap_or(0.0),
        })
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let fts = self.store.fts_search(query, limit)?;
        let vec_hits = match self.embedder.embed(query).await {
            Ok(emb) => self.store.vector_search(&emb, limit).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        Ok(Self::fuse(fts, vec_hits, limit))
    }
}

fn excerpt(text: &str, max: usize) -> String {
    let t = text.trim();
    if t.chars().count() <= max {
        return t.to_string();
    }
    t.chars().take(max).collect::<String>() + "…"
}
