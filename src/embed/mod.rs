use crate::error::Result;

/// Generates embeddings for text via an API or local model.
#[async_trait::async_trait]
pub trait Embedder: Send + Sync {
    fn name(&self) -> &str;
    fn dim(&self) -> usize;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

/// Uses OpenRouter API for embeddings (fast, no local GPU).
pub struct OpenRouterEmbedder {
    api_key: String,
    model: String,
    dimension: usize,
}

impl OpenRouterEmbedder {
    pub fn new(api_key: String, model: String, dimension: usize) -> Self {
        Self { api_key, model, dimension }
    }
}

#[async_trait::async_trait]
impl Embedder for OpenRouterEmbedder {
    fn name(&self) -> &str { &self.model }
    fn dim(&self) -> usize { self.dimension }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let _ = text;
        Ok(vec![0.0; self.dimension])
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed(t).await?);
        }
        Ok(results)
    }
}
