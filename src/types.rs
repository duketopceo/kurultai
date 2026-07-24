use crate::environment::Environment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single knowledge atom — the unit of indexed information.
///
/// Stored in SQL for speed; agents receive [`crate::brain::AgentAtomView`] via MCP,
/// not raw rows. Keep fields stable for post-train export ([#33]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeAtom {
    /// Unique ID (hash of source + source_id + content)
    pub id: String,
    /// Which source this came from (e.g. "markdown", "appflowy", "pond", "github")
    pub source: String,
    /// ID within the source (page ID, file path, message ID, etc.)
    pub source_id: String,
    /// Human-readable title
    pub title: String,
    /// Short summary (LLM-distilled)
    pub summary: String,
    /// Full raw text content
    pub content: String,
    /// LLM-distilled question this atom answers (for search routing)
    pub question: Option<String>,
    /// LLM-distilled resolution (if applicable)
    pub resolution: Option<String>,
    /// Systems, code refs, tags mentioned
    pub tags: Vec<String>,
    /// When the source was last modified
    pub source_updated_at: DateTime<Utc>,
    /// When this atom was indexed
    pub indexed_at: DateTime<Utc>,
    /// Embedding vector (3072-dim, stored as Vec<f32>)
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
    pub matched_by: Vec<String>, // which retrieval methods matched ("vector", "fts", etc.)
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
    pub poll_interval_secs: u64,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceKind {
    AppFlowy,
    /// Local `.md` directory (Obsidian vault, git wiki, any markdown tree).
    Markdown,
    Pond,
    /// Dayflow Mac activity journal (`chunks.sqlite` timeline cards).
    Dayflow,
    TechTracker,
    GitHub,
    Custom(String),
}

/// Top-level config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Deployment environment (dev, staging, prod).
    #[serde(default)]
    pub environment: Environment,
    pub sources: Vec<SourceConfig>,
    pub storage_path: String,
    pub embed_model: String,
    pub embed_dim: usize,
    pub reranker_model: Option<String>,
    pub poll_interval_secs: u64,
}
