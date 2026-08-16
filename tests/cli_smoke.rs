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
        .stdout(predicate::str::contains(format!(
            "Version: {}",
            env!("CARGO_PKG_VERSION")
        )))
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("Reranker: none"))
        .stdout(predicate::str::contains("Features"))
        .stdout(predicate::str::contains("fts"))
        .stdout(predicate::str::contains("brain_ui"));
}

#[test]
fn help_groups_setup_knowledge_serve() {
    bin()
        .args(["--plain", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup"))
        .stdout(predicate::str::contains("init --docs"))
        .stdout(predicate::str::contains("Knowledge"))
        .stdout(predicate::str::contains("Serve"))
        .stdout(predicate::str::contains("Packs"))
        .stdout(predicate::str::contains("Maintenance"))
        .stdout(predicate::str::contains("who-knows"))
        .stdout(predicate::str::contains("127.0.0.1:8421/ui/"));
}

#[test]
fn who_knows_underscore_alias_works() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&tmp);
    bin()
        .args(["--config", cfg.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();
    bin()
        .args(["--config", cfg.to_str().unwrap(), "who_knows", "migration"])
        .assert()
        .success();
}

#[test]
fn index_and_search_fixture_phrase() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = fixture_config(&tmp);
    bin()
        .args(["--config", cfg.to_str().unwrap(), "index", "--full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains(format!(
            "Kurultai {}",
            env!("CARGO_PKG_VERSION")
        )))
        .stdout(predicate::str::contains(format!(
            "embedded UI {}",
            env!("CARGO_PKG_VERSION")
        )));

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

    // Destination gets its own vault with a unique marker so we can prove
    // combine preserves local atoms alongside imported ones.
    let dest_vault = b.path().join("vault");
    fs::create_dir_all(dest_vault.join("ops")).unwrap();
    fs::write(
        dest_vault.join("ops/local.md"),
        "---\ntags: [local]\n---\n\nDEST_ONLY_MARKER_KURULTAI_99 stays on device B with enough detail for the quality gate.\n",
    )
    .unwrap();
    let dest_db = b.path().join("store.db");
    let cfg_b = b.path().join("config.toml");
    fs::write(
        &cfg_b,
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
            vault = dest_vault.display()
        ),
    )
    .unwrap();

    bin()
        .args(["--config", cfg_a.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();

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

    bin()
        .args([
            "--config",
            cfg_b.to_str().unwrap(),
            "search",
            "DEST_ONLY_MARKER_KURULTAI_99",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("DEST_ONLY_MARKER_KURULTAI_99"));
}

#[test]
fn export_import_replace_refuses_nonempty_store() {
    let src = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let cfg_src = fixture_config(&src);

    let dest_vault = dest.path().join("vault");
    fs::create_dir_all(dest_vault.join("ops")).unwrap();
    fs::write(
        dest_vault.join("ops/keep.md"),
        "---\ntags: [keep]\n---\n\nDEST_REFUSE_MARKER_KURULTAI_77 must survive a refused replace with enough detail for the quality gate.\n",
    )
    .unwrap();
    let dest_db = dest.path().join("store.db");
    let cfg_dest = dest.path().join("config.toml");
    fs::write(
        &cfg_dest,
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
            vault = dest_vault.display()
        ),
    )
    .unwrap();

    bin()
        .args(["--config", cfg_src.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();
    bin()
        .args(["--config", cfg_dest.to_str().unwrap(), "index", "--full"])
        .assert()
        .success();

    let pack = src.path().join("brain.kurultai");
    bin()
        .args([
            "--config",
            cfg_src.to_str().unwrap(),
            "export",
            "-o",
            pack.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .args([
            "--config",
            cfg_dest.to_str().unwrap(),
            "import",
            pack.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--force").or(predicate::str::contains("--combine")));

    bin()
        .args([
            "--config",
            cfg_dest.to_str().unwrap(),
            "search",
            "DEST_REFUSE_MARKER_KURULTAI_77",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("DEST_REFUSE_MARKER_KURULTAI_77"));
}

#[test]
fn init_help_lists_docs_index_and_agent_none() {
    bin()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--docs"))
        .stdout(predicate::str::contains("--index"))
        .stdout(predicate::str::contains("none"));
}

#[test]
fn init_docs_agent_none_provisions_tagged_starter() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let cfg = tmp.path().join("config.toml");
    let docs = tmp.path().join("vault");

    bin()
        .args([
            "--plain",
            "init",
            "--docs",
            docs.to_str().unwrap(),
            "--agent",
            "none",
        ])
        .env("HOME", &home)
        .env("KURULTAI_CONFIG", &cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("Docs folder:"))
        .stdout(predicate::str::contains("welcome.md"))
        .stdout(predicate::str::contains("FTS-only"))
        .stdout(predicate::str::contains("http://127.0.0.1:8421/ui/"))
        .stdout(predicate::str::contains("MCP: skipped"))
        .stdout(predicate::str::contains("kurultai index --full"));

    let welcome = fs::read_to_string(docs.join("welcome.md")).unwrap();
    assert!(welcome.contains("tags:"));
    let raw = fs::read_to_string(&cfg).unwrap();
    assert!(raw.contains("[sources.notes]"));
    assert!(raw.contains("kind = \"markdown\""));
    assert!(!home.join(".cursor/mcp.json").exists());
}

#[test]
fn init_without_docs_points_at_docs_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let cfg = tmp.path().join("config.toml");

    bin()
        .args(["--plain", "init", "--agent", "none"])
        .env("HOME", &home)
        .env("KURULTAI_CONFIG", &cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("kurultai init --docs"))
        .stdout(predicate::str::contains("MCP: skipped"));

    assert!(cfg.exists());
    let raw = fs::read_to_string(&cfg).unwrap();
    assert!(!raw.contains("[sources.notes]"));
}

#[test]
fn init_docs_does_not_overwrite_existing_note() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let cfg = tmp.path().join("config.toml");
    let docs = tmp.path().join("vault");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("welcome.md"), "KEEP\n").unwrap();
    fs::write(
        docs.join("already.md"),
        "---\ntitle: Existing\ntags:\n  - notes\n---\n\nkeep me\n",
    )
    .unwrap();

    bin()
        .args([
            "--plain",
            "init",
            "--docs",
            docs.to_str().unwrap(),
            "--agent",
            "none",
        ])
        .env("HOME", &home)
        .env("KURULTAI_CONFIG", &cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("not overwritten"));

    assert_eq!(
        fs::read_to_string(docs.join("welcome.md")).unwrap(),
        "KEEP\n"
    );
    assert!(fs::read_to_string(docs.join("already.md"))
        .unwrap()
        .contains("keep me"));
}

#[test]
fn init_docs_index_makes_starter_searchable() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    fs::create_dir_all(&home).unwrap();
    let cfg = tmp.path().join("config.toml");
    let docs = tmp.path().join("vault");

    bin()
        .args([
            "--plain",
            "init",
            "--docs",
            docs.to_str().unwrap(),
            "--agent",
            "none",
            "--index",
        ])
        .env("HOME", &home)
        .env("KURULTAI_CONFIG", &cfg)
        .assert()
        .success()
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("Indexed sources"))
        .stdout(predicate::str::contains("indexed 1").or(predicate::str::contains("indexed 2")));

    bin()
        .args(["--plain", "search", "welcome", "--limit", "5"])
        .env("HOME", &home)
        .env("KURULTAI_CONFIG", &cfg)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("welcome")
                .or(predicate::str::contains("Welcome"))
                .or(predicate::str::contains("getting-started")),
        );
}

// ── Track A / A2: `mcp` gained --agent-id / --namespace ──────────────────────

#[test]
fn mcp_help_lists_agent_identity_flags() {
    bin()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--agent-id"))
        .stdout(predicate::str::contains("--namespace"));
}

#[test]
fn mcp_accepts_no_flags_and_exits_cleanly_on_closed_stdin() {
    // The subcommand changed from a unit variant to a struct variant; bare
    // `kurultai mcp` must still parse and run.
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = fixture_config(&tmp);
    bin()
        .args(["--config", cfg.to_str().unwrap(), "mcp"])
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn mcp_accepts_agent_identity_flags() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cfg = fixture_config(&tmp);
    bin()
        .args([
            "--config",
            cfg.to_str().unwrap(),
            "mcp",
            "--agent-id",
            "session-3",
            "--namespace",
            "proj-a",
        ])
        .write_stdin("")
        .assert()
        .success();
}
