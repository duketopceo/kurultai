//! O1 labeled property graph beside `KnowledgeAtom` (Wave E / #116).
//!
//! Promote-to-entity does not delete or un-index the atom. Unsupervised `is_a`
//! writes belong to O3, not this module.

use crate::error::{KurultaiError, Result};
use crate::store::Store;
use crate::types::{OntologyEntity, OntologyLink, OntologyLinkType};

pub const CLASS_MEMORY: &str = "class:memory";
pub const CLASS_NOTE: &str = "class:note";
pub const CLASS_CODE: &str = "class:code";
pub const CLASS_DECISION: &str = "class:decision";
pub const CLASS_PERSON: &str = "class:person";
pub const CLASS_SYSTEM: &str = "class:system";

pub const SEEDED_CLASS_IDS: &[&str] = &[
    CLASS_MEMORY,
    CLASS_NOTE,
    CLASS_CODE,
    CLASS_DECISION,
    CLASS_PERSON,
    CLASS_SYSTEM,
];

/// Map an existing atom onto an instance entity + `instance_of` class link.
/// Does not change `trust_lane`.
pub async fn promote_atom_to_entity(
    store: &dyn Store,
    atom_id: &str,
    class_id: &str,
    actor: &str,
) -> Result<OntologyEntity> {
    let atom = store.get(atom_id).await?.ok_or_else(|| {
        KurultaiError::Store(format!("ontology_promote: atom {atom_id} not found"))
    })?;
    let class = store.get_ontology_entity(class_id).await?.ok_or_else(|| {
        KurultaiError::Store(format!("ontology_promote: class {class_id} not found"))
    })?;
    if class.kind != "class" {
        return Err(KurultaiError::Store(format!(
            "ontology_promote: {class_id} is kind={}, expected class",
            class.kind
        )));
    }

    let entity_id = format!("ent:{atom_id}");
    let entity = match store.get_ontology_entity(&entity_id).await? {
        Some(existing) => existing,
        None => {
            let created = OntologyEntity {
                id: entity_id.clone(),
                kind: "instance".into(),
                name: atom.title.clone(),
                atom_id: Some(atom.id.clone()),
                attributes: serde_json::json!({}),
            };
            store.upsert_ontology_entity(&created).await?;
            created
        }
    };

    let link = OntologyLink {
        id: format!("link:{entity_id}:instance_of:{class_id}"),
        from_id: entity.id.clone(),
        to_id: class_id.to_string(),
        rel: OntologyLinkType::InstanceOf,
        confidence: 1.0,
        status: "approved".into(),
        actor: actor.to_string(),
    };
    store.upsert_ontology_link(&link).await?;
    Ok(entity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{migrations, SearchFilter, SqliteVecStore};
    use crate::types::{KnowledgeAtom, OntologyLinkType};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_store() -> SqliteVecStore {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "kurultai-ontology-{}-{}-{}",
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
            title: "Fixture note".into(),
            summary: "KNOWN_PHRASE_ONTOLOGY_42".into(),
            content: "KNOWN_PHRASE_ONTOLOGY_42 body".into(),
            question: None,
            resolution: None,
            tags: vec!["test".into()],
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: None,
            metadata: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn seed_classes_and_is_a_links() {
        let store = temp_store();
        assert_eq!(migrations::CURRENT_SCHEMA_VERSION, 9);

        let entities = store.list_ontology_entities(50).await.unwrap();
        let ids: Vec<&str> = entities.iter().map(|e| e.id.as_str()).collect();
        for class_id in SEEDED_CLASS_IDS {
            assert!(ids.contains(class_id), "missing {class_id}");
        }
        assert_eq!(entities.iter().filter(|e| e.kind == "class").count(), 6);

        let links = store.list_ontology_links(None).await.unwrap();
        let is_a: Vec<_> = links
            .iter()
            .filter(|l| l.rel == OntologyLinkType::IsA)
            .collect();
        assert_eq!(is_a.len(), 5);
        for child in [
            CLASS_NOTE,
            CLASS_CODE,
            CLASS_DECISION,
            CLASS_PERSON,
            CLASS_SYSTEM,
        ] {
            assert!(is_a
                .iter()
                .any(|l| l.from_id == child && l.to_id == CLASS_MEMORY));
        }
    }

    #[tokio::test]
    async fn promote_creates_entity_and_keeps_atom() {
        let store = temp_store();
        let atom = sample_atom("note-1");
        store.upsert(&atom).await.unwrap();

        let entity = promote_atom_to_entity(&store, "note-1", CLASS_NOTE, "test")
            .await
            .unwrap();
        assert_eq!(entity.id, "ent:note-1");
        assert_eq!(entity.kind, "instance");
        assert_eq!(entity.atom_id.as_deref(), Some("note-1"));

        let got = store
            .get_ontology_entity("ent:note-1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got, entity);

        let links = store.list_ontology_links(Some("ent:note-1")).await.unwrap();
        assert!(links
            .iter()
            .any(|l| { l.rel == OntologyLinkType::InstanceOf && l.to_id == CLASS_NOTE }));
        let by_atom = store.list_ontology_links(Some("note-1")).await.unwrap();
        assert!(by_atom
            .iter()
            .any(|l| l.rel == OntologyLinkType::InstanceOf));

        assert!(store.get("note-1").await.unwrap().is_some());
        let hits = store
            .fts_search("KNOWN_PHRASE_ONTOLOGY_42", 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(hits.iter().any(|(a, _)| a.id == "note-1"));
    }

    #[tokio::test]
    async fn promote_missing_class_writes_nothing() {
        let store = temp_store();
        store.upsert(&sample_atom("note-2")).await.unwrap();
        let err = promote_atom_to_entity(&store, "note-2", "class:nope", "test")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("class:nope"));
        assert!(store
            .get_ontology_entity("ent:note-2")
            .await
            .unwrap()
            .is_none());
    }

    #[test]
    fn parse_unknown_rel_is_none() {
        assert_eq!(OntologyLinkType::parse("nope"), None);
        assert_eq!(OntologyLinkType::parse("is_a"), Some(OntologyLinkType::IsA));
    }
}
