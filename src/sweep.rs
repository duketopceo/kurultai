//! Consolidation sweep (competitive sweep U5) — "sleep-time compute".
//!
//! One pass over the store that keeps the brain tidy without an LLM:
//!
//! 1. **Dedupe** — trusted atoms sharing an identical `content_hash` are
//!    folded into the earliest-indexed copy via `mark_superseded` (non-
//!    destructive; they remain reachable with `--include-superseded`).
//! 2. **Stale ontology repair** — links whose endpoints vanished and
//!    entities whose backing atom was deleted are pruned.
//! 3. **Tier + lane census** — hot/warm/cold counts and quarantine depth
//!    are reported (tiers are derived, so "decay" is automatic; we surface
//!    the distribution rather than mutate timestamps).
//! 4. **Hey report** — optionally posts the summary to the `kurultai-sweep`
//!    thread so agents see what the night shift did.

use crate::error::Result;
use crate::memory::TierPolicy;
use crate::store::{PostMessageInput, SearchFilter, Store};
use crate::types::TrustLane;
use chrono::Utc;

/// What a sweep pass changed / observed.
#[derive(Debug, Default, Clone)]
pub struct SweepReport {
    /// Content-hash groups containing duplicates.
    pub dup_groups: usize,
    /// Atoms marked superseded by the group's canonical survivor.
    pub dup_atoms_merged: usize,
    /// Ontology links with a missing endpoint that were deleted.
    pub links_pruned: u64,
    /// Ontology entities whose backing atom vanished that were deleted.
    pub entities_pruned: u64,
    /// Hot / warm / cold atom counts after the pass.
    pub tiers: (u64, u64, u64),
    /// Atoms still sitting in quarantine.
    pub quarantined: u64,
    /// Whether this was a dry run (nothing mutated).
    pub dry_run: bool,
}

impl SweepReport {
    /// Human-readable summary, also used as the Hey post body.
    pub fn summary(&self) -> String {
        format!(
            "{}sweep: {} dup group(s), {} atom(s) folded into canonical copies; \
             {} stale link(s), {} orphan entity(ies) pruned; \
             tiers hot={} warm={} cold={}; {} atom(s) in quarantine",
            if self.dry_run { "[dry-run] " } else { "" },
            self.dup_groups,
            self.dup_atoms_merged,
            self.links_pruned,
            self.entities_pruned,
            self.tiers.0,
            self.tiers.1,
            self.tiers.2,
            self.quarantined,
        )
    }
}

/// Run one consolidation pass. `dry_run` computes counts without mutating.
/// `post_report` publishes the summary into Hey thread `kurultai-sweep`.
pub async fn run(
    store: &dyn Store,
    policy: &TierPolicy,
    dry_run: bool,
    post_report: bool,
) -> Result<SweepReport> {
    let mut report = SweepReport {
        dry_run,
        ..Default::default()
    };

    // 1. Dedupe: fold every non-canonical member into the earliest-indexed
    //    copy of its content-hash group via the U4 supersede machinery.
    let groups = store.duplicate_content_groups().await?;
    report.dup_groups = groups.len();
    for (_, ids) in &groups {
        let (canonical, dups) = ids.split_first().unwrap();
        report.dup_atoms_merged += dups.len();
        if !dry_run {
            store.mark_superseded(dups, canonical, Utc::now()).await?;
        }
    }

    // 2. Stale ontology links + orphan entities.
    let (links, entities) = if dry_run {
        count_stale_ontology(store).await?
    } else {
        store.prune_stale_ontology().await?
    };
    report.links_pruned = links;
    report.entities_pruned = entities;

    // 3. Census: tier distribution + quarantine depth.
    report.tiers = store.count_by_tier(policy.clone()).await?;
    let all = SearchFilter {
        trusted_only: false,
        ..Default::default()
    };
    report.quarantined = store
        .list_atoms(100_000, all)
        .await?
        .iter()
        .filter(|a| a.trust_lane == TrustLane::Quarantine)
        .count() as u64;

    // 4. Hey report — agent_id is the agent uuid; reuse the registered
    //    `kurultai` agent or register it on first sweep.
    if post_report {
        let agent_id = match store
            .list_agents()
            .await?
            .into_iter()
            .find(|a| a.codename.eq_ignore_ascii_case("kurultai"))
        {
            Some(a) => a.id,
            None => store.register_agent("kurultai").await?.0,
        };
        // Thread must exist before posting (messages.thread_id FK).
        let thread_id = match store.get_thread_by_name("kurultai-sweep").await? {
            Some(t) => t.id,
            None => store.create_thread("kurultai-sweep", None, None).await?.id,
        };
        store
            .post_message(&PostMessageInput {
                thread_id,
                agent_id,
                parent_id: None,
                content: report.summary(),
                request_reply: false,
                repo: None,
                instance_id: None,
            })
            .await?;
    }

    Ok(report)
}

