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

/// How the app selects an embedder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmbedBackend {
    #[default]
    Auto,
    OpenRouter,
    Local,
    Null,
}

impl EmbedBackend {
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" | "" => Ok(Self::Auto),
            "openrouter" | "cloud" => Ok(Self::OpenRouter),
            "local" => Ok(Self::Local),
            "null" | "none" | "fts" => Ok(Self::Null),
            other => Err(KurultaiError::config(format!(
                "unknown embed.backend '{other}' (expected auto|openrouter|local|null)"
            ))),
        }
    }
}

pub const OPENROUTER_EMBED_URL: &str = "https://openrouter.ai/api/v1/embeddings";
pub const DEFAULT_LOCAL_EMBED_URL: &str = "http://127.0.0.1:11434/v1/embeddings";
pub const DEFAULT_LOCAL_EMBED_MODEL: &str = "nomic-embed-text";
const DEFAULT_CLOUD_EMBED_MODEL: &str = "openai/text-embedding-3-large";
const BATCH_SIZE: usize = 32;
const MAX_RETRIES: u32 = 3;

/// OpenAI-compatible `/v1/embeddings` client (OpenRouter cloud or local Ollama/TEI).
pub struct HttpEmbedder {
    label: String,
    url: String,
    api_key: Option<String>,
    model: String,
    dimension: usize,
    client: reqwest::Client,
}

impl HttpEmbedder {
    pub fn openrouter(api_key: String, model: String, dimension: usize) -> Self {
        Self::new(
            "openrouter".into(),
            OPENROUTER_EMBED_URL.into(),
            Some(api_key),
            model,
            dimension,
        )
    }

    pub fn local(url: String, api_key: Option<String>, model: String, dimension: usize) -> Self {
        Self::new("local".into(), url, api_key, model, dimension)
    }

    pub fn new(
        label: String,
        url: String,
        api_key: Option<String>,
        model: String,
        dimension: usize,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            label,
            url,
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
            let mut req = self
                .client
                .post(&self.url)
                .header("Content-Type", "application/json")
                .json(&body);
            if let Some(key) = &self.api_key {
                req = req.bearer_auth(key);
            }

            let response = req.send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 429 || status.is_server_error() {
                        last_err = format!("{} {status}", self.label);
                        let backoff = Duration::from_millis(200 * 2u64.pow(attempt));
                        tracing::warn!(attempt, ?backoff, status = %status, "embed retry");
                        tokio::time::sleep(backoff).await;
                        continue;
                    }
                    if !status.is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        return Err(KurultaiError::Embed(format!(
                            "{} {status}: {}",
                            self.label,
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

/// Cloud OpenRouter embedder (HTTP) — same type as [`HttpEmbedder`].
pub type OpenRouterEmbedder = HttpEmbedder;

#[async_trait::async_trait]
impl Embedder for HttpEmbedder {
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

/// Resolve local model name when config still has the cloud default.
pub fn resolve_local_model(configured: &str) -> String {
    if configured.trim().is_empty() || configured == DEFAULT_CLOUD_EMBED_MODEL {
        DEFAULT_LOCAL_EMBED_MODEL.into()
    } else {
        configured.to_string()
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
    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::{json, Value};
    use tokio::net::TcpListener;

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
            HttpEmbedder::openrouter("test-key".into(), "openai/text-embedding-3-large".into(), 4);
        let err = e.embed("   ").await.unwrap_err().to_string();
        assert!(err.contains("empty"), "{err}");
    }

    #[tokio::test]
    async fn local_http_embedder_roundtrip() {
        async fn embeddings(Json(body): Json<Value>) -> Json<Value> {
            let n = body["input"].as_array().map(|a| a.len()).unwrap_or(1);
            let data: Vec<Value> = (0..n)
                .map(|i| {
                    json!({
                        "index": i,
                        "embedding": [0.1, 0.2, 0.3, 0.4],
                    })
                })
                .collect();
            Json(json!({ "data": data }))
        }

        let app = Router::new().route("/v1/embeddings", post(embeddings));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("http://{addr}/v1/embeddings");
        let e = HttpEmbedder::local(url, None, "nomic-embed-text".into(), 4);
        assert!(e.is_live());
        let v = e.embed("hello local").await.unwrap();
        assert_eq!(v, vec![0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn resolve_local_model_swaps_cloud_default() {
        assert_eq!(
            resolve_local_model(DEFAULT_CLOUD_EMBED_MODEL),
            DEFAULT_LOCAL_EMBED_MODEL
        );
        assert_eq!(resolve_local_model("my-model"), "my-model");
    }

    #[test]
    fn parse_embed_backend() {
        assert_eq!(EmbedBackend::parse("local").unwrap(), EmbedBackend::Local);
        assert_eq!(EmbedBackend::parse("AUTO").unwrap(), EmbedBackend::Auto);
        assert!(EmbedBackend::parse("weird").is_err());
    }
}
