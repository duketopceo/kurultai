//! Deterministic gap analysis for `ask` — reports what the brain *lacks*,
//! not just what it found. No LLM in the loop: coverage is judged from
//! retrieval shape (hit count, trust lanes) plus unmatched salient terms.

use crate::types::{SearchResult, TrustLane};
use std::collections::HashSet;

/// Below this many hits, coverage counts as thin.
const THIN_HIT_FLOOR: usize = 3;
/// Cap on reported unmatched terms so gap text stays one-line.
const MAX_GAP_TERMS: usize = 4;

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "are", "was", "were", "with", "that", "this", "from", "have", "has",
    "what", "when", "where", "which", "who", "whom", "why", "how", "does", "did", "can", "could",
    "should", "would", "will", "shall", "about", "into", "over", "under", "between", "there",
    "their", "they", "them", "then", "than", "its", "our", "your", "you", "not", "but", "all",
    "any", "each", "other", "some", "such", "only", "own", "same", "also", "just",
    // interrogative/filler verbs — presence in a question carries no topic signal
    "work", "works", "mean", "means", "tell", "show", "list", "give", "find", "know", "need",
    "want", "like", "look", "explain",
];

/// Describe knowledge gaps for a question given its retrieved hits.
/// Empty vec = adequate coverage. Deterministic — same inputs, same gaps.
pub fn analyze_gaps(question: &str, hits: &[SearchResult]) -> Vec<String> {
    let mut gaps = Vec::new();

    if hits.is_empty() {
        gaps.push("no coverage — zero atoms matched this question".to_string());
    } else {
        if hits.len() < THIN_HIT_FLOOR {
            gaps.push(format!(
                "thin coverage — only {} atom{} matched",
                hits.len(),
                if hits.len() == 1 { "" } else { "s" }
            ));
        }
        if hits
            .iter()
            .all(|h| h.atom.trust_lane == TrustLane::Quarantine)
        {
            gaps.push("coverage exists only in quarantine — nothing trusted".to_string());
        }
    }

    let missing = unmatched_terms(question, hits);
    if !missing.is_empty() {
        gaps.push(format!(
            "the brain lacks coverage of: {}",
            missing.join(", ")
        ));
    }

    gaps
}

/// Salient question terms absent from every hit's title/tags/content.
/// Matching is case-insensitive substring on the joined hit text.
fn unmatched_terms(question: &str, hits: &[SearchResult]) -> Vec<String> {
    let mut terms = HashSet::new();
    for tok in question.split(|c: char| !c.is_alphanumeric() && c != '-') {
        let t = tok.trim_matches('-').to_lowercase();
        if t.len() >= 4 && !STOPWORDS.contains(&t.as_str()) {
            terms.insert(t);
        }
    }
    if terms.is_empty() {
        return Vec::new();
    }

    let corpus: String = hits
        .iter()
        .map(|h| {
            format!(
                "{} {} {}",
                h.atom.title,
                h.atom.tags.join(" "),
                h.atom.content.chars().take(500).collect::<String>()
            )
        })
        .collect::<String>()
        .to_lowercase();

    let mut missing: Vec<String> = terms
        .into_iter()
        .filter(|t| !corpus.contains(t.as_str()))
        .collect();
    missing.sort();
    missing.truncate(MAX_GAP_TERMS);
    missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KnowledgeAtom;

    fn hit(title: &str, content: &str, lane: TrustLane) -> SearchResult {
        let atom = KnowledgeAtom {
            title: title.into(),
            content: content.into(),
            trust_lane: lane,
            ..Default::default()
        };
        SearchResult {
            atom,
            score: 1.0,
            rank: 0,
            matched_by: vec!["fts".into()],
        }
    }

    #[test]
    fn empty_hits_report_no_coverage_and_missing_terms() {
        let gaps = analyze_gaps("what is the flux capacitor retention policy", &[]);
        assert!(gaps.iter().any(|g| g.contains("no coverage")));
        assert!(gaps
            .iter()
            .any(|g| g.contains("lacks coverage of: capacitor, flux, policy, retention")));
    }

    #[test]
    fn thin_and_quarantine_only() {
        let hits = vec![hit(
            "Deploy runbook",
            "how we deploy",
            TrustLane::Quarantine,
        )];
        let gaps = analyze_gaps("deploy runbook", &hits);
        assert!(gaps.iter().any(|g| g.contains("thin coverage")));
        assert!(gaps.iter().any(|g| g.contains("quarantine")));
    }

    #[test]
    fn adequate_coverage_reports_nothing() {
        let hits: Vec<_> = (0..4)
            .map(|_| {
                hit(
                    "Embedding cache",
                    "the embedding cache stores vectors",
                    TrustLane::Trusted,
                )
            })
            .collect();
        let gaps = analyze_gaps("how does the embedding cache work", &hits);
        assert!(gaps.is_empty(), "{gaps:?}");
    }
}
