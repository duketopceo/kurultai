use serde::Deserialize;
use std::collections::HashMap;

/// On-disk TOML shape. Kept separate from runtime `Config` so we can evolve
/// the file format without breaking internal APIs.
#[derive(Debug, Clone, Deserialize)]
pub struct FileConfig {
    #[serde(default)]
    pub sources: HashMap<String, FileSourceConfig>,

    #[serde(default)]
    pub storage: FileStorageConfig,

    #[serde(default)]
    pub embed: FileEmbedConfig,

    #[serde(default)]
    pub runtime: FileRuntimeConfig,

    /// Deployment environment: dev | staging | prod
    #[serde(default)]
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileSourceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub kind: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileStorageConfig {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileEmbedConfig {
    pub model: Option<String>,
    pub dimension: Option<usize>,
}

impl Default for FileEmbedConfig {
    fn default() -> Self {
        Self {
            model: Some("openai/text-embedding-3-large".into()),
            dimension: Some(3072),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileRuntimeConfig {
    pub poll_interval_secs: Option<u64>,
    pub reranker_model: Option<String>,
    pub synthesis_model: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_poll_interval() -> u64 {
    300
}
