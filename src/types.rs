use crate::environment::Environment;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn chrono_utc_now() -> DateTime<Utc> {
    Utc::now()
}

/// Trust lane for quality gating — trusted atoms are default-retrieval; quarantine is opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrustLane {
    #[default]
    Trusted,
    Quarantine,
}

impl TrustLane {
    /// Canonical DB / wire string (`"trusted"` or `"quarantine"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Quarantine => "quarantine",
        }
    }

    /// Parse a stored lane value. Fail-closed: only exact `"trusted"` is Trusted;
    /// `"quarantine"` and any other/unknown value map to Quarantine so corrupt rows
    /// never leak into default retrieval. DB migration default remains `'trusted'`.
    pub fn parse(s: &str) -> Self {
        match s {
            "trusted" => Self::Trusted,
            "quarantine" => Self::Quarantine,
            _ => Self::Quarantine,
        }
    }
}

/// Visibility scope for tiered access (HUB-1 / #178).
///
/// Solo / no-hub deployments leave every atom at [`VisibilityScope::Personal`].
/// Shared hub slices (HUB-2+) honor `team` / `company`; this field does not
/// imply a remote store by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityScope {
    #[default]
    Personal,
    Team,
    Company,
}

impl VisibilityScope {
    /// Canonical DB / wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Team => "team",
            Self::Company => "company",
        }
    }

    /// Parse a stored scope. Fail-closed toward private: only exact
    /// `"team"` / `"company"` promote; anything else → [`VisibilityScope::Personal`].
    pub fn parse(s: &str) -> Self {
        match s {
            "team" => Self::Team,
            "company" => Self::Company,
            "personal" => Self::Personal,
            _ => Self::Personal,
        }
    }
}

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
    /// Soft multi-label scores (#113) — does **not** satisfy the hard tag trust gate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soft_labels: Vec<SoftLabel>,
    /// When the source was last modified
    pub source_updated_at: DateTime<Utc>,
    /// When this atom was indexed
    pub indexed_at: DateTime<Utc>,
    /// Last search / cite / UI focus — drives hot/warm/cold ([`crate::memory::MemoryTier`]).
    #[serde(default = "chrono_utc_now")]
    pub last_accessed_at: DateTime<Utc>,
    /// Embedding vector (3072-dim, stored as Vec<f32>)
    pub embedding: Option<Vec<f32>>,
    /// Arbitrary source-specific metadata
    pub metadata: HashMap<String, String>,
    /// Quality lane (`trusted` or `quarantine`). Legacy rows migrate as trusted.
    #[serde(default)]
    pub trust_lane: TrustLane,
    /// Why the atom was quarantined (when `trust_lane = quarantine`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quarantine_reason: Option<String>,
    /// Tiered visibility (`personal` | `team` | `company`). Default personal (#178).
    #[serde(default)]
    pub visibility: VisibilityScope,
}

impl Default for KnowledgeAtom {
    fn default() -> Self {
        Self {
            id: String::new(),
            source: String::new(),
            source_id: String::new(),
            title: String::new(),
            summary: String::new(),
            content: String::new(),
            question: None,
            resolution: None,
            tags: Vec::new(),
            soft_labels: Vec::new(),
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            last_accessed_at: Utc::now(),
            embedding: None,
            metadata: HashMap::new(),
            trust_lane: TrustLane::Trusted,
            quarantine_reason: None,
            visibility: VisibilityScope::Personal,
        }
    }
}

/// Soft multi-label with confidence (#113). Hard `tags` remain the trust gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoftLabel {
    pub label_id: i64,
    pub name: String,
    /// Confidence in `[0.0, 1.0]`.
    pub score: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

/// Max soft labels persisted per atom (cardinality clamp).
pub const SOFT_LABEL_MAX: usize = 16;

