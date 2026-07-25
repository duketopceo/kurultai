//! Personal installer script smoke (#72) — bash syntax + dry-run / help.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command as StdCommand;

fn install_sh() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/install/install.sh")
}

#[test]
fn install_script_passes_bash_syntax_check() {
    let path = install_sh();
    assert!(path.is_file(), "missing {}", path.display());
    let status = StdCommand::new("bash")
        .args(["-n"])
        .arg(&path)
        .status()
        .expect("bash -n");
    assert!(status.success(), "bash -n failed for {}", path.display());
}

#[test]
fn install_script_help_exits_zero() {
    let path = install_sh();
    Command::new("bash")
        .arg(&path)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal installer"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn install_script_dry_run_does_not_require_cargo_install() {
    let path = install_sh();
    let home = tempfile::tempdir().expect("temp HOME");
    let xdg = tempfile::tempdir().expect("temp XDG");
    // Plain --dry-run (with init enabled) must not write config/MCP state.
    Command::new("bash")
        .arg(&path)
        .arg("--dry-run")
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", xdg.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("DRY-RUN"))
        .stderr(predicate::str::contains("personal install"));

    let kurultai_cfg = xdg.path().join("kurultai");
    assert!(
        !kurultai_cfg.exists(),
        "dry-run must not create {}",
        kurultai_cfg.display()
    );
    let home_cfg = home.path().join(".config").join("kurultai");
    assert!(
        !home_cfg.exists(),
        "dry-run must not create {}",
        home_cfg.display()
    );
}

#[test]
fn install_script_rejects_unknown_flag() {
    let path = install_sh();
    Command::new("bash")
        .arg(&path)
        .arg("--not-a-real-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown argument"));
}
