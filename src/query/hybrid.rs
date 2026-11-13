//! Diamond hybrid search: FTS ∥ vector → RRF barrier → soft-label / quality boost → optional rerank.

use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::Result;
use crate::ingest::dump::QUALITY_SCORE_KEY;
use crate::query::rrf::{candidate_limit, fuse_rrf_ids, RRF_K};
use crate::rerank::{apply_rerank_order, Reranker};
use crate::store::{SearchFilter, Store};
use crate::types::{KnowledgeAtom, SearchResult, TrustLane};
use std::collections::HashMap;
use std::sync::Arc;

/// Multiplier weight for soft-label match boost: `score * (1 + α * best_score)`.
const SOFT_LABEL_BOOST_ALPHA: f64 = 0.5;

/// Light post-RRF quality boost: `score * (1 + α * quality_score)` for trusted atoms.
const QUALITY_BOOST_ALPHA: f64 = 0.15;

/// True when the query mentions a soft label name or alias (case-insensitive substring).
fn query_matches_soft_label(query_lc: &str, atom: &KnowledgeAtom) -> Option<f32> {
    let mut best: Option<f32> = None;
    for sl in &atom.soft_labels {
        let name = sl.name.to_lowercase();
        if !name.is_empty() && query_lc.contains(&name) {
            best = Some(best.map_or(sl.score, |b| b.max(sl.score)));
        }
        for alias in &sl.aliases {
            let a = alias.to_lowercase();
            if !a.is_empty() && query_lc.contains(&a) {
                best = Some(best.map_or(sl.score, |b| b.max(sl.score)));
            }
        }
    }
    best
}

fn resort_by_score(results: &mut [SearchResult]) {
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.atom.id.cmp(&b.atom.id))
    });
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i;
    }
}

/// Post-RRF boost when the query matches a soft label (#113).
fn apply_soft_label_boost(query: &str, results: &mut [SearchResult]) {
    let q = query.to_lowercase();
    let mut boosted = false;
    for r in results.iter_mut() {
        if let Some(best) = query_matches_soft_label(&q, &r.atom) {
            r.score *= 1.0 + SOFT_LABEL_BOOST_ALPHA * f64::from(best);
            if !r.matched_by.iter().any(|m| m == "soft_label") {
                r.matched_by.push("soft_label".into());
            }
            boosted = true;
        }
    }
    if boosted {
        resort_by_score(results);
    }
}

/// Post-RRF light boost from atom `quality_score` metadata (trusted only).
fn apply_quality_score_boost(results: &mut [SearchResult]) {
    let mut boosted = false;
    for r in results.iter_mut() {
        if r.atom.trust_lane != TrustLane::Trusted {
            continue;
        }
        let Some(raw) = r.atom.metadata.get(QUALITY_SCORE_KEY) else {
            continue;
        };
        let Ok(qs) = raw.parse::<f64>() else {
            continue;
        };
        let qs = qs.clamp(0.0, 1.0);
        if qs <= 0.0 {
            continue;
        }
        r.score *= 1.0 + QUALITY_BOOST_ALPHA * qs;
        if !r.matched_by.iter().any(|m| m == "quality") {
            r.matched_by.push("quality".into());
        }
        boosted = true;
    }
    if boosted {
        resort_by_score(results);
    }
}

/// Hybrid results plus which retrieval arms degraded (failed and were skipped).
#[derive(Debug, Clone, Default)]
pub struct HybridOutcome {
    pub results: Vec<SearchResult>,
    /// Subset of `fts`, `vector`, `rerank` (`rerank` = kept RRF order). Empty when healthy.
    pub degraded: Vec<String>,
}

/// Parallel FTS + optional vector, fused with RRF (`k=60`), optional LLM rerank.
pub async fn hybrid_search(
    store: &Arc<dyn Store>,
    embedder: &Arc<dyn Embedder>,
    reranker: &Arc<dyn Reranker>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    hybrid_search_filtered(
        store,
        embedder,
        reranker,
        query,
        limit,
        SearchFilter::default(),
    )
    .await
}

