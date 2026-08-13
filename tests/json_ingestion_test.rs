//! Integration tests for U1.1–U1.3: schema migration v006, ingestion_jobs store
//! methods, and JSON/NDJSON connector.

use kurultai::connectors::json::JsonConnector;
use kurultai::connectors::Connector;
use kurultai::embed::{Embedder, NullEmbedder};
use kurultai::pipeline::IndexPipeline;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::types::{SourceConfig, SourceKind};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Arc;

// ── Migration v006 ──────────────────────────────────────────────────────────

#[test]
fn migration_v006_creates_ingestion_jobs_table() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap();
    drop(store); // flush

    // Open the raw SQLite connection and probe the table.
    let conn = Connection::open(dir.path().join("store.db")).unwrap();

    // schema_migrations must record version 7
    let max_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(max_version, 7, "schema_migrations should record version 7");

    // ingestion_jobs table must exist with expected columns
    let mut stmt = conn.prepare("PRAGMA table_info(ingestion_jobs)").unwrap();
    let columns: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    for expected in &[
        "id",
        "batch_id",
        "source",
        "file_path",
        "status",
        "atoms_count",
        "error_message",
        "created_at",
        "completed_at",
    ] {
        assert!(
            columns.contains(&expected.to_string()),
            "column {expected} missing from ingestion_jobs; found: {columns:?}"
        );
    }
}

#[test]
fn migration_v006_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    // Open twice — second open must not error even though version 7 already applied.
    let _ = SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap();
    let _ = SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap();
}

// ── Ingestion job store methods ─────────────────────────────────────────────

#[tokio::test]
async fn ingestion_jobs_record_start_returns_id() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());

    let id = store
        .record_ingestion_start("batch-001", "json-src", "/data/file.json")
        .await
        .unwrap();
    assert!(id > 0, "expected positive row id, got {id}");
}

#[tokio::test]
async fn ingestion_jobs_list_pending() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());

    // Insert two pending jobs.
    store
        .record_ingestion_start("batch-A", "json", "/a/f1.json")
        .await
        .unwrap();
    store
        .record_ingestion_start("batch-A", "json", "/a/f2.json")
        .await
        .unwrap();

    let pending = store.list_pending_ingestion_jobs().await.unwrap();
    assert_eq!(pending.len(), 2);
    assert!(pending.iter().all(|j| j.status == "pending"));
    assert!(pending.iter().any(|j| j.file_path == "/a/f1.json"));
}

#[tokio::test]
async fn ingestion_jobs_finish_success_sets_completed() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());

    let job_id = store
        .record_ingestion_start("batch-B", "json", "/b/data.json")
        .await
        .unwrap();

    store
        .record_ingestion_finish(job_id, Some(42), None)
        .await
        .unwrap();

    // After finishing, the job should no longer be pending.
    let pending = store.list_pending_ingestion_jobs().await.unwrap();
    assert!(
        pending.iter().all(|j| j.id != job_id),
        "completed job should not appear in pending list"
    );
}

#[tokio::test]
async fn ingestion_jobs_finish_failure_records_error() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());

    let job_id = store
        .record_ingestion_start("batch-C", "json", "/c/broken.json")
        .await
        .unwrap();

    store
        .record_ingestion_finish(job_id, None, Some("parse error: unexpected EOF"))
        .await
        .unwrap();

    // Should no longer be in the pending list (status = 'failed').
    let pending = store.list_pending_ingestion_jobs().await.unwrap();
    assert!(pending.iter().all(|j| j.id != job_id));
}

// ── JSON connector ──────────────────────────────────────────────────────────

#[tokio::test]
async fn json_connector_full_sync_indexes_json_array() {
    let dir = tempfile::tempdir().unwrap();
    let content = r#"[
      {"id": "j1", "title": "Record One",   "content": "INTEGRATION_JSON_KNOWN_55 with enough operational detail for the quality gate to pass.", "tags": ["integration"]},
      {"id": "j2", "title": "Record Two",   "content": "second record content with enough operational detail for indexing under quarantine."}
    ]"#;
    std::fs::write(dir.path().join("data.json"), content).unwrap();

    let store_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(store_dir.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let mut connector = JsonConnector::new();
    let config = SourceConfig {
        name: "json-integ".into(),
        kind: SourceKind::Json,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([(
            "root_path".into(),
            dir.path().to_string_lossy().into_owned(),
        )]),
    };
    connector.init(&config).await.unwrap();
    pipeline
        .index_connector("json-integ", &connector, true)
        .await
        .unwrap();

    let count = store.count().await.unwrap();
    assert_eq!(count, 2, "expected 2 atoms indexed, got {count}");

    let atoms = store
        .list_atoms(10, kurultai::store::SearchFilter::default())
        .await
        .unwrap();
    assert!(
        atoms
            .iter()
            .any(|a| a.content.contains("INTEGRATION_JSON_KNOWN_55")),
        "expected INTEGRATION_JSON_KNOWN_55 in indexed atoms"
    );
    assert!(
        atoms
            .iter()
            .any(|a| a.tags.contains(&"integration".to_string())),
        "expected tag 'integration' in indexed atoms"
    );
    assert!(
        atoms.iter().all(|a| a.source == "json-integ"),
        "all atoms should carry source=json-integ"
    );
}

