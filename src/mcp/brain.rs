//! Brain facade — AgentRead / AgentWrite over the SQLite store.

use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use crate::hashutil::atom_id;
use crate::mcp::interface::{AgentRead, AgentWrite};
use crate::store::Store;
use crate::types::{Answer, Citation, KnowledgeAtom, SearchResult};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

/// MCP-facing brain bound to the app store + embedder.
pub struct BrainService {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
}

impl BrainService {
    pub fn new(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self { store, embedder }
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

#[async_trait::async_trait]
impl AgentRead for BrainService {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let limit = limit.clamp(1, 50);
        let mut by_id: HashMap<String, SearchResult> = HashMap::new();

        // FTS is local; embedding may hit the network — overlap them when live.
        let fts_fut = self.store.fts_search(query, limit);
        let embed_fut = async {
            if !self.embedder.is_live() {
                return None;
            }
            match self.embedder.embed(query).await {
                Ok(emb) => Some(emb),
                Err(err) => {
                    tracing::warn!(error = %err, "semantic search skipped; using FTS only");
                    None
                }
            }
        };
        let (fts_res, emb_opt) = tokio::join!(fts_fut, embed_fut);
        let fts = fts_res?;

        for (rank, (atom, score)) in fts.into_iter().enumerate() {
            by_id.insert(
                atom.id.clone(),
                SearchResult {
                    atom,
                    score,
                    rank,
                    matched_by: vec!["fts".into()],
                },
            );
        }

        if let Some(emb) = emb_opt {
            let vec_hits = self.store.vector_search(&emb, limit).await?;
            for (rank, (atom, score)) in vec_hits.into_iter().enumerate() {
                by_id
                    .entry(atom.id.clone())
                    .and_modify(|existing| {
                        existing.score = existing.score.max(score);
                        if !existing.matched_by.iter().any(|m| m == "vector") {
                            existing.matched_by.push("vector".into());
                        }
                    })
                    .or_insert(SearchResult {
                        atom,
                        score,
                        rank,
                        matched_by: vec!["vector".into()],
                    });
            }
        }

        let mut results: Vec<SearchResult> = by_id.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        for (i, r) in results.iter_mut().enumerate() {
            r.rank = i;
        }
        Ok(results)
    }

    async fn cite(&self, source: &str, source_id: &str) -> Result<Option<Citation>> {
        let Some(atom) = self.store.get_by_source_id(source, source_id).await? else {
            return Ok(None);
        };
        Ok(Some(citation_from_atom(&atom, 1.0, true)))
    }

    async fn ask(&self, question: &str) -> Result<Answer> {
        // Phase 1: thin FTS synthesis stub — full planner is #7.
        let hits = self.search(question, 5).await?;
        let citations: Vec<Citation> = hits
            .iter()
            .map(|r| citation_from_atom(&r.atom, r.score, false))
            .collect();
        let sources_used: Vec<String> = citations.iter().map(|c| c.source.clone()).collect();
        let answer = if citations.is_empty() {
            "No indexed atoms matched. Run `kurultai index` first.".into()
        } else {
            format!(
                "Top matches (synthesis deferred to #7):\n{}",
                citations
                    .iter()
                    .take(3)
                    .map(|c| format!(
                        "- {} ({}/{}): {}",
                        c.title, c.source, c.source_id, c.excerpt
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        Ok(Answer {
            question: question.into(),
            answer,
            citations,
            sources_used,
            confidence: if hits.is_empty() { 0.0 } else { 0.4 },
        })
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
        let source_id = format!("remember/{}", Utc::now().timestamp_millis());
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

        BrainService::new(store, embedder)
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
        assert!(hits
            .iter()
            .any(|h| h.atom.source == "agent" && h.atom.id == id));
    }
}
