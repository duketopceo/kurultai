use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single knowledge atom — the unit of indexed information.
/// Raw-first in Phase 1: distillation fields may be empty/None.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeAtom {
    /// Content-addressed ID (hash of source + source_id + content_hash)
    pub id: String,
    /// Which source this came from (e.g. "filesystem", "obsidian")
    pub source: String,
    /// ID within the source (file path, page ID, etc.)
    pub source_id: String,
    /// Stable URI for export / citations (e.g. file://...)
    pub source_uri: Option<String>,
    /// Human-readable title
    pub title: String,
    /// Short summary (empty until distillation exists)
    pub summary: String,
    /// Full raw text content
    pub content: String,
    /// LLM-distilled question (optional; Phase 1 leaves None)
    pub question: Option<String>,
    /// LLM-distilled resolution (optional)
    pub resolution: Option<String>,
    /// Durable labels for future post-train export
    pub tags: Vec<String>,
    /// Free-form / JSON provenance for multi-year export
    pub provenance: Option<String>,
    /// When the source was last modified
    pub source_updated_at: DateTime<Utc>,
    /// When this atom was indexed
    pub indexed_at: DateTime<Utc>,
    /// SHA-256 of file/content bytes for change detection
    pub content_hash: String,
    /// Embedding vector (stored when keyed; never all-zeros)
    pub embedding: Option<Vec<f32>>,
    /// Arbitrary source-specific metadata
    pub metadata: HashMap<String, String>,
}

/// A search result returned by the query pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub atom: KnowledgeAtom,
    pub score: f64,
    pub rank: usize,
    pub matched_by: Vec<String>,
}

/// A synthesized answer with citations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub question: String,
    pub answer: String,
    pub citations: Vec<Citation>,
    pub sources_used: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source: String,
    pub source_id: String,
    pub title: String,
    pub url: Option<String>,
    pub excerpt: String,
}

/// Configuration for a data source connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub kind: SourceKind,
    pub enabled: bool,
    #[serde(default = "default_poll")]
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

fn default_poll() -> u64 {
    60
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Filesystem,
    Obsidian,
    AppFlowy,
    Pond,
    TechTracker,
    GitHub,
    Custom,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Filesystem => "filesystem",
            SourceKind::Obsidian => "obsidian",
            SourceKind::AppFlowy => "appflowy",
            SourceKind::Pond => "pond",
            SourceKind::TechTracker => "tech_tracker",
            SourceKind::GitHub => "github",
            SourceKind::Custom => "custom",
        }
    }

    /// Kinds with a registered connector implementation in this build.
    pub fn registered() -> &'static [SourceKind] {
        &[SourceKind::Filesystem, SourceKind::Obsidian]
    }

    pub fn is_implemented(self) -> bool {
        matches!(self, SourceKind::Filesystem | SourceKind::Obsidian)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KurultaiEnv {
    #[default]
    Dev,
    Staging,
    Prod,
}

impl KurultaiEnv {
    pub fn as_str(self) -> &'static str {
        match self {
            KurultaiEnv::Dev => "dev",
            KurultaiEnv::Staging => "staging",
            KurultaiEnv::Prod => "prod",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "dev" | "development" => Some(KurultaiEnv::Dev),
            "staging" | "stage" => Some(KurultaiEnv::Staging),
            "prod" | "production" => Some(KurultaiEnv::Prod),
            _ => None,
        }
    }
}

/// Top-level config (written by `kurultai init`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub env: KurultaiEnv,
    pub sources: Vec<SourceConfig>,
    /// Absolute or `~`-prefixed path; empty means default data-dir store for `env`.
    #[serde(default)]
    pub storage_path: String,
    pub embed_model: String,
    pub embed_dim: usize,
    #[serde(default)]
    pub reranker_model: Option<String>,
    #[serde(default = "default_poll")]
    pub poll_interval_secs: u64,
    /// Env var name holding OpenRouter API key (default OPENROUTER_API_KEY).
    #[serde(default = "default_key_env")]
    pub openrouter_api_key_env: String,
}

fn default_key_env() -> String {
    "OPENROUTER_API_KEY".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            env: KurultaiEnv::Dev,
            sources: vec![],
            storage_path: String::new(),
            embed_model: "openai/text-embedding-3-small".into(),
            embed_dim: 1536,
            reranker_model: None,
            poll_interval_secs: 60,
            openrouter_api_key_env: default_key_env(),
        }
    }
}
