//! Hot / warm / cold classification from access + index timestamps.
//!
//! - **Hot** — recently accessed or freshly indexed (full payload in graph UI).
//! - **Warm** — local SQLite, loadable on demand (stubs until focused).
//! - **Cold** — aged out of active use (still local until object-storage archive; #34).

use crate::types::KnowledgeAtom;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Default policy — matches foveated “look here first” without object storage yet.
#[derive(Debug, Clone, Copy)]
pub struct TierPolicy {
    /// `last_accessed_at` within this many days ⇒ hot.
    pub hot_access_days: i64,
    /// `indexed_at` within this many hours ⇒ hot (fresh ingest).
    pub hot_index_hours: i64,
    /// Both timestamps older than this many days ⇒ cold.
    pub cold_days: i64,
}

impl Default for TierPolicy {
    fn default() -> Self {
        Self {
            hot_access_days: 7,
            hot_index_hours: 48,
            cold_days: 180,
        }
    }
}

/// Memory temperature for graph / retrieval foveation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTier {
    Hot,
    Warm,
    Cold,
}

impl MemoryTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Warm => "warm",
            Self::Cold => "cold",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "hot" => Some(Self::Hot),
            "warm" => Some(Self::Warm),
            "cold" => Some(Self::Cold),
            _ => None,
        }
    }
}

/// Classify an atom at `now` under `policy`.
pub fn classify(
    indexed_at: DateTime<Utc>,
    last_accessed_at: DateTime<Utc>,
    now: DateTime<Utc>,
    policy: TierPolicy,
) -> MemoryTier {
    let hot_access_cut = now - Duration::days(policy.hot_access_days);
    let hot_index_cut = now - Duration::hours(policy.hot_index_hours);
    if last_accessed_at >= hot_access_cut || indexed_at >= hot_index_cut {
        return MemoryTier::Hot;
    }
    let cold_cut = now - Duration::days(policy.cold_days);
    if last_accessed_at < cold_cut && indexed_at < cold_cut {
        return MemoryTier::Cold;
    }
    MemoryTier::Warm
}

/// Lightweight graph vertex — hot may carry summary; warm/cold are stubs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub source: String,
    pub source_id: String,
    pub tier: MemoryTier,
    pub indexed_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_y: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl GraphNode {
    pub fn from_atom(atom: &KnowledgeAtom, tier: MemoryTier, include_summary: bool) -> Self {
        let layout_x = atom
            .metadata
            .get("layout_x")
            .and_then(|s| s.parse::<f64>().ok());
        let layout_y = atom
            .metadata
            .get("layout_y")
            .and_then(|s| s.parse::<f64>().ok());
        Self {
            id: atom.id.clone(),
            title: atom.title.clone(),
            source: atom.source.clone(),
            source_id: atom.source_id.clone(),
            tier,
            indexed_at: atom.indexed_at,
            last_accessed_at: atom.last_accessed_at,
            layout_x,
            layout_y,
            summary: if include_summary {
                Some(atom.summary.clone())
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_access_is_hot() {
        let now = Utc::now();
        let t = classify(now - Duration::days(30), now - Duration::days(1), now, TierPolicy::default());
        assert_eq!(t, MemoryTier::Hot);
    }

    #[test]
    fn fresh_index_is_hot() {
        let now = Utc::now();
        let t = classify(now - Duration::hours(12), now - Duration::days(30), now, TierPolicy::default());
        assert_eq!(t, MemoryTier::Hot);
    }

    #[test]
    fn mid_age_is_warm() {
        let now = Utc::now();
        let t = classify(now - Duration::days(30), now - Duration::days(30), now, TierPolicy::default());
        assert_eq!(t, MemoryTier::Warm);
    }

    #[test]
    fn ancient_is_cold() {
        let now = Utc::now();
        let t = classify(now - Duration::days(200), now - Duration::days(200), now, TierPolicy::default());
        assert_eq!(t, MemoryTier::Cold);
    }
}
