//! Diamond hybrid search: FTS ∥ vector → RRF barrier.

use crate::embed::Embedder;
use crate::error::Result;
use crate::query::rrf::{candidate_limit, fuse_rrf_ids, RRF_K};
use crate::store::Store;
use crate::types::SearchResult;
use std::collections::HashMap;
use std::sync::Arc;

/// Parallel FTS + optional vector, fused with RRF (`k=60`).
///
/// Arms return id ranks only; full atoms are batch-loaded for the fused top-N.
pub async fn hybrid_search(
    store: &Arc<dyn Store>,
    embedder: &Arc<dyn Embedder>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let limit = limit.clamp(1, 50);
    let cand = candidate_limit(limit);

    let fts_fut = async {
        match store.fts_search_ids(query, cand).await {
            Ok(hits) => hits,
            Err(err) => {
                tracing::warn!(error = %err, "FTS search failed; continuing without FTS arm");
                Vec::new()
            }
        }
    };

    let vec_fut = async {
        if !embedder.is_live() {
            return Vec::new();
        }
        let emb = match embedder.embed(query).await {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(error = %err, "semantic search skipped; using FTS only");
                return Vec::new();
            }
        };
        match store.vector_search_ids(&emb, cand).await {
            Ok(hits) => hits,
            Err(err) => {
                tracing::warn!(error = %err, "vector search failed; using FTS only");
                Vec::new()
            }
        }
    };

    let (fts_hits, vec_hits) = tokio::join!(fts_fut, vec_fut);

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
        });
    }
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i;
    }
    Ok(results)
}