/// Hybrid search with an explicit trust-lane filter.
pub async fn hybrid_search_filtered(
    store: &Arc<dyn Store>,
    embedder: &Arc<dyn Embedder>,
    reranker: &Arc<dyn Reranker>,
    query: &str,
    limit: usize,
    filter: SearchFilter,
) -> Result<Vec<SearchResult>> {
    Ok(
        hybrid_search_outcome(store, embedder, reranker, query, limit, filter)
            .await?
            .results,
    )
}

/// Like [`hybrid_search_filtered`] but also reports degraded arms; every returned
/// hit carries the same `degraded` list.
pub async fn hybrid_search_outcome(
    store: &Arc<dyn Store>,
    embedder: &Arc<dyn Embedder>,
    reranker: &Arc<dyn Reranker>,
    query: &str,
    limit: usize,
    filter: SearchFilter,
) -> Result<HybridOutcome> {
    if query.trim().is_empty() {
        return Ok(HybridOutcome::default());
    }

    let limit = limit.clamp(1, 50);
    let cand = candidate_limit(limit);

    let fts_filter = filter.clone();
    let fts_fut = async {
        match store.fts_search_ids(query, cand, fts_filter).await {
            Ok(hits) => (hits, false),
            Err(err) => {
                tracing::warn!(error = %err, "FTS search failed; continuing without FTS arm");
                (Vec::new(), true)
            }
        }
    };

    let vec_fut = async {
        if !embedder.is_live() {
            // Intentionally not configured: not a degradation.
            return (Vec::new(), false);
        }
        let emb = match embedder.embed(query).await {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(error = %err, "semantic search skipped; using FTS only");
                return (Vec::new(), true);
            }
        };
        match store.vector_search_ids(&emb, cand, filter.clone()).await {
            Ok(hits) => (hits, false),
            Err(err) => {
                tracing::warn!(error = %err, "vector search failed; using FTS only");
                (Vec::new(), true)
            }
        }
    };

    let ((fts_hits, fts_failed), (vec_hits, vec_failed)) = tokio::join!(fts_fut, vec_fut);
    let mut degraded: Vec<String> = Vec::new();
    if fts_failed {
        degraded.push("fts".into());
    }
    if vec_failed {
        degraded.push("vector".into());
    }

    let mut fused = fuse_rrf_ids(&[(fts_hits, "fts"), (vec_hits, "vector")], RRF_K);
    fused.truncate(limit);

    let ids: Vec<String> = fused.iter().map(|f| f.id.clone()).collect();
    let atoms = store.get_many(&ids).await?;
    let by_id: HashMap<String, _> = atoms.into_iter().map(|a| (a.id.clone(), a)).collect();

    let mut results = Vec::with_capacity(fused.len());
    for (rank, f) in fused.into_iter().enumerate() {
        let Some(atom) = by_id.get(&f.id).cloned() else {
            continue;
        };
        results.push(SearchResult {
            atom,
            score: f.score,
            rank,
            matched_by: f.matched_by,
            degraded: Vec::new(),
        });
    }
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i;
    }

    apply_soft_label_boost(query, &mut results);
    apply_quality_score_boost(&mut results);

    if reranker.is_live() && !results.is_empty() {
        let candidates: Vec<(String, String)> = results
            .iter()
            .map(|r| {
                let view = AgentAtomView::from_atom(&r.atom, r.score, DEFAULT_EXCERPT_CAP);
                (r.atom.id.clone(), view.excerpt)
            })
            .collect();
        match reranker.rerank(query, &candidates).await {
            Ok(order) if !order.is_empty() => {
                results = apply_rerank_order(results, &order);
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(error = %err, "rerank failed; keeping RRF order");
                degraded.push("rerank".into());
            }
        }
    }

    stamp_degraded(&mut results, &degraded);
    Ok(HybridOutcome { results, degraded })
}

