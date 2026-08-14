//! Inbox tray + dump format parity + gate heuristics (AE1–AE7).

use kurultai::connectors::inbox::InboxConnector;
use kurultai::connectors::json::JsonConnector;
use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::pipeline::IndexPipeline;
use kurultai::store::{SearchFilter, SqliteVecStore, Store};
use kurultai::types::{SourceConfig, SourceKind, TrustLane};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

fn pipeline(store: Arc<SqliteVecStore>) -> IndexPipeline {
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    IndexPipeline::new(store as Arc<dyn Store>, embedder)
}

fn source(name: &str, kind: SourceKind, root: &std::path::Path) -> SourceConfig {
    SourceConfig {
        name: name.into(),
        kind,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([("root_path".into(), root.to_string_lossy().into_owned())]),
    }
}

#[tokio::test]
async fn ae1_tagged_markdown_dump_trusted_and_searchable() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("note.md"),
        "---\ntags: [ops]\n---\n\nDeploy checklist with verification steps for the production cluster rollout and post-deploy health checks.\n",
    )
    .unwrap();

    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let pipe = pipeline(Arc::clone(&store));
    let mut connector = MarkdownConnector::new();
    connector
        .init(&source("notes", SourceKind::Markdown, dir.path()))
        .await
        .unwrap();
    pipe.index_connector("notes", &connector, true)
        .await
        .unwrap();

    let hits = store
        .fts_search("Deploy checklist", 5, SearchFilter::default())
        .await
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].0.trust_lane, TrustLane::Trusted);
}

#[tokio::test]
async fn ae2_untagged_plain_text_inbox_fails_but_stores() {
    let dir = tempfile::tempdir().unwrap();
    let inbox = dir.path().join("tray");
    fs::create_dir_all(&inbox).unwrap();
    fs::write(
        inbox.join("raw.txt"),
        "untagged plain dump with enough characters to pass the length heuristic alone.\n",
    )
    .unwrap();

    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let pipe = pipeline(Arc::clone(&store));
    let mut connector = InboxConnector::new();
    connector
        .init(&source("tray", SourceKind::Inbox, &inbox))
        .await
        .unwrap();
    pipe.index_connector("tray", &connector, true)
        .await
        .unwrap();

    assert!(!inbox.join("raw.txt").exists());
    let failed = inbox.join("failed");
    assert!(failed.read_dir().unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains("raw")));

    let all = store
        .list_atoms(50, SearchFilter::trusted(false))
        .await
        .unwrap();
    assert!(all.iter().any(|a| {
        a.trust_lane == TrustLane::Quarantine && a.quarantine_reason.as_deref() == Some("untagged")
    }));
}

#[tokio::test]
async fn ae3_too_short_quarantines() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("short.md"),
        "---\ntags: [ops]\n---\n\ntiny\n",
    )
    .unwrap();

    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let pipe = pipeline(Arc::clone(&store));
    let mut connector = MarkdownConnector::new();
    connector
        .init(&source("notes", SourceKind::Markdown, dir.path()))
        .await
        .unwrap();
    pipe.index_connector("notes", &connector, true)
        .await
        .unwrap();

    let atoms = store
        .list_atoms(20, SearchFilter::trusted(false))
        .await
        .unwrap();
    assert!(atoms
        .iter()
        .any(|a| { a.quarantine_reason.as_deref() == Some("low_quality:too_short") }));
}

#[tokio::test]
async fn ae4_thin_boilerplate_quarantines() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("thin.md"),
        "---\ntags: [ops]\n---\n\nlorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore.\n",
    )
    .unwrap();

    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let pipe = pipeline(Arc::clone(&store));
    let mut connector = MarkdownConnector::new();
    connector
        .init(&source("notes", SourceKind::Markdown, dir.path()))
        .await
        .unwrap();
    pipe.index_connector("notes", &connector, true)
        .await
        .unwrap();

    let atoms = store
        .list_atoms(20, SearchFilter::trusted(false))
        .await
        .unwrap();
    assert!(atoms
        .iter()
        .any(|a| a.quarantine_reason.as_deref() == Some("low_quality:thin")));
}

#[tokio::test]
async fn ae5_soft_labels_do_not_satisfy_tag_gate() {
    use chrono::Utc;
    use kurultai::quality::gate::{evaluate, GateOutcome};
    use kurultai::types::{KnowledgeAtom, SoftLabel};

    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());

    let atom = KnowledgeAtom {
        id: "x".into(),
        source: "t".into(),
        source_id: "x".into(),
        title: "t".into(),
        summary: "s".into(),
        content: "Detailed soft-label-only body with plenty of operational words for length."
            .into(),
        tags: vec![],
        soft_labels: vec![SoftLabel {
            label_id: 1,
            name: "infra".into(),
            score: 1.0,
            aliases: vec![],
        }],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        ..Default::default()
    };
    let out = evaluate(store.as_ref() as &dyn Store, &atom).await.unwrap();
    assert_eq!(
        out,
        GateOutcome::Quarantine {
            reason: "untagged".into()
        }
    );
}

#[tokio::test]
async fn ae7_inbox_trusted_to_processed_and_format_parity() {
    let dir = tempfile::tempdir().unwrap();
    let inbox = dir.path().join("tray");
    fs::create_dir_all(&inbox).unwrap();
    fs::write(
        inbox.join("ok.md"),
        "---\ntags: [ops]\n---\n\nTrusted inbox dump covering migration verification and rollback notes for operators.\n",
    )
    .unwrap();

    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
    let pipe = pipeline(Arc::clone(&store));
    let mut connector = InboxConnector::new();
    connector
        .init(&source("tray", SourceKind::Inbox, &inbox))
        .await
        .unwrap();
    pipe.index_connector("tray", &connector, true)
        .await
        .unwrap();

    assert!(!inbox.join("ok.md").exists());
    assert!(inbox
        .join("processed")
        .read_dir()
        .unwrap()
        .any(|e| { e.unwrap().file_name().to_string_lossy().starts_with("ok") }));

    // Format parity: json source reads markdown dumps with path-stable ids.
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(
        data.join("note.md"),
        "---\ntags: [j]\n---\n\nMarkdown via json source with enough detail for parity coverage.\n",
    )
    .unwrap();
    fs::write(
        data.join("recs.json"),
        r#"[{"title":"R","content":"JSON via json source with enough detail for parity coverage.","tags":["j"]}]"#,
    )
    .unwrap();

    let mut json = JsonConnector::new();
    json.init(&source("data", SourceKind::Json, &data))
        .await
        .unwrap();
    let atoms = json.full_sync().await.unwrap();
    assert!(atoms
        .iter()
        .any(|a| a.source_id == "note.md" || a.source_id.starts_with("note.md")));
    assert!(atoms.iter().any(|a| a.source_id.ends_with("/0")));
}
