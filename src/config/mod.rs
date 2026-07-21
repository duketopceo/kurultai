mod file;
mod loader;

pub use file::FileConfig;
pub use loader::{config_path, load_config, load_config_from, load_config_with_env};

/// Canonical on-disk default matching [`FileConfig`] section shape.
pub fn default_config_toml() -> &'static str {
    r#"environment = "dev"

[storage]
path = "~/.local/share/kurultai/dev/store.db"

[embed]
model = "openai/text-embedding-3-large"
dimension = 3072

[runtime]
poll_interval_secs = 300
"#
}

use crate::error::{KurultaiError, Result};
use crate::types::{Config, SourceKind};
use std::path::{Path, PathBuf};

/// Validate a loaded config before wiring the app.
pub fn validate(config: &Config) -> Result<()> {
    if config.storage_path.trim().is_empty() {
        return Err(KurultaiError::config("storage.path must not be empty"));
    }

    if config.embed_dim == 0 {
        return Err(KurultaiError::config("embed.dimension must be > 0"));
    }

    if config.embed_model.trim().is_empty() {
        return Err(KurultaiError::config("embed.model must not be empty"));
    }

    let mut names = std::collections::HashSet::new();
    for source in &config.sources {
        if !names.insert(&source.name) {
            return Err(KurultaiError::config(format!(
                "duplicate source name: {}",
                source.name
            )));
        }

        if source.enabled && matches!(source.kind, SourceKind::Markdown) {
            let root = source
                .extra
                .get("root_path")
                .or_else(|| source.extra.get("vault_path"))
                .map(String::as_str)
                .unwrap_or("");
            if root.trim().is_empty() {
                return Err(KurultaiError::config(format!(
                    "source '{}' (markdown) requires root_path",
                    source.name
                )));
            }
        }
    }

    Ok(())
}

/// Expand `~` and normalize storage path.
pub fn expand_path(path: &str) -> Result<PathBuf> {
    let expanded = if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| KurultaiError::config("could not resolve home directory"))?;
        home.join(rest)
    } else if path == "~" {
        dirs::home_dir().ok_or_else(|| KurultaiError::config("could not resolve home directory"))?
    } else {
        PathBuf::from(path)
    };

    Ok(expanded)
}

/// Resolve and create parent dirs for the storage database.
pub fn ensure_storage_parent(storage_path: &Path) -> Result<()> {
    if let Some(parent) = storage_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::types::Config;

    #[test]
    fn validate_rejects_empty_storage() {
        let config = Config {
            environment: Environment::Dev,
            sources: vec![],
            storage_path: "  ".into(),
            embed_model: "model".into(),
            embed_dim: 3072,
            reranker_model: None,
            poll_interval_secs: 300,
        };
        assert!(validate(&config).is_err());
    }
}
