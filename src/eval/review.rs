//! Pre-merge commit review: grade each commit in a range with the Jev
//! judge (`noul`/`score` typed questions over `git show`) and flag commits
//! that cross probability thresholds — secrets, security regressions,
//! message/diff mismatch, missing tests.
//!
//! Same env-gating as the eval judge: no OpenRouter key → caller should
//! skip with a warning (see `Commands::Review`).

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

use super::judge::{Judge, Question};

const DIFF_CAP: usize = 12_000;
const STAT_CAP: usize = 3_000;
const BODY_CAP: usize = 1_500;

/// Hard-fail thresholds (noul probabilities).
pub const SECRET_FLAG: f64 = 0.5;
pub const SECURITY_FLAG: f64 = 0.5;
/// `matches_message` BELOW this is a flag (inverted check).
pub const MSG_MISMATCH: f64 = 0.5;
/// `tests_adequate` BELOW this is an advisory flag.
pub const TEST_GAP: f64 = 0.3;
/// `needs_followup` AT OR ABOVE this is an advisory flag.
pub const FOLLOWUP_FLAG: f64 = 0.7;

/// One commit's material for review.
#[derive(Debug)]
pub struct Commit {
    pub sha: String,
    pub author: String,
    pub subject: String,
    pub body: String,
    pub stat: String,
    pub diff: String,
}

