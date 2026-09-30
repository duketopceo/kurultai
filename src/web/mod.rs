//! Ephemeral web search for `ask --web` — Perplexity Search API, REST-direct.
//!
//! Results are used as ask-time context only: they are never written to the
//! store, never embedded, and never touch `touch_access`/activity/quality
//! paths. No key → [`NullWebSearcher`] and `--web` degrades to a local answer.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::security::{api_key_from_env_optional, SecretString};

const SEARCH_URL: &str = "https://api.perplexity.ai/search";

/// One ranked web result — mirrors the `/search` response hit shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebHit {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub snippet: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Web search backend for ask-time augmentation.
#[async_trait::async_trait]
pub trait WebSearcher: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_live(&self) -> bool;
    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<WebHit>>;
}

/// No web backend configured.
pub struct NullWebSearcher;

#[async_trait::async_trait]
impl WebSearcher for NullWebSearcher {
    fn name(&self) -> &'static str {
        "null"
    }
    fn is_live(&self) -> bool {
        false
    }
    async fn search(&self, _query: &str, _max_results: usize) -> Result<Vec<WebHit>> {
        anyhow::bail!("no web searcher configured (set PERPLEXITY_API_KEY)")
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    results: Vec<WebHit>,
}

/// Perplexity `/search` client over reqwest (same pattern as the OpenRouter
/// surfaces — timeout client, bearer auth, soft-fail at call sites).
pub struct PerplexitySearcher {
    client: reqwest::Client,
    api_key: SecretString,
}

impl PerplexitySearcher {
    pub fn new(api_key: SecretString) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, api_key }
    }
}

#[async_trait::async_trait]
impl WebSearcher for PerplexitySearcher {
    fn name(&self) -> &'static str {
        "perplexity"
    }

    fn is_live(&self) -> bool {
        true
    }

    async fn search(&self, query: &str, max_results: usize) -> Result<Vec<WebHit>> {
        let body = serde_json::json!({
            "query": query,
            "max_results": max_results.clamp(1, 20),
        });
        let resp: SearchResponse = self
            .client
            .post(SEARCH_URL)
            .bearer_auth(self.api_key.expose())
            .json(&body)
            .send()
            .await
            .context("perplexity search request failed")?
            .error_for_status()
            .context("perplexity search returned an error")?
            .json()
            .await
            .context("perplexity search response was not valid JSON")?;
        Ok(resp.results)
    }
}

/// Build from env: `PERPLEXITY_API_KEY` only (deliberately a separate key —
/// OpenRouter keys must not be sent to api.perplexity.ai). Returns
/// [`NullWebSearcher`] when unset.
pub fn web_searcher_from_env() -> Arc<dyn WebSearcher> {
    match api_key_from_env_optional("PERPLEXITY_API_KEY") {
        Some(key) => Arc::new(PerplexitySearcher::new(key)),
        None => Arc::new(NullWebSearcher),
    }
}
