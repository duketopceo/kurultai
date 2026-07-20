use chrono::Utc;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::types::KnowledgeAtom;
use std::collections::HashMap;
use tempfile::tempdir;

fn atom(id: &str, content: &str, emb: Option<Vec<f32>>) -> KnowledgeAtom {
    KnowledgeAtom {
        id: id.into(),
        source: "test".into(),
        source_id: id.into(),
        source_uri: Some(format!("file://{id}")),
        title: id.into(),
        summary: String::new(),
        content: content.into(),
        question: None,
        resolution: None,
        tags: vec!["export".into()],
        provenance: Some("schema-test".into()),
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        content_hash: format!("h-{id}"),
        embedding: emb,
        metadata: HashMap::new(),
    }
}

#[test]
fn open_empty_db_creates_tables() {
    let dir = tempdir().unwrap();
    let store = SqliteVecStore::open(dir.path().join("s.db"), "model-a", 4).unwrap();
    assert_eq!(store.count().unwrap(), 0);
    assert_eq!(store.embed_model().unwrap().as_deref(), Some("model-a"));
}

#[test]
fn upsert_fts_roundtrip_and_export_fields() {
    let dir = tempdir().unwrap();
    let store = SqliteVecStore::open(dir.path().join("s.db"), "m", 4).unwrap();
    store
        .upsert(&atom("a", "hello schema-export-token", None))
        .unwrap();
    let hits = store.fts_search("schema-export-token", 5).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].0.tags, vec!["export".to_string()]);
    assert_eq!(hits[0].0.provenance.as_deref(), Some("schema-test"));
    assert!(hits[0].0.source_uri.is_some());
}

#[test]
fn zero_vector_and_dim_mismatch() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("s.db");
    let store = SqliteVecStore::open(&path, "m", 4).unwrap();
    assert!(store.upsert(&atom("z", "x", Some(vec![0.0; 4]))).is_err());
    assert!(store.upsert(&atom("d", "x", Some(vec![0.1, 0.2]))).is_err());
    drop(store);
    assert!(SqliteVecStore::open(&path, "m", 8).is_err());
    assert!(SqliteVecStore::open(&path, "other", 4).is_err());
}
