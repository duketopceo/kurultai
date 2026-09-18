//! Optional LLM judge for graded relevance and groundedness scoring.
//!
//! Backend: TypeSafe Jev via the OpenRouter decisions API
//! (`POST https://openrouter.ai/api/alpha/decisions`). Jev returns typed,
//! calibrated answers (`noul`/`choice`/`score`) rather than text — a good fit
//! for per-pair relevance grading and rubric scoring.
//!
//! Env-gated like the other OpenRouter surfaces: no key → [`NullJudge`] and
//! the eval run still emits labels-only metrics.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::security::{api_key_from_env_optional, api_key_from_keyfile, SecretString};

const DECISIONS_URL: &str = "https://openrouter.ai/api/alpha/decisions";
/// Pinned judge model — thresholds and reports are only comparable within a
/// model version. `~typesafe/jev-latest` is a moving alias; do not use it.
pub const DEFAULT_JUDGE_MODEL: &str = "typesafe/jev-1.13";

/// One typed question for the decisions API.
#[derive(Debug, Clone)]
pub enum Question {
    /// yes/no probability ("noul" — true/false descriptions are required).
    Noul {
        instructions: String,
        on_true: String,
        on_false: String,
    },
    /// Ordered rubric; `criteria` is low → high (2–10 levels).
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
}

impl Question {
    fn to_json(&self) -> Value {
        match self {
            Question::Noul {
                instructions,
                on_true,
                on_false,
            } => json!({
                "type": "noul",
                "instructions": instructions,
                "true": on_true,
                "false": on_false,
            }),
            Question::Score {
                instructions,
                criteria,
            } => json!({
                "type": "score",
                "instructions": instructions,
                "criteria": criteria,
            }),
        }
    }
}

/// Answers for one decisions call, keyed by question id.
#[derive(Debug, Clone, Default)]
pub struct DecisionAnswers {
    /// qid → probability of "yes" for noul questions.
    pub noul: Map<String, Value>,
    /// qid → fractional rubric score for score questions.
    pub score: Map<String, Value>,
    /// Resolved model slug that answered (e.g. `jev-1.13-20260917`).
    pub resolved_model: Option<String>,
    /// Total USD cost reported by OpenRouter, when present.
    pub cost_usd: Option<f64>,
}

/// Grading backend for eval scoring.
#[async_trait::async_trait]
pub trait Judge: Send + Sync {
    fn name(&self) -> &'static str;
    /// Whether a live backend is configured.
    fn is_live(&self) -> bool;
    /// Ask typed questions about `state`. Questions evaluate in parallel.
    async fn decide(
        &self,
        state: &Value,
        questions: &[(String, Question)],
    ) -> Result<DecisionAnswers>;
}

/// No judge configured — graded metrics are skipped.
pub struct NullJudge;

#[async_trait::async_trait]
impl Judge for NullJudge {
    fn name(&self) -> &'static str {
        "null"
    }
    fn is_live(&self) -> bool {
        false
    }
    async fn decide(
        &self,
        _state: &Value,
        _questions: &[(String, Question)],
    ) -> Result<DecisionAnswers> {
        anyhow::bail!("no judge configured (set OPENROUTER_API_KEY)")
    }
}

/// Jev decisions client over reqwest (mirrors `OpenRouterSynthesizer`).
pub struct OpenRouterJudge {
    client: reqwest::Client,
    api_key: SecretString,
    model: String,
}

#[derive(Debug, Deserialize)]
struct DecisionsResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    answers: Map<String, Value>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    #[serde(default)]
    cost: Option<f64>,
}

impl OpenRouterJudge {
    pub fn new(api_key: SecretString, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            api_key,
            model,
        }
    }
}

#[async_trait::async_trait]
impl Judge for OpenRouterJudge {
    fn name(&self) -> &'static str {
        "openrouter-decisions"
    }

    fn is_live(&self) -> bool {
        true
    }

    async fn decide(
        &self,
        state: &Value,
        questions: &[(String, Question)],
    ) -> Result<DecisionAnswers> {
        let questions_json: Map<String, Value> = questions
            .iter()
            .map(|(id, q)| (id.clone(), q.to_json()))
            .collect();
        let body = json!({
            "model": self.model,
            "state": state,
            "questions": questions_json,
        });
        let resp: DecisionsResponse = self
            .client
            .post(DECISIONS_URL)
            .bearer_auth(self.api_key.expose())
            .header("HTTP-Referer", "https://github.com/duketopceo/kurultai")
            .header("X-OpenRouter-Title", "kurultai-eval")
            .json(&body)
            .send()
            .await
            .context("decisions request failed")?
            .error_for_status()
            .context("decisions API returned an error")?
            .json()
            .await
            .context("decisions response was not valid JSON")?;

        let mut out = DecisionAnswers {
            resolved_model: resp.model,
            cost_usd: resp.usage.and_then(|u| u.cost),
            ..Default::default()
        };
        for (qid, answer) in resp.answers {
            if let Some(p) = answer.get("noul") {
                out.noul.insert(qid.clone(), p.clone());
            }
            if let Some(s) = answer.get("score") {
                out.score.insert(qid.clone(), s.clone());
            }
        }
        Ok(out)
    }
}

/// Build the judge from env: `OPENROUTER_API_KEY` → `KURULTAI_API_KEY` →
/// keyfile (same chain as embedder/synthesizer). `model` falls back to the
/// pinned default. Never logs the key.
pub fn judge_from_env(model: Option<String>) -> Arc<dyn Judge> {
    match api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"))
        .or_else(api_key_from_keyfile)
    {
        Some(key) => Arc::new(OpenRouterJudge::new(
            key,
            model.unwrap_or_else(|| DEFAULT_JUDGE_MODEL.to_string()),
        )),
        None => Arc::new(NullJudge),
    }
}

/// Build the judge honoring `[judge]` config: `enabled = false` forces
/// [`NullJudge`] even when a key resolves; `model` overrides the pin.
pub fn judge_from_config(cfg: &crate::types::Config) -> Arc<dyn Judge> {
    if !cfg.judge_enabled {
        return Arc::new(NullJudge);
    }
    judge_from_env(cfg.judge_model.clone())
}
