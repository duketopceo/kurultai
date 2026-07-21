//! Phase 1 CLI smoke (#5 / #23) — binary against fixture vault.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn bin() -> Command {
    let mut cmd = Command::cargo_bin("kurultai").expect("kurultai binary");
    // Force FTS-only — ambient API keys must not hit OpenRouter or trip dim mismatch.
    cmd.env_remove("OPENROUTER_API_KEY");
    cmd.env_remove("KURULTAI_API_KEY");
    cmd
}

fn fixture_config(tmp: &tempfile::TempDir) -> PathBuf {
    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let db = tmp.path().join("store.db");
    let cfg_path = tmp.path().join("config.toml");
    let body = format!(
        r#"environment = "dev"

[storage]
path = "{db}"

[embed]
model = "openai/text-embedding-3-large"
dimension = 4

[runtime]
poll_interval_secs = 300

[sources.notes]
kind = "markdown"
enabled = true
root_path = "{vault}"
"#,
        db = db.display(),
        vault = vault.display()
    );
    fs::write(&cfg_path, body).unwrap();
    cfg_path
}

#[test]
fn status_shows_environment_and_sources() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&tmp);
    bin()
        .args(["--config", cfg.to_str().unwrap(), "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Kurultai status"))
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("Reranker: none"));
}

#[test]
fn index_and_search_fixture_phrase() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&tmp);
    bin()
        .args(["--config", cfg.to_str().unwrap(), "index", "--full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes"));

    bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "search",
            "KNOWN_PHRASE_KURULTAI_42",
            "--limit",
            "5",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes"));
}

#[test]
fn invalid_config_errors_clearly() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = tmp.path().join("bad.toml");
    fs::write(&cfg, "[[[broken").unwrap();
    bin()
        .args(["--config", cfg.to_str().unwrap(), "status"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("TOML")
                .or(predicate::str::contains("parse"))
                .or(predicate::str::contains("config"))
                .or(predicate::str::contains("Error")),
        );
}
