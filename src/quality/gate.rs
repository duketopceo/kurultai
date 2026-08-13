//! Synchronous write barrier: tags + exact trusted content_hash duplicate + cheap heuristics.

use crate::error::Result;
use crate::hashutil::sha256_hex;
use crate::store::Store;
use crate::types::{KnowledgeAtom, TrustLane};

/// Minimum trimmed content length for trusted lane (characters).
const MIN_TRUSTED_CHARS: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateOutcome {
    Trusted,
    Quarantine { reason: String },
}

/// True when at least one tag is non-empty after trim.
pub fn has_non_empty_tags(tags: &[String]) -> bool {
    tags.iter().any(|t| !t.trim().is_empty())
}

/// Cheap thin/boilerplate detector (not an LLM judge).
///
/// Runs after the length gate; does not re-check short char counts.
fn is_thin_or_boilerplate(content: &str) -> bool {
    let trimmed = content.trim();
    let lower = trimmed.to_ascii_lowercase();
    // Common dump boilerplate / placeholder patterns.
    const BOILERPLATE: &[&str] = &[
        "lorem ipsum",
        "todo: replace",
        "placeholder",
        "test test test",
        "asdfasdf",
        "xxx xxx xxx",
    ];
    if BOILERPLATE.iter().any(|p| lower.contains(p)) {
        return true;
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    // Extremely low lexical diversity (e.g. same word repeated).
    let unique: std::collections::HashSet<&str> = words.iter().copied().collect();
    if words.len() >= 12 && unique.len() * 4 <= words.len() {
        return true;
    }
    false
}

/// Evaluate the sync quality gate (no embed / near-dupe / LLM).
pub async fn evaluate(store: &dyn Store, atom: &KnowledgeAtom) -> Result<GateOutcome> {
    if !has_non_empty_tags(&atom.tags) {
        return Ok(GateOutcome::Quarantine {
            reason: "untagged".into(),
        });
    }

    let hash = sha256_hex(&atom.content);
    if let Some(existing) = store.find_trusted_by_content_hash(&hash).await? {
        if existing != atom.id {
            return Ok(GateOutcome::Quarantine {
                reason: format!("exact_duplicate:{existing}"),
            });
        }
    }

    let trimmed_len = atom.content.trim().chars().count();
    if trimmed_len < MIN_TRUSTED_CHARS {
        return Ok(GateOutcome::Quarantine {
            reason: "low_quality:too_short".into(),
        });
    }

    if is_thin_or_boilerplate(&atom.content) {
        return Ok(GateOutcome::Quarantine {
            reason: "low_quality:thin".into(),
        });
    }

    Ok(GateOutcome::Trusted)
}

/// Apply gate outcome onto atom fields (does not persist).
pub fn apply_gate(atom: &mut KnowledgeAtom, outcome: GateOutcome) {
    match outcome {
        GateOutcome::Trusted => {
            atom.trust_lane = TrustLane::Trusted;
            atom.quarantine_reason = None;
        }
        GateOutcome::Quarantine { reason } => {
            atom.trust_lane = TrustLane::Quarantine;
            atom.quarantine_reason = Some(reason);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::NullEmbedder;
    use crate::store::SqliteVecStore;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn atom(id: &str, content: &str, tags: Vec<&str>) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "agent".into(),
            source_id: format!("/{id}"),
            title: "t".into(),
            summary: content.chars().take(80).collect(),
            content: content.into(),
            tags: tags.into_iter().map(str::to_string).collect(),
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            ..Default::default()
        }
    }

    fn long_body(seed: &str) -> String {
        format!(
            "{seed} — detailed operational notes about deployments, migrations, \
             rollbacks, checksums, and verification steps for the production cluster."
        )
    }

    fn temp_store() -> Arc<SqliteVecStore> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "kurultai-gate-{}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Arc::new(SqliteVecStore::open(dir.join("store.db"), 4).unwrap())
    }

    #[tokio::test]
    async fn untagged_quarantines() {
        let store = temp_store();
        let a = atom("a1", &long_body("hello world unique"), vec![]);
        let out = evaluate(store.as_ref() as &dyn Store, &a).await.unwrap();
        assert_eq!(
            out,
            GateOutcome::Quarantine {
                reason: "untagged".into()
            }
        );
    }

    #[tokio::test]
    async fn soft_labels_alone_do_not_satisfy_tag_gate() {
        use crate::types::SoftLabel;
        let store = temp_store();
        let mut a = atom("soft-only", &long_body("soft only body"), vec![]);
        a.soft_labels = vec![SoftLabel {
            label_id: 0,
            name: "infra".into(),
            score: 0.99,
            aliases: vec![],
        }];
        let out = evaluate(store.as_ref() as &dyn Store, &a).await.unwrap();
        assert_eq!(
            out,
            GateOutcome::Quarantine {
                reason: "untagged".into()
            }
        );
    }

    #[tokio::test]
    async fn tagged_unique_is_trusted() {
        let store = temp_store();
        let a = atom("a1", &long_body("hello world unique"), vec!["ops"]);
        let out = evaluate(store.as_ref() as &dyn Store, &a).await.unwrap();
        assert_eq!(out, GateOutcome::Trusted);
    }

    #[tokio::test]
    async fn exact_duplicate_of_trusted_quarantines() {
        let store = temp_store();
        let body = long_body("same body");
        let mut first = atom("a1", &body, vec!["ops"]);
        apply_gate(&mut first, GateOutcome::Trusted);
        store.upsert(&first).await.unwrap();

        let second = atom("a2", &body, vec!["ops"]);
        let out = evaluate(store.as_ref() as &dyn Store, &second)
            .await
            .unwrap();
        match out {
            GateOutcome::Quarantine { reason } => {
                assert!(reason.starts_with("exact_duplicate:a1"));
            }
            GateOutcome::Trusted => panic!("expected quarantine"),
        }
    }

    #[tokio::test]
    async fn same_id_reupsert_stays_trusted() {
        let store = temp_store();
        let body = long_body("stable");
        let mut first = atom("a1", &body, vec!["ops"]);
        apply_gate(&mut first, GateOutcome::Trusted);
        store.upsert(&first).await.unwrap();

        let again = atom("a1", &body, vec!["ops"]);
        let out = evaluate(store.as_ref() as &dyn Store, &again)
            .await
            .unwrap();
        assert_eq!(out, GateOutcome::Trusted);
    }

    #[tokio::test]
    async fn too_short_quarantines() {
        let store = temp_store();
        let a = atom("short", "tiny tagged note", vec!["ops"]);
        let out = evaluate(store.as_ref() as &dyn Store, &a).await.unwrap();
        assert_eq!(
            out,
            GateOutcome::Quarantine {
                reason: "low_quality:too_short".into()
            }
        );
    }

    #[tokio::test]
    async fn thin_boilerplate_quarantines() {
        let store = temp_store();
        // Long enough to pass too_short, but boilerplate + low diversity.
        let body = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt";
        let a = atom("thin", body, vec!["ops"]);
        let out = evaluate(store.as_ref() as &dyn Store, &a).await.unwrap();
        assert_eq!(
            out,
            GateOutcome::Quarantine {
                reason: "low_quality:thin".into()
            }
        );
    }

    #[tokio::test]
    async fn gate_does_not_call_embedder() {
        let _ = NullEmbedder::new(4);
        let store = temp_store();
        let a = atom("a1", &long_body("no embed"), vec!["t"]);
        let _ = evaluate(store.as_ref() as &dyn Store, &a).await.unwrap();
    }
}
