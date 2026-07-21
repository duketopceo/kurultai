use crate::error::{KurultaiError, Result};
use serde::Deserialize;
use std::time::Duration;

/// Generates embeddings for text via an API or local model.
#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    fn name(&self) -> &str;
    fn dim(&self) -> usize;

    /// When false, the pipeline skips embedding (FTS-only / no API key).
    fn is_live(&self) -> bool {
        true
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/embeddings";
const BATCH_SIZE: usize = 32;
const MAX_RETRIES: u32 = 3;

/// Uses OpenRouter API for embeddings (fast, no local GPU).
pub struct OpenRouterEmbedder {
    api_key: String,
    model: String,
    dimension: usize,
    client: reqwest::Client,
}

impl OpenRouterEmbedder {
    pub fn new(api_key: String, model: String, dimension: usize) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            api_key,
            model,
            dimension,
            client,
        }
    }

    async fn embed_chunk(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        for t in texts {
            if t.trim().is_empty() {
                return Err(KurultaiError::Embed("empty text cannot be embedded".into()));
            }
        }

        let body = serde_json::json!({
            "model": self.model,
            "input": texts,
        });

        let mut last_err = String::new();
        for attempt in 0..MAX_RETRIES {
            let response = self
                .client
                .post(OPENROUTER_URL)
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 429 || status.is_server_error() {
                        last_err = format!("OpenRouter {status}");
                        let backoff = Duration::from_millis(200 * 2u64.pow(attempt));
                        tracing::warn!(attempt, ?backoff, status = %status, "embed retry");
                        tokio::time::sleep(backoff).await;
                        continue;
                    }
                    if !status.is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        return Err(KurultaiError::Embed(format!(
                            "OpenRouter {status}: {}",
                            body.chars().take(200).collect::<String>()
                        )));
                    }

                    let parsed: EmbeddingsResponse = resp
                        .json()
                        .await
                        .map_err(|e| KurultaiError::Embed(format!("decode response: {e}")))?;

                    let mut by_index: Vec<(usize, Vec<f32>)> = parsed
                        .data
                        .into_iter()
                        .map(|d| (d.index, d.embedding))
                        .collect();
                    by_index.sort_by_key(|(i, _)| *i);

                    if by_index.len() != texts.len() {
                        return Err(KurultaiError::Embed(format!(
                            "expected {} embeddings, got {}",
                            texts.len(),
                            by_index.len()
                        )));
                    }

                    let mut out = Vec::with_capacity(by_index.len());
                    for (_, emb) in by_index {
                        if emb.len() != self.dimension {
                            return Err(KurultaiError::Embed(format!(
                                "expected dim {}, got {}",
                                self.dimension,
                                emb.len()
                            )));
                        }
                        out.push(emb);
                    }
                    return Ok(out);
                }
                Err(e) => {
                    last_err = e.to_string();
                    let backoff = Duration::from_millis(200 * 2u64.pow(attempt));
                    tracing::warn!(attempt, ?backoff, error = %last_err, "embed network retry");
                    tokio::time::sleep(backoff).await;
                }
            }
        }

        Err(KurultaiError::Embed(format!(
            "embed failed after {MAX_RETRIES} retries: {last_err}"
        )))
    }
}

#[async_trait::async_trait]
impl Embedder for OpenRouterEmbedder {
    fn name(&self) -> &str {
        &self.model
    }
    fn dim(&self) -> usize {
        self.dimension
    }
    fn is_live(&self) -> bool {
        true
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut batch = self.embed_batch(&[text]).await?;
        batch
            .pop()
            .ok_or_else(|| KurultaiError::Embed("empty embed batch result".into()))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(BATCH_SIZE) {
            let mut part = self.embed_chunk(chunk).await?;
            results.append(&mut part);
        }
        Ok(results)
    }
}

/// FTS-first embedder when no API key is set — never produces vectors.
pub struct NullEmbedder {
    dimension: usize,
}

impl NullEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl Embedder for NullEmbedder {
    fn name(&self) -> &str {
        "none"
    }
    fn dim(&self) -> usize {
        self.dimension
    }
    fn is_live(&self) -> bool {
        false
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Err(KurultaiError::Embed(
            "NullEmbedder: no API key — FTS-only mode".into(),
        ))
    }

    async fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Err(KurultaiError::Embed(
            "NullEmbedder: no API key — FTS-only mode".into(),
        ))
    }
}

#[derive(Debug, Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn null_embedder_is_not_live() {
        let e = NullEmbedder::new(3072);
        assert!(!e.is_live());
        assert_eq!(e.name(), "none");
        assert!(e.embed("hi").await.is_err());
    }

    #[tokio::test]
    async fn openrouter_rejects_empty_text() {
        let e =
            OpenRouterEmbedder::new("test-key".into(), "openai/text-embedding-3-large".into(), 4);
        let err = e.embed("   ").await.unwrap_err().to_string();
        assert!(err.contains("empty"), "{err}");
    }
}
