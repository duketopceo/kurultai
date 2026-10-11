//! Late-interaction (ColBERT-style) multi-vector embeddings.
//!
//! Dense [`Embedder`] emits one vector per atom and drives `atoms_vec` KNN
//! recall. `MultiVectorEmbedder` emits one `token_dim` vector per *token*;
//! documents are scored against query token vectors with MaxSim and used as a
//! second-stage reranker over the hybrid candidate set (`[embed.late]`).

use crate::error::{KurultaiError, Result};
use serde::Deserialize;
use std::time::Duration;

/// Per-token multi-vector embedder (late interaction).
#[async_trait::async_trait]
pub trait MultiVectorEmbedder: Send + Sync {
    fn name(&self) -> &str;
    /// Per-token vector width (128 for pplx-embed-v2-late, 1024 for BGE-M3).
    fn token_dim(&self) -> usize;

    /// When false, indexing and rerank skip the lane entirely.
    fn is_live(&self) -> bool {
        true
    }

    /// Embed a query into per-token vectors.
    async fn embed_query(&self, text: &str) -> Result<Vec<Vec<f32>>>;
    /// Embed a document into per-token vectors (capped by the backend).
    async fn embed_document(&self, text: &str) -> Result<Vec<Vec<f32>>>;

    /// Batch documents — default loops; backends override to send one request.
    async fn embed_document_batch(&self, texts: &[&str]) -> Result<Vec<Vec<Vec<f32>>>> {
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed_document(t).await?);
        }
        Ok(out)
    }
}

/// Disabled lane (`[embed.late] backend = "off"` or unset).
#[derive(Default)]
pub struct NullMultiVectorEmbedder;

impl NullMultiVectorEmbedder {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl MultiVectorEmbedder for NullMultiVectorEmbedder {
    fn name(&self) -> &str {
        "none"
    }
    fn token_dim(&self) -> usize {
        0
    }
    fn is_live(&self) -> bool {
        false
    }
    async fn embed_query(&self, _text: &str) -> Result<Vec<Vec<f32>>> {
        Ok(Vec::new())
    }
    async fn embed_document(&self, _text: &str) -> Result<Vec<Vec<f32>>> {
        Ok(Vec::new())
    }
}

/// ColBERT MaxSim: for each query token vector take the max cosine similarity
/// over all document token vectors, then sum. Vectors are L2-normalized before
/// scoring so cosine reduces to dot product.
pub fn maxsim(query_tokens: &[Vec<f32>], doc_tokens: &[Vec<f32>]) -> f32 {
    if query_tokens.is_empty() || doc_tokens.is_empty() {
        return 0.0;
    }
    let doc_norms: Vec<Vec<f32>> = doc_tokens.iter().map(|t| l2_normalize(t)).collect();
    query_tokens
        .iter()
        .map(|qt| {
            let qn = l2_normalize(qt);
            doc_norms
                .iter()
                .map(|dt| dot(&qn, dt))
                .fold(f32::MIN, f32::max)
        })
        .sum()
}

fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm <= f32::EPSILON {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

const DEFAULT_MAX_DOC_TOKENS: usize = 512;

/// HTTP backend: a sidecar server (e.g. `scripts/late-embed-server.py` running
/// `sentence-transformers` `MultiVectorEncoder` on `pplx-embed-v2-late-0.6b`).
///
/// Contract:
/// - `POST {url}/embed_query`  `{ "texts": [..] }` → `{ "vectors": [[[f32];dim];T] }`
/// - `POST {url}/embed_document` `{ "texts": [..] }` → same shape.
///   Each list item is the document's per-token matrix.
pub struct LateHttpEmbedder {
    url: String,
    token_dim: usize,
    max_doc_tokens: usize,
    client: reqwest::Client,
}

impl LateHttpEmbedder {
    pub fn new(url: String, token_dim: usize, max_doc_tokens: Option<usize>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            url: url.trim_end_matches('/').to_string(),
            token_dim,
            max_doc_tokens: max_doc_tokens.unwrap_or(DEFAULT_MAX_DOC_TOKENS),
            client,
        }
    }

    async fn call(&self, path: &str, text: &str) -> Result<Vec<Vec<f32>>> {
        if text.trim().is_empty() {
            return Err(KurultaiError::Embed("empty text cannot be embedded".into()));
        }
        let resp = self
            .client
            .post(format!("{}{}", self.url, path))
            .json(&serde_json::json!({ "texts": [text] }))
            .send()
            .await
            .map_err(|e| KurultaiError::Embed(format!("late-embed http: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(KurultaiError::Embed(format!(
                "late-embed {status}: {}",
                &body[..body.len().min(200)]
            )));
        }
        let parsed: LateEmbedResponse = resp
            .json()
            .await
            .map_err(|e| KurultaiError::Embed(format!("late-embed decode: {e}")))?;
        let mut mat = parsed
            .vectors
            .into_iter()
            .next()
            .ok_or_else(|| KurultaiError::Embed("late-embed returned no vectors".into()))?;
        mat.truncate(self.max_doc_tokens);
        for (i, v) in mat.iter().enumerate() {
            if v.len() != self.token_dim {
                return Err(KurultaiError::Embed(format!(
                    "late-embed token {i} dim {} != token_dim {}",
                    v.len(),
                    self.token_dim
                )));
            }
        }
        Ok(mat)
    }
}

