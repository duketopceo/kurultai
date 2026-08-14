//! Acceptance tests — ontology O1 (KHAN-251).
//!
//! Covers: seeded class tree, entity CRUD, typed links, promote atom→entity
//! (instance_of), list entities/links, and the fail-closed link parser.

use chrono::Utc;
use kurultai::ontology::{
    promote_atom_to_entity, CLASS_CODE, CLASS_DECISION, CLASS_MEMORY, CLASS_NOTE, CLASS_PERSON,
    CLASS_SYSTEM, SEEDED_CLASS_IDS,
};
use kurultai::store::{migrations, SqliteVecStore, Store};
use kurultai::types::{KnowledgeAtom, OntologyEntity, OntologyLink, OntologyLinkType};
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_store() -> SqliteVecStore {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "khan251-ont-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    SqliteVecStore::open(dir.join("store.db"), 4).unwrap()
}

fn sample_atom(id: &str) -> KnowledgeAtom {
    KnowledgeAtom {
        id: id.into(),
        source: "markdown".into(),
        source_id: format!("/{id}.md"),
        title: format!("Ontology note {id}"),
        summary: "KNOWN_PHRASE_ONTOLOGY_42".into(),
        content: "KNOWN_PHRASE_ONTOLOGY_42 body detail for the gate".into(),
        tags: vec!["test".into()],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        ..Default::default()
    }
}

// ── Seeded class tree ────────────────────────────────────────────────────────

#[tokio::test]
async fn schema_version_is_9() {
    let _store = temp_store();
    assert_eq!(migrations::CURRENT_SCHEMA_VERSION, 9);
}

#[tokio::test]
async fn seeded_classes_present_after_migration() {
    let store = temp_store();
    let entities = store.list_ontology_entities(500).await.unwrap();
    let ids: Vec<&str> = entities.iter().map(|e| e.id.as_str()).collect();
    for class in SEEDED_CLASS_IDS {
        assert!(ids.contains(class), "seeded class {class} missing");
    }
    assert!(ids.contains(&CLASS_MEMORY));
    assert!(ids.contains(&CLASS_NOTE));
    assert!(ids.contains(&CLASS_CODE));
    assert!(ids.contains(&CLASS_DECISION));
    assert!(ids.contains(&CLASS_PERSON));
    assert!(ids.contains(&CLASS_SYSTEM));
}

#[tokio::test]
async fn seeded_is_a_links_present() {
    let store = temp_store();
    let links = store.list_ontology_links(None).await.unwrap();
    // At least one is_a link per class in the seed hierarchy.
    assert!(links.len() >= 5, "seeded links must exist: {links:?}");
    assert!(
        links.iter().any(|l| l.rel == OntologyLinkType::IsA),
        "seeded links must include is_a"
    );
}

// ── Entity CRUD ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn upsert_and_get_ontology_entity() {
    let store = temp_store();
    let entity = OntologyEntity {
        id: "ent:custom1".into(),
        kind: "instance".into(),
        name: "Custom Entity".into(),
        atom_id: None,
        attributes: serde_json::json!({"team": "platform"}),
    };
    store.upsert_ontology_entity(&entity).await.unwrap();
    let loaded = store
        .get_ontology_entity("ent:custom1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded.name, "Custom Entity");
    assert_eq!(loaded.kind, "instance");
    assert_eq!(loaded.attributes["team"], "platform");
}

#[tokio::test]
async fn upsert_entity_is_idempotent_update() {
    let store = temp_store();
    let entity = OntologyEntity {
        id: "ent:up1".into(),
        kind: "instance".into(),
        name: "V1".into(),
        atom_id: None,
        attributes: serde_json::json!({}),
    };
    store.upsert_ontology_entity(&entity).await.unwrap();
    let mut updated = entity.clone();
    updated.name = "V2".into();
    store.upsert_ontology_entity(&updated).await.unwrap();
    let loaded = store.get_ontology_entity("ent:up1").await.unwrap().unwrap();
    assert_eq!(loaded.name, "V2");
}

#[tokio::test]
async fn list_ontology_entities_respects_limit() {
    let store = temp_store();
    let n = store.list_ontology_entities(3).await.unwrap();
    assert_eq!(n.len(), 3, "limit must clamp result count");
}

// ── Typed links ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn upsert_and_list_ontology_link() {
    let store = temp_store();
    let link = OntologyLink {
        id: "link:test:associates:note".into(),
        from_id: "ent:test1".into(),
        to_id: CLASS_NOTE.into(),
        rel: OntologyLinkType::AssociatesWith,
        confidence: 0.9,
        status: "approved".into(),
        actor: "test".into(),
    };
    store.upsert_ontology_link(&link).await.unwrap();
    let links = store.list_ontology_links(None).await.unwrap();
    assert!(links.iter().any(|l| l.id == "link:test:associates:note"));
}

