//! O1 labeled property graph beside `KnowledgeAtom` (Wave E / #116).
//!
//! Promote-to-entity does not delete or un-index the atom. O3 (#118) adds the
//! proposal queue: agents submit drafts, humans approve/reject. Nothing in
//! this module mutates entities or links without an explicit decide.

use crate::error::{KurultaiError, Result};
use crate::store::Store;
use crate::types::{OntologyEntity, OntologyLink, OntologyLinkType, OntologyProposal};

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

// ── O3: proposal queue (#118) ────────────────────────────────────────────────

pub const PROPOSAL_PENDING: &str = "pending";
pub const PROPOSAL_APPROVED: &str = "approved";
pub const PROPOSAL_REJECTED: &str = "rejected";

pub const PROPOSAL_PROMOTE_ATOM: &str = "promote_atom";
pub const PROPOSAL_NEW_LINK: &str = "new_link";
pub const PROPOSAL_NEW_ENTITY: &str = "new_entity";

pub const PROPOSAL_KINDS: &[&str] = &[
    PROPOSAL_PROMOTE_ATOM,
    PROPOSAL_NEW_LINK,
    PROPOSAL_NEW_ENTITY,
];

fn now_sql() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn payload_str<'a>(payload: &'a serde_json::Value, key: &str) -> Result<&'a str> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| KurultaiError::Store(format!("ontology_propose: payload.{key} required")))
}

/// Slugify a display name into an entity-id suffix (`"Vector DB"` → `vector-db`).
fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

