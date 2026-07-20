use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// Generates embeddings for text via an API or local model.
#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    fn name(&self) -> &str;
    fn dim(&self) -> usize;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed(t).await?);
        }
        Ok(results)
    }
}

/// Reject all-zero (and empty) embedding vectors — fail loud.
pub fn reject_zero_vector(v: &[f32]) -> Result<()> {
    if v.is_empty() {
        bail!("refuse empty embedding vector");
    }
    if v.iter().all(|x| *x == 0.0) {
        bail!("refuse zero-vector embedding");
    }
    Ok(())
}

/// No-op embedder for FTS-only mode (no API key). Always errors if called.
pub struct FtsOnlyEmbedder {
    dimension: usize,
}

impl FtsOnlyEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait::async_trait]
impl Embedder for FtsOnlyEmbedder {
    fn name(&self) -> &str {
        "fts-only"
    }
    fn dim(&self) -> usize {
        self.dimension
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        bail!("embeddings unavailable (no API key); use FTS search")
    }
}

/// Uses OpenRouter API for embeddings.
pub struct OpenRouterEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
    base_url: String,
}

impl OpenRouterEmbedder {
    pub fn new(api_key: String, model: String, dimension: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            dimension,
            base_url: "https://openrouter.ai/api/v1/embeddings".into(),
        }
    }

    /// Test helper / custom endpoint.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

#[async_trait::async_trait]
impl Embedder for OpenRouterEmbedder {
    fn name(&self) -> &str {
        &self.model
    }
    fn dim(&self) -> usize {
        self.dimension
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
            "dimensions": self.dimension,
        });
        let resp = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("openrouter embeddings request")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("openrouter embeddings HTTP {status}: {text}");
        }
        let parsed: EmbedResponse = resp.json().await.context("decode embeddings response")?;
        let emb = parsed
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| anyhow::anyhow!("empty embeddings response"))?;
        if emb.len() != self.dimension {
            bail!(
                "openrouter returned dim {} expected {}",
                emb.len(),
                self.dimension
            );
        }
        reject_zero_vector(&emb)?;
        Ok(emb)
    }
}

/// Build embedder from env: OpenRouter if key present, else FTS-only.
pub fn embedder_from_env(model: &str, dim: usize, key_env: &str) -> Box<dyn Embedder> {
    match std::env::var(key_env) {
        Ok(key) if !key.trim().is_empty() => {
            Box::new(OpenRouterEmbedder::new(key, model.to_string(), dim))
        }
        _ => Box::new(FtsOnlyEmbedder::new(dim)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_vector_rejected() {
        assert!(reject_zero_vector(&[0.0, 0.0]).is_err());
        assert!(reject_zero_vector(&[0.1, 0.0]).is_ok());
    }

    #[tokio::test]
    async fn fts_only_errors_on_embed() {
        let e = FtsOnlyEmbedder::new(8);
        assert!(e.embed("hi").await.is_err());
    }

    /// Stub that returns zeros — callers must reject via reject_zero_vector.
    struct ZeroStub;
    #[async_trait::async_trait]
    impl Embedder for ZeroStub {
        fn name(&self) -> &str {
            "zero"
        }
        fn dim(&self) -> usize {
            2
        }
        async fn embed(&self, _: &str) -> Result<Vec<f32>> {
            Ok(vec![0.0, 0.0])
        }
    }

    #[tokio::test]
    async fn stub_zero_rejected_by_guard() {
        let v = ZeroStub.embed("x").await.unwrap();
        assert!(reject_zero_vector(&v).is_err());
    }
}
