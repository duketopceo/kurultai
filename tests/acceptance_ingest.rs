//! Acceptance tests — ingest surface (KHAN-251).
//!
//! Covers: markdown ingest (frontmatter + heading chunks), format parity
//! (json/ndjson/txt), inbox tray connector (processed/failed moves),
//! loopback `POST /ingest`, and the hashtag-line corpora feature.

use chrono::Utc;
use kurultai::connectors::inbox::InboxConnector;
use kurultai::connectors::markdown::MarkdownConnector;
use kurultai::connectors::Connector;
use kurultai::embed::NullEmbedder;
use kurultai::ingest::dump::{atomize_bytes, atomize_path, detect_format, DumpFormat};
use kurultai::pipeline::IndexPipeline;
use kurultai::store::{SearchFilter, SqliteVecStore, Store};
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Unique temp dir per test run (avoids sqlite-vec / FTS cross-test locking).
fn temp_dir(_label: &str) -> tempfile::TempDir {
    tempfile::TempDir::new().unwrap()
}

fn temp_store(dim: usize) -> (Arc<SqliteVecStore>, tempfile::TempDir) {
    let dir = temp_dir("store");
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), dim).unwrap());
    (store, dir)
}

fn fixture_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

// ── A0-1: Markdown ingest ────────────────────────────────────────────────────

#[tokio::test]
async fn markdown_full_sync_indexes_fixture_vault() {
    let (store, _db) = temp_store(4);
    let embedder: Arc<dyn kurultai::embed::Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);

    let mut connector = MarkdownConnector::new();
    let mut extra = HashMap::new();
    extra.insert(
        "root_path".into(),
        fixture_vault().to_string_lossy().into_owned(),
    );
    connector
        .init(&SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();

    let stats = pipeline
        .index_connector("notes", &connector, true)
        .await
        .unwrap();
    assert!(stats.atoms_indexed > 0, "fixture vault must index atoms");

    // FTS hit on the golden phrase proves the content reached atoms_fts.
    let hits = store
        .fts_search("KNOWN_PHRASE_KURULTAI_42", 5, SearchFilter::default())
        .await
        .unwrap();
    assert!(!hits.is_empty());
    assert_eq!(hits[0].0.source, "notes");
}

#[tokio::test]
async fn markdown_frontmatter_tags_survive_index() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("note.md"),
        "---\ntitle: Frontmatter Note\ntags: [ops, deploy]\n---\n\n## Body\n\
         Enough operational detail for the quality gate to pass acceptance here with more words.\n",
    )
    .unwrap();

    let mut connector = MarkdownConnector::new();
    connector
        .init(&SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();
    let atoms = connector.full_sync().await.unwrap();
    assert!(atoms.iter().any(|a| a.tags.iter().any(|t| t == "ops")));
    assert!(atoms.iter().any(|a| a.title.contains("Frontmatter Note")));
}

// ── Format parity (json / ndjson / txt) ───────────────────────────────────────

#[test]
fn detect_format_by_extension() {
    assert_eq!(
        detect_format(&PathBuf::from("a.md")),
        Some(DumpFormat::Markdown)
    );
    assert_eq!(
        detect_format(&PathBuf::from("a.JSON")),
        Some(DumpFormat::Json)
    );
    assert_eq!(
        detect_format(&PathBuf::from("a.ndjson")),
        Some(DumpFormat::Ndjson)
    );
    assert_eq!(
        detect_format(&PathBuf::from("a.txt")),
        Some(DumpFormat::PlainText)
    );
    assert_eq!(detect_format(&PathBuf::from("a.bin")), None);
}

#[test]
fn atomize_json_array_produces_atoms() {
    let json = r#"[{"title":"J1","content":"JSON body one with detail","tags":["x"]},
                   {"title":"J2","content":"JSON body two with detail","tags":["y"]}]"#;
    let atoms = atomize_bytes(
        "inbox",
        "dump.json",
        json.as_bytes(),
        DumpFormat::Json,
        Utc::now(),
    )
    .unwrap();
    assert_eq!(atoms.len(), 2);
    assert!(atoms.iter().any(|a| a.title == "J1"));
    assert!(atoms.iter().any(|a| a.tags.iter().any(|t| t == "y")));
}

#[test]
fn atomize_ndjson_produces_one_atom_per_line() {
    let ndjson = "{\"title\":\"N1\",\"content\":\"ndjson body one detail\",\"tags\":[]}\n\
                  {\"title\":\"N2\",\"content\":\"ndjson body two detail\",\"tags\":[]}\n";
    let atoms = atomize_bytes(
        "inbox",
        "dump.ndjson",
        ndjson.as_bytes(),
        DumpFormat::Ndjson,
        Utc::now(),
    )
    .unwrap();
    assert_eq!(atoms.len(), 2);
    assert!(atoms.iter().any(|a| a.title == "N1"));
}

#[test]
fn atomize_plain_text_single_atom() {
    let atoms = atomize_bytes(
        "inbox",
        "dump.txt",
        b"plain text body with enough operational detail for the gate",
        DumpFormat::PlainText,
        Utc::now(),
    )
    .unwrap();
    assert_eq!(atoms.len(), 1);
    assert!(atoms[0].content.contains("plain text body"));
}

// ── TA-9: Inbox connector (config-not-code adapter) ──────────────────────────

