//! Optional LLM rerank of fused search candidates (Phase 2 / #6).

use crate::error::{KurultaiError, Result};
use crate::security::SecretString;
use serde::Deserialize;
use std::time::Duration;

/// Reorders fused candidates; soft-skipped when not live.
#[async_trait::async_trait]
pub trait Reranker: Send + Sync {
    fn name(&self) -> &str;

    fn is_live(&self) -> bool {
        true
    }

    /// Return candidate ids in preferred order. Unknown ids are ignored by the caller.
    async fn rerank(&self, query: &str, candidates: &[(String, String)]) -> Result<Vec<String>>;
}

/// No-op when rerank is unconfigured or no API key.
pub struct NullReranker;

impl NullReranker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullReranker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Reranker for NullReranker {
    fn name(&self) -> &str {
        "none"
    }
    fn is_live(&self) -> bool {
        false
    }
    async fn rerank(&self, _query: &str, _candidates: &[(String, String)]) -> Result<Vec<String>> {
        Err(KurultaiError::Query(
            "NullReranker: rerank disabled".into(),
        ))
    }
}

const OPENROUTER_CHAT_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const MAX_CANDIDATES: usize = 20;
const EXCERPT_CAP: usize = 400;

/// OpenRouter chat-completions reranker (JSON id list).
pub struct OpenRouterReranker {
    api_key: SecretString,
    model: String,
    client: reqwest::Client,
}

impl OpenRouterReranker {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(45))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            api_key: SecretString::new(api_key),
            model,
            client,
        }
    }
}

#[async_trait::async_trait]
impl Reranker for OpenRouterReranker {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn rerank(&self, query: &str, candidates: &[(String, String)]) -> Result<Vec<String>> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }
        let slice: Vec<&(String, String)> = candidates.iter().take(MAX_CANDIDATES).collect();
        let mut body_lines = String::from("Candidates (id | excerpt):\n");
        for (id, excerpt) in &slice {
            let capped: String = excerpt.chars().take(EXCERPT_CAP).collect();
            body_lines.push_str(&format!("- {id} | {capped}\n"));
        }
        let user = format!(
            "Query: {query}\n\n{body_lines}\nReturn ONLY a JSON array of candidate ids best-first, using only the ids above."
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You reorder search candidates. Reply with a JSON array of strings only."
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
            .map_err(|e| KurultaiError::Query(format!("rerank request: {e}")))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| KurultaiError::Query(format!("rerank body: {e}")))?;
        if !status.is_success() {
            return Err(KurultaiError::Query(format!(
                "rerank HTTP {status}: {}",
                text.chars().take(200).collect::<String>()
            )));
        }

        let parsed: ChatResponse = serde_json::from_str(&text)
            .map_err(|e| KurultaiError::Query(format!("rerank json: {e}")))?;
        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .unwrap_or("")
            .trim();
        parse_id_list(content)
    }
}

fn parse_id_list(content: &str) -> Result<Vec<String>> {
    // Strip optional markdown fences.
    let trimmed = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let ids: Vec<String> = serde_json::from_str(trimmed).map_err(|e| {
        KurultaiError::Query(format!(
            "rerank expected JSON array of ids: {e} (got {})",
            trimmed.chars().take(120).collect::<String>()
        ))
    })?;
    Ok(ids)
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

/// Apply an ordered id list onto fused results; unknown ids skipped; leftovers append in RRF order.
pub fn apply_rerank_order(
    mut results: Vec<crate::types::SearchResult>,
    order: &[String],
) -> Vec<crate::types::SearchResult> {
    use std::collections::HashMap;
    let mut by_id: HashMap<String, crate::types::SearchResult> = results
        .drain(..)
        .map(|r| (r.atom.id.clone(), r))
        .collect();
    let mut out = Vec::with_capacity(by_id.len());
    for id in order {
        if let Some(r) = by_id.remove(id) {
            out.push(r);
        }
    }
    // Preserve relative RRF order for leftovers.
    let mut rest: Vec<_> = by_id.into_values().collect();
    rest.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.atom.id.cmp(&b.atom.id))
    });
    out.extend(rest);
    for (i, r) in out.iter_mut().enumerate() {
        r.rank = i;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{KnowledgeAtom, SearchResult};
    use chrono::Utc;
    use std::collections::HashMap;

    fn sr(id: &str, score: f64) -> SearchResult {
        SearchResult {
            atom: KnowledgeAtom {
                id: id.into(),
                source: "t".into(),
                source_id: id.into(),
                title: id.into(),
                summary: id.into(),
                content: id.into(),
                question: None,
                resolution: None,
                tags: vec![],
                source_updated_at: Utc::now(),
                indexed_at: Utc::now(),
                embedding: None,
                metadata: HashMap::new(),
            },
            score,
            rank: 0,
            matched_by: vec!["fts".into()],
        }
    }

    #[test]
    fn apply_rerank_reorders_and_keeps_unknown_tail() {
        let results = vec![sr("a", 0.9), sr("b", 0.8), sr("c", 0.7)];
        let out = apply_rerank_order(results, &["b".into(), "a".into()]);
        assert_eq!(out[0].atom.id, "b");
        assert_eq!(out[1].atom.id, "a");
        assert_eq!(out[2].atom.id, "c");
        assert_eq!(out[0].rank, 0);
    }

    #[test]
    fn parse_id_list_accepts_fence() {
        let ids = parse_id_list("```json\n[\"x\",\"y\"]\n```").unwrap();
        assert_eq!(ids, vec!["x", "y"]);
    }

    #[tokio::test]
    async fn null_reranker_not_live() {
        let r = NullReranker::new();
        assert!(!r.is_live());
    }
}
