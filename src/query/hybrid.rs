//! Diamond hybrid search: FTS ∥ vector → RRF barrier.

use crate::embed::Embedder;
use crate::error::Result;
use crate::query::rrf::{candidate_limit, fuse_rrf, RRF_K};
use crate::store::Store;
use crate::types::SearchResult;
use std::sync::Arc;

/// Parallel FTS + optional vector, fused with RRF (`k=60`).
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
        match store.fts_search(query, cand).await {
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
        match store.vector_search(&emb, cand).await {
            Ok(hits) => hits,
            Err(err) => {
                tracing::warn!(error = %err, "vector search failed; using FTS only");
                Vec::new()
            }
        }
    };

    // True diamond: FTS arm || (embed → vector) arm share one join barrier.
    let (fts_hits, vec_hits) = tokio::join!(fts_fut, vec_fut);

    let mut results = fuse_rrf(&[(fts_hits, "fts"), (vec_hits, "vector")], RRF_K);
    results.truncate(limit);
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i;
    }
    Ok(results)
}