#[tokio::test]
async fn inbox_connector_indexes_dump_and_moves_to_processed() {
    let tray = tempfile::tempdir().unwrap();
    // A markdown file with frontmatter tags (passes quality gate → trusted).
    fs::write(
        tray.path().join("inbox.md"),
        "---\ntitle: Inbox Doc\ntags: [ops]\n---\n\nInbox body with sufficient operational detail to pass the quality gate acceptance threshold.\n",
    )
    .unwrap();

    let (store, _db) = temp_store(4);
    let embedder: Arc<dyn kurultai::embed::Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);

    let mut connector = InboxConnector::new();
    connector
        .init(&SourceConfig {
            name: "inbox".into(),
            kind: SourceKind::Inbox,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                tray.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();

    pipeline
        .index_connector("inbox", &connector, true)
        .await
        .unwrap();

    // Trusted atoms are searchable.
    let hits = store
        .fts_search("Inbox body", 5, SearchFilter::default())
        .await
        .unwrap();
    assert!(!hits.is_empty(), "inbox atom must be searchable");

    // Tray finalization moved the file to processed/.
    let processed = tray.path().join("processed");
    assert!(processed.is_dir(), "processed/ tray must exist");
    let moved: Vec<_> = fs::read_dir(&processed)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(!moved.is_empty(), "trusted file must move to processed/");
}

#[tokio::test]
async fn inbox_connector_moves_unparseable_to_failed() {
    let tray = tempfile::tempdir().unwrap();
    // A .json file with invalid JSON content — parse fails → moves to failed/.
    fs::write(
        tray.path().join("broken.json"),
        b"{ this is not valid json ::",
    )
    .unwrap();

    let mut connector = InboxConnector::new();
    connector
        .init(&SourceConfig {
            name: "inbox".into(),
            kind: SourceKind::Inbox,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                tray.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();

    let atoms = connector.full_sync().await.unwrap();
    assert!(atoms.is_empty(), "unparseable file yields no atoms");

    let failed = tray.path().join("failed");
    assert!(failed.is_dir(), "failed/ tray must exist");
    let moved: Vec<_> = fs::read_dir(&failed)
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(!moved.is_empty(), "unparseable file must move to failed/");
}

// ── TA-6: Hashtag-line ingest (whitespace #tag lines) ───────────────────────
// NOTE: This feature is declared in commit b43d43d ("feat(connectors): accept
// dedicated hashtag-line tags") but no parsing implementation exists on main.
// The markdown connector only reads YAML-frontmatter tags. This test is
// #[ignore]'d and the gap is documented in ACCEPTANCE_REPORT.md.

#[tokio::test]
#[ignore = "KHAN-251: hashtag-line ingest not implemented; see ACCEPTANCE_REPORT.md"]
async fn hashtag_line_tags_without_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    // No YAML frontmatter; tags declared on a dedicated whitespace-separated
    // `#tag` line (the kb-it-docs corpora pattern).
    fs::write(
        dir.path().join("itdoc.md"),
        "# IT Doc\n\n#ops #deploy\n\nRun the migration scripts before cutover with full detail.\n",
    )
    .unwrap();

    let mut connector = MarkdownConnector::new();
    connector
        .init(&SourceConfig {
            name: "itdocs".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();
    let atoms = connector.full_sync().await.unwrap();
    // Expected: hashtag-line tags are parsed so the atom is NOT quarantined.
    assert!(
        atoms.iter().any(|a| a.tags.iter().any(|t| t == "ops")),
        "hashtag-line tags should be parsed: {:?}",
        atoms.iter().map(|a| &a.tags).collect::<Vec<_>>()
    );
}

// ── TA-10: Loopback POST /ingest ─────────────────────────────────────────────
// The /ingest route is mounted only when KURULTAI_INGEST_SECRET is set.
// We exercise the atomizer directly (route-level auth tested in
// acceptance_http.rs) plus a pipeline upsert to confirm round-trip.

#[tokio::test]
async fn loopback_atomize_then_upsert_searchable() {
    let (store, _db) = temp_store(4);
    // JSON dump with tags so the quality gate trusts it.
    let body = "{\"title\":\"Loopback Doc\",\"content\":\"Loopback ingest body with sufficient operational detail to clear the quality gate threshold.\",\"tags\":[\"ops\"]}";
    let mut atoms = atomize_bytes(
        "loopback",
        "ingest/body.json",
        body.as_bytes(),
        DumpFormat::Json,
        Utc::now(),
    )
    .unwrap();
    // Apply the quality gate like the /ingest handler does.
    for atom in &mut atoms {
        let outcome = kurultai::quality::evaluate(store.as_ref(), atom)
            .await
            .unwrap();
        kurultai::quality::apply_gate(atom, outcome);
    }
    store.upsert_batch(&atoms).await.unwrap();

    let hits = store
        .fts_search("Loopback ingest", 5, SearchFilter::default())
        .await
        .unwrap();
    assert!(
        !hits.is_empty(),
        "loopback-ingested atom must be searchable"
    );
}

#[test]
fn atomize_path_rejects_unsupported_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.bin");
    fs::write(&path, b"bytes").unwrap();
    let err = atomize_path("src", dir.path(), &path, Utc::now()).unwrap_err();
    assert!(
        err.to_string().contains("unsupported dump format"),
        "expected format error, got: {err}"
    );
}
