//! Offline eval-harness acceptance tests.
//!
//! Serves the real Axum stack on an ephemeral port over the fixture vault,
//! then runs `eval::run_eval` against it — same code path as `kurultai eval`
//! against a live daemon. No network, no judge, no API keys.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::eval::{self, EvalConfig, GoldenSet};
use kurultai::http::{build_app, HubAuth, HubGate};
use kurultai::mcp::BrainService;
use kurultai::pipeline::IndexPipeline;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use kurultai::types::{SourceConfig, SourceKind};

fn fixture_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

fn fixture_golden() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/evals/golden.json")
}

/// Serve a fixture-vault brain on `127.0.0.1:<ephemeral>`; returns base URL.
async fn serve_fixture_brain() -> (String, tempfile::TempDir) {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = tempfile::tempdir().unwrap();
    let _ = N.fetch_add(1, Ordering::Relaxed);
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert(
        "root_path".into(),
        fixture_vault().to_string_lossy().into_owned(),
    );
    connector
        .init(&SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();
    pipeline
        .index_connector("notes", &connector, true)
        .await
        .unwrap();

    let brain = BrainService::new(
        Arc::clone(&store) as Arc<dyn Store>,
        embedder,
        Arc::new(NullReranker::new()),
        Arc::new(ExtractiveSynthesizer::new()),
    );
    let app = build_app(
        brain,
        Arc::new(kurultai::daemon::DaemonStatus::default()),
        HubGate {
            auth: HubAuth::None,
            api_keys: vec![],
            agent_store: None,
            cf_access: None,
            #[cfg(feature = "postgres")]
            key_store: None,
        },
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), dir)
}

#[tokio::test]
async fn eval_runner_scores_fixture_golden_set() {
    let (base_url, _dir) = serve_fixture_brain().await;
    let golden = GoldenSet::load(&fixture_golden()).unwrap();
    let report = eval::run_eval(
        &EvalConfig {
            base_url,
            k: 5,
            judge: false,
            judge_model: None,
            judge_override: None,
        },
        &golden,
    )
    .await
    .unwrap();

    assert_eq!(report.queries.len(), 3);
    // All three fixture queries should fully recall the Deploy Guide at k=5.
    assert!(
        report.aggregate.mean_recall_at_k >= 1.0,
        "expected full recall on fixture set, got {}",
        report.aggregate.mean_recall_at_k
    );
    assert_eq!(report.aggregate.noise_violations, 0);
    // No judge configured: graded metrics stay absent.
    assert!(report.aggregate.mean_ndcg_at_k.is_none());
    assert!(report.aggregate.mean_groundedness.is_none());
    assert_eq!(report.judge, "null");
}

#[tokio::test]
async fn eval_runner_reports_missing_daemon_cleanly() {
    let golden = GoldenSet::load(&fixture_golden()).unwrap();
    // Port 1 is never listening.
    let err = eval::run_eval(
        &EvalConfig {
            base_url: "http://127.0.0.1:1".into(),
            k: 5,
            judge: false,
            judge_model: None,
            judge_override: None,
        },
        &golden,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("search request failed"));
}

#[test]
fn golden_set_requires_queries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.json");
    std::fs::write(&path, r#"{"version":1,"queries":[]}"#).unwrap();
    assert!(GoldenSet::load(&path).is_err());
}

#[test]
fn matcher_variants_hit_expected_fields() {
    use kurultai::eval::Matcher;
    let m = Matcher {
        title_contains: Some("deploy".into()),
        ..Default::default()
    };
    assert!(m.matches("Deploy Guide", "/x", None));
    assert!(!m.matches("Other", "/x", None));

    let m = Matcher {
        source_id_contains: Some("ops/".into()),
        ..Default::default()
    };
    assert!(m.matches("T", "vault/ops/deploy.md", None));

    let m = Matcher {
        title_hash: Some("abc123".into()),
        ..Default::default()
    };
    assert!(m.matches("T", "s", Some("abc123")));
    assert!(!m.matches("T", "s", Some("def456")));
    assert!(!m.matches("T", "s", None));
}

/// Canned judge: every hit graded 3.0/3 → nDCG must be 1.0.
struct PerfectJudge;

#[async_trait::async_trait]
impl kurultai::eval::judge::Judge for PerfectJudge {
    fn name(&self) -> &'static str {
        "stub"
    }
    fn is_live(&self) -> bool {
        true
    }
    async fn decide(
        &self,
        _state: &serde_json::Value,
        questions: &[(String, kurultai::eval::judge::Question)],
    ) -> anyhow::Result<kurultai::eval::judge::DecisionAnswers> {
        let mut answers = kurultai::eval::judge::DecisionAnswers {
            resolved_model: Some("stub-1.0".into()),
            cost_usd: Some(0.001),
            ..Default::default()
        };
        for (qid, q) in questions {
            match q {
                kurultai::eval::judge::Question::Score { .. } => {
                    answers.score.insert(qid.clone(), serde_json::json!(3.0));
                }
                kurultai::eval::judge::Question::Noul { .. } => {
                    answers.noul.insert(qid.clone(), serde_json::json!(1.0));
                }
            }
        }
        Ok(answers)
    }
}

