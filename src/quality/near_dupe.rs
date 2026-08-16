//! Background near-duplicate pass (daemon / post-index). Not on remember hot path.

use crate::embed::Embedder;
use crate::error::Result;
use crate::hashutil::sha256_hex;
use crate::quality::merge::{bodies_similar, has_merge_conflict, merge_additive, survivor_loser};
use crate::store::{SearchFilter, Store};
use crate::types::{KnowledgeAtom, TrustLane};
use std::collections::HashSet;
use std::sync::Arc;

/// Near-dupe Jaccard threshold for pairing (plan ≥ 0.92).
const JACCARD_CANDIDATE: f64 = 0.92;
/// Safe auto-merge requires ≥ 0.95 (or identical content_hash).
const JACCARD_SAFE_MERGE: f64 = 0.95;
const SCAN_CAP: usize = 500;

/// Run one near-dupe pass. Returns (auto_merges, candidates_inserted).
pub async fn run_near_dupe_pass(
    store: &dyn Store,
    _embedder: &Arc<dyn Embedder>,
) -> Result<(usize, usize)> {
    let mut candidates = store.list_near_dupe_candidates(SCAN_CAP).await?;
    if candidates.is_empty() {
        return Ok((0, 0));
    }

    let trusted = store
        .list_atoms(SCAN_CAP, SearchFilter::trusted(true))
        .await?;
    for t in trusted {
        if !candidates.iter().any(|c| c.id == t.id) {
            candidates.push(t);
        }
    }
    candidates.truncate(SCAN_CAP);

    let mut auto_merges = 0usize;
    let mut pending = 0usize;
    let mut removed: HashSet<String> = HashSet::new();

    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            let a = &candidates[i];
            let b = &candidates[j];
            if removed.contains(&a.id) || removed.contains(&b.id) {
                continue;
            }
            // Never pair across project namespaces (#184): two sessions recording
            // the same fact under different projects are two atoms on purpose.
            // Merging them would delete one namespace's copy.
            if a.project_id() != b.project_id() {
                continue;
            }
            if !bodies_similar(a, b, JACCARD_CANDIDATE) {
                continue;
            }

            if has_merge_conflict(a, b) {
                if store
                    .insert_merge_candidate(&a.id, &b.id, "near_dupe_conflict")
                    .await?
                {
                    pending += 1;
                }
                continue;
            }

            let exact_hash = sha256_hex(&a.content) == sha256_hex(&b.content);
            let safe_similar = exact_hash || bodies_similar(a, b, JACCARD_SAFE_MERGE);
            if !safe_similar {
                if store
                    .insert_merge_candidate(&a.id, &b.id, "near_dupe_below_safe")
                    .await?
                {
                    pending += 1;
                }
                continue;
            }

            let both_trusted =
                a.trust_lane == TrustLane::Trusted && b.trust_lane == TrustLane::Trusted;
            // Conservative: paraphrase among trusted → candidate unless exact hash.
            if both_trusted && !exact_hash {
                if store
                    .insert_merge_candidate(&a.id, &b.id, "near_dupe_trusted_paraphrase")
                    .await?
                {
                    pending += 1;
                }
                continue;
            }

            if try_auto_merge(store, a, b).await? {
                let (_, loser) = survivor_loser(a, b);
                removed.insert(loser.id.clone());
                auto_merges += 1;
            }
        }
    }

    Ok((auto_merges, pending))
}

async fn try_auto_merge(store: &dyn Store, a: &KnowledgeAtom, b: &KnowledgeAtom) -> Result<bool> {
    let (survivor, loser) = survivor_loser(a, b);
    let merged = merge_additive(survivor, loser);
    let detail = serde_json::json!({
        "loser_id": loser.id,
        "survivor_id": survivor.id,
    });
    store.apply_auto_merge(&merged, &loser.id, &detail).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::NullEmbedder;
    use crate::quality::gate::{apply_gate, GateOutcome};
    use crate::store::SqliteVecStore;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_store() -> Arc<SqliteVecStore> {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "kurultai-neardupe-{}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap())
    }

    fn atom(id: &str, title: &str, content: &str, lane: TrustLane) -> KnowledgeAtom {
        let mut a = KnowledgeAtom {
            id: id.into(),
            source: "agent".into(),
            source_id: format!("/{id}"),
            title: title.into(),
            summary: content.into(),
            content: content.into(),
            tags: vec!["ops".into()],
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            ..Default::default()
        };
        match lane {
            TrustLane::Trusted => apply_gate(&mut a, GateOutcome::Trusted),
            TrustLane::Quarantine => apply_gate(
                &mut a,
                GateOutcome::Quarantine {
                    reason: "untagged".into(),
                },
            ),
        }
        a
    }

    #[tokio::test]
    async fn empty_store_noop() {
        let store = temp_store();
        let emb: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let (m, p) = run_near_dupe_pass(store.as_ref() as &dyn Store, &emb)
            .await
            .unwrap();
        assert_eq!((m, p), (0, 0));
    }

    #[tokio::test]
    async fn identical_trusted_and_quarantine_auto_merges() {
        let store = temp_store();
        let emb: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let t = atom("t1", "Same", "identical body text", TrustLane::Trusted);
        let q = atom("q1", "Same", "identical body text", TrustLane::Quarantine);
        store.upsert(&t).await.unwrap();
        store.upsert(&q).await.unwrap();

        let (m, _) = run_near_dupe_pass(store.as_ref() as &dyn Store, &emb)
            .await
            .unwrap();
        assert!(m >= 1);
        let t_gone = store.get("t1").await.unwrap().is_none();
        let q_gone = store.get("q1").await.unwrap().is_none();
        assert!(t_gone || q_gone);
        // Trusted must survive over quarantine.
        assert!(store.get("t1").await.unwrap().is_some());
        assert!(store.get("q1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn three_identical_atoms_skip_stale_pairs_after_merge() {
        let store = temp_store();
        let emb: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let body = "triple identical body for merge skip";
        let a = atom("a1", "Same", body, TrustLane::Trusted);
        let b = atom("b1", "Same", body, TrustLane::Quarantine);
        let c = atom("c1", "Same", body, TrustLane::Quarantine);
        store.upsert(&a).await.unwrap();
        store.upsert(&b).await.unwrap();
        store.upsert(&c).await.unwrap();

        let (m, pending) = run_near_dupe_pass(store.as_ref() as &dyn Store, &emb)
            .await
            .unwrap();
        // Two quarantine copies merge into trusted; no pending for stale pairs.
        assert_eq!(m, 2);
        assert_eq!(pending, 0);
        assert!(store.get("a1").await.unwrap().is_some());
        assert!(store.get("b1").await.unwrap().is_none());
        assert!(store.get("c1").await.unwrap().is_none());
        assert_eq!(store.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn conflicting_titles_create_candidate() {
        let store = temp_store();
        let emb: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let a = atom("c1", "Title A", "shared exact body", TrustLane::Trusted);
        let b = atom("c2", "Title B", "shared exact body", TrustLane::Trusted);
        store.upsert(&a).await.unwrap();
        store.upsert(&b).await.unwrap();

        let (_, pending) = run_near_dupe_pass(store.as_ref() as &dyn Store, &emb)
            .await
            .unwrap();
        assert!(pending >= 1);
        assert!(store.get("c1").await.unwrap().is_some());
        assert!(store.get("c2").await.unwrap().is_some());
        assert!(store.count_merge_candidates_pending().await.unwrap() >= 1);
    }
}