#[tokio::test]
async fn json_connector_full_sync_indexes_ndjson() {
    let dir = tempfile::tempdir().unwrap();
    let content = [
        r#"{"uid": "n1", "title": "NDJSON One", "content": "INTEGRATION_NDJSON_KNOWN_99 with enough operational detail for the quality gate.", "tags": ["ndjson"]}"#,
        r#"{"uid": "n2", "title": "NDJSON Two", "content": "second ndjson line with enough operational detail for indexing.", "tags": ["ndjson"]}"#,
    ]
    .join("\n");
    std::fs::write(dir.path().join("data.ndjson"), &content).unwrap();

    let store_dir2 = tempfile::tempdir().unwrap();
    let store = Arc::new(SqliteVecStore::open(store_dir2.path().join("store.db"), 4).unwrap());
    let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
    let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, Arc::clone(&embedder));

    let mut connector = JsonConnector::new();
    let config = SourceConfig {
        name: "ndjson-integ".into(),
        kind: SourceKind::Json,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([
            (
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            ),
            ("id_field".into(), "uid".into()),
        ]),
    };
    connector.init(&config).await.unwrap();
    pipeline
        .index_connector("ndjson-integ", &connector, true)
        .await
        .unwrap();

    let count = store.count().await.unwrap();
    assert_eq!(count, 2);

    let atoms = store
        .list_atoms(10, kurultai::store::SearchFilter::default())
        .await
        .unwrap();
    assert!(
        atoms
            .iter()
            .any(|a| a.content.contains("INTEGRATION_NDJSON_KNOWN_99")),
        "expected NDJSON phrase in indexed atoms"
    );
    // Path-stable source_id (relative path + record index).
    assert!(
        atoms.iter().any(|a| a.source_id.ends_with("/0")),
        "expected path-index source_id, got {:?}",
        atoms.iter().map(|a| &a.source_id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn json_connector_rejects_malformed_json_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bad.json"), "this is not json").unwrap();

    let mut connector = JsonConnector::new();
    let config = SourceConfig {
        name: "json-bad".into(),
        kind: SourceKind::Json,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([(
            "root_path".into(),
            dir.path().to_string_lossy().into_owned(),
        )]),
    };
    connector.init(&config).await.unwrap();
    let err = connector.full_sync().await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("malformed JSON") || msg.contains("connector"),
        "expected a connector/parse error, got: {msg}"
    );
}

#[tokio::test]
async fn json_connector_stable_source_id_from_relative_path() {
    let dir = tempfile::tempdir().unwrap();
    let content = r#"[
      {"record_id": "stable-001", "title": "Stable ID Test", "content": "hello with enough detail for a dump atom"}
    ]"#;
    std::fs::write(dir.path().join("stable.json"), content).unwrap();

    let mut connector = JsonConnector::new();
    let config = SourceConfig {
        name: "json-stable".into(),
        kind: SourceKind::Json,
        enabled: true,
        poll_interval_secs: 60,
        extra: HashMap::from([(
            "root_path".into(),
            dir.path().to_string_lossy().into_owned(),
        )]),
    };
    connector.init(&config).await.unwrap();
    let atoms = connector.full_sync().await.unwrap();

    assert_eq!(atoms.len(), 1);
    assert_eq!(
        atoms[0].source_id, "stable.json/0",
        "source_id should be relative path + record index"
    );
    assert_eq!(
        atoms[0].metadata.get("external_id").map(String::as_str),
        None,
        "record_id is not the default id field; external_id only from `id`"
    );
}

#[tokio::test]
async fn json_connector_from_registry_via_config() {
    use kurultai::connectors::ConnectorRegistry;
    use kurultai::environment::Environment;
    use kurultai::types::Config;

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("reg.json"),
        r#"[{"id":"r1","title":"Reg","content":"registry test"}]"#,
    )
    .unwrap();

    let config = Config {
        environment: Environment::Dev,
        sources: vec![SourceConfig {
            name: "json-reg".into(),
            kind: SourceKind::Json,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        }],
        storage_path: "/tmp/kurultai-json-reg-test.db".into(),
        embed_model: "m".into(),
        embed_dim: 4,
        embed_backend: None,
        reranker_model: None,
        poll_interval_secs: 300,
        nightly_full_sync_hour: None,
        inactivity_threshold_hours: None,
        mcp_http_secret: None,
        banner: kurultai::art::BannerMode::Auto,
    };

    let registry = ConnectorRegistry::from_config(&config).await.unwrap();
    assert_eq!(registry.len(), 1);
    assert!(registry.get("json-reg").is_some());
}
