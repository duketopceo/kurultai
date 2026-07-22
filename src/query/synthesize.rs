//! Answer synthesis over retrieved hits (Phase 3 / #7 WO1).

use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::error::{KurultaiError, Result};
use crate::security::SecretString;
use crate::types::SearchResult;
use serde::Deserialize;
use std::time::Duration;

/// Builds the answer body from a question + ranked hits.
#[async_trait::async_trait]
pub trait Synthesizer: Send + Sync {
    fn name(&self) -> &str;

    fn is_live(&self) -> bool {
        true
    }

    /// Return the answer string only; caller owns citations / confidence.
    async fn synthesize(&self, question: &str, hits: &[SearchResult]) -> Result<String>;
}

/// Deterministic extractive synthesis — CI path and soft-fail fallback.
pub struct NullSynthesizer;

impl NullSynthesizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Synthesizer for NullSynthesizer {
    fn name(&self) -> &str {
        "extractive"
    }

    fn is_live(&self) -> bool {
        false
    }

    async fn synthesize(&self, _question: &str, hits: &[SearchResult]) -> Result<String> {
        Ok(extractive_answer(hits))
    }
}

/// OpenRouter chat-completions synthesizer (cite-only-from-excerpts).
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

const OPENROUTER_CHAT_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_HITS: usize = 8;

#[async_trait::async_trait]
impl Synthesizer for OpenRouterSynthesizer {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn synthesize(&self, question: &str, hits: &[SearchResult]) -> Result<String> {
        if hits.is_empty() {
            return Ok(extractive_answer(hits));
        }
        let evidence = format_evidence(hits);
        let user = format!(
            "Question: {question}\n\nEvidence (use only this):\n{evidence}\n\nWrite a concise answer grounded only in the evidence. Do not invent facts."
        );
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You answer using only the provided evidence excerpts. If evidence is insufficient, say so briefly."
                },
                {"role": "user", "content": user}
            ],
            "temperature": 0
        });

        let resp = self
            .client
            .post(OPENROUTER_CHAT_URL)
            .bearer_auth(self.api_key.expose())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| KurultaiError::Query(format!("synthesize request: {e}")))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| KurultaiError::Query(format!("synthesize body: {e}")))?;
        if !status.is_success() {
            return Err(KurultaiError::Query(format!(
                "synthesize HTTP {status}: {}",
                text.chars().take(200).collect::<String>()
            )));
        }

        let parsed: ChatResponse = serde_json::from_str(&text)
            .map_err(|e| KurultaiError::Query(format!("synthesize json: {e}")))?;
        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .unwrap_or("")
            .trim();
        if content.is_empty() {
            return Err(KurultaiError::Query(
                "synthesize returned empty content".into(),
            ));
        }
        Ok(content.to_string())
    }
}

/// Extractive answer from ranked hits (no LLM).
pub fn extractive_answer(hits: &[SearchResult]) -> String {
    if hits.is_empty() {
        return "No indexed atoms matched. Run `kurultai index` first.".into();
    }
    let mut lines = Vec::new();
    lines.push("Based on indexed atoms:".to_string());
    for r in hits.iter().take(3) {
        let view = AgentAtomView::from_atom(&r.atom, r.score, DEFAULT_EXCERPT_CAP);
        lines.push(format!(
            "- {} ({}/{}): {}",
            view.title, view.source, view.source_id, view.excerpt
        ));
    }
    lines.join("\n")
}

fn format_evidence(hits: &[SearchResult]) -> String {
    let mut out = String::new();
    for (i, r) in hits.iter().take(MAX_HITS).enumerate() {
        let view = AgentAtomView::from_atom(&r.atom, r.score, DEFAULT_EXCERPT_CAP);
        out.push_str(&format!(
            "{}. [{}] {} / {} — {}\n   {}\n",
            i + 1,
            r.atom.id,
            view.source,
            view.source_id,
            view.title,
            view.excerpt
        ));
    }
    out
}

/// Confidence heuristic (not calibrated probability).
pub fn confidence_for(hits: &[SearchResult], used_live_synth: bool) -> f64 {
    if hits.is_empty() {
        0.0
    } else if used_live_synth {
        0.7
    } else {
        0.45
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KnowledgeAtom;
    use chrono::Utc;
    use std::collections::HashMap;

    fn hit(id: &str, title: &str, summary: &str) -> SearchResult {
        SearchResult {
            atom: KnowledgeAtom {
                id: id.into(),
                source: "notes".into(),
                source_id: id.into(),
                title: title.into(),
                summary: summary.into(),
                content: summary.into(),
                question: None,
                resolution: None,
                tags: vec![],
                source_updated_at: Utc::now(),
                indexed_at: Utc::now(),
                embedding: None,
                metadata: HashMap::new(),
            },
            score: 0.5,
            rank: 0,
            matched_by: vec!["fts".into()],
        }
    }

    #[tokio::test]
    async fn null_extractive_includes_title() {
        let s = NullSynthesizer::new();
        assert!(!s.is_live());
        let text = s
            .synthesize("q", &[hit("a1", "Alpha", "alpha excerpt body")])
            .await
            .unwrap();
        assert!(text.contains("Alpha"));
        assert!(text.contains("alpha excerpt"));
        assert!(!text.contains("deferred to #7"));
    }

    #[tokio::test]
    async fn null_empty_hits_index_first() {
        let text = NullSynthesizer::new().synthesize("q", &[]).await.unwrap();
        assert!(text.contains("index"));
        assert_eq!(confidence_for(&[], false), 0.0);
    }

    #[test]
    fn confidence_bands() {
        let hits = vec![hit("a", "T", "e")];
        assert_eq!(confidence_for(&hits, false), 0.45);
        assert_eq!(confidence_for(&hits, true), 0.7);
    }
}
