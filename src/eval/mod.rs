//! Retrieval eval harness: run a frozen golden query set against a live
//! daemon (`POST /api/search`, `POST /api/ask`) and score the results.
//!
//! Labels in `evals/golden.json` are hand-maintained — they match on title /
//! source_id / title_hash, never on atom `id` (a content hash that changes
//! when the underlying text is edited). Optional [`judge`] grading adds nDCG
//! and groundedness; without it the run is labels-only.

pub mod judge;
pub mod metrics;

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use judge::{Judge, Question};

/// A frozen benchmark: queries plus expected-relevance matchers.
#[derive(Debug, Deserialize)]
pub struct GoldenSet {
    pub version: u32,
    pub queries: Vec<GoldenQuery>,
}

#[derive(Debug, Deserialize)]
pub struct GoldenQuery {
    pub id: String,
    /// `search` (default) or `ask`.
    #[serde(default = "default_kind")]
    pub kind: String,
    pub query: String,
    /// A hit is relevant when ANY matcher field matches.
    #[serde(default)]
    pub relevant: Vec<Matcher>,
    /// Sources that must not appear in results (noise exclusion).
    #[serde(default)]
    pub deny_sources: Vec<String>,
    /// For `ask` queries: judge the answer for groundedness when a judge is
    /// configured.
    #[serde(default)]
    pub judge_answer: bool,
}

fn default_kind() -> String {
    "search".to_string()
}

/// Relevance matcher — one object, first matching field wins.
#[derive(Debug, Deserialize, Default)]
pub struct Matcher {
    /// Case-insensitive substring match on `atom.title`.
    pub title_contains: Option<String>,
    /// Exact match on `atom.title`.
    pub title: Option<String>,
    /// Substring match on `atom.source_id`.
    pub source_id_contains: Option<String>,
    /// Exact match on `atom.source_id`.
    pub source_id: Option<String>,
    /// `Citation.title_hash` short hash (stable across content edits).
    pub title_hash: Option<String>,
}

impl Matcher {
    /// Does this matcher hit the given title / source_id / title_hash?
    pub fn matches(&self, title: &str, source_id: &str, title_hash: Option<&str>) -> bool {
        if let Some(t) = &self.title_contains {
            if title.to_lowercase().contains(&t.to_lowercase()) {
                return true;
            }
        }
        if let Some(t) = &self.title {
            if title == t {
                return true;
            }
        }
        if let Some(s) = &self.source_id_contains {
            if source_id.contains(s.as_str()) {
                return true;
            }
        }
        if let Some(s) = &self.source_id {
            if source_id == s {
                return true;
            }
        }
        if let (Some(want), Some(got)) = (&self.title_hash, title_hash) {
            if want == got {
                return true;
            }
        }
        false
    }
}

impl GoldenSet {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("cannot read golden set {}", path.display()))?;
        let set: GoldenSet =
            serde_json::from_str(&text).with_context(|| "golden set is not valid JSON")?;
        anyhow::ensure!(!set.queries.is_empty(), "golden set has no queries");
        Ok(set)
    }
}

/// CLI knobs for one eval run.
pub struct EvalConfig {
    pub base_url: String,
    pub k: usize,
    pub judge: bool,
    pub judge_model: Option<String>,
    /// Test seam: inject a judge instead of building from env.
    #[doc(hidden)]
    pub judge_override: Option<Arc<dyn Judge>>,
}

#[derive(Debug, Serialize)]
pub struct QueryReport {
    pub id: String,
    pub kind: String,
    pub query: String,
    pub hits: usize,
    pub recall_at_k: Option<f64>,
    pub precision_at_k: Option<f64>,
    pub mrr: Option<f64>,
    /// Graded nDCG — present only when the judge is live.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndcg_at_k: Option<f64>,
    /// Judge-groundedness rubric for ask queries (0–1 normalized).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groundedness: Option<f64>,
    /// Extractive/LLM `Answer.confidence` for ask queries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer_confidence: Option<f64>,
    /// Sources found in results that the query denied.
    pub noise_violations: Vec<String>,
    pub latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct EvalReport {
    pub base_url: String,
    pub k: usize,
    pub git_sha: Option<String>,
    pub judge: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_cost_usd: Option<f64>,
    /// First judge error when grading was disabled mid-run (e.g. insufficient
    /// inference credits). Absent when grading stayed healthy or was off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_disabled_reason: Option<String>,
    pub queries: Vec<QueryReport>,
    pub aggregate: Aggregate,
}

#[derive(Debug, Serialize)]
pub struct Aggregate {
    pub mean_recall_at_k: f64,
    pub mean_precision_at_k: f64,
    pub mean_mrr: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_ndcg_at_k: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_groundedness: Option<f64>,
    pub noise_violations: usize,
    pub queries: usize,
}

#[derive(Debug, Deserialize)]
struct SearchHitView {
    title: String,
    source_id: String,
    source: String,
}

/// Minimal `/api/search` hit shape — avoids depending on full `SearchResult`
/// deserialization staying in lockstep with the daemon.
#[derive(Debug, Deserialize)]
struct SearchResultWire {
    atom: SearchHitView,
}

#[derive(Debug, Deserialize)]
struct CitationWire {
    title: String,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnswerWire {
    answer: String,
    #[serde(default)]
    citations: Vec<CitationWire>,
    #[serde(default)]
    confidence: f64,
}

/// Run the golden set against `base_url`. Blocking (eval runs from the CLI,
/// not inside the daemon runtime).
pub async fn run_eval(cfg: &EvalConfig, golden: &GoldenSet) -> Result<EvalReport> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let judge: Arc<dyn Judge> = match (&cfg.judge_override, cfg.judge) {
        (Some(j), _) => Arc::clone(j),
        (None, true) => judge::judge_from_env(cfg.judge_model.clone()),
        (None, false) => Arc::new(judge::NullJudge),
    };
    if cfg.judge && !judge.is_live() {
        tracing::warn!("--judge set but no OpenRouter key — falling back to labels-only metrics");
    }

