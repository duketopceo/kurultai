//! Acceptance tests — mesh partitioning schema (KHAN-mesh).
//!
//! Unlike `acceptance_visibility.rs`'s TA-1..TA-4 (which only prove
//! `corpus_tier` / `visibility_labels` round-trip through the store), these
//! tests prove *retrieval-time* exclusion: a `SearchFilter::mesh_scope` /
//! `max_tier` set on the query must actually keep an out-of-scope atom out of
//! FTS results, vector results, and the raw id-list variants of both — not
//! just persist correctly on write. That's the exact gap the mesh schema is
//! meant to close (see FEATURE_MATRIX.md's note on TA-1..TA-4).

use chrono::Utc;
use kurultai::store::{SearchFilter, SqliteVecStore, Store};
use kurultai::types::{CorpusTier, KnowledgeAtom};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_store(dim: usize) -> SqliteVecStore {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "khan-mesh-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    SqliteVecStore::open(dir.join("store.db"), dim).unwrap()
}

fn mesh_atom(id: &str, mesh_ids: &[&str], embed: Vec<f32>) -> KnowledgeAtom {
    KnowledgeAtom {
        id: id.into(),
        source: "test".into(),
        source_id: format!("/{id}"),
        title: format!("Atom {id}"),
        summary: "mesh partition test summary".into(),
        content: format!("mesh partition unique-token-{id} test content with enough detail"),
        tags: vec!["t".into()],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        embedding: Some(embed),
        mesh_ids: mesh_ids.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

fn scope(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// Two atoms in different meshes; a query scoped to only one mesh must exclude
/// the other atom from FTS, vector, and both ID-list variants.
#[tokio::test]
async fn mesh_scope_excludes_atom_outside_scope_on_every_retrieval_path() {
    let store = temp_store(4);

    let in_scope = mesh_atom("mesh-in", &["crew-a"], vec![0.9, 0.9, 0.9, 0.9]);
    let out_of_scope = mesh_atom("mesh-out", &["crew-b"], vec![0.9, 0.9, 0.9, 0.9]);
    store.upsert(&in_scope).await.unwrap();
    store.upsert(&out_of_scope).await.unwrap();

    let filter = SearchFilter {
        mesh_scope: Some(scope(&["crew-a"])),
        ..Default::default()
    };

    // FTS (hydrated)
    let fts = store
        .fts_search("mesh partition test", 10, filter.clone())
        .await
        .unwrap();
    assert!(fts.iter().any(|(a, _)| a.id == "mesh-in"));
    assert!(
        !fts.iter().any(|(a, _)| a.id == "mesh-out"),
        "mesh-out leaked into fts_search: {fts:?}"
    );

    // FTS id-list
    let fts_ids = store
        .fts_search_ids("mesh partition test", 10, filter.clone())
        .await
        .unwrap();
    assert!(fts_ids.iter().any(|(id, _)| id == "mesh-in"));
    assert!(
        !fts_ids.iter().any(|(id, _)| id == "mesh-out"),
        "mesh-out leaked into fts_search_ids: {fts_ids:?}"
    );

    // Vector (hydrated) — both atoms share the same embedding, so a naive kNN
    // would return both; mesh_scope must still exclude mesh-out.
    let vec_hits = store
        .vector_search(&[0.9, 0.9, 0.9, 0.9], 10, filter.clone())
        .await
        .unwrap();
    assert!(vec_hits.iter().any(|(a, _)| a.id == "mesh-in"));
    assert!(
        !vec_hits.iter().any(|(a, _)| a.id == "mesh-out"),
        "mesh-out leaked into vector_search: {vec_hits:?}"
    );

    // Vector id-list
    let vec_ids = store
        .vector_search_ids(&[0.9, 0.9, 0.9, 0.9], 10, filter.clone())
        .await
        .unwrap();
    assert!(vec_ids.iter().any(|(id, _)| id == "mesh-in"));
    assert!(
        !vec_ids.iter().any(|(id, _)| id == "mesh-out"),
        "mesh-out leaked into vector_search_ids: {vec_ids:?}"
    );

    // list_atoms
    let listed = store.list_atoms(50, filter.clone()).await.unwrap();
    assert!(listed.iter().any(|a| a.id == "mesh-in"));
    assert!(
        !listed.iter().any(|a| a.id == "mesh-out"),
        "mesh-out leaked into list_atoms: {listed:?}"
    );
}

/// No `mesh_scope` set = no mesh filtering (backward-compatible default) —
/// both atoms are visible to an unscoped caller.
#[tokio::test]
async fn no_mesh_scope_set_sees_every_mesh() {
    let store = temp_store(4);
    store
        .upsert(&mesh_atom("mesh-a", &["crew-a"], vec![0.5, 0.5, 0.5, 0.5]))
        .await
        .unwrap();
    store
        .upsert(&mesh_atom("mesh-b", &["crew-b"], vec![0.5, 0.5, 0.5, 0.5]))
        .await
        .unwrap();

    let listed = store
        .list_atoms(50, SearchFilter::default())
        .await
        .unwrap();
    assert!(listed.iter().any(|a| a.id == "mesh-a"));
    assert!(listed.iter().any(|a| a.id == "mesh-b"));
}

/// An explicitly empty mesh scope (`Some(HashSet::new())`) is a fail-closed
/// request — distinct from `None` — and must match nothing.
#[tokio::test]
async fn empty_mesh_scope_fails_closed_to_no_results() {
    let store = temp_store(4);
    store
        .upsert(&mesh_atom("mesh-only", &["crew-a"], vec![0.5, 0.5, 0.5, 0.5]))
        .await
        .unwrap();

    let filter = SearchFilter {
        mesh_scope: Some(HashSet::new()),
        ..Default::default()
    };
    let listed = store.list_atoms(50, filter.clone()).await.unwrap();
    assert!(listed.is_empty(), "empty scope must match nothing: {listed:?}");

    let vec_ids = store
        .vector_search_ids(&[0.5, 0.5, 0.5, 0.5], 10, filter)
        .await
        .unwrap();
    assert!(vec_ids.is_empty(), "empty scope must match nothing: {vec_ids:?}");
}

/// `max_tier: Public` excludes a `Private` atom from every retrieval path,
/// even though both atoms are otherwise unscoped by mesh.
#[tokio::test]
async fn max_tier_public_excludes_private_atom_on_every_retrieval_path() {
    let store = temp_store(4);

    let mut public_atom = mesh_atom("tier-pub", &[], vec![0.8, 0.8, 0.8, 0.8]);
    public_atom.corpus_tier = CorpusTier::Public;
    let mut private_atom = mesh_atom("tier-priv", &[], vec![0.8, 0.8, 0.8, 0.8]);
    private_atom.corpus_tier = CorpusTier::Private;
    store.upsert(&public_atom).await.unwrap();
    store.upsert(&private_atom).await.unwrap();

    let filter = SearchFilter {
        max_tier: Some(CorpusTier::Public),
        ..Default::default()
    };

    let fts = store
        .fts_search("mesh partition test", 10, filter.clone())
        .await
        .unwrap();
    assert!(fts.iter().any(|(a, _)| a.id == "tier-pub"));
    assert!(!fts.iter().any(|(a, _)| a.id == "tier-priv"));

    let vec_hits = store
        .vector_search(&[0.8, 0.8, 0.8, 0.8], 10, filter.clone())
        .await
        .unwrap();
    assert!(vec_hits.iter().any(|(a, _)| a.id == "tier-pub"));
    assert!(!vec_hits.iter().any(|(a, _)| a.id == "tier-priv"));

    let listed = store.list_atoms(50, filter).await.unwrap();
    assert!(listed.iter().any(|a| a.id == "tier-pub"));
    assert!(!listed.iter().any(|a| a.id == "tier-priv"));
}

/// `mesh_ids` survives the store round-trip (baseline persistence check,
/// analogous to TA-1..TA-4 — kept alongside the retrieval-exclusion tests
/// above rather than in place of them).
#[tokio::test]
async fn mesh_ids_round_trip_through_store() {
    let store = temp_store(4);
    let atom = mesh_atom("mesh-rt", &["crew-a", "crew-b"], vec![0.1, 0.1, 0.1, 0.1]);
    store.upsert(&atom).await.unwrap();
    let loaded = store.get("mesh-rt").await.unwrap().unwrap();
    assert_eq!(loaded.mesh_ids, vec!["crew-a".to_string(), "crew-b".to_string()]);
}
