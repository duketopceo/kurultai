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
    /// Multi-hop chain of `source_id` values used for this answer (#74).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graph_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source: String,
    pub source_id: String,
    pub title: String,
    pub url: Option<String>,
    pub excerpt: String,
    /// File path when the atom is filesystem-backed (often same as source_id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Section / heading when known (metadata or title split).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Short content-address of title for exact-match debugging (#75).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_hash: Option<String>,
    /// Character offsets of the excerpt within `content` when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt_start: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt_end: Option<usize>,
}

impl Citation {
    /// Build a citation from an atom with provenance fields (#75).
    pub fn from_atom(atom: &KnowledgeAtom, excerpt: String) -> Self {
        let file_path = atom.metadata.get("file_path").cloned().or_else(|| {
            if atom.source == "markdown" || atom.source == "github" {
                Some(atom.source_id.clone())
            } else {
                None
            }
        });
        let section = atom
            .metadata
            .get("section")
            .cloned()
            .or_else(|| section_from_title(&atom.title));
        let (excerpt_start, excerpt_end) = excerpt_range_in_content(&atom.content, &excerpt);
        Self {
            source: atom.source.clone(),
            source_id: atom.source_id.clone(),
            title: atom.title.clone(),
            url: atom.metadata.get("source_uri").cloned(),
            excerpt,
            file_path,
            section,
            title_hash: Some(short_title_hash(&atom.title)),
            excerpt_start,
            excerpt_end,
        }
    }
}

fn short_title_hash(title: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    title.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn section_from_title(title: &str) -> Option<String> {
    // Titles often look like "Deploy Guide — Database migration"
    for sep in [" — ", " - ", " > ", " / "] {
        if let Some((_, rest)) = title.split_once(sep) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

fn excerpt_range_in_content(content: &str, excerpt: &str) -> (Option<usize>, Option<usize>) {
    if excerpt.is_empty() {
        return (None, None);
    }
    if let Some(start) = content.find(excerpt) {
        return (Some(start), Some(start + excerpt.len()));
    }
    // summary-based excerpts may not be substrings of content
    (None, None)
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
    /// Local hour (0–23) for nightly full reindex; `None` disables (#73).
    #[serde(default)]
    pub nightly_full_sync_hour: Option<u8>,
    /// Skip incremental poll when no index activity for this many hours (#73).
    #[serde(default)]
    pub inactivity_threshold_hours: Option<u64>,
}
