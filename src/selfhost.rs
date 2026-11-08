//! Zero-config bootstrap for self-hosted containers (`KURULTAI_SELFHOST=1`).
//!
//! A container must bind a non-loopback address, and the daemon refuses that
//! without auth. Rather than make users invent a key before they see anything,
//! the self-host flag:
//!
//! * creates `config.toml` on first start with `[sources.notes]` pointing at
//!   `$KURULTAI_NOTES_DIR` (default `/data/notes`), so mounting a folder there
//!   indexes it with no further setup;
//! * mints one random access key on first start, stores it `0600` under the
//!   data dir, and prints it once. Later starts reuse it.
//!
//! Without the flag nothing here runs, so hosted instances (which pass their own
//! keys) and local installs are unchanged.

use crate::error::{KurultaiError, Result};
use std::path::{Path, PathBuf};

const KEY_FILE: &str = "selfhost.key";

/// `KURULTAI_SELFHOST` is `1` / `true`.
pub fn enabled() -> bool {
    std::env::var("KURULTAI_SELFHOST")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn notes_dir() -> PathBuf {
    std::env::var("KURULTAI_NOTES_DIR")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/data/notes"))
}

/// Run before the daemon resolves its bind/auth: first-run config plus an
/// access key exported as `KURULTAI_HUB_AUTH=api_key` / `KURULTAI_HUB_API_KEYS`
/// (only when the operator has not configured keys or Cloudflare Access).
pub fn bootstrap(config_override: Option<&Path>) -> Result<()> {
    bootstrap_config(config_override)?;
    let has = |k: &str| {
        std::env::var(k)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    };
    if !has("KURULTAI_HUB_API_KEYS") && !has("KURULTAI_CF_ACCESS_TEAM") {
        let (key, created, path) = ensure_access_key()?;
        std::env::set_var("KURULTAI_HUB_AUTH", "api_key");
        std::env::set_var("KURULTAI_HUB_API_KEYS", &key);
        if created {
            eprintln!(
                "\nKURULTAI ACCESS KEY (shown once, stored at {}):\n  {key}\n",
                path.display()
            );
        } else {
            tracing::info!(path = %path.display(), "self-host access key loaded from file");
        }
    }
    Ok(())
}

/// Create the default config (if absent) with the notes source enabled.
pub fn bootstrap_config(config_override: Option<&Path>) -> Result<()> {
    let path = match config_override {
        Some(p) => p.to_path_buf(),
        None => crate::config::config_path()?,
    };
    if path.exists() {
        return Ok(());
    }
    let path = crate::mcp::init::ensure_default_config_at(path)?;
    crate::mcp::init::enable_notes_source(&path, &notes_dir())?;
    tracing::info!(config = %path.display(), notes = %notes_dir().display(), "self-host: wrote first-run config");
    Ok(())
}

/// Key file location: `<data dir>/kurultai/selfhost.key`.
pub fn key_path() -> Result<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve data directory"))?;
    Ok(base.join("kurultai").join(KEY_FILE))
}

/// Load the access key, creating it on first call. Returns `(key, created, path)`.
pub fn ensure_access_key() -> Result<(String, bool, PathBuf)> {
    let path = key_path()?;
    let (key, created) = ensure_access_key_at(&path)?;
    Ok((key, created, path))
}

pub fn ensure_access_key_at(path: &Path) -> Result<(String, bool)> {
    if let Ok(existing) = std::fs::read_to_string(path) {
        let key = existing.trim().to_string();
        if !key.is_empty() {
            return Ok((key, false));
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let key = format!(
        "kur_{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    write_private(path, &key)?;
    Ok((key, true))
}

fn write_private(path: &Path, contents: &str) -> Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    f.write_all(contents.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_is_created_once_then_reused_with_private_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join(KEY_FILE);
        let (k1, created1) = ensure_access_key_at(&path).unwrap();
        assert!(created1);
        assert!(k1.starts_with("kur_") && k1.len() > 40);
        let (k2, created2) = ensure_access_key_at(&path).unwrap();
        assert!(!created2);
        assert_eq!(k1, k2);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn bootstrap_writes_config_once_and_enables_notes_without_starter() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("config.toml");
        std::env::set_var("KURULTAI_NOTES_DIR", dir.path().join("notes"));
        bootstrap_config(Some(&cfg)).unwrap();
        std::env::remove_var("KURULTAI_NOTES_DIR");
        let raw = std::fs::read_to_string(&cfg).unwrap();
        assert!(raw.contains("[sources.notes]"), "{raw}");
        assert!(dir.path().join("notes").is_dir());
        assert_eq!(
            std::fs::read_dir(dir.path().join("notes")).unwrap().count(),
            0
        );
        // existing config is left alone
        std::fs::write(&cfg, "environment = \"dev\"\n").unwrap();
        bootstrap_config(Some(&cfg)).unwrap();
        assert_eq!(
            std::fs::read_to_string(&cfg).unwrap(),
            "environment = \"dev\"\n"
        );
    }
}