#[derive(Debug, Serialize)]
pub struct Flag {
    pub check: String,
    pub probability: f64,
    pub threshold: f64,
    /// `hard` fails the review; `advisory` is reported only.
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct CommitReview {
    pub sha: String,
    pub subject: String,
    /// noul probabilities by question id.
    pub noul: serde_json::Map<String, Value>,
    /// rubric scores by question id.
    pub score: serde_json::Map<String, Value>,
    pub flags: Vec<Flag>,
}

#[derive(Debug, Serialize)]
pub struct ReviewReport {
    pub range: String,
    pub judge: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub judge_cost_usd: Option<f64>,
    pub commits: Vec<CommitReview>,
    /// Commits with at least one hard flag.
    pub failed: usize,
    /// Total commits reviewed (bot commits excluded).
    pub reviewed: usize,
}

fn review_questions() -> Vec<(String, Question)> {
    vec![
        (
            "leaks_secret".into(),
            Question::Noul {
                instructions: "Does the diff expose a secret, credential, API key, token, or private key in plaintext?".into(),
                on_true: "A secret/credential is visible in the diff".into(),
                on_false: "No secrets in the diff".into(),
            },
        ),
        (
            "security_risk".into(),
            Question::Noul {
                instructions: "Does this change plausibly introduce a security vulnerability (auth bypass, injection, XSS, CSRF, unsafe deserialization, missing access check)?".into(),
                on_true: "Plausible security vulnerability".into(),
                on_false: "No plausible security issue".into(),
            },
        ),
        (
            "unauth_write".into(),
            Question::Noul {
                instructions: "Does this diff create a path where an unauthenticated or remote caller gains write/admin access?".into(),
                on_true: "Unauthenticated write/admin path exists".into(),
                on_false: "All write paths require auth or are loopback-only".into(),
            },
        ),
        (
            "network_exposure".into(),
            Question::Noul {
                instructions: "Does this diff bind a service to a non-loopback interface or widen network exposure without an auth check on the new surface?".into(),
                on_true: "New/widened unauthenticated network surface".into(),
                on_false: "No unauthenticated network exposure".into(),
            },
        ),
        (
            "injection_xss".into(),
            Question::Noul {
                instructions: "Does this diff render user/agent-controlled content without escaping (XSS), or build commands/SQL/paths from untrusted input (injection)?".into(),
                on_true: "XSS or injection possible".into(),
                on_false: "Content is escaped/parameterized".into(),
            },
        ),
        (
            "csrf".into(),
            Question::Noul {
                instructions: "Does this diff add or modify state-changing HTTP endpoints without CSRF protection (token, Origin check, or SameSite)?".into(),
                on_true: "State-changing endpoint lacks CSRF protection".into(),
                on_false: "CSRF-protected or not applicable".into(),
            },
        ),
        (
            "matches_message".into(),
            Question::Noul {
                instructions: "Does the diff plausibly implement what the commit message claims?".into(),
                on_true: "Diff matches the stated intent".into(),
                on_false: "Diff does not match the claim".into(),
            },
        ),
        (
            "tests_adequate".into(),
            Question::Noul {
                instructions: "Is the change accompanied by appropriate tests or verification for its risk level?".into(),
                on_true: "Adequate tests/verification".into(),
                on_false: "Missing or inadequate tests".into(),
            },
        ),
        (
            "needs_followup".into(),
            Question::Noul {
                instructions: "Does this diff leave an obvious bug, unfinished edge, or issue a reviewer should flag for follow-up?".into(),
                on_true: "There is a flaggable issue".into(),
                on_false: "Nothing obvious to flag".into(),
            },
        ),
        (
            "quality".into(),
            Question::Score {
                instructions: "Rate the code quality and clarity of this change.".into(),
                criteria: vec![
                    "sloppy or harmful".into(),
                    "works but rough".into(),
                    "solid work".into(),
                    "exemplary".into(),
                ],
            },
        ),
        (
            "risk".into(),
            Question::Score {
                instructions: "Rate the regression/deployment risk of this change.".into(),
                criteria: vec![
                    "docs/tests only, safe".into(),
                    "low risk".into(),
                    "moderate risk".into(),
                    "high risk — needs careful review".into(),
                ],
            },
        ),
    ]
}

fn noul_p(m: &serde_json::Map<String, Value>, key: &str) -> f64 {
    m.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn push_flag(flags: &mut Vec<Flag>, check: &str, p: f64, threshold: f64, severity: &str) {
    if p >= threshold {
        flags.push(Flag {
            check: check.into(),
            probability: p,
            threshold,
            severity: severity.into(),
        });
    }
}

fn flags_for(noul: &serde_json::Map<String, Value>) -> Vec<Flag> {
    let mut flags = Vec::new();
    for check in [
        "leaks_secret",
        "security_risk",
        "unauth_write",
        "network_exposure",
        "injection_xss",
        "csrf",
    ] {
        let t = if check == "leaks_secret" {
            SECRET_FLAG
        } else {
            SECURITY_FLAG
        };
        push_flag(&mut flags, check, noul_p(noul, check), t, "hard");
    }
    // Inverted: low confidence the diff matches its message.
    let m = noul_p(noul, "matches_message");
    if m < MSG_MISMATCH {
        flags.push(Flag {
            check: "matches_message".into(),
            probability: m,
            threshold: MSG_MISMATCH,
            severity: "hard".into(),
        });
    }
    // Inverted advisory: low confidence tests are adequate.
    let t = noul_p(noul, "tests_adequate");
    if t < TEST_GAP {
        flags.push(Flag {
            check: "tests_adequate".into(),
            probability: t,
            threshold: TEST_GAP,
            severity: "advisory".into(),
        });
    }
    push_flag(
        &mut flags,
        "needs_followup",
        noul_p(noul, "needs_followup"),
        FOLLOWUP_FLAG,
        "advisory",
    );
    flags
}

/// Review already-collected commits — testable without git.
pub async fn review_commits(
    range: &str,
    commits: &[Commit],
    judge: &Arc<dyn Judge>,
) -> Result<ReviewReport> {
    let questions = review_questions();
    let mut reviews = Vec::new();
    let mut cost = 0.0f64;
    let mut model: Option<String> = None;
    for c in commits {
        let state = json!({
            "commit": c.sha,
            "subject": c.subject,
            "body": c.body,
            "stat": c.stat,
            "diff": c.diff,
        });
        let answers = judge
            .decide(&state, &questions)
            .await
            .with_context(|| format!("judge failed for commit {}", c.sha))?;
        cost += answers.cost_usd.unwrap_or(0.0);
        if model.is_none() {
            model = answers.resolved_model.clone();
        }
        let flags = flags_for(&answers.noul);
        reviews.push(CommitReview {
            sha: c.sha.clone(),
            subject: c.subject.clone(),
            flags,
            noul: answers.noul,
            score: answers.score,
        });
    }
    let failed = reviews
        .iter()
        .filter(|r| r.flags.iter().any(|f| f.severity == "hard"))
        .count();
    Ok(ReviewReport {
        range: range.into(),
        judge: judge.name().into(),
        judge_model: model,
        judge_cost_usd: (cost > 0.0).then_some(cost),
        reviewed: reviews.len(),
        failed,
        commits: reviews,
    })
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .with_context(|| format!("git {} failed to spawn", args.join(" ")))?;
    anyhow::ensure!(
        out.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Collect commits in `range` (e.g. `origin/main..HEAD`), excluding bot
/// authors (dependabot etc.) — automated bumps aren't worth judging.
pub fn collect_commits(repo: &Path, range: &str) -> Result<Vec<Commit>> {
    let list = git(
        repo,
        &["rev-list", "--format=%H%x1f%an%x1f%s", "--no-merges", range],
    )?;
    let mut commits = Vec::new();
    for line in list.lines().filter(|l| l.contains('\x1f')) {
        let mut parts = line.splitn(3, '\x1f');
        let sha = parts.next().unwrap_or_default().to_string();
        let author = parts.next().unwrap_or_default().to_string();
        let subject = parts.next().unwrap_or_default().to_string();
        if author.contains("dependabot") || author.contains("[bot]") {
            continue;
        }
        let body = git(repo, &["log", "-1", "--format=%b", &sha])
            .unwrap_or_default()
            .chars()
            .take(BODY_CAP)
            .collect();
        let stat: String = git(repo, &["show", "--stat", "--format=", &sha])
            .unwrap_or_default()
            .chars()
            .take(STAT_CAP)
            .collect();
        let diff: String = git(repo, &["show", "--format=", &sha])
            .unwrap_or_default()
            .chars()
            .take(DIFF_CAP)
            .collect();
        commits.push(Commit {
            sha,
            author,
            subject,
            body,
            stat,
            diff,
        });
    }
    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn noul(pairs: &[(&str, f64)]) -> serde_json::Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect()
    }

    #[test]
    fn flags_secret_and_security_as_hard() {
        let flags = flags_for(&noul(&[
            ("leaks_secret", 0.9),
            ("security_risk", 0.6),
            ("matches_message", 0.9),
            ("tests_adequate", 0.8),
            ("needs_followup", 0.2),
        ]));
        assert_eq!(
            flags
                .iter()
                .filter(|f| f.severity == "hard")
                .map(|f| f.check.as_str())
                .collect::<Vec<_>>(),
            vec!["leaks_secret", "security_risk"]
        );
    }

    #[test]
    fn clean_commit_has_no_flags() {
        let flags = flags_for(&noul(&[
            ("leaks_secret", 0.02),
            ("security_risk", 0.05),
            ("unauth_write", 0.1),
            ("network_exposure", 0.1),
            ("injection_xss", 0.05),
            ("csrf", 0.1),
            ("matches_message", 0.92),
            ("tests_adequate", 0.85),
            ("needs_followup", 0.3),
        ]));
        assert!(flags.is_empty());
    }

    #[test]
    fn message_mismatch_and_test_gap_and_followup_flag() {
        let flags = flags_for(&noul(&[
            ("matches_message", 0.3),
            ("tests_adequate", 0.1),
            ("needs_followup", 0.85),
        ]));
        let by_check: std::collections::HashMap<_, _> = flags
            .iter()
            .map(|f| (f.check.as_str(), f.severity.as_str()))
            .collect();
        assert_eq!(by_check["matches_message"], "hard");
        assert_eq!(by_check["tests_adequate"], "advisory");
        assert_eq!(by_check["needs_followup"], "advisory");
    }
}
