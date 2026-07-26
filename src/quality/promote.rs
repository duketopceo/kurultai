//! Explicit quarantine → trusted transition with audit log.

use crate::error::{KurultaiError, Result};
use crate::quality::gate::{apply_gate, evaluate, GateOutcome};
use crate::store::Store;
use crate::types::TrustLane;

#[derive(Debug, Clone)]
pub struct PromoteResult {
    pub atom_id: String,
    pub actor: String,
}

/// Promote a quarantined atom after re-running the gate. Never a side effect of remember.
pub async fn promote_atom(
    store: &dyn Store,
    atom_id: &str,
    actor: &str,
    reason_note: Option<&str>,
) -> Result<PromoteResult> {
    let Some(mut atom) = store.get(atom_id).await? else {
        return Err(KurultaiError::Store(format!("atom not found: {atom_id}")));
    };

    if atom.trust_lane != TrustLane::Quarantine {
        return Err(KurultaiError::config(format!(
            "atom {atom_id} is not in quarantine (lane={})",
            atom.trust_lane.as_str()
        )));
    }

    let outcome = evaluate(store, &atom).await?;
    match outcome {
        GateOutcome::Trusted => {
            apply_gate(&mut atom, GateOutcome::Trusted);
            store
                .set_trust_lane(atom_id, TrustLane::Trusted, None)
                .await?;
            let mut detail = serde_json::json!({ "from": "quarantine", "to": "trusted" });
            if let Some(note) = reason_note.filter(|s| !s.trim().is_empty()) {
                detail["note"] = serde_json::Value::String(note.to_string());
            }
            store
                .insert_quality_audit("promote", atom_id, actor, &detail)
                .await?;
            Ok(PromoteResult {
                atom_id: atom_id.to_string(),
                actor: actor.to_string(),
            })
        }
        GateOutcome::Quarantine { reason } => Err(KurultaiError::config(format!(
            "promote refused: gate still fails ({reason})"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::gate::{apply_gate, GateOutcome};
    use crate::store::SqliteVecStore;
    use crate::types::KnowledgeAtom;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn temp_store() -> Arc<SqliteVecStore> {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-promote-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap())
    }

    fn sample(id: &str, tags: Vec<&str>, content: &str) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "agent".into(),
            source_id: format!("/{id}"),
            title: "t".into(),
            summary: content.into(),
            content: content.into(),
            tags: tags.into_iter().map(str::to_string).collect(),
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn promote_untagged_errors() {
        let store = temp_store();
        let mut a = sample("p1", vec![], "body");
        apply_gate(
            &mut a,
            GateOutcome::Quarantine {
                reason: "untagged".into(),
            },
        );
        store.upsert(&a).await.unwrap();
        let err = promote_atom(store.as_ref() as &dyn Store, "p1", "test", None)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("untagged"));
    }

    #[tokio::test]
    async fn promote_after_tags_succeeds() {
        let store = temp_store();
        let mut a = sample("p2", vec![], "unique promote body");
        apply_gate(
            &mut a,
            GateOutcome::Quarantine {
                reason: "untagged".into(),
            },
        );
        store.upsert(&a).await.unwrap();

        a.tags = vec!["ops".into()];
        store.upsert(&a).await.unwrap();
        // still quarantine until promote
        store
            .set_trust_lane("p2", TrustLane::Quarantine, Some("untagged"))
            .await
            .unwrap();

        let res = promote_atom(
            store.as_ref() as &dyn Store,
            "p2",
            "cli",
            Some("fixed tags"),
        )
        .await
        .unwrap();
        assert_eq!(res.atom_id, "p2");
        let loaded = store.get("p2").await.unwrap().unwrap();
        assert_eq!(loaded.trust_lane, TrustLane::Trusted);
        assert!(loaded.quarantine_reason.is_none());
    }
}
