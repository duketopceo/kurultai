#![allow(clippy::field_reassign_with_default)]
//! Acceptance tests — visibility / tiered access (KHAN-251).
//!
//! Covers: atom visibility scope (personal/team/company) round-trip,
//! corpus tiers (public/private) type + parse, KTD15 visibility labels,
//! and the SourceConfig tier/label helpers. Documents broken persistence
//! for corpus_tier / visibility_labels (see ACCEPTANCE_REPORT.md).

use chrono::Utc;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::types::{CorpusTier, KnowledgeAtom, SourceConfig, SourceKind, VisibilityScope};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_store() -> SqliteVecStore {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "khan251-vis-{}-{}-{}",
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
        source: "test".into(),
        source_id: format!("/{id}"),
        title: format!("Atom {id}"),
        summary: "visibility test summary".into(),
        content: "visibility test content with enough detail for the gate".into(),
        tags: vec!["t".into()],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        ..Default::default()
    }
}

// ── TA-1: Visibility scope round-trip ────────────────────────────────────────

#[tokio::test]
async fn visibility_scope_team_round_trips_through_store() {
    let store = temp_store();
    let mut atom = sample_atom("v-team");
    atom.visibility = VisibilityScope::Team;
    store.upsert(&atom).await.unwrap();

    let loaded = store.get("v-team").await.unwrap().unwrap();
    assert_eq!(loaded.visibility, VisibilityScope::Team);
}

#[tokio::test]
async fn visibility_scope_company_round_trips_through_store() {
    let store = temp_store();
    let mut atom = sample_atom("v-co");
    atom.visibility = VisibilityScope::Company;
    store.upsert(&atom).await.unwrap();

    let loaded = store.get("v-co").await.unwrap().unwrap();
    assert_eq!(loaded.visibility, VisibilityScope::Company);
}

#[tokio::test]
async fn visibility_scope_default_is_personal() {
    let store = temp_store();
    let atom = sample_atom("v-def");
    store.upsert(&atom).await.unwrap();
    let loaded = store.get("v-def").await.unwrap().unwrap();
    assert_eq!(loaded.visibility, VisibilityScope::Personal);
}

#[test]
fn visibility_scope_parse_round_trips() {
    for scope in [
        VisibilityScope::Personal,
        VisibilityScope::Team,
        VisibilityScope::Company,
    ] {
        assert_eq!(VisibilityScope::parse(scope.as_str()), scope);
    }
}

#[test]
fn visibility_scope_parse_unknown_fail_closed_to_personal() {
    assert_eq!(VisibilityScope::parse(""), VisibilityScope::Personal);
    assert_eq!(VisibilityScope::parse("Team"), VisibilityScope::Personal);
    assert_eq!(VisibilityScope::parse("public"), VisibilityScope::Personal);
}

// ── TA-2: Corpus tier type + parse ──────────────────────────────────────────

#[test]
fn corpus_tier_as_str_and_parse_round_trips() {
    assert_eq!(CorpusTier::Public.as_str(), "public");
    assert_eq!(CorpusTier::Private.as_str(), "private");
    assert_eq!(CorpusTier::parse("public"), CorpusTier::Public);
    assert_eq!(CorpusTier::parse("private"), CorpusTier::Private);
}

#[test]
fn corpus_tier_parse_unknown_fail_closed_to_private() {
    assert_eq!(CorpusTier::parse(""), CorpusTier::Private);
    assert_eq!(CorpusTier::parse("Public"), CorpusTier::Private);
    assert_eq!(CorpusTier::parse("team"), CorpusTier::Private);
}

#[test]
fn corpus_tier_default_is_public() {
    assert_eq!(CorpusTier::default(), CorpusTier::Public);
}

// ── TA-3: Corpus tier persistence ─────────────────────────────────────────

#[tokio::test]
async fn corpus_tier_private_round_trips_through_store() {
    let store = temp_store();
    let mut atom = sample_atom("c-priv");
    atom.corpus_tier = CorpusTier::Private;
    store.upsert(&atom).await.unwrap();
    let loaded = store.get("c-priv").await.unwrap().unwrap();
    assert_eq!(loaded.corpus_tier, CorpusTier::Private);
}

// ── TA-4: Visibility labels persistence ─────────────────────────────────────

#[tokio::test]
async fn visibility_labels_round_trip_through_store() {
    let store = temp_store();
    let mut atom = sample_atom("v-labels");
    atom.visibility_labels = vec!["finance".into(), "exec".into()];
    store.upsert(&atom).await.unwrap();
    let loaded = store.get("v-labels").await.unwrap().unwrap();
    assert_eq!(loaded.visibility_labels, vec!["finance", "exec"]);
}

// ── TA-5: SourceConfig tier / label helpers ─────────────────────────────────

#[test]
fn source_config_default_corpus_tier_public_when_absent() {
    let cfg = SourceConfig {
        name: "s".into(),
        kind: SourceKind::Markdown,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::new(),
    };
    assert_eq!(cfg.default_corpus_tier(), CorpusTier::Public);
}

#[test]
fn source_config_default_corpus_tier_private_when_set() {
    let cfg = SourceConfig {
        name: "s".into(),
        kind: SourceKind::Markdown,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([("default_corpus_tier".into(), "private".into())]),
    };
    assert_eq!(cfg.default_corpus_tier(), CorpusTier::Private);
}

#[test]
fn source_config_default_visibility_labels_parsed() {
    let cfg = SourceConfig {
        name: "s".into(),
        kind: SourceKind::Markdown,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([(
            "default_visibility_labels".into(),
            "finance, exec ,ops".into(),
        )]),
    };
    let labels = cfg.default_visibility_labels();
    assert_eq!(labels, vec!["finance", "exec", "ops"]);
}

#[test]
fn source_config_default_visibility_labels_empty_when_absent() {
    let cfg = SourceConfig {
        name: "s".into(),
        kind: SourceKind::Markdown,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::new(),
    };
    assert!(cfg.default_visibility_labels().is_empty());
}

// ── TA-11: Hub store gate (feature-flagged) ──────────────────────────────────
// The Postgres+pgvector hub store is behind `--features postgres` AND the
// `KURULTAI_FEATURE_HUB=1` runtime flag. Without the flag it must refuse.

#[tokio::test]
async fn open_hub_store_refuses_without_feature_flag() {
    // Explicitly disable the hub flag — CI's Postgres job sets KURULTAI_FEATURE_HUB=1.
    let prev = std::env::var("KURULTAI_FEATURE_HUB").ok();
    std::env::set_var("KURULTAI_FEATURE_HUB", "0");
    let result = kurultai::store::open_hub_store("postgres://localhost/x", 4).await;
    match prev {
        Some(v) => std::env::set_var("KURULTAI_FEATURE_HUB", v),
        None => std::env::remove_var("KURULTAI_FEATURE_HUB"),
    }
    assert!(
        result.is_err(),
        "hub store must refuse without the feature flag"
    );
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("KURULTAI_FEATURE_HUB"),
        "hub store must refuse without the feature flag: {err}"
    );
}
