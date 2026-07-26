use crate::art::BannerMode;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

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
    pub banner: BannerSetting,
}

/// File-form of [`BannerMode`]; accepts TOML bool or `"auto"` / `"true"` / `"false"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BannerSetting(pub BannerMode);

impl From<BannerSetting> for BannerMode {
    fn from(value: BannerSetting) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for BannerSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = BannerSetting;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("true, false, or \"auto\"")
            }

            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(BannerSetting(if v {
                    BannerMode::Always
                } else {
                    BannerMode::Never
                }))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v.trim().to_ascii_lowercase().as_str() {
                    "auto" => Ok(BannerSetting(BannerMode::Auto)),
                    "true" | "always" | "on" => Ok(BannerSetting(BannerMode::Always)),
                    "false" | "never" | "off" => Ok(BannerSetting(BannerMode::Never)),
                    other => Err(E::custom(format!(
                        "invalid [cli].banner value {other:?}; expected true, false, or \"auto\""
                    ))),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
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