/// Dry-run equivalent of `prune_stale_ontology` — counts without deleting.
async fn count_stale_ontology(store: &dyn Store) -> Result<(u64, u64)> {
    let links = store.list_ontology_links(None).await?;
    let mut stale_links = 0u64;
    for l in &links {
        if store.get_ontology_entity(&l.from_id).await?.is_none()
            || store.get_ontology_entity(&l.to_id).await?.is_none()
        {
            stale_links += 1;
        }
    }
    let entities = store.list_ontology_entities(100_000).await?;
    let mut orphans = 0u64;
    for e in &entities {
        if let Some(atom_id) = &e.atom_id {
            if store.get(atom_id).await?.is_none() {
                orphans += 1;
            }
        }
    }
    Ok((stale_links, orphans))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::SqliteVecStore;
    use crate::types::KnowledgeAtom;

    fn atom(id: &str, content: &str) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "test".into(),
            source_id: format!("sid-{id}"),
            content: content.into(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn dedupe_marks_later_copies_superseded() {
        let store = SqliteVecStore::open(std::path::PathBuf::from(":memory:"), 4).unwrap();
        let mut a = atom("a1", "same body");
        a.indexed_at = Utc::now() - chrono::Duration::days(2);
        let mut b = atom("a2", "same body");
        b.indexed_at = Utc::now() - chrono::Duration::days(1);
        let c = atom("a3", "different");
        store.upsert_batch(&[a, b, c]).await.unwrap();

        let report = run(&store, &TierPolicy::default(), false, false)
            .await
            .unwrap();
        assert_eq!(report.dup_groups, 1);
        assert_eq!(report.dup_atoms_merged, 1);
        let kept = store.get("a1").await.unwrap().unwrap();
        let folded = store.get("a2").await.unwrap().unwrap();
        assert!(kept.superseded_at.is_none());
        assert_eq!(folded.superseded_by.as_deref(), Some("a1"));
    }

    #[tokio::test]
    async fn dry_run_counts_without_mutating() {
        let store = SqliteVecStore::open(std::path::PathBuf::from(":memory:"), 4).unwrap();
        let mut a = atom("a1", "same body");
        a.indexed_at = Utc::now() - chrono::Duration::days(2);
        let b = atom("a2", "same body");
        store.upsert_batch(&[a, b]).await.unwrap();

        let report = run(&store, &TierPolicy::default(), true, false)
            .await
            .unwrap();
        assert_eq!(report.dup_atoms_merged, 1);
        assert!(store
            .get("a2")
            .await
            .unwrap()
            .unwrap()
            .superseded_at
            .is_none());
    }

    #[tokio::test]
    async fn prune_removes_links_with_missing_endpoints() {
        let store = SqliteVecStore::open(std::path::PathBuf::from(":memory:"), 4).unwrap();
        store
            .upsert_ontology_entity(&crate::types::OntologyEntity {
                id: "ent:live".into(),
                kind: "instance".into(),
                name: "live".into(),
                atom_id: None,
                attributes: serde_json::json!({}),
            })
            .await
            .unwrap();
        store
            .upsert_ontology_link(&crate::types::OntologyLink {
                id: "lnk:dangling".into(),
                from_id: "ent:live".into(),
                to_id: "ent:gone".into(),
                rel: crate::types::OntologyLinkType::References,
                confidence: 1.0,
                status: "approved".into(),
                actor: "test".into(),
            })
            .await
            .unwrap();

        let report = run(&store, &TierPolicy::default(), false, false)
            .await
            .unwrap();
        assert_eq!(report.links_pruned, 1);
        assert!(store
            .list_ontology_links(Some("ent:live"))
            .await
            .unwrap()
            .iter()
            .all(|l| l.id != "lnk:dangling"));
        assert!(store
            .get_ontology_entity("ent:live")
            .await
            .unwrap()
            .is_some());
    }
}
