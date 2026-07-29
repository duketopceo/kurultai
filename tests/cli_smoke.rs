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
    fixture_config_with_cli(tmp, None)
}

fn fixture_config_with_cli(tmp: &tempfile::TempDir, cli_banner: Option<&str>) -> PathBuf {
    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let db = tmp.path().join("store.db");
    let cfg_path = tmp.path().join("config.toml");
    let cli_section = match cli_banner {
        Some(v) => format!("\n[cli]\nbanner = {v}\n"),
        None => String::new(),
    };
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
{cli_section}"#,
        db = db.display(),
        vault = vault.display(),
        cli_section = cli_section
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

#[test]
fn cli_ask_extractive_fixture() {
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
            "ask",
            "what is KNOWN_PHRASE_KURULTAI_42",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("KNOWN_PHRASE_KURULTAI_42"));
}

#[test]
fn status_banner_true_shows_compact_art() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config_with_cli(&tmp, Some("true"));
    bin()
        .args(["--config", cfg.to_str().unwrap(), "status"])
        .env_remove("NO_COLOR")
        .env_remove("KURULTAI_PLAIN")
        .assert()
        .success()
        .stdout(predicate::str::contains("Kurultai status"))
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_BOX))
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_YURT));
}

#[test]
fn status_plain_flag_suppresses_art() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config_with_cli(&tmp, Some("true"));
    bin()
        .args(["--config", cfg.to_str().unwrap(), "--plain", "status"])
        .env_remove("NO_COLOR")
        .env_remove("KURULTAI_PLAIN")
        .assert()
        .success()
        .stdout(predicate::str::contains("Kurultai status"))
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_BOX).not())
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_YURT).not());
}

#[test]
fn status_no_color_suppresses_art() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config_with_cli(&tmp, Some("true"));
    bin()
        .args(["--config", cfg.to_str().unwrap(), "status"])
        .env("NO_COLOR", "1")
        .env_remove("KURULTAI_PLAIN")
        .assert()
        .success()
        .stdout(predicate::str::contains("Kurultai status"))
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_BOX).not())
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_YURT).not());
}

#[test]
fn status_kurultai_plain_env_suppresses_art() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config_with_cli(&tmp, Some("true"));
    bin()
        .args(["--config", cfg.to_str().unwrap(), "status"])
        .env("KURULTAI_PLAIN", "1")
        .env_remove("NO_COLOR")
        .assert()
        .success()
        .stdout(predicate::str::contains("Kurultai status"))
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_BOX).not())
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_YURT).not());
}

#[test]
fn mcp_help_never_prints_yurt_art() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config_with_cli(&tmp, Some("true"));
    bin()
        .args(["--config", cfg.to_str().unwrap(), "mcp", "--help"])
        .env_remove("NO_COLOR")
        .env_remove("KURULTAI_PLAIN")
        .assert()
        .success()
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_BOX).not())
        .stdout(predicate::str::contains(kurultai::art::ART_MARKER_YURT).not());
}

#[test]
fn cli_who_knows_fixture() {
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
            "who-knows",
            "KNOWN_PHRASE_KURULTAI_42",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes"));
}

#[test]
fn export_import_replace_preserves_search() {
    let src = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&src);
    bin()
        .args(["--config", cfg.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();

    let pack = src.path().join("brain.kurultai");
    bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "export",
            "-o",
            pack.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    let dest = tempfile::tempdir().unwrap();
    let dest_db = dest.path().join("store.db");
    let dest_cfg = dest.path().join("config.toml");
    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    fs::write(
        &dest_cfg,
        format!(
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
            db = dest_db.display(),
            vault = vault.display()
        ),
    )
    .unwrap();

    bin()
        .args([
            "--config",
            dest_cfg.to_str().unwrap(),
            "import",
            pack.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported"));

    bin()
        .args([
            "--config",
            dest_cfg.to_str().unwrap(),
            "search",
            "KNOWN_PHRASE_KURULTAI_42",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("KNOWN_PHRASE_KURULTAI_42"));
}

#[test]
fn export_import_combine_keeps_both_brains() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    let cfg_a = fixture_config(&a);
    let cfg_b = fixture_config(&b);

    bin()
        .args(["--config", cfg_a.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();

    // Seed destination with its own index first.
    bin()
        .args(["--config", cfg_b.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();

    let pack = a.path().join("a.kurultai");
    bin()
        .args([
            "--config",
            cfg_a.to_str().unwrap(),
            "export",
            "-o",
            pack.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .args([
            "--config",
            cfg_b.to_str().unwrap(),
            "import",
            "--combine",
            pack.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("combine"));

    bin()
        .args([
            "--config",
            cfg_b.to_str().unwrap(),
            "search",
            "KNOWN_PHRASE_KURULTAI_42",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("KNOWN_PHRASE_KURULTAI_42"));
}
