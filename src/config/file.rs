use crate::art::BannerMode;
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

    #[serde(default)]
    pub cli: FileCliConfig,

    /// Deployment environment: dev | staging | prod
    #[serde(default)]
    pub environment: Option<String>,
}

/// `[cli]` presentation settings.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileCliConfig {
    /// `true` | `false` | `"auto"` (default auto = TTY only).
    #[serde(default)]
    pub banner: BannerMode,
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
    /// `local` enables on-device ONNX (feature `local-embed`). Omit for OpenRouter/Null.
    pub backend: Option<String>,
    pub model: Option<String>,
    pub dimension: Option<usize>,
}

impl Default for FileEmbedConfig {
    fn default() -> Self {
        Self {
            backend: None,
            model: Some("openai/text-embedding-3-large".into()),
            dimension: Some(3072),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileRuntimeConfig {
    pub poll_interval_secs: Option<u64>,
    pub reranker_model: Option<String>,
    /// Local hour 0–23 for nightly full reindex (#73).
    pub nightly_full_sync_hour: Option<u8>,
    /// Skip poll when idle this many hours (#73).
    pub inactivity_threshold_hours: Option<u64>,
}

fn default_true() -> bool {
    true
}

fn default_poll_interval() -> u64 {
    300
}