/// Validate a proposal payload against current ontology state. Pure read —
/// returns a normalized payload for storage.
async fn validate_proposal(
    store: &dyn Store,
    kind: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value> {
    match kind {
        PROPOSAL_PROMOTE_ATOM => {
            let atom_id = payload_str(payload, "atom_id")?;
            let class_id = payload_str(payload, "class_id")?;
            store.get(atom_id).await?.ok_or_else(|| {
                KurultaiError::Store(format!("ontology_propose: atom {atom_id} not found"))
            })?;
            let class = store.get_ontology_entity(class_id).await?.ok_or_else(|| {
                KurultaiError::Store(format!("ontology_propose: class {class_id} not found"))
            })?;
            if class.kind != "class" {
                return Err(KurultaiError::Store(format!(
                    "ontology_propose: {class_id} is kind={}, expected class",
                    class.kind
                )));
            }
            Ok(serde_json::json!({ "atom_id": atom_id, "class_id": class_id }))
        }
        PROPOSAL_NEW_LINK => {
            let from_id = payload_str(payload, "from_id")?;
            let to_id = payload_str(payload, "to_id")?;
            let rel = payload_str(payload, "rel")?;
            if OntologyLinkType::parse(rel).is_none() {
                return Err(KurultaiError::Store(format!(
                    "ontology_propose: unknown rel {rel}"
                )));
            }
            for id in [from_id, to_id] {
                if store.get_ontology_entity(id).await?.is_none() {
                    return Err(KurultaiError::Store(format!(
                        "ontology_propose: entity {id} not found"
                    )));
                }
            }
            let confidence = payload
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            let exists = store
                .list_ontology_links(Some(from_id))
                .await?
                .iter()
                .any(|l| l.from_id == from_id && l.to_id == to_id && l.rel.as_str() == rel);
            if exists {
                return Err(KurultaiError::Store(format!(
                    "ontology_propose: link {from_id} -{rel}-> {to_id} already exists"
                )));
            }
            Ok(serde_json::json!({
                "from_id": from_id,
                "to_id": to_id,
                "rel": rel,
                "confidence": confidence,
            }))
        }
        PROPOSAL_NEW_ENTITY => {
            let kind_field = payload_str(payload, "entity_kind")?;
            if !matches!(kind_field, "class" | "instance" | "metric") {
                return Err(KurultaiError::Store(format!(
                    "ontology_propose: entity_kind must be class|instance|metric, got {kind_field}"
                )));
            }
            let name = payload_str(payload, "name")?;
            let prefix = match kind_field {
                "class" => "class",
                "metric" => "metric",
                _ => "ent",
            };
            let id = payload
                .get("id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("{prefix}:{}", slugify(name)));
            if id.is_empty() || store.get_ontology_entity(&id).await?.is_some() {
                return Err(KurultaiError::Store(format!(
                    "ontology_propose: entity id {id} empty or already exists"
                )));
            }
            let atom_id = payload.get("atom_id").and_then(|v| v.as_str());
            if let Some(aid) = atom_id {
                if store.get(aid).await?.is_none() {
                    return Err(KurultaiError::Store(format!(
                        "ontology_propose: atom {aid} not found"
                    )));
                }
            }
            let attributes = payload
                .get("attributes")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            Ok(serde_json::json!({
                "id": id,
                "entity_kind": kind_field,
                "name": name,
                "atom_id": atom_id,
                "attributes": attributes,
            }))
        }
        other => Err(KurultaiError::Store(format!(
            "ontology_propose: unknown kind {other} (expected one of {PROPOSAL_KINDS:?})"
        ))),
    }
}

/// Submit a draft ontology mutation. Validates references, dedupes against the
/// pending queue, and inserts a `pending` row. Never mutates entities/links.
pub async fn submit_proposal(
    store: &dyn Store,
    kind: &str,
    payload: serde_json::Value,
    proposed_by: &str,
    reason: Option<String>,
) -> Result<OntologyProposal> {
    if proposed_by.trim().is_empty() {
        return Err(KurultaiError::Store(
            "ontology_propose: proposed_by required".into(),
        ));
    }
    if let Some(r) = &reason {
        if r.chars().count() > 500 {
            return Err(KurultaiError::Store(
                "ontology_propose: reason must be at most 500 characters".into(),
            ));
        }
    }
    let normalized = validate_proposal(store, kind, &payload).await?;

    // Cheap spam guard: an identical pending proposal is a no-op submit.
    for p in store
        .list_ontology_proposals(Some(PROPOSAL_PENDING), 500)
        .await?
    {
        if p.kind == kind && p.payload == normalized {
            return Err(KurultaiError::Store(format!(
                "ontology_propose: duplicate of pending proposal {}",
                p.id
            )));
        }
    }

    let proposal = OntologyProposal {
        id: format!("prop:{}", uuid::Uuid::new_v4()),
        kind: kind.to_string(),
        payload: normalized,
        status: PROPOSAL_PENDING.into(),
        proposed_by: proposed_by.to_string(),
        reason,
        created_at: now_sql(),
        decided_by: None,
        decided_at: None,
    };
    store.insert_ontology_proposal(&proposal).await?;
    Ok(proposal)
}

/// Apply an approved proposal's mutation. Idempotent — entity/link upserts
/// make a repeated approve safe.
async fn apply_proposal(store: &dyn Store, p: &OntologyProposal) -> Result<()> {
    match p.kind.as_str() {
        PROPOSAL_PROMOTE_ATOM => {
            promote_atom_to_entity(
                store,
                payload_str(&p.payload, "atom_id")?,
                payload_str(&p.payload, "class_id")?,
                &p.proposed_by,
            )
            .await?;
        }
        PROPOSAL_NEW_LINK => {
            let from_id = payload_str(&p.payload, "from_id")?;
            let to_id = payload_str(&p.payload, "to_id")?;
            let rel_str = payload_str(&p.payload, "rel")?;
            let rel = OntologyLinkType::parse(rel_str).ok_or_else(|| {
                KurultaiError::Store(format!("ontology_decide: unknown rel {rel_str}"))
            })?;
            let confidence = p
                .payload
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            store
                .upsert_ontology_link(&OntologyLink {
                    id: format!("link:{from_id}:{rel_str}:{to_id}"),
                    from_id: from_id.to_string(),
                    to_id: to_id.to_string(),
                    rel,
                    confidence,
                    status: "approved".into(),
                    actor: p.proposed_by.clone(),
                })
                .await?;
        }
        PROPOSAL_NEW_ENTITY => {
            let atom_id = p
                .payload
                .get("atom_id")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            store
                .upsert_ontology_entity(&OntologyEntity {
                    id: payload_str(&p.payload, "id")?.to_string(),
                    kind: payload_str(&p.payload, "entity_kind")?.to_string(),
                    name: payload_str(&p.payload, "name")?.to_string(),
                    atom_id,
                    attributes: p
                        .payload
                        .get("attributes")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                })
                .await?;
        }
        other => {
            return Err(KurultaiError::Store(format!(
                "ontology_decide: unknown kind {other}"
            )))
        }
    }
    Ok(())
}

/// Human decision on a pending proposal. Approval applies the mutation first,
/// then records the decision — a failed apply leaves the proposal pending so
/// the queue stays truthful. Rejection mutates nothing.
pub async fn decide_proposal(
    store: &dyn Store,
    id: &str,
    approve: bool,
    decided_by: &str,
) -> Result<OntologyProposal> {
    if decided_by.trim().is_empty() {
        return Err(KurultaiError::Store(
            "ontology_decide: decided_by required".into(),
        ));
    }
    let proposal = store
        .get_ontology_proposal(id)
        .await?
        .ok_or_else(|| KurultaiError::Store(format!("ontology_decide: proposal {id} not found")))?;
    if proposal.status != PROPOSAL_PENDING {
        return Err(KurultaiError::Store(format!(
            "ontology_decide: proposal {id} already decided ({})",
            proposal.status
        )));
    }
    if approve {
        apply_proposal(store, &proposal).await?;
    }
    let status = if approve {
        PROPOSAL_APPROVED
    } else {
        PROPOSAL_REJECTED
    };
    store
        .decide_ontology_proposal(id, status, decided_by, &now_sql())
        .await
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
        assert_eq!(migrations::CURRENT_SCHEMA_VERSION, 15);

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

    // ── O3 proposal queue ────────────────────────────────────────────────────

    #[tokio::test]
    async fn propose_promote_is_pending_and_mutates_nothing() {
        let store = temp_store();
        store.upsert(&sample_atom("prop-atom")).await.unwrap();

        let p = submit_proposal(
            &store,
            PROPOSAL_PROMOTE_ATOM,
            serde_json::json!({"atom_id": "prop-atom", "class_id": CLASS_NOTE}),
            "agent:claude",
            Some("looks like a note".into()),
        )
        .await
        .unwrap();

        assert_eq!(p.status, PROPOSAL_PENDING);
        assert_eq!(p.proposed_by, "agent:claude");
        assert!(p.decided_by.is_none());
        // Draft only — no entity or link exists yet.
        assert!(store
            .get_ontology_entity("ent:prop-atom")
            .await
            .unwrap()
            .is_none());
        assert!(store
            .list_ontology_links(Some("ent:prop-atom"))
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn approve_promote_applies_and_records_decision() {
        let store = temp_store();
        store.upsert(&sample_atom("prop-atom2")).await.unwrap();
        let p = submit_proposal(
            &store,
            PROPOSAL_PROMOTE_ATOM,
            serde_json::json!({"atom_id": "prop-atom2", "class_id": CLASS_NOTE}),
            "agent:claude",
            None,
        )
        .await
        .unwrap();

        let decided = decide_proposal(&store, &p.id, true, "human:luke")
            .await
            .unwrap();
        assert_eq!(decided.status, PROPOSAL_APPROVED);
        assert_eq!(decided.decided_by.as_deref(), Some("human:luke"));
        assert!(decided.decided_at.is_some());

        let entity = store
            .get_ontology_entity("ent:prop-atom2")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(entity.atom_id.as_deref(), Some("prop-atom2"));
        let links = store
            .list_ontology_links(Some("ent:prop-atom2"))
            .await
            .unwrap();
        assert!(links.iter().any(|l| l.rel == OntologyLinkType::InstanceOf
            && l.to_id == CLASS_NOTE
            && l.actor == "agent:claude"));

        // A second decide must fail — already decided.
        let err = decide_proposal(&store, &p.id, true, "human:luke")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("already decided"));
    }

    #[tokio::test]
    async fn reject_mutates_nothing() {
        let store = temp_store();
        store.upsert(&sample_atom("prop-atom3")).await.unwrap();
        let p = submit_proposal(
            &store,
            PROPOSAL_PROMOTE_ATOM,
            serde_json::json!({"atom_id": "prop-atom3", "class_id": CLASS_NOTE}),
            "agent:codex",
            None,
        )
        .await
        .unwrap();

        let decided = decide_proposal(&store, &p.id, false, "human:luke")
            .await
            .unwrap();
        assert_eq!(decided.status, PROPOSAL_REJECTED);
        assert!(store
            .get_ontology_entity("ent:prop-atom3")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn propose_new_link_applies_on_approve() {
        let store = temp_store();
        for (id, kind, name) in [
            ("ent:a", "instance", "A"),
            ("class:custom", "class", "Custom"),
        ] {
            store
                .upsert_ontology_entity(&OntologyEntity {
                    id: id.into(),
                    kind: kind.into(),
                    name: name.into(),
                    atom_id: None,
                    attributes: serde_json::json!({}),
                })
                .await
                .unwrap();
        }
        let p = submit_proposal(
            &store,
            PROPOSAL_NEW_LINK,
            serde_json::json!({
                "from_id": "ent:a",
                "to_id": "class:custom",
                "rel": "instance_of",
                "confidence": 0.8,
            }),
            "agent:claude",
            None,
        )
        .await
        .unwrap();

        decide_proposal(&store, &p.id, true, "human:luke")
            .await
            .unwrap();
        let links = store.list_ontology_links(Some("ent:a")).await.unwrap();
        let link = links
            .iter()
            .find(|l| l.to_id == "class:custom")
            .expect("link applied");
        assert_eq!(link.rel, OntologyLinkType::InstanceOf);
        assert!((link.confidence - 0.8).abs() < 1e-6);
        assert_eq!(link.status, "approved");
        assert_eq!(link.actor, "agent:claude");
    }

    #[tokio::test]
    async fn propose_new_entity_slugs_id_and_applies() {
        let store = temp_store();
        let p = submit_proposal(
            &store,
            PROPOSAL_NEW_ENTITY,
            serde_json::json!({"entity_kind": "class", "name": "Vector DB"}),
            "agent:hermes",
            None,
        )
        .await
        .unwrap();
        assert_eq!(p.payload["id"], "class:vector-db");

        decide_proposal(&store, &p.id, true, "human:luke")
            .await
            .unwrap();
        let e = store
            .get_ontology_entity("class:vector-db")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(e.kind, "class");
        assert_eq!(e.name, "Vector DB");
    }

    #[tokio::test]
    async fn invalid_proposals_write_nothing() {
        let store = temp_store();
        store.upsert(&sample_atom("prop-atom4")).await.unwrap();

        // Unknown kind.
        assert!(
            submit_proposal(&store, "bogus", serde_json::json!({}), "agent:x", None)
                .await
                .is_err()
        );
        // Missing atom.
        assert!(submit_proposal(
            &store,
            PROPOSAL_PROMOTE_ATOM,
            serde_json::json!({"atom_id": "ghost", "class_id": CLASS_NOTE}),
            "agent:x",
            None
        )
        .await
        .is_err());
        // Non-class target.
        assert!(submit_proposal(
            &store,
            PROPOSAL_PROMOTE_ATOM,
            serde_json::json!({"atom_id": "prop-atom4", "class_id": "ent:other"}),
            "agent:x",
            None
        )
        .await
        .is_err());
        // Unknown rel.
        assert!(submit_proposal(
            &store,
            PROPOSAL_NEW_LINK,
            serde_json::json!({
                "from_id": CLASS_NOTE, "to_id": CLASS_MEMORY, "rel": "nope"
            }),
            "agent:x",
            None
        )
        .await
        .is_err());

        assert!(store
            .list_ontology_proposals(None, 10)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn duplicate_pending_proposal_rejected() {
        let store = temp_store();
        store.upsert(&sample_atom("prop-atom5")).await.unwrap();
        let payload = serde_json::json!({"atom_id": "prop-atom5", "class_id": CLASS_NOTE});
        submit_proposal(
            &store,
            PROPOSAL_PROMOTE_ATOM,
            payload.clone(),
            "agent:a",
            None,
        )
        .await
        .unwrap();
        let err = submit_proposal(&store, PROPOSAL_PROMOTE_ATOM, payload, "agent:b", None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }
}