#[async_trait::async_trait]
impl MultiVectorEmbedder for LateHttpEmbedder {
    fn name(&self) -> &str {
        "late-http"
    }
    fn token_dim(&self) -> usize {
        self.token_dim
    }
    async fn embed_query(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        self.call("/embed_query", text).await
    }
    async fn embed_document(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        self.call("/embed_document", text).await
    }
    async fn embed_document_batch(&self, texts: &[&str]) -> Result<Vec<Vec<Vec<f32>>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        if texts.iter().any(|t| t.trim().is_empty()) {
            return Err(KurultaiError::Embed("empty text cannot be embedded".into()));
        }
        let resp = self
            .client
            .post(format!("{}/embed_document", self.url))
            .json(&serde_json::json!({ "texts": texts }))
            .send()
            .await
            .map_err(|e| KurultaiError::Embed(format!("late-embed http: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(KurultaiError::Embed(format!(
                "late-embed {status}: {}",
                &body[..body.len().min(200)]
            )));
        }
        let parsed: LateEmbedResponse = resp
            .json()
            .await
            .map_err(|e| KurultaiError::Embed(format!("late-embed decode: {e}")))?;
        if parsed.vectors.len() != texts.len() {
            return Err(KurultaiError::Embed(format!(
                "late-embed returned {} docs for {} inputs",
                parsed.vectors.len(),
                texts.len()
            )));
        }
        let mut out = parsed.vectors;
        for mat in &mut out {
            mat.truncate(self.max_doc_tokens);
            for (i, v) in mat.iter().enumerate() {
                if v.len() != self.token_dim {
                    return Err(KurultaiError::Embed(format!(
                        "late-embed token {i} dim {} != token_dim {}",
                        v.len(),
                        self.token_dim
                    )));
                }
            }
        }
        Ok(out)
    }
}

#[derive(Deserialize)]
struct LateEmbedResponse {
    vectors: Vec<Vec<Vec<f32>>>,
}

/// On-device late interaction via fastembed's BGE-M3 (feature `local-embed`).
/// Uses the model's `colbert` output head; token_dim = 1024.
#[cfg(feature = "local-embed")]
pub struct LateBgem3Embedder {
    inner: std::sync::Arc<std::sync::Mutex<fastembed::Bgem3Embedding>>,
    max_doc_tokens: usize,
}

#[cfg(feature = "local-embed")]
impl LateBgem3Embedder {
    pub fn try_new(max_doc_tokens: Option<usize>) -> Result<Self> {
        let options = fastembed::Bgem3InitOptions::new(fastembed::Bgem3Model::BGEM3Q)
            .with_show_download_progress(true);
        let inner = fastembed::Bgem3Embedding::try_new(options)
            .map_err(|e| KurultaiError::Embed(format!("init late bgem3: {e}")))?;
        Ok(Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
            max_doc_tokens: max_doc_tokens.unwrap_or(DEFAULT_MAX_DOC_TOKENS),
        })
    }

    async fn embed_texts(&self, texts: &[&str]) -> Result<Vec<Vec<Vec<f32>>>> {
        if texts.iter().any(|t| t.trim().is_empty()) {
            return Err(KurultaiError::Embed("empty text cannot be embedded".into()));
        }
        let owned: Vec<String> = texts.iter().map(|t| t.to_string()).collect();
        let inner = std::sync::Arc::clone(&self.inner);
        let cap = self.max_doc_tokens;
        tokio::task::spawn_blocking(move || {
            let mut out = inner
                .lock()
                .map_err(|_| KurultaiError::Embed("late bgem3 lock poisoned".into()))?
                .embed(owned, None)
                .map_err(|e| KurultaiError::Embed(format!("late bgem3 embed: {e}")))?;
            for mat in &mut out.colbert {
                mat.truncate(cap);
            }
            Ok(out.colbert)
        })
        .await
        .map_err(|e| KurultaiError::Embed(format!("late bgem3 join: {e}")))?
    }
}

#[cfg(feature = "local-embed")]
#[async_trait::async_trait]
impl MultiVectorEmbedder for LateBgem3Embedder {
    fn name(&self) -> &str {
        "late-bgem3"
    }
    fn token_dim(&self) -> usize {
        1024
    }
    async fn embed_query(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        Ok(self
            .embed_texts(&[text])
            .await?
            .into_iter()
            .next()
            .unwrap_or_default())
    }
    async fn embed_document(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        self.embed_query(text).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maxsim_prefers_matching_docs() {
        let q = vec![vec![1.0, 0.0, 0.0]];
        let matching = vec![vec![0.9, 0.1, 0.0], vec![0.0, 1.0, 0.0]];
        let orthogonal = vec![vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]];
        assert!(maxsim(&q, &matching) > maxsim(&q, &orthogonal));
    }

    #[test]
    fn maxsim_empty_inputs_score_zero() {
        assert_eq!(maxsim(&[], &[vec![1.0]]), 0.0);
        assert_eq!(maxsim(&[vec![1.0]], &[]), 0.0);
    }

    #[test]
    fn maxsim_norms_so_scale_invariant() {
        let q = vec![vec![1.0, 0.0]];
        let small = vec![vec![1.0, 0.0]];
        let big = vec![vec![10.0, 0.0]];
        assert!((maxsim(&q, &small) - maxsim(&q, &big)).abs() < 1e-6);
    }

    #[tokio::test]
    async fn null_late_is_not_live() {
        let e = NullMultiVectorEmbedder::new();
        assert!(!e.is_live());
        assert_eq!(e.token_dim(), 0);
    }
}
