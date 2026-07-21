//! Reciprocal Rank Fusion helpers (Phase 2 / #6).

use crate::types::{KnowledgeAtom, SearchResult};
use std::collections::HashMap;

/// Cerebras / Supabase-style RRF constant.
pub const RRF_K: f64 = 60.0;

/// Prefetch pool per retrieval arm before fusion.
pub fn candidate_limit(final_limit: usize) -> usize {
    final_limit.saturating_mul(4).clamp(20, 100)
}

/// Fused id with RRF score and provenance methods.
#[derive(Debug, Clone)]
pub struct FusedId {
    pub id: String,
    pub score: f64,
    pub matched_by: Vec<String>,
}

/// Fuse ranked id lists with RRF (`k=60`, 1-based ranks).
pub fn fuse_rrf_ids(lists: &[(Vec<(String, f64)>, &'static str)], k: f64) -> Vec<FusedId> {
    #[derive(Default)]
    struct Acc {
        score: f64,
        matched_by: Vec<String>,
    }

    let mut by_id: HashMap<String, Acc> = HashMap::new();

    for (list, method) in lists {
        for (i, (id, _raw_score)) in list.iter().enumerate() {
            let rank = (i + 1) as f64;
            let contrib = 1.0 / (k + rank);
            let entry = by_id.entry(id.clone()).or_default();
            entry.score += contrib;
            let method_s = (*method).to_string();
            if !entry.matched_by.iter().any(|m| m == &method_s) {
                entry.matched_by.push(method_s);
            }
        }
    }

    let mut results: Vec<FusedId> = by_id
        .into_iter()
        .map(|(id, mut acc)| {
            acc.matched_by.sort();
            if acc.matched_by.iter().any(|m| m == "fts")
                && acc.matched_by.iter().any(|m| m == "vector")
            {
                acc.matched_by = vec!["fts".into(), "vector".into()];
            }
            FusedId {
                id,
                score: acc.score,
                matched_by: acc.matched_by,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    results
}

/// Fuse ranked atom lists (test helper) via id fusion.
pub fn fuse_rrf(lists: &[(Vec<(KnowledgeAtom, f64)>, &'static str)], k: f64) -> Vec<SearchResult> {
    let id_lists: Vec<(Vec<(String, f64)>, &'static str)> = lists
        .iter()
        .map(|(list, method)| {
            (
                list.iter()
                    .map(|(a, s)| (a.id.clone(), *s))
                    .collect::<Vec<_>>(),
                *method,
            )
        })
        .collect();

    let mut atoms: HashMap<String, KnowledgeAtom> = HashMap::new();
    for (list, _) in lists {
        for (atom, _) in list {
            atoms.entry(atom.id.clone()).or_insert_with(|| atom.clone());
        }
    }

    fuse_rrf_ids(&id_lists, k)
        .into_iter()
        .enumerate()
        .filter_map(|(rank, fused)| {
            let atom = atoms.remove(&fused.id)?;
            Some(SearchResult {
                atom,
                score: fused.score,
                rank,
                matched_by: fused.matched_by,
            })
        })
        .collect()
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

    #[test]
    fn fuse_rrf_ids_orders_by_score() {
        let fused = fuse_rrf_ids(
            &[(vec![("a".into(), 1.0), ("b".into(), 0.5)], "fts")],
            RRF_K,
        );
        assert_eq!(fused[0].id, "a");
        assert_eq!(fused[1].id, "b");
    }
}
