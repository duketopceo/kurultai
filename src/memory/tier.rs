//! Hot / warm / cold classification from access + index timestamps.
//!
//! - **Hot** — recently accessed or freshly indexed (full payload in graph UI).
//! - **Warm** — local SQLite, loadable on demand (stubs until focused).
//! - **Cold** — aged out of active use (still local until object-storage archive; #34).

use crate::types::KnowledgeAtom;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Default policy — matches foveated “look here first” without object storage yet.
#[derive(Debug, Clone)]
pub struct TierPolicy {
    /// `last_accessed_at` within this many days ⇒ hot.
    pub hot_access_days: i64,
    /// `indexed_at` within this many hours ⇒ hot (fresh ingest).
    pub hot_index_hours: i64,
    /// Both timestamps older than this many days ⇒ cold.
    pub cold_days: i64,
    /// Declarative sequester rules (Pillar "subsets" analog, #325). First match
    /// wins; a rule can only cap a tier, never promote it.
    pub rules: Vec<TierRule>,
}

impl Default for TierPolicy {
    fn default() -> Self {
        Self {
            hot_access_days: 7,
            hot_index_hours: 48,
            cold_days: 180,
            rules: Vec::new(),
        }
    }
}

/// A declarative membership rule: atoms matching every set field are capped at
/// `cap`. All present fields must match (AND); unset fields match anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TierRule {
    /// Matches `atom.source` (e.g. `"pond"`).
    pub source: Option<String>,
    /// Matches membership in `atom.tags`.
    pub tag: Option<String>,
    /// Matches `atom.trust_lane`.
    pub trust_lane: Option<crate::types::TrustLane>,
    /// Highest tier a matching atom may reach (`warm` or `cold`).
    pub cap: MemoryTier,
}

impl TierRule {
    fn matches(&self, atom: &KnowledgeAtom) -> bool {
        if let Some(s) = &self.source {
            if &atom.source != s {
                return false;
            }
        }
        if let Some(t) = &self.tag {
            if !atom.tags.iter().any(|tag| tag == t) {
                return false;
            }
        }
        if let Some(l) = self.trust_lane {
            if atom.trust_lane != l {
                return false;
            }
        }
        true
    }
}

/// Memory temperature for graph / retrieval foveation.
/// Declared cold-last so `Ord` ranks Hot < Warm < Cold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

/// Classify an atom at `now` under `policy` — time-only path.
pub fn classify(
    indexed_at: DateTime<Utc>,
    last_accessed_at: DateTime<Utc>,
    now: DateTime<Utc>,
    policy: &TierPolicy,
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

/// Classify a full atom under `policy`: time-based tier, then the first
/// matching [`TierRule`] caps it. Rules only lower, never promote.
pub fn classify_atom(atom: &KnowledgeAtom, now: DateTime<Utc>, policy: &TierPolicy) -> MemoryTier {
    let t = classify(atom.indexed_at, atom.last_accessed_at, now, policy);
    match policy.rules.iter().find(|r| r.matches(atom)) {
        Some(rule) => t.max(rule.cap),
        None => t,
    }
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
    /// Labels only — enough for Brain colouring / edges without full content.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
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
            tags: atom.tags.clone(),
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
        let t = classify(
            now - Duration::days(30),
            now - Duration::days(1),
            now,
            &TierPolicy::default(),
        );
        assert_eq!(t, MemoryTier::Hot);
    }

    #[test]
    fn fresh_index_is_hot() {
        let now = Utc::now();
        let t = classify(
            now - Duration::hours(12),
            now - Duration::days(30),
            now,
            &TierPolicy::default(),
        );
        assert_eq!(t, MemoryTier::Hot);
    }

    #[test]
    fn mid_age_is_warm() {
        let now = Utc::now();
        let t = classify(
            now - Duration::days(30),
            now - Duration::days(30),
            now,
            &TierPolicy::default(),
        );
        assert_eq!(t, MemoryTier::Warm);
    }

    #[test]
    fn ancient_is_cold() {
        let now = Utc::now();
        let t = classify(
            now - Duration::days(200),
            now - Duration::days(200),
            now,
            &TierPolicy::default(),
        );
        assert_eq!(t, MemoryTier::Cold);
    }

    fn atom_with(
        source: &str,
        tags: &[&str],
        trust_lane: crate::types::TrustLane,
    ) -> KnowledgeAtom {
        KnowledgeAtom {
            source: source.into(),
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
            trust_lane,
            ..Default::default()
        }
    }

    fn policy_with(rules: Vec<TierRule>) -> TierPolicy {
        TierPolicy {
            rules,
            ..Default::default()
        }
    }

    #[test]
    fn rule_caps_fresh_atom() {
        // Freshly indexed pond atom would be hot by time; source rule caps cold.
        let now = Utc::now();
        let mut atom = atom_with("pond", &[], crate::types::TrustLane::Trusted);
        atom.indexed_at = now;
        atom.last_accessed_at = now;
        let policy = policy_with(vec![TierRule {
            source: Some("pond".into()),
            tag: None,
            trust_lane: None,
            cap: MemoryTier::Cold,
        }]);
        assert_eq!(classify_atom(&atom, now, &policy), MemoryTier::Cold);
    }

    #[test]
    fn unmatched_rule_leaves_tier() {
        let now = Utc::now();
        let mut atom = atom_with("markdown", &[], crate::types::TrustLane::Trusted);
        atom.indexed_at = now;
        atom.last_accessed_at = now;
        let policy = policy_with(vec![TierRule {
            source: Some("pond".into()),
            tag: None,
            trust_lane: None,
            cap: MemoryTier::Cold,
        }]);
        assert_eq!(classify_atom(&atom, now, &policy), MemoryTier::Hot);
    }

    #[test]
    fn rules_never_promote() {
        let now = Utc::now();
        let mut atom = atom_with("pond", &[], crate::types::TrustLane::Trusted);
        atom.indexed_at = now - Duration::days(200);
        atom.last_accessed_at = now - Duration::days(200);
        let policy = policy_with(vec![TierRule {
            source: Some("pond".into()),
            tag: None,
            trust_lane: None,
            cap: MemoryTier::Warm,
        }]);
        // Time says cold; a warm cap cannot promote it.
        assert_eq!(classify_atom(&atom, now, &policy), MemoryTier::Cold);
    }

    #[test]
    fn first_match_wins() {
        let now = Utc::now();
        let mut atom = atom_with("pond", &["session"], crate::types::TrustLane::Trusted);
        atom.indexed_at = now;
        atom.last_accessed_at = now;
        let policy = policy_with(vec![
            TierRule {
                source: Some("pond".into()),
                tag: None,
                trust_lane: None,
                cap: MemoryTier::Warm,
            },
            TierRule {
                source: Some("pond".into()),
                tag: Some("session".into()),
                trust_lane: None,
                cap: MemoryTier::Cold,
            },
        ]);
        assert_eq!(classify_atom(&atom, now, &policy), MemoryTier::Warm);
    }

    #[test]
    fn multi_field_rule_requires_all() {
        let now = Utc::now();
        let mut atom = atom_with("pond", &[], crate::types::TrustLane::Quarantine);
        atom.indexed_at = now;
        atom.last_accessed_at = now;
        // Rule wants tag "session" AND quarantine — atom has neither the tag.
        let policy = policy_with(vec![TierRule {
            source: None,
            tag: Some("session".into()),
            trust_lane: Some(crate::types::TrustLane::Quarantine),
            cap: MemoryTier::Cold,
        }]);
        assert_eq!(classify_atom(&atom, now, &policy), MemoryTier::Hot);
    }
}
