//! Answer synthesis after hybrid retrieval (Phase 3 / #7).
//!
//! Live path: OpenRouter chat completion over capped excerpts.
//! Degrade path: extractive bullets from hits (no API key).

use crate::error::{KurultaiError, Result};
use crate::security::SecretString;
use crate::types::{Answer, Citation, SearchResult};
use serde::Deserialize;
use std::time::Duration;

const OPENROUTER_CHAT_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_HITS: usize = 8;
const EXCERPT_CAP: usize = 400;

/// Builds an [`Answer`] from a question and ranked search hits.
#[async_trait::async_trait]
pub trait Synthesizer: Send + Sync {
    fn name(&self) -> &str;

    fn is_live(&self) -> bool {
        true
    }

    async fn synthesize(&self, question: &str, hits: &[SearchResult]) -> Result<Answer>;
}

/// Extractive synthesis — no network.
pub struct ExtractiveSynthesizer;

impl ExtractiveSynthesizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExtractiveSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Synthesizer for ExtractiveSynthesizer {
    fn name(&self) -> &str {
        "extractive"
    }

    fn is_live(&self) -> bool {
        false
    }

    async fn synthesize(&self, question: &str, hits: &[SearchResult]) -> Result<Answer> {
        Ok(extractive_answer(question, hits))
    }
}

/// OpenRouter chat synthesizer.
pub struct OpenRouterSynthesizer {
    api_key: SecretString,
    model: String,
    client: reqwest::Client,
}

impl OpenRouterSynthesizer {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            api_key: SecretString::new(api_key),
            model,
            client,
        }
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[async_trait::async_trait]
impl Synthesizer for OpenRouterSynthesizer {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn synthesize(&self, question: &str, hits: &[SearchResult]) -> Result<Answer> {
        if hits.is_empty() {
            return Ok(empty_answer(question));
        }
        let citations = citations_from_hits(hits);
        let context = hits
            .iter()
            .take(MAX_HITS)
            .enumerate()
            .map(|(i, r)| {
                let excerpt: String = r.atom.content.chars().take(EXCERPT_CAP).collect();
                format!(
                    "[{}] title={} source={}/{} excerpt={}",
                    i + 1,
                    r.atom.title,
                    r.atom.source,
                    r.atom.source_id,
                    excerpt
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let system = "You answer using ONLY the numbered excerpts. Do not invent facts. \
If excerpts are insufficient, say so briefly. Keep the answer concise.";
        let user = format!("Question: {question}\n\nExcerpts:\n{context}");

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "temperature": 0.2,
        });

        let resp = self
            .client
            .post(OPENROUTER_CHAT_URL)
            .header("Authorization", format!("Bearer {}", self.api_key.expose()))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| KurultaiError::Query(format!("synthesize request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            tracing::warn!(%status, body = %text, "synthesize HTTP error — falling back extractive");
            return Ok(extractive_answer(question, hits));
        }

        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| KurultaiError::Query(format!("synthesize decode failed: {e}")))?;
        let prose = parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default()
            .trim()
            .to_string();

        if prose.is_empty() {
            return Ok(extractive_answer(question, hits));
        }

        Ok(Answer {
            question: question.into(),
            answer: prose,
            sources_used: citations.iter().map(|c| c.source.clone()).collect(),
            citations,
            confidence: confidence_from_hits(hits),
        })
    }
}

/// Prefer live synthesizer when keyed; otherwise extractive.
pub fn synthesizer_from_env(model: Option<&str>) -> std::sync::Arc<dyn Synthesizer> {
    let key = crate::security::api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| crate::security::api_key_from_env_optional("KURULTAI_API_KEY"));
    match key {
        Some(k) => {
            let model = model
                .filter(|m| !m.trim().is_empty())
                .unwrap_or("openai/gpt-4o-mini")
                .to_string();
            std::sync::Arc::new(OpenRouterSynthesizer::new(k.expose().to_string(), model))
        }
        None => std::sync::Arc::new(ExtractiveSynthesizer::new()),
    }
}

pub fn empty_answer(question: &str) -> Answer {
    Answer {
        question: question.into(),
        answer: "No indexed atoms matched. Run `kurultai index` first.".into(),
        citations: vec![],
        sources_used: vec![],
        confidence: 0.0,
    }
}