#[tokio::test]
async fn eval_runner_uses_injected_judge_for_ndcg() {
    let (base_url, _dir) = serve_fixture_brain().await;
    let golden = GoldenSet::load(&fixture_golden()).unwrap();
    let report = eval::run_eval(
        &EvalConfig {
            base_url,
            k: 5,
            judge: false, // override wins regardless of flag
            judge_model: None,
            judge_override: Some(Arc::new(PerfectJudge)),
        },
        &golden,
    )
    .await
    .unwrap();

    assert_eq!(report.judge, "stub");
    assert_eq!(report.judge_model.as_deref(), Some("stub-1.0"));
    assert!(report.judge_cost_usd.unwrap() > 0.0);
    let ndcg = report.aggregate.mean_ndcg_at_k.expect("judge ran");
    assert!(
        (ndcg - 1.0).abs() < 1e-9,
        "all-3.0 grades → nDCG 1.0, got {ndcg}"
    );
}

/// Judge that always fails — simulates a dead key / insufficient credits.
struct FailingJudge {
    calls: AtomicU64,
}

#[async_trait::async_trait]
impl kurultai::eval::judge::Judge for FailingJudge {
    fn name(&self) -> &'static str {
        "failing"
    }
    fn is_live(&self) -> bool {
        true
    }
    async fn decide(
        &self,
        _state: &serde_json::Value,
        _questions: &[(String, kurultai::eval::judge::Question)],
    ) -> anyhow::Result<kurultai::eval::judge::DecisionAnswers> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        anyhow::bail!("402 insufficient credits")
    }
}

/// A dead/out-of-credit judge must trip the circuit breaker: stop calling
/// after 3 failures, finish the run labels-only, and flag the report.
#[tokio::test]
async fn eval_runner_disables_judge_after_consecutive_failures() {
    let (base_url, _dir) = serve_fixture_brain().await;
    // Five queries over the fixture vault — enough to exceed the breaker.
    let golden = GoldenSet {
        version: 1,
        queries: (0..5)
            .map(|i| eval::GoldenQuery {
                id: format!("breaker-{i}"),
                kind: "search".into(),
                query: "deploy".into(),
                relevant: vec![eval::Matcher {
                    source_id_contains: Some("ops/deploy.md".into()),
                    ..Default::default()
                }],
                deny_sources: vec![],
                judge_answer: false,
            })
            .collect(),
    };
    let judge = Arc::new(FailingJudge {
        calls: AtomicU64::new(0),
    });
    let report = eval::run_eval(
        &EvalConfig {
            base_url,
            k: 5,
            judge: false,
            judge_model: None,
            judge_override: Some(judge.clone()),
        },
        &golden,
    )
    .await
    .unwrap();

    assert_eq!(judge.calls.load(Ordering::Relaxed), 3, "breaker at 3");
    assert!(report.judge_disabled_reason.is_some());
    assert!(report.aggregate.mean_ndcg_at_k.is_none());
    // Labels-only metrics still produced for all queries.
    assert_eq!(report.queries.len(), 5);
    assert!(report.queries.iter().all(|r| r.recall_at_k.is_some()));
}

/// Live smoke for the Jev decisions path — manual only:
/// `OPENROUTER_API_KEY=... cargo test --test evals_search judge_live -- --ignored`
#[tokio::test]
#[ignore = "live OpenRouter call; set OPENROUTER_API_KEY and pass --ignored"]
async fn judge_live_decisions_smoke() {
    let judge = kurultai::eval::judge::judge_from_env(None);
    assert!(judge.is_live(), "no OpenRouter key configured");
    let answers = judge
        .decide(
            &serde_json::json!({"query": "deploy", "hit": "ops/deploy.md"}),
            &[(
                "rel".to_string(),
                kurultai::eval::judge::Question::Noul {
                    instructions: "Is the hit relevant to the query?".into(),
                    on_true: "Relevant".into(),
                    on_false: "Not relevant".into(),
                },
            )],
        )
        .await
        .unwrap();
    let p = answers
        .noul
        .get("rel")
        .and_then(serde_json::Value::as_f64)
        .expect("noul answer present");
    assert!(p > 0.5, "deploy hit should grade relevant, got {p}");
    assert!(answers.resolved_model.is_some());
}
