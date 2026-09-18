//! Pure retrieval metrics over ranked hit lists.
//!
//! Binary metrics take a per-rank relevance flag list plus the total number of
//! relevant items for the query. `ndcg_at_k` takes graded gains instead.

/// Fraction of all relevant items that appear in the top `k` ranks.
/// `total_relevant == 0` returns 1.0 (nothing to recall).
pub fn recall_at_k(ranked_relevant: &[bool], total_relevant: usize, k: usize) -> f64 {
    if total_relevant == 0 {
        return 1.0;
    }
    let hits = ranked_relevant.iter().take(k).filter(|r| **r).count();
    hits as f64 / total_relevant as f64
}

/// Fraction of the top `k` ranks that are relevant.
pub fn precision_at_k(ranked_relevant: &[bool], k: usize) -> f64 {
    let k = k.min(ranked_relevant.len());
    if k == 0 {
        return 0.0;
    }
    let hits = ranked_relevant.iter().take(k).filter(|r| **r).count();
    hits as f64 / k as f64
}

/// Reciprocal rank of the first relevant hit; 0.0 when none is present.
pub fn mrr(ranked_relevant: &[bool]) -> f64 {
    match ranked_relevant.iter().position(|r| *r) {
        Some(pos) => 1.0 / (pos + 1) as f64,
        None => 0.0,
    }
}

/// Normalized discounted cumulative gain at `k` over graded `gains`.
/// `ideal` is the best achievable gain ordering (usually the same gains
/// sorted descending). Zero-ideal returns 1.0.
pub fn ndcg_at_k(gains: &[f64], ideal: &[f64], k: usize) -> f64 {
    let dcg = |g: &[f64]| -> f64 {
        g.iter()
            .take(k)
            .enumerate()
            .map(|(i, g)| g / (i as f64 + 2.0).log2())
            .sum()
    };
    let ideal_dcg = dcg(ideal);
    if ideal_dcg == 0.0 {
        return 1.0;
    }
    dcg(gains) / ideal_dcg
}

/// Arithmetic mean, 0.0 on empty input.
pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recall_counts_hits_in_top_k() {
        let flags = [true, false, true, false];
        assert_eq!(recall_at_k(&flags, 2, 10), 1.0);
        assert_eq!(recall_at_k(&flags, 2, 1), 0.5);
        assert_eq!(recall_at_k(&flags, 4, 10), 0.5);
        assert_eq!(recall_at_k(&flags, 0, 10), 1.0);
    }

    #[test]
    fn precision_counts_relevant_in_window() {
        let flags = [true, false, true, false];
        assert_eq!(precision_at_k(&flags, 4), 0.5);
        assert_eq!(precision_at_k(&flags, 2), 0.5);
        assert_eq!(precision_at_k(&flags, 0), 0.0);
    }

    #[test]
    fn mrr_is_reciprocal_of_first_hit() {
        assert_eq!(mrr(&[true]), 1.0);
        assert_eq!(mrr(&[false, false, true]), 1.0 / 3.0);
        assert_eq!(mrr(&[false, false]), 0.0);
    }

    #[test]
    fn ndcg_perfect_ranking_is_one() {
        let gains = [3.0, 2.0, 1.0, 0.0];
        assert!((ndcg_at_k(&gains, &gains, 4) - 1.0).abs() < 1e-9);
        // Swapped order scores below 1.
        let shuffled = [1.0, 3.0, 2.0, 0.0];
        assert!(ndcg_at_k(&shuffled, &gains, 4) < 1.0);
        assert!((ndcg_at_k(&[0.0, 0.0], &[0.0, 0.0], 2) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mean_handles_empty() {
        assert_eq!(mean(&[]), 0.0);
        assert_eq!(mean(&[1.0, 3.0]), 2.0);
    }
}
