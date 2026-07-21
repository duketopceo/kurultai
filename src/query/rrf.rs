//! Reciprocal Rank Fusion helpers (Phase 2 / #6).

use crate::types::{KnowledgeAtom, SearchResult};
use std::collections::HashMap;

/// Cerebras / Supabase-style RRF constant.
pub const RRF_K: f64 = 60.0;

/// Prefetch pool per retrieval arm before fusion.
pub fn candidate_limit(final_limit: usize) -> usize {
    final_limit.saturating_mul(4).clamp(20, 100)
}

/// Fuse ranked atom lists with RRF (`k=60`, 1-based ranks).
///
/// Each list is already ordered best-first. Duplicate ids sum contributions and
/// merge `matched_by`. Ties break by `id` ascending.
pub fn fuse_rrf(
    lists: &[(Vec<(KnowledgeAtom, f64)>, &'static str)],
    k: f64,
) -> Vec<SearchResult> {
    #[derive(Default)]
    struct Acc {
        atom: Option<KnowledgeAtom>,
        score: f64,
        matched_by: Vec<String>,
    }

    let mut by_id: HashMap<String, Acc> = HashMap::new();

    for (list, method) in lists {
        for (i, (atom, _raw_score)) in list.iter().enumerate() {
            let rank = (i + 1) as f64;
            let contrib = 1.0 / (k + rank);
            let entry = by_id.entry(atom.id.clone()).or_default();
            if entry.atom.is_none() {
                entry.atom = Some(atom.clone());
            }
            entry.score += contrib;
            let method_s = (*method).to_string();
            if !entry.matched_by.iter().any(|m| m == &method_s) {
                entry.matched_by.push(method_s);
            }
        }
    }

    let mut results: Vec<SearchResult> = by_id
        .into_values()
        .filter_map(|acc| {
            let atom = acc.atom?;
            Some(SearchResult {
                atom,
                score: acc.score,
                rank: 0,
                matched_by: acc.matched_by,
            })
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.atom.id.cmp(&b.atom.id))
    });

    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i;
        // Stable matched_by order: fts before vector when both present.
        r.matched_by.sort();
        if r.matched_by.iter().any(|m| m == "fts") && r.matched_by.iter().any(|m| m == "vector") {
            r.matched_by = vec!["fts".into(), "vector".into()];
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn atom(id: &str, title: &str) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "test".into(),
            source_id: id.into(),
            title: title.into(),
            summary: title.into(),
            content: title.into(),
            question: None,
            resolution: None,
            tags: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn candidate_limit_clamps() {
        assert_eq!(candidate_limit(1), 20);
        assert_eq!(candidate_limit(10), 40);
        assert_eq!(candidate_limit(50), 100);
    }

    #[test]
    fn rrf_sums_shared_id() {
        let a = atom("shared", "Shared");
        let b = atom("fts-only", "Fts");
        let c = atom("vec-only", "Vec");

        let fts = vec![(a.clone(), 0.9), (b, 0.8)];
        let vec_hits = vec![(a.clone(), 0.5), (c, 0.4)];

        let fused = fuse_rrf(&[(fts, "fts"), (vec_hits, "vector")], RRF_K);
        assert_eq!(fused[0].atom.id, "shared");
        assert_eq!(fused[0].matched_by, vec!["fts", "vector"]);
        let expected = 1.0 / (60.0 + 1.0) + 1.0 / (60.0 + 1.0);
        assert!((fused[0].score - expected).abs() < 1e-9);
        assert_eq!(fused.len(), 3);
    }

    #[test]
    fn rrf_tie_breaks_by_id() {
        let a = atom("aaa", "A");
        let b = atom("bbb", "B");
        // Each alone at rank 1 in its list → equal RRF; id order wins.
        let fused = fuse_rrf(
            &[(vec![(a, 1.0)], "fts"), (vec![(b, 1.0)], "vector")],
            RRF_K,
        );
        assert_eq!(fused[0].atom.id, "aaa");
        assert_eq!(fused[1].atom.id, "bbb");
    }

    #[test]
    fn single_list_preserves_order() {
        let a = atom("1", "One");
        let b = atom("2", "Two");
        let fused = fuse_rrf(&[(vec![(a, 1.0), (b, 0.5)], "fts")], RRF_K);
        assert_eq!(fused[0].atom.id, "1");
        assert_eq!(fused[1].atom.id, "2");
        assert_eq!(fused[0].matched_by, vec!["fts"]);
    }
}