    let mut reports = Vec::new();
    let mut judge_cost = 0.0f64;
    let mut judge_model: Option<String> = None;
    // Circuit breaker: stop calling the judge after N consecutive failures
    // (dead key, out of credits, API down) so a broken backend doesn't burn a
    // request per remaining query. Report flags the degradation.
    const JUDGE_FAIL_MAX: usize = 3;
    let mut judge_failures = 0usize;
    let mut judge_disabled_reason: Option<String> = None;

    for q in &golden.queries {
        let start = Instant::now();
        let mut rep = QueryReport {
            id: q.id.clone(),
            kind: q.kind.clone(),
            query: q.query.clone(),
            hits: 0,
            recall_at_k: None,
            precision_at_k: None,
            mrr: None,
            ndcg_at_k: None,
            groundedness: None,
            answer_confidence: None,
            noise_violations: Vec::new(),
            latency_ms: 0,
        };

        match q.kind.as_str() {
            "search" => {
                let hits: Vec<SearchResultWire> = client
                    .post(format!("{}/api/search", cfg.base_url))
                    .json(&json!({"query": q.query, "limit": cfg.k}))
                    .send()
                    .await
                    .with_context(|| format!("search request failed for {}", q.id))?
                    .error_for_status()
                    .with_context(|| format!("search returned error for {}", q.id))?
                    .json()
                    .await
                    .context("search response was not valid JSON")?;
                rep.hits = hits.len();

                let flags: Vec<bool> = hits
                    .iter()
                    .map(|h| {
                        q.relevant
                            .iter()
                            .any(|m| m.matches(&h.atom.title, &h.atom.source_id, None))
                    })
                    .collect();
                rep.recall_at_k = Some(metrics::recall_at_k(&flags, q.relevant.len(), cfg.k));
                rep.precision_at_k = Some(metrics::precision_at_k(&flags, cfg.k));
                rep.mrr = Some(metrics::mrr(&flags));

                for h in &hits {
                    if q.deny_sources.iter().any(|s| s == &h.atom.source) {
                        rep.noise_violations.push(h.atom.source.clone());
                    }
                }

                if judge.is_live() && judge_disabled_reason.is_none() && !hits.is_empty() {
                    match grade_hits(&*judge, q, &hits, cfg.k).await {
                        Ok((ndcg, cost, model)) => {
                            rep.ndcg_at_k = Some(ndcg);
                            judge_cost += cost;
                            judge_failures = 0;
                            if judge_model.is_none() {
                                judge_model = model;
                            }
                        }
                        Err(e) => {
                            judge_failures += 1;
                            if judge_failures >= JUDGE_FAIL_MAX {
                                judge_disabled_reason = Some(format!("{e:#}"));
                                tracing::warn!(
                                    "judge disabled after {JUDGE_FAIL_MAX} consecutive failures \
                                     — remaining queries labels-only"
                                );
                            } else {
                                tracing::warn!("judge failed for {}: {e:#} — skipping nDCG", q.id)
                            }
                        }
                    }
                }
            }
            "ask" => {
                let ans: AnswerWire = client
                    .post(format!("{}/api/ask", cfg.base_url))
                    .json(&json!({"question": q.query}))
                    .send()
                    .await
                    .with_context(|| format!("ask request failed for {}", q.id))?
                    .error_for_status()
                    .with_context(|| format!("ask returned error for {}", q.id))?
                    .json()
                    .await
                    .context("ask response was not valid JSON")?;
                rep.hits = ans.citations.len();
                rep.answer_confidence = Some(ans.confidence);

                if judge.is_live() && judge_disabled_reason.is_none() && q.judge_answer {
                    match grade_answer(&*judge, q, &ans).await {
                        Ok((grounded, cost, model)) => {
                            rep.groundedness = Some(grounded);
                            judge_cost += cost;
                            judge_failures = 0;
                            if judge_model.is_none() {
                                judge_model = model;
                            }
                        }
                        Err(e) => {
                            judge_failures += 1;
                            if judge_failures >= JUDGE_FAIL_MAX {
                                judge_disabled_reason = Some(format!("{e:#}"));
                                tracing::warn!(
                                    "judge disabled after {JUDGE_FAIL_MAX} consecutive failures \
                                     — remaining queries labels-only"
                                );
                            } else {
                                tracing::warn!("judge failed for {}: {e:#} — skipping", q.id)
                            }
                        }
                    }
                }
            }
            other => anyhow::bail!("unknown golden query kind {other:?} in {}", q.id),
        }

        rep.latency_ms = start.elapsed().as_millis() as u64;
        reports.push(rep);
    }

