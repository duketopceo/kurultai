use crate::error::{KurultaiError, Result};
use base64::Engine;
use serde::Deserialize;
use std::time::Duration;

use super::{reject_empty_embed_texts, Embedder, BATCH_SIZE, MAX_RETRIES};

const PPLX_URL: &str = "https://api.perplexity.ai/v1/embeddings";
/// Perplexity returns quantized vectors only — int8 is the widest format.
const ENCODING: &str = "base64_int8";
/// int8 quantization scale; cosine distance is scale-invariant so this only
/// keeps stored floats in a conventional [-1, 1] range.
const INT8_SCALE: f32 = 127.0;

/// Perplexity `pplx-embed-v1-*` API embedder.
///
/// The API has no float output — every response is base64-encoded signed
/// int8 (`base64_int8`) or packed bits (`base64_binary`, unsupported here
/// because the store keeps `Vec<f32>`). We decode int8 → f32 client-side.
/// Models are Matryoshka, so `dimensions` ≤ the model max can be requested
/// server-side instead of truncating.
pub struct PerplexityEmbedder {
    api_key: String,
    model: String,
    dimension: usize,
    client: reqwest::Client,
}

impl PerplexityEmbedder {
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

    fn decode_int8(&self, encoded: &str) -> Result<Vec<f32>> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|e| KurultaiError::Embed(format!("perplexity base64 decode: {e}")))?;
        if bytes.len() != self.dimension {
            return Err(KurultaiError::Embed(format!(
                "expected dim {}, got {} int8 values",
                self.dimension,
                bytes.len()
            )));
        }
        Ok(bytes
            .iter()
            .map(|b| (*b as i8) as f32 / INT8_SCALE)
            .collect())
    }

    async fn embed_chunk(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        reject_empty_embed_texts(texts)?;

        let body = serde_json::json!({
            "model": self.model,
            "input": texts,
            "dimensions": self.dimension,
            "encoding_format": ENCODING,
        });

        let mut last_err = String::new();
        for attempt in 0..MAX_RETRIES {
            let response = self
                .client
                .post(PPLX_URL)
                .bearer_auth(&self.api_key)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 429 || status.is_server_error() {
                        last_err = format!("Perplexity {status}");
                        let backoff = Duration::from_millis(200 * 2u64.pow(attempt));
                        tracing::warn!(attempt, ?backoff, status = %status, "embed retry");
                        tokio::time::sleep(backoff).await;
                        continue;
                    }
                    if !status.is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        return Err(KurultaiError::Embed(format!(
                            "Perplexity {status}: {}",
                            body.chars().take(200).collect::<String>()
                        )));
                    }

                    let parsed: PplxEmbeddingsResponse = resp
                        .json()
                        .await
                        .map_err(|e| KurultaiError::Embed(format!("decode response: {e}")))?;

                    let mut by_index: Vec<(usize, String)> = parsed
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

                    return by_index
                        .into_iter()
                        .map(|(_, emb)| self.decode_int8(&emb))
                        .collect();
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
impl Embedder for PerplexityEmbedder {
    fn name(&self) -> &str {
        &self.model
    }
    fn dim(&self) -> usize {
        self.dimension
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

#[derive(Debug, Deserialize)]
struct PplxEmbeddingsResponse {
    data: Vec<PplxEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct PplxEmbeddingData {
    embedding: String,
    index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn perplexity_rejects_empty_text() {
        let e = PerplexityEmbedder::new("test-key".into(), "pplx-embed-v1-0.6b".into(), 8);
        let err = e.embed("   ").await.unwrap_err().to_string();
        assert!(err.contains("empty"), "{err}");
    }

    #[test]
    fn decodes_int8_base64() {
        // [-127, -1, 0, 127] as int8 → scaled f32.
        let e = PerplexityEmbedder::new("k".into(), "m".into(), 4);
        let encoded = base64::engine::general_purpose::STANDARD.encode([129u8, 255, 0, 127]);
        let v = e.decode_int8(&encoded).unwrap();
        assert_eq!(v.len(), 4);
        assert!((v[0] - (-127.0 / 127.0)).abs() < 1e-6);
        assert!((v[1] - (-1.0 / 127.0)).abs() < 1e-6);
        assert_eq!(v[2], 0.0);
        assert!((v[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn decode_rejects_wrong_dim() {
        let e = PerplexityEmbedder::new("k".into(), "m".into(), 4);
        let encoded = base64::engine::general_purpose::STANDARD.encode([0u8; 8]);
        let err = e.decode_int8(&encoded).unwrap_err().to_string();
        assert!(err.contains("expected dim 4"), "{err}");
    }
}
