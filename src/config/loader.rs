use crate::config::file::FileConfig;
use crate::config::validate;
use crate::environment::Environment;
use crate::error::{KurultaiError, Result};
use crate::types::{Config, SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_RELATIVE: &str = ".config/kurultai/config.toml";

/// Default config file path: `KURULTAI_CONFIG` or `~/.config/kurultai/config.toml`.
pub fn config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("KURULTAI_CONFIG") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;
    Ok(home.join(DEFAULT_CONFIG_RELATIVE))
}

/// Load config from the default path.
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    load_config_from(&path)
}

/// Load config with optional path and `--env` / `KURULTAI_ENV` override.
pub fn load_config_with_env(path: Option<&Path>, env_override: Option<&str>) -> Result<Config> {
    let path = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| config_path().expect("home directory required"));
    load_config_from_with_env(&path, env_override)
}

/// Load config from an explicit path.
pub fn load_config_from(path: &Path) -> Result<Config> {
    load_config_from_with_env(path, None)
}

fn load_config_from_with_env(path: &Path, env_override: Option<&str>) -> Result<Config> {
    if !path.exists() {
        tracing::warn!(path = %path.display(), "config file not found, using defaults");
        let env = Environment::resolve(env_override)?;
        let config = default_config(env)?;
        validate(&config)?;
        return Ok(config);
    }

    let raw = std::fs::read_to_string(path)?;
    let file: FileConfig = toml::from_str(&raw)?;
    let explicit_storage = file.storage.path.is_some();
    let env = Environment::resolve(env_override.or(file.environment.as_deref()))?;
    let config = file_to_runtime(file, env, explicit_storage)?;
    validate(&config)?;
    tracing::debug!(
        path = %path.display(),
        env = %config.environment,
        sources = config.sources.len(),
        "config loaded"
    );
    Ok(config)
}

fn default_config(env: Environment) -> Result<Config> {
    let home = dirs::home_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;
    Ok(Config {
        environment: env,
        sources: vec![],
        storage_path: home
            .join(env.storage_relative())
            .to_string_lossy()
            .into_owned(),
        embed_model: "openai/text-embedding-3-large".into(),
        embed_dim: 3072,
        reranker_model: None,
        poll_interval_secs: 300,
    })
}

fn file_to_runtime(file: FileConfig, env: Environment, explicit_storage: bool) -> Result<Config> {
    let home = dirs::home_dir()
        .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;

    let storage_path = if explicit_storage {
        file.storage.path.unwrap_or_else(|| {
            home.join(env.storage_relative())
                .to_string_lossy()
                .into_owned()
        })
    } else {
        home.join(env.storage_relative())
            .to_string_lossy()
            .into_owned()
    };

    let sources = file
        .sources
        .into_iter()
        .map(|(name, source)| {
            let mut extra = HashMap::new();
            for (key, value) in source.extra {
                extra.insert(key, value_to_string(&value));
            }

            Ok(SourceConfig {
                name,
                kind: parse_source_kind(&source.kind),
                enabled: source.enabled,
                poll_interval_secs: source.poll_interval_secs,
                extra,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Config {
        environment: env,
        sources,
        storage_path,
        embed_model: file
            .embed
            .model
            .unwrap_or_else(|| "openai/text-embedding-3-large".into()),
        embed_dim: file.embed.dimension.unwrap_or(3072),
        reranker_model: file.runtime.reranker_model,
        poll_interval_secs: file.runtime.poll_interval_secs.unwrap_or(300),
    })
}

fn value_to_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn parse_source_kind(kind: &str) -> SourceKind {
    match kind.to_ascii_lowercase().as_str() {
        "appflowy" => SourceKind::AppFlowy,
        "markdown" | "filesystem" | "fs" => SourceKind::Markdown,
        "obsidian" => {
            tracing::warn!(
                "source kind 'obsidian' is deprecated — use kind = \"markdown\" with root_path"
            );
            SourceKind::Markdown
        }
        "pond" => SourceKind::Pond,
        "tech_tracker" | "techtracker" => SourceKind::TechTracker,
        "github" => SourceKind::GitHub,
        other => SourceKind::Custom(other.to_string()),
    }
}