    let collect =
        |f: fn(&QueryReport) -> Option<f64>| -> Vec<f64> { reports.iter().filter_map(f).collect() };
    let ndcgs = collect(|r| r.ndcg_at_k);
    let grounded = collect(|r| r.groundedness);
    let aggregate = Aggregate {
        mean_recall_at_k: metrics::mean(&collect(|r| r.recall_at_k)),
        mean_precision_at_k: metrics::mean(&collect(|r| r.precision_at_k)),
        mean_mrr: metrics::mean(&collect(|r| r.mrr)),
        mean_ndcg_at_k: (!ndcgs.is_empty()).then(|| metrics::mean(&ndcgs)),
        mean_groundedness: (!grounded.is_empty()).then(|| metrics::mean(&grounded)),
        noise_violations: reports.iter().map(|r| r.noise_violations.len()).sum(),
        queries: reports.len(),
    };

    Ok(EvalReport {
        base_url: cfg.base_url.clone(),
        k: cfg.k,
        git_sha: std::env::var("KURULTAI_EVAL_GIT_SHA")
            .ok()
            .or_else(|| option_env!("VERGEN_GIT_SHA").map(String::from)),
        judge: judge.name().to_string(),
        judge_model,
        judge_cost_usd: (judge_cost > 0.0).then_some(judge_cost),
        judge_disabled_reason,
        queries: reports,
        aggregate,
    })
}

/// Grade each hit's relevance on a 0–3 rubric in one batched decisions call,
/// then compute nDCG@k against the ideal ordering of those gains.
async fn grade_hits(
    judge: &dyn Judge,
    q: &GoldenQuery,
    hits: &[SearchResultWire],
    k: usize,
) -> Result<(f64, f64, Option<String>)> {
    let state = json!({
        "query": q.query,
        "hits": hits.iter().take(k).enumerate().map(|(i, h)| {
            json!({"rank": i + 1, "title": h.atom.title, "source_id": h.atom.source_id})
        }).collect::<Vec<_>>(),
    });
    let questions: Vec<(String, Question)> = hits
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, _)| {
            (
                format!("rel_{}", i),
                Question::Score {
                    instructions: format!(
                        "How relevant is the hit at rank {} to the query? Judge title and source path only.",
                        i + 1
                    ),
                    criteria: vec![
                        "irrelevant".into(),
                        "tangential".into(),
                        "relevant".into(),
                        "directly answers".into(),
                    ],
                },
            )
        })
        .collect();
    let answers = judge.decide(&state, &questions).await?;
    let gains: Vec<f64> = (0..hits.len().min(k))
        .map(|i| {
            answers
                .score
                .get(&format!("rel_{i}"))
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        })
        .collect();
    let mut ideal = gains.clone();
    ideal.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Ok((
        metrics::ndcg_at_k(&gains, &ideal, k),
        answers.cost_usd.unwrap_or(0.0),
        answers.resolved_model,
    ))
}

/// Score an ask answer for groundedness (0–1) via rubric + support check.
async fn grade_answer(
    judge: &dyn Judge,
    q: &GoldenQuery,
    ans: &AnswerWire,
) -> Result<(f64, f64, Option<String>)> {
    let state = json!({
        "question": q.query,
        "answer": ans.answer,
        "citations": ans.citations.iter().map(|c| {
            json!({"title": c.title, "url": c.url})
        }).collect::<Vec<_>>(),
    });
    let questions = vec![
        (
            "grounded".to_string(),
            Question::Score {
                instructions: "How well is the answer supported by the cited excerpts?".to_string(),
                criteria: vec![
                    "unsupported or wrong".into(),
                    "partially supported".into(),
                    "well supported".into(),
                    "fully grounded in citations".into(),
                ],
            },
        ),
        (
            "correct".to_string(),
            Question::Noul {
                instructions: "Is the answer responsive to the question asked?".to_string(),
                on_true: "Directly answers the question".to_string(),
                on_false: "Off-topic, evasive, or empty".to_string(),
            },
        ),
    ];
    let answers = judge.decide(&state, &questions).await?;
    let rubric = answers
        .score
        .get("grounded")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        / 3.0;
    let correct = answers
        .noul
        .get("correct")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let grounded = (rubric * 0.7 + correct * 0.3).clamp(0.0, 1.0);
    Ok((
        grounded,
        answers.cost_usd.unwrap_or(0.0),
        answers.resolved_model,
    ))
}
