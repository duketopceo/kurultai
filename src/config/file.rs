use crate::art::BannerMode;
use serde::{Deserialize, Serialize};
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

    /// `[tiers]` hot/warm/cold policy + declarative sequester rules (#325).
    #[serde(default)]
    pub tiers: FileTiersConfig,

    /// `[judge]` — Jev judge enable/model (evals, ask --web, review).
    #[serde(default)]
    pub judge: FileJudgeConfig,

    /// `[broker]` — local device broker (`kurultai broker`).
    #[serde(default)]
    pub broker: FileBrokerConfig,
}

/// `[tiers]` — thresholds override the `TierPolicy` defaults; `[[tiers.rule]]`
/// entries cap matching atoms. Missing section = defaults, no rules.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileTiersConfig {
    pub hot_access_days: Option<i64>,
    pub hot_index_hours: Option<i64>,
    pub cold_days: Option<i64>,
    #[serde(default)]
    pub rule: Vec<FileTierRule>,
}

/// `[[tiers.rule]]` — all set fields must match (AND); `cap` is required and
/// must be `warm` or `cold` (rules never promote).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileTierRule {
    pub source: Option<String>,
    pub tag: Option<String>,
    pub trust_lane: Option<String>,
    pub cap: Option<String>,
}

/// `[broker]` — the per-device broker daemon that holds the single upstream
/// seat session to a hosted instance; agents board it locally and never see
/// upstream credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBrokerConfig {
    /// Upstream base URL, e.g. https://knowledge.shippedit.dev
    pub upstream_url: Option<String>,
    /// Loopback port for the broker HTTP surface (agents board here).
    #[serde(default = "default_broker_port")]
    pub port: u16,
    /// Optional unix socket path — `~` expanded; binds in addition to TCP.
    pub socket: Option<String>,
    /// Agent-key name to load for the upstream session
    /// (default `{lane}-broker-{hostname-slug}-agent-token`); set after
    /// `kurultai connect --codename broker`.
    pub key_name: Option<String>,
}

impl Default for FileBrokerConfig {
    fn default() -> Self {
        Self {
            upstream_url: None,
            port: default_broker_port(),
            socket: None,
            key_name: None,
        }
    }
}

fn default_broker_port() -> u16 {
    8420
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

/// `[judge]` — the Jev judge used by evals, `ask --web` sufficiency, and
/// `kurultai review`. Enabled by default when an OpenRouter key resolves.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileJudgeConfig {
    /// `false` forces `NullJudge` regardless of keys (labels-only everywhere).
    pub enabled: Option<bool>,
    /// Judge model override; default is the pinned `typesafe/jev-1.13`.
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileRuntimeConfig {
    pub poll_interval_secs: Option<u64>,
    pub reranker_model: Option<String>,
    /// Local hour 0–23 for nightly full reindex (#73).
    pub nightly_full_sync_hour: Option<u8>,
    /// Skip poll when idle this many hours (#73).
    pub inactivity_threshold_hours: Option<u64>,
    /// Shared secret for daemon MCP HTTP/SSE (`POST /mcp`). Prefer env `KURULTAI_MCP_HTTP_SECRET`.
    pub mcp_http_secret: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_poll_interval() -> u64 {
    300
}
