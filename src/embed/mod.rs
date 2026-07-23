use crate::error::{KurultaiError, Result};
use serde::Deserialize;

/// Operational mode of an embedder. Drives whether the pipeline attempts
/// embeddings or stays in FTS-only mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedMode {
    /// Embeddings are produced and stored.
    Full,
    /// No API key available; the pipeline skips embedding and relies on FTS.
    FtsOnly,
}

/// Generates embeddings for text via an API or local model.
#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    fn name(&self) -> &str;
    fn dim(&self) -> usize;
    fn mode(&self) -> EmbedMode;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

/// Uses OpenRouter API for embeddings (fast, no local GPU).
pub struct OpenRouterEmbedder {
    api_key: String,
    model: String,
    dimension: usize,
    mode: EmbedMode,
    client: reqwest::Client,
}

impl OpenRouterEmbedder {
    pub fn new(api_key: String, model: String, dimension: usize) -> Self {
        let mode = if api_key.trim().is_empty() {
            EmbedMode::FtsOnly
        } else {
            EmbedMode::Full
        };
        Self {
            api_key,
            model,
            dimension,
            mode,
            client: reqwest::Client::new(),
        }
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
    fn mode(&self) -> EmbedMode {
        self.mode
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if self.mode == EmbedMode::FtsOnly {
            return Err(KurultaiError::Embed(
                "embedder is in FTS-only mode (no API key)".into(),
            ));
        }

        if text.trim().is_empty() {
            return Err(KurultaiError::Embed("cannot embed empty text".into()));
        }

        #[derive(Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            data: Vec<EmbeddingData>,
        }

        let body = serde_json::json!({
            "model": self.model,
            "input": text,
        });

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| KurultaiError::Embed(format!("OpenRouter request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "<could not read response>".into());
            return Err(KurultaiError::Embed(format!(
                "OpenRouter returned {status}: {body_text}"
            )));
        }

        let payload: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| KurultaiError::Embed(format!("OpenRouter JSON parse failed: {e}")))?;

        let embedding = payload
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| {
                KurultaiError::Embed("OpenRouter returned empty embedding data".into())
            })?;

        if embedding.len() != self.dimension {
            return Err(KurultaiError::Embed(format!(
                "embedding dimension mismatch: got {}, expected {}",
                embedding.len(),
                self.dimension
            )));
        }

        if embedding.iter().all(|v| *v == 0.0) {
            return Err(KurultaiError::Embed(
                "OpenRouter returned zero vector — refusing to store".into(),
            ));
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed(t).await?);
        }
        Ok(results)
    }
}

/// Deterministic embedder for tests. Returns a unit vector rotated by the
/// text content so the same text always produces the same embedding.
pub struct FixedEmbedder {
    dimension: usize,
}

impl FixedEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl Embedder for FixedEmbedder {
    fn name(&self) -> &str {
        "fixed-test-embedder"
    }
    fn dim(&self) -> usize {
        self.dimension
    }
    fn mode(&self) -> EmbedMode {
        EmbedMode::Full
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut vec = vec![0.0f32; self.dimension];
        let bytes = text.as_bytes();
        if !bytes.is_empty() {
            for (i, byte) in bytes.iter().enumerate() {
                let idx = i % self.dimension;
                vec[idx] += (*byte as f32) / 255.0;
            }
        }
        // normalize
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            vec.iter_mut().for_each(|v| *v /= norm);
        } else {
            vec[0] = 1.0;
        }
        Ok(vec)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed(t).await?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn openrouter_without_key_is_fts_only() {
        let embedder = OpenRouterEmbedder::new("".into(), "model".into(), 4);
        assert_eq!(embedder.mode(), EmbedMode::FtsOnly);
        assert!(embedder.embed("hello").await.is_err());
    }

    #[tokio::test]
    async fn fixed_embedder_is_deterministic() {
        let e = FixedEmbedder::new(8);
        let a = e.embed("hello").await.unwrap();
        let b = e.embed("hello").await.unwrap();
        assert_eq!(a, b);
        assert!(a.iter().any(|v| *v != 0.0));
    }

    #[tokio::test]
    async fn fixed_embedder_differs_for_different_text() {
        let e = FixedEmbedder::new(8);
        let a = e.embed("hello").await.unwrap();
        let b = e.embed("world").await.unwrap();
        assert_ne!(a, b);
    }
}