/// Copy the degraded list onto every hit (no-op when healthy).
pub fn stamp_degraded(results: &mut [SearchResult], degraded: &[String]) {
    if degraded.is_empty() {
        return;
    }
    for r in results.iter_mut() {
        r.degraded = degraded.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{KnowledgeAtom, SoftLabel};

    #[test]
    fn soft_label_boost_reorders_when_query_matches() {
        let mut low = SearchResult {
            atom: KnowledgeAtom {
                id: "low".into(),
                soft_labels: vec![],
                ..Default::default()
            },
            score: 0.010,
            rank: 0,
            matched_by: vec!["fts".into()],
            degraded: Vec::new(),
        };
        let mut high = SearchResult {
            atom: KnowledgeAtom {
                id: "high".into(),
                soft_labels: vec![SoftLabel {
                    label_id: 1,
                    name: "kubernetes".into(),
                    score: 1.0,
                    aliases: vec!["k8s".into()],
                }],
                ..Default::default()
            },
            score: 0.009,
            rank: 1,
            matched_by: vec!["fts".into()],
            degraded: Vec::new(),
        };
        // Before boost, low ranks first by score.
        let mut results = vec![low.clone(), high.clone()];
        apply_soft_label_boost("how do we run kubernetes", &mut results);
        assert_eq!(results[0].atom.id, "high");
        assert!(results[0].matched_by.iter().any(|m| m == "soft_label"));
        assert!(results[0].score > results[1].score);

        // Alias match.
        low.score = 0.010;
        high.score = 0.009;
        let mut results = vec![low, high];
        apply_soft_label_boost("k8s rollout", &mut results);
        assert_eq!(results[0].atom.id, "high");
    }

    #[test]
    fn quality_score_boost_reorders_trusted() {
        use crate::ingest::dump::QUALITY_SCORE_KEY;
        use std::collections::HashMap;

        let mut low = SearchResult {
            atom: KnowledgeAtom {
                id: "low".into(),
                trust_lane: TrustLane::Trusted,
                metadata: HashMap::from([(QUALITY_SCORE_KEY.into(), "0.1".into())]),
                ..Default::default()
            },
            score: 0.010,
            rank: 0,
            matched_by: vec!["fts".into()],
            degraded: Vec::new(),
        };
        let mut high = SearchResult {
            atom: KnowledgeAtom {
                id: "high".into(),
                trust_lane: TrustLane::Trusted,
                metadata: HashMap::from([(QUALITY_SCORE_KEY.into(), "1.0".into())]),
                ..Default::default()
            },
            score: 0.0095,
            rank: 1,
            matched_by: vec!["fts".into()],
            degraded: Vec::new(),
        };
        let mut results = vec![low.clone(), high.clone()];
        apply_quality_score_boost(&mut results);
        assert_eq!(results[0].atom.id, "high");
        assert!(results[0].matched_by.iter().any(|m| m == "quality"));

        // Quarantine must not receive the boost even with a high score.
        high.atom.trust_lane = TrustLane::Quarantine;
        high.score = 0.0095;
        low.score = 0.010;
        let mut results = vec![low, high];
        apply_quality_score_boost(&mut results);
        assert_eq!(results[0].atom.id, "low");
    }

    // ---- degraded indicator (one test per failing arm) ----

    use crate::error::KurultaiError;
    use crate::store::SqliteVecStore;

    struct TestEmbedder {
        dim: usize,
        fail: bool,
    }

    #[async_trait::async_trait]
    impl Embedder for TestEmbedder {
        fn name(&self) -> &str {
            "test"
        }
        fn dim(&self) -> usize {
            self.dim
        }
        async fn embed(&self, _t: &str) -> Result<Vec<f32>> {
            if self.fail {
                Err(KurultaiError::Embed("boom".into()))
            } else {
                Ok(vec![1.0; self.dim])
            }
        }
        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts.iter().map(|_| vec![1.0; self.dim]).collect())
        }
    }

    struct TestReranker {
        fail: bool,
    }

    #[async_trait::async_trait]
    impl Reranker for TestReranker {
        fn name(&self) -> &str {
            "test"
        }
        async fn rerank(&self, _q: &str, c: &[(String, String)]) -> Result<Vec<String>> {
            if self.fail {
                Err(KurultaiError::Other(anyhow::anyhow!("rerank boom")))
            } else {
                Ok(c.iter().map(|(id, _)| id.clone()).collect())
            }
        }
    }

    async fn degraded_fixture(dir: &tempfile::TempDir) -> (Arc<dyn Store>, std::path::PathBuf) {
        let path = dir.path().join("s.db");
        let store = SqliteVecStore::open(path.clone(), 4).unwrap();
        store
            .upsert(&KnowledgeAtom {
                id: "a1".into(),
                source: "t".into(),
                source_id: "a1".into(),
                title: "Quokka handbook".into(),
                summary: "quokka care".into(),
                content: "quokka care and feeding".into(),
                embedding: Some(vec![1.0; 4]),
                ..Default::default()
            })
            .await
            .unwrap();
        (Arc::new(store), path)
    }

    async fn run(
        store: &Arc<dyn Store>,
        embed_dim: usize,
        embed_fail: bool,
        rerank_fail: bool,
    ) -> HybridOutcome {
        let e: Arc<dyn Embedder> = Arc::new(TestEmbedder {
            dim: embed_dim,
            fail: embed_fail,
        });
        let r: Arc<dyn Reranker> = Arc::new(TestReranker { fail: rerank_fail });
        hybrid_search_outcome(store, &e, &r, "quokka", 5, SearchFilter::default())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn healthy_search_is_not_degraded() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = degraded_fixture(&dir).await;
        let out = run(&store, 4, false, false).await;
        assert!(out.degraded.is_empty());
        assert_eq!(out.results.len(), 1);
        let json = serde_json::to_value(&out.results[0]).unwrap();
        assert!(json.get("degraded").is_none(), "omitted when healthy");
    }

    #[tokio::test]
    async fn fts_arm_failure_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let (store, path) = degraded_fixture(&dir).await;
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch("DROP TABLE atoms_fts;")
            .unwrap();
        let out = run(&store, 4, false, false).await;
        assert_eq!(out.degraded, vec!["fts"]);
        // Vector arm still serves the hit, stamped degraded.
        assert_eq!(out.results.len(), 1);
        assert_eq!(out.results[0].degraded, vec!["fts"]);
        let json = serde_json::to_value(&out.results[0]).unwrap();
        assert_eq!(json["degraded"][0], "fts");
    }

    #[tokio::test]
    async fn vector_arm_embed_failure_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = degraded_fixture(&dir).await;
        let out = run(&store, 4, true, false).await;
        assert_eq!(out.degraded, vec!["vector"]);
        assert_eq!(out.results.len(), 1, "FTS still answers");
    }

    #[tokio::test]
    async fn vector_arm_store_failure_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = degraded_fixture(&dir).await;
        // Query embedding dim 3 != store dim 4 -> store error.
        let out = run(&store, 3, false, false).await;
        assert_eq!(out.degraded, vec!["vector"]);
    }

    #[tokio::test]
    async fn rerank_failure_is_reported_and_keeps_rrf_order() {
        let dir = tempfile::tempdir().unwrap();
        let (store, _) = degraded_fixture(&dir).await;
        let out = run(&store, 4, false, true).await;
        assert_eq!(out.degraded, vec!["rerank"]);
        assert_eq!(out.results.len(), 1);
        assert_eq!(out.results[0].degraded, vec!["rerank"]);
    }

    #[tokio::test]
    async fn all_arms_failing_reports_all_with_empty_hits() {
        let dir = tempfile::tempdir().unwrap();
        let (store, path) = degraded_fixture(&dir).await;
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch("DROP TABLE atoms_fts;")
            .unwrap();
        let out = run(&store, 4, true, true).await;
        assert_eq!(out.degraded, vec!["fts", "vector"]);
        assert!(out.results.is_empty());
    }
}