/// Clamp score into `[0.0, 1.0]` and truncate to [`SOFT_LABEL_MAX`].
pub fn normalize_soft_labels(labels: &[SoftLabel]) -> Vec<SoftLabel> {
    labels
        .iter()
        .take(SOFT_LABEL_MAX)
        .map(|l| SoftLabel {
            label_id: l.label_id,
            name: l.name.trim().to_string(),
            score: l.score.clamp(0.0, 1.0),
            aliases: l.aliases.clone(),
        })
        .filter(|l| !l.name.is_empty())
        .collect()
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
    /// `source_id`s consulted for this answer (primary + multi-hop hits), score order (#74).
    /// Not a graph edge walk — use for provenance, not path reconstruction.
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
    // Stable across compiler releases (unlike DefaultHasher).
    crate::hashutil::sha256_hex(title)
        .chars()
        .take(16)
        .collect()
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
    if let Some(byte_start) = content.find(excerpt) {
        let char_start = content[..byte_start].chars().count();
        let char_len = excerpt.chars().count();
        return (Some(char_start), Some(char_start + char_len));
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
    /// JSON / NDJSON file ingestion. Reads `.json` array-of-objects or `.jsonl`/`.ndjson`
    /// line-delimited files from a `root_path`. Optional `id_field` in `extra` for stable
    /// `source_id`.
    Json,
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
    /// When `Some("local")`, use on-device embeddings (requires `--features local-embed`).
    #[serde(default)]
    pub embed_backend: Option<String>,
    pub reranker_model: Option<String>,
    pub poll_interval_secs: u64,
    /// Local hour (0–23) for nightly full reindex; `None` disables (#73).
    #[serde(default)]
    pub nightly_full_sync_hour: Option<u8>,
    /// Skip incremental poll when no client queries for this many hours (#73).
    #[serde(default)]
    pub inactivity_threshold_hours: Option<u64>,
    /// Shared secret for MCP HTTP/SSE on the daemon. Prefer `KURULTAI_MCP_HTTP_SECRET`.
    #[serde(default)]
    pub mcp_http_secret: Option<String>,
    /// CLI banner presentation (`[cli] banner`); default auto = TTY only.
    #[serde(default)]
    pub banner: crate::art::BannerMode,
}

#[cfg(test)]
mod trust_lane_tests {
    use super::*;

    #[test]
    fn parse_exact_trusted_and_quarantine() {
        assert_eq!(TrustLane::parse("trusted"), TrustLane::Trusted);
        assert_eq!(TrustLane::parse("quarantine"), TrustLane::Quarantine);
    }

    #[test]
    fn parse_invalid_fail_closed_to_quarantine() {
        assert_eq!(TrustLane::parse(""), TrustLane::Quarantine);
        assert_eq!(TrustLane::parse("Trusted"), TrustLane::Quarantine);
        assert_eq!(TrustLane::parse("TRUSTED"), TrustLane::Quarantine);
        assert_eq!(TrustLane::parse("unknown"), TrustLane::Quarantine);
        assert_eq!(TrustLane::parse("trusted "), TrustLane::Quarantine);
    }

    #[test]
    fn as_str_round_trips_with_parse() {
        assert_eq!(
            TrustLane::parse(TrustLane::Trusted.as_str()),
            TrustLane::Trusted
        );
        assert_eq!(
            TrustLane::parse(TrustLane::Quarantine.as_str()),
            TrustLane::Quarantine
        );
    }
}

#[cfg(test)]
mod visibility_scope_tests {
    use super::*;

    #[test]
    fn parse_exact_scopes() {
        assert_eq!(
            VisibilityScope::parse("personal"),
            VisibilityScope::Personal
        );
        assert_eq!(VisibilityScope::parse("team"), VisibilityScope::Team);
        assert_eq!(VisibilityScope::parse("company"), VisibilityScope::Company);
    }

    #[test]
    fn parse_invalid_fail_closed_to_personal() {
        assert_eq!(VisibilityScope::parse(""), VisibilityScope::Personal);
        assert_eq!(VisibilityScope::parse("Team"), VisibilityScope::Personal);
        assert_eq!(VisibilityScope::parse("unknown"), VisibilityScope::Personal);
        assert_eq!(
            VisibilityScope::parse("personal "),
            VisibilityScope::Personal
        );
    }

    #[test]
    fn as_str_round_trips_with_parse() {
        for scope in [
            VisibilityScope::Personal,
            VisibilityScope::Team,
            VisibilityScope::Company,
        ] {
            assert_eq!(VisibilityScope::parse(scope.as_str()), scope);
        }
    }
}

#[cfg(test)]
mod citation_tests {
    use super::*;

    #[test]
    fn title_hash_is_stable_sha256_prefix() {
        let h = short_title_hash("Deploy Guide");
        assert_eq!(h.len(), 16);
        assert_eq!(h, &crate::hashutil::sha256_hex("Deploy Guide")[..16]);
        assert_eq!(h, short_title_hash("Deploy Guide"));
    }

    #[test]
    fn excerpt_range_uses_char_offsets_for_multibyte() {
        let content = "abécaféxy";
        let excerpt = "café";
        let (start, end) = excerpt_range_in_content(content, excerpt);
        assert_eq!(start, Some(3));
        assert_eq!(end, Some(7));
        // Byte index of café would be 4 (é is 2 bytes), not 3.
        assert_ne!(content.find(excerpt), start);
    }

    #[test]
    fn excerpt_range_empty_or_missing() {
        assert_eq!(excerpt_range_in_content("abc", ""), (None, None));
        assert_eq!(excerpt_range_in_content("abc", "zzz"), (None, None));
    }
}
