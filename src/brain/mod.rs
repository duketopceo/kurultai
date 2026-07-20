//! Knowledge brain design — agent-optimized SQL, not document storage.
//!
//! # North star
//!
//! **Speed at read time, structure at write time, tokens never wasted.**
//!
//! Heavy work (embed, distill, dedupe) happens at **index time**.
//! Agents get **pre-structured excerpts** at **query time** — sub-second, minimal tokens.
//!
//! # Doctrine
//!
//! 1. **SQL agent-optimized** — SQLite + FTS5 + vectors; one row = one `KnowledgeAtom`
//! 2. **Pristine structure** — fixed schema, stable IDs, provenance (`source`, `source_id`, `source_uri`)
//! 3. **Structuring rules** — MCP never returns raw `content` by default; see [`AgentAtomView`]
//! 4. **Token budget** — `search` → excerpts; `cite` → one slice; `ask` → synthesis + cites only
//! 5. **Bleeding-edge speed** — cache hot queries, skip re-embed on content-hash match, batch index
//!
//! Neural-net / post-train export reuses the same schema ([#33]) — no second pipeline.

use crate::types::KnowledgeAtom;
use serde::{Deserialize, Serialize};

/// What agents receive from read operations — never the full atom unless explicitly requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAtomView {
    pub id: String,
    pub source: String,
    pub source_id: String,
    pub title: String,
    /// Distilled summary — primary token payload for search results.
    pub summary: String,
    /// Short excerpt for grounding; capped at ingest/MCP boundary.
    pub excerpt: String,
    pub score: f64,
    pub tags: Vec<String>,
    /// Optional routing fields when present (cheap to include, high signal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// Default excerpt cap for MCP read responses (characters, not tokens — tune per model later).
pub const DEFAULT_EXCERPT_CAP: usize = 400;

impl AgentAtomView {
    /// Build a token-efficient view from a stored atom. Full `content` stays in SQL.
    pub fn from_atom(atom: &KnowledgeAtom, score: f64, excerpt_cap: usize) -> Self {
        let cap = excerpt_cap.max(1);
        let excerpt = if atom.summary.len() >= cap {
            atom.summary.chars().take(cap).collect()
        } else {
            let remainder = cap.saturating_sub(atom.summary.len());
            format!(
                "{}{}",
                atom.summary,
                atom.content.chars().take(remainder).collect::<String>()
            )
        };

        Self {
            id: atom.id.clone(),
            source: atom.source.clone(),
            source_id: atom.source_id.clone(),
            title: atom.title.clone(),
            summary: atom.summary.clone(),
            excerpt,
            score,
            tags: atom.tags.clone(),
            question: atom.question.clone(),
            resolution: atom.resolution.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn view_caps_excerpt() {
        let atom = KnowledgeAtom {
            id: "a".into(),
            source: "markdown".into(),
            source_id: "/x.md".into(),
            title: "T".into(),
            summary: "short".into(),
            content: "x".repeat(1000),
            question: None,
            resolution: None,
            tags: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: None,
            metadata: Default::default(),
        };
        let view = AgentAtomView::from_atom(&atom, 0.9, 50);
        assert!(view.excerpt.len() <= 50);
        assert!(!view.excerpt.contains(&"x".repeat(500)));
    }
}