pub fn extractive_answer(question: &str, hits: &[SearchResult]) -> Answer {
    if hits.is_empty() {
        return empty_answer(question);
    }
    let citations = citations_from_hits(hits);
    let body = citations
        .iter()
        .take(5)
        .map(|c| {
            format!(
                "- {} ({}/{}): {}",
                c.title, c.source, c.source_id, c.excerpt
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Answer {
        question: question.into(),
        answer: format!("Based on indexed atoms:\n{body}"),
        sources_used: citations.iter().map(|c| c.source.clone()).collect(),
        citations,
        confidence: confidence_from_hits(hits),
    }
}

fn citations_from_hits(hits: &[SearchResult]) -> Vec<Citation> {
    hits.iter()
        .take(MAX_HITS)
        .map(|r| {
            let excerpt: String = if !r.atom.summary.is_empty() {
                r.atom.summary.chars().take(EXCERPT_CAP).collect()
            } else {
                r.atom.content.chars().take(EXCERPT_CAP).collect()
            };
            Citation {
                source: r.atom.source.clone(),
                source_id: r.atom.source_id.clone(),
                title: r.atom.title.clone(),
                url: r.atom.metadata.get("source_uri").cloned(),
                excerpt,
            }
        })
        .collect()
}

fn confidence_from_hits(hits: &[SearchResult]) -> f64 {
    if hits.is_empty() {
        return 0.0;
    }
    let mean: f64 = hits.iter().take(5).map(|h| h.score).sum::<f64>() / hits.len().min(5) as f64;
    (mean.clamp(0.0, 1.0) * 0.85 + 0.1).clamp(0.0, 1.0)
}

/// Aggregate distinct sources that match a topic (who_knows).
pub fn who_knows_from_hits(hits: &[SearchResult]) -> Vec<WhoKnowsEntry> {
    let mut out: Vec<WhoKnowsEntry> = Vec::new();
    for r in hits {
        if let Some(existing) = out.iter_mut().find(|e| e.source == r.atom.source) {
            if existing.sample_titles.len() < 3
                && !existing.sample_titles.iter().any(|t| t == &r.atom.title)
            {
                existing.sample_titles.push(r.atom.title.clone());
            }
            existing.hit_count += 1;
            continue;
        }
        out.push(WhoKnowsEntry {
            source: r.atom.source.clone(),
            hit_count: 1,
            sample_titles: vec![r.atom.title.clone()],
        });
    }
    out
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhoKnowsEntry {
    pub source: String,
    pub hit_count: usize,
    pub sample_titles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KnowledgeAtom;
    use chrono::Utc;
    use std::collections::HashMap;

    fn hit(title: &str, content: &str, score: f64) -> SearchResult {
        SearchResult {
            atom: KnowledgeAtom {
                id: title.into(),
                source: "markdown".into(),
                source_id: format!("{title}.md"),
                title: title.into(),
                summary: String::new(),
                content: content.into(),
                question: None,
                resolution: None,
                tags: vec![],
                source_updated_at: Utc::now(),
                indexed_at: Utc::now(),
                embedding: None,
                metadata: HashMap::new(),
            },
            score,
            rank: 1,
            matched_by: vec!["fts".into()],
        }
    }

    #[tokio::test]
    async fn empty_hits_zero_confidence() {
        let s = ExtractiveSynthesizer::new();
        let a = s.synthesize("q", &[]).await.unwrap();
        assert_eq!(a.confidence, 0.0);
        assert!(a.citations.is_empty());
    }

    #[tokio::test]
    async fn extractive_includes_excerpt() {
        let s = ExtractiveSynthesizer::new();
        let hits = vec![hit("Deploy", "fixture-ops-runbook checklist", 0.8)];
        let a = s.synthesize("ops?", &hits).await.unwrap();
        assert!(a.answer.contains("Deploy") || a.answer.contains("fixture-ops-runbook"));
        assert_eq!(a.citations.len(), 1);
        assert!(a.confidence > 0.0);
    }

    #[test]
    fn who_knows_aggregates_sources() {
        let hits = vec![hit("A", "x", 0.5), hit("B", "y", 0.4)];
        let w = who_knows_from_hits(&hits);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].source, "markdown");
        assert_eq!(w[0].hit_count, 2);
    }

    /// Test-only synthesizer that returns fixed prose while preserving citations.
    struct FixedSynthesizer {
        prose: String,
    }

    impl FixedSynthesizer {
        fn new(prose: impl Into<String>) -> Self {
            Self {
                prose: prose.into(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Synthesizer for FixedSynthesizer {
        fn name(&self) -> &str {
            "fixed"
        }

        fn is_live(&self) -> bool {
            false
        }

        async fn synthesize(&self, question: &str, hits: &[SearchResult]) -> Result<Answer> {
            if hits.is_empty() {
                return Ok(empty_answer(question));
            }
            let citations = citations_from_hits(hits);
            Ok(Answer {
                question: question.into(),
                answer: format!("{} (hits: {})", self.prose, hits.len()),
                sources_used: citations.iter().map(|c| c.source.clone()).collect(),
                citations,
                confidence: confidence_from_hits(hits),
            })
        }
    }

    #[tokio::test]
    async fn fixed_synthesizer_returns_preset_prose_and_citations() {
        let s = FixedSynthesizer::new("Mock synthesis result");
        let hits = vec![hit("Deploy", "fixture-ops-runbook checklist", 0.8)];
        let a = s.synthesize("what is ops?", &hits).await.unwrap();
        assert!(a.answer.contains("Mock synthesis result"));
        assert!(a.answer.contains("hits: 1"));
        assert_eq!(a.citations.len(), 1);
        assert_eq!(a.citations[0].source, "markdown");
        assert!(a.confidence > 0.0);
    }
}
