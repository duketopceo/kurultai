//! On-device ONNX embeddings via fastembed (feature `local-embed`).

use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use std::sync::{Arc, Mutex};

/// Local ONNX text embedder (fastembed).
pub struct LocalEmbedder {
    model_name: String,
    dimension: usize,
    inner: Arc<Mutex<fastembed::TextEmbedding>>,
}

impl LocalEmbedder {
    /// Build from a fastembed model id string (e.g. `AllMiniLML6V2`).
    pub fn try_new(model: &str, expected_dim: usize) -> Result<Self> {
        let embedding_model = parse_model(model)?;
        let info = fastembed::TextEmbedding::get_model_info(&embedding_model)
            .map_err(|e| KurultaiError::Embed(format!("local model info: {e}")))?;
        if info.dim != expected_dim {
            return Err(KurultaiError::config(format!(
                "local embed model {model} outputs dim {}, but embed.dimension is {expected_dim} — align config or use a new storage.path",
                info.dim
            )));
        }
        let options =
            fastembed::TextInitOptions::new(embedding_model).with_show_download_progress(true);
        let inner = fastembed::TextEmbedding::try_new(options)
            .map_err(|e| KurultaiError::Embed(format!("init local embedder: {e}")))?;
        Ok(Self {
            model_name: format!("local:{model}"),
            dimension: expected_dim,
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

fn parse_model(model: &str) -> Result<fastembed::EmbeddingModel> {
    let key = model.trim();
    match key {
        "AllMiniLML6V2" | "all-MiniLM-L6-v2" | "sentence-transformers/all-MiniLM-L6-v2" => {
            Ok(fastembed::EmbeddingModel::AllMiniLML6V2)
        }
        "BGESmallENV15" | "BAAI/bge-small-en-v1.5" => Ok(fastembed::EmbeddingModel::BGESmallENV15),
        "BGEBaseENV15" | "BAAI/bge-base-en-v1.5" => Ok(fastembed::EmbeddingModel::BGEBaseENV15),
        other => Err(KurultaiError::config(format!(
            "unsupported local embed.model {other:?}; try AllMiniLML6V2 (dim 384)"
        ))),
    }
}

#[async_trait::async_trait]
impl Embedder for LocalEmbedder {
    fn name(&self) -> &str {
        &self.model_name
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
            .ok_or_else(|| KurultaiError::Embed("local embed returned empty".into()))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        super::reject_empty_embed_texts(texts)?;
        let owned: Vec<String> = texts.iter().map(|t| (*t).to_string()).collect();
        let dimension = self.dimension;
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || {
            let mut guard = inner
                .lock()
                .map_err(|_| KurultaiError::Embed("local embedder mutex poisoned".into()))?;
            let embeddings = guard
                .embed(owned, None)
                .map_err(|e| KurultaiError::Embed(format!("local embed: {e}")))?;
            for emb in &embeddings {
                if emb.len() != dimension {
                    return Err(KurultaiError::Embed(format!(
                        "expected dim {dimension}, got {}",
                        emb.len()
                    )));
                }
            }
            Ok(embeddings)
        })
        .await
        .map_err(|e| KurultaiError::Embed(format!("local embed join: {e}")))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minilm_aliases() {
        assert!(parse_model("AllMiniLML6V2").is_ok());
        assert!(parse_model("all-MiniLM-L6-v2").is_ok());
        assert!(parse_model("nope").is_err());
    }
}
