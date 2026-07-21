//! Diamond hybrid search: FTS ∥ vector → RRF barrier → optional rerank.

use crate::brain::{AgentAtomView, DEFAULT_EXCERPT_CAP};
use crate::embed::Embedder;
use crate::error::Result;
use crate::query::rrf::{candidate_limit, fuse_rrf_ids, RRF_K};
use crate::rerank::{apply_rerank_order, Reranker};
use crate::store::Store;
use crate::types::SearchResult;
use std::collections::HashMap;
use std::sync::Arc;

/// Parallel FTS + optional vector, fused with RRF (`k=60`), optional LLM rerank.
pub async fn hybrid_search(
    store: &Arc<dyn Store>,
    embedder: &Arc<dyn Embedder>,
    reranker: &Arc<dyn Reranker>,
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
            }
        }
    }

    Ok(results)
}
