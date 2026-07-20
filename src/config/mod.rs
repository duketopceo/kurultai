use crate::types::{Config, KurultaiEnv, SourceConfig, SourceKind};
use anyhow::{bail, Context, Result};
use directories::ProjectDirs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("io", "kurultai", "kurultai")
        .ok_or_else(|| anyhow::anyhow!("could not resolve project directories"))
}

pub fn expand_path(p: &str) -> Result<PathBuf> {
    if let Some(rest) = p.strip_prefix("~/") {
        let home = dirs_home()?;
        return Ok(home.join(rest));
    }
    if p == "~" {
        return dirs_home();
    }
    Ok(PathBuf::from(p))
}

fn dirs_home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("HOME not set"))
}

pub fn resolve_env(cli_env: Option<&str>) -> Result<KurultaiEnv> {
    if let Some(s) = cli_env {
        return KurultaiEnv::parse(s).ok_or_else(|| anyhow::anyhow!("invalid --env: {s}"));
    }
    if let Ok(s) = std::env::var("KURULTAI_ENV") {
        return KurultaiEnv::parse(&s).ok_or_else(|| anyhow::anyhow!("invalid KURULTAI_ENV: {s}"));
    }
    Ok(KurultaiEnv::Dev)
}

pub fn config_path(env: KurultaiEnv) -> Result<PathBuf> {
    let dirs = project_dirs()?;
    Ok(dirs.config_dir().join(env.as_str()).join("config.toml"))
}

pub fn default_storage_path(env: KurultaiEnv) -> Result<PathBuf> {
    let dirs = project_dirs()?;
    Ok(dirs.data_local_dir().join(env.as_str()).join("store.db"))
}

pub fn resolve_storage_path(config: &Config) -> Result<PathBuf> {
    if config.storage_path.trim().is_empty() {
        default_storage_path(config.env)
    } else {
        expand_path(&config.storage_path)
    }
}

pub fn load_config(env: KurultaiEnv) -> Result<Config> {
    let path = config_path(env)?;
    if !path.exists() {
        bail!(
            "config not found at {} — run `kurultai init` first",
            path.display()
        );
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read config {}", path.display()))?;
    let mut cfg: Config = toml::from_str(&text).context("parse config.toml")?;
    cfg.env = env;
    Ok(cfg)
}

pub fn write_config(config: &Config, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(config).context("serialize config")?;
    std::fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Default config for `kurultai init`.
pub fn default_init_config(env: KurultaiEnv, vault_hint: Option<PathBuf>) -> Config {
    let mut extra = HashMap::new();
    if let Some(v) = vault_hint {
        extra.insert("path".into(), v.display().to_string());
    } else {
        extra.insert("path".into(), "~/Documents/Notes".into());
    }
    Config {
        env,
        sources: vec![SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Filesystem,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        }],
        storage_path: String::new(),
        embed_model: "openai/text-embedding-3-small".into(),
        embed_dim: 1536,
        reranker_model: None,
        poll_interval_secs: 60,
        openrouter_api_key_env: "OPENROUTER_API_KEY".into(),
    }
}