#[tokio::test]
async fn upsert_link_updates_confidence_on_duplicate() {
    let store = temp_store();
    let mk = |conf: f32| OntologyLink {
        id: "link:dup".into(),
        from_id: "ent:a".into(),
        to_id: CLASS_NOTE.into(),
        rel: OntologyLinkType::AssociatesWith,
        confidence: conf,
        status: "approved".into(),
        actor: "test".into(),
    };
    store.upsert_ontology_link(&mk(0.3)).await.unwrap();
    store.upsert_ontology_link(&mk(0.8)).await.unwrap();
    let links = store.list_ontology_links(Some("ent:a")).await.unwrap();
    let l = links.iter().find(|l| l.from_id == "ent:a").unwrap();
    assert!((l.confidence - 0.8).abs() < 1e-6, "confidence must update");
}

#[tokio::test]
async fn list_links_by_endpoint_matches_atom_id() {
    let store = temp_store();
    // Promote an atom to an entity, then query links by the atom_id.
    let atom = sample_atom("ont-atom1");
    store.upsert(&atom).await.unwrap();
    let entity = promote_atom_to_entity(&store, "ont-atom1", CLASS_NOTE, "test")
        .await
        .unwrap();
    let links = store.list_ontology_links(Some("ont-atom1")).await.unwrap();
    assert!(
        links.iter().any(|l| l.from_id == entity.id),
        "listing by atom_id must resolve the entity's links"
    );
}

// ── Promote atom → entity ────────────────────────────────────────────────────

#[tokio::test]
async fn promote_atom_creates_instance_entity_and_instance_of_link() {
    let store = temp_store();
    let atom = sample_atom("promote1");
    store.upsert(&atom).await.unwrap();

    let entity = promote_atom_to_entity(&store, "promote1", CLASS_NOTE, "mcp")
        .await
        .unwrap();
    assert_eq!(entity.kind, "instance");
    assert_eq!(entity.atom_id.as_deref(), Some("promote1"));
    assert!(entity.id.starts_with("ent:promote1"));

    let links = store.list_ontology_links(Some(&entity.id)).await.unwrap();
    assert!(
        links.iter().any(|l| {
            l.from_id == entity.id
                && l.to_id == CLASS_NOTE
                && l.rel == OntologyLinkType::InstanceOf
                && l.status == "approved"
        }),
        "instance_of link must be created"
    );
}

#[tokio::test]
async fn promote_atom_is_idempotent() {
    let store = temp_store();
    let atom = sample_atom("promote2");
    store.upsert(&atom).await.unwrap();
    let e1 = promote_atom_to_entity(&store, "promote2", CLASS_NOTE, "a")
        .await
        .unwrap();
    let e2 = promote_atom_to_entity(&store, "promote2", CLASS_NOTE, "b")
        .await
        .unwrap();
    assert_eq!(e1.id, e2.id, "second promote must reuse existing entity");
}

#[tokio::test]
async fn promote_atom_missing_atom_errors() {
    let store = temp_store();
    let err = promote_atom_to_entity(&store, "no-such-atom", CLASS_NOTE, "t")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

#[tokio::test]
async fn promote_atom_missing_class_errors() {
    let store = temp_store();
    let atom = sample_atom("promote3");
    store.upsert(&atom).await.unwrap();
    let err = promote_atom_to_entity(&store, "promote3", "class:nonexistent", "t")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}

#[tokio::test]
async fn promote_atom_rejects_non_class_target() {
    let store = temp_store();
    let atom = sample_atom("promote4");
    store.upsert(&atom).await.unwrap();
    // Create an instance entity, then try to promote onto it (must fail).
    let instance = OntologyEntity {
        id: "ent:instance1".into(),
        kind: "instance".into(),
        name: "Instance".into(),
        atom_id: None,
        attributes: serde_json::json!({}),
    };
    store.upsert_ontology_entity(&instance).await.unwrap();
    let err = promote_atom_to_entity(&store, "promote4", "ent:instance1", "t")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("expected class"), "{err}");
}

// ── LinkType parse / as_str ──────────────────────────────────────────────────

#[test]
fn ontology_link_type_round_trips() {
    for (lt, s) in [
        (OntologyLinkType::IsA, "is_a"),
        (OntologyLinkType::InstanceOf, "instance_of"),
        (OntologyLinkType::AssociatesWith, "associates_with"),
        (OntologyLinkType::TriggeredBy, "triggered_by"),
        (OntologyLinkType::Contradicts, "contradicts"),
    ] {
        assert_eq!(lt.as_str(), s);
        assert_eq!(OntologyLinkType::parse(s), Some(lt));
    }
}

#[test]
fn ontology_link_type_parse_unknown_returns_none() {
    assert_eq!(OntologyLinkType::parse("unknown"), None);
    assert_eq!(OntologyLinkType::parse(""), None);
}

#[test]
fn promote_does_not_change_trust_lane() {
    // promote_atom_to_entity explicitly does not alter trust_lane (doc guarantee).
    // Verified structurally: the function reads the atom but never calls set_trust_lane.
    // This test asserts the documented contract by checking the atom stays trusted.
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let store = temp_store();
        let mut atom = sample_atom("promote5");
        atom.trust_lane = kurultai::types::TrustLane::Trusted;
        store.upsert(&atom).await.unwrap();
        promote_atom_to_entity(&store, "promote5", CLASS_NOTE, "t")
            .await
            .unwrap();
        let reloaded = store.get("promote5").await.unwrap().unwrap();
        assert_eq!(reloaded.trust_lane, kurultai::types::TrustLane::Trusted);
    });
}
