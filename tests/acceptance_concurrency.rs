//! Acceptance tests — multi-process SQLite safety (Track A: shared crew brain).
//!
//! The shared-brain deployment is N independent OS processes (one `kurultai
//! mcp` per agent session, plus periodic `kurultai index` runs) all opening the
//! same `store.db`. `SqliteVecStore`'s internal `Mutex<Connection>` only
//! serializes within one process, so the database itself must be opened in a
//! mode that tolerates cross-process concurrency.
//!
//! These tests pin the three properties that makes true:
//!   * `journal_mode` reads back as `wal` (readers do not block on the writer),
//!   * `busy_timeout` is non-zero (a contended open waits instead of failing),
//!   * concurrent handles on one file can interleave writes and searches
//!     without surfacing `SQLITE_BUSY`.

use chrono::Utc;
use kurultai::store::{SearchFilter, SqliteVecStore, Store};
use kurultai::types::KnowledgeAtom;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

fn temp_db_path(tag: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kurultai-conc-{tag}-{}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("store.db")
}

fn atom(id: &str, body: &str) -> KnowledgeAtom {
    KnowledgeAtom {
        id: id.into(),
        source: "test".into(),
        source_id: format!("/{id}"),
        title: format!("Atom {id}"),
        summary: "concurrency acceptance summary".into(),
        content: body.into(),
        tags: vec!["concurrency".into()],
        source_updated_at: Utc::now(),
        indexed_at: Utc::now(),
        ..Default::default()
    }
}

// ── TA-C1: WAL is actually on ────────────────────────────────────────────────

/// `PRAGMA journal_mode = WAL` is a *query* pragma — `execute_batch` swallows a
/// failed switch silently. Assert on the value the database reports back, not
/// on the statement having run.
#[test]
fn journal_mode_reads_back_as_wal() {
    let path = temp_db_path("wal");
    let store = SqliteVecStore::open(path.clone(), 4).unwrap();
    assert_eq!(store.journal_mode().unwrap().to_ascii_lowercase(), "wal");
    drop(store);

    // WAL lives in the database header, so a fresh handle sees it too.
    let reopened = SqliteVecStore::open(path, 4).unwrap();
    let mode = reopened.journal_mode().unwrap();
    assert_eq!(
        mode.to_ascii_lowercase(),
        "wal",
        "store.db must persist WAL journal mode; got {mode}"
    );
}

// ── TA-C2: busy_timeout is set on every connection ───────────────────────────

/// `busy_timeout` is per-connection, not persisted in the file, so it has to be
/// re-applied on each `open`. Zero means a contended reader gets `SQLITE_BUSY`
/// immediately rather than waiting.
#[test]
fn busy_timeout_is_nonzero_on_each_open() {
    let path = temp_db_path("busy");
    let first = SqliteVecStore::open(path.clone(), 4).unwrap();
    let second = SqliteVecStore::open(path.clone(), 4).unwrap();

    for store in [&first, &second] {
        let timeout = store.pragma_i64("busy_timeout").unwrap();
        assert!(
            timeout >= 1000,
            "busy_timeout must be set on every open, got {timeout}"
        );
    }
}

/// The timeout is overridable for operators who need a longer wait under a
/// heavier crew.
#[test]
fn busy_timeout_honours_env_override() {
    // Serialized against other env readers by running in its own process is
    // not available here; the value is read at open time, so set/restore
    // around a single open.
    let path = temp_db_path("busyenv");
    let prev = std::env::var(kurultai::store::BUSY_TIMEOUT_ENV).ok();
    std::env::set_var(kurultai::store::BUSY_TIMEOUT_ENV, "12345");
    let store = SqliteVecStore::open(path, 4).unwrap();
    let timeout = store.pragma_i64("busy_timeout").unwrap();
    match prev {
        Some(v) => std::env::set_var(kurultai::store::BUSY_TIMEOUT_ENV, v),
        None => std::env::remove_var(kurultai::store::BUSY_TIMEOUT_ENV),
    }
    assert_eq!(timeout, 12345);
}

// ── TA-C3: concurrent handles do not surface SQLITE_BUSY ─────────────────────

/// Nine handles — one per crew session — on a single database file, writing and
/// searching at the same time. Under the previous `DELETE` journal with a zero
/// busy timeout this fails with "database is locked".
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn nine_concurrent_handles_interleave_writes_and_searches() {
    let path = temp_db_path("nine");
    // Open once up front so migrations are settled before the fan-out.
    let seed = SqliteVecStore::open(path.clone(), 4).unwrap();
    seed.upsert(&atom("seed", "kurultai shared crew brain seed content"))
        .await
        .unwrap();
    drop(seed);

    let mut tasks = Vec::new();
    for session in 0..9u32 {
        let path = path.clone();
        tasks.push(tokio::spawn(async move {
            let store = Arc::new(SqliteVecStore::open(path, 4).unwrap());
            for round in 0..5u32 {
                store
                    .upsert(&atom(
                        &format!("s{session}-r{round}"),
                        "kurultai shared crew brain content for concurrency round",
                    ))
                    .await
                    .expect("upsert must not hit SQLITE_BUSY");
                let hits = store
                    .fts_search(
                        "kurultai",
                        10,
                        SearchFilter {
                            trusted_only: false,
                        },
                    )
                    .await
                    .expect("fts_search must not hit SQLITE_BUSY");
                assert!(!hits.is_empty(), "seed atom should always be findable");
            }
        }));
    }
    for t in tasks {
        t.await.expect("session task panicked");
    }

    let store = SqliteVecStore::open(path, 4).unwrap();
    assert_eq!(store.count().await.unwrap(), 9 * 5 + 1);
}

// ── TA-C4: stale WAL sidecars are cleaned when the db file is replaced ───────

/// `import --replace` copies a new database over `store.db` in place. A `-wal`
/// left behind from the old database would be replayed onto the new one.
#[test]
fn remove_wal_sidecars_clears_stale_files() {
    let path = temp_db_path("sidecar");
    let store = SqliteVecStore::open(path.clone(), 4).unwrap();
    drop(store);

    let [wal, shm] = kurultai::store::wal_sidecar_paths(&path);
    std::fs::write(&wal, b"stale").unwrap();
    std::fs::write(&shm, b"stale").unwrap();

    kurultai::store::remove_wal_sidecars(&path).unwrap();
    assert!(!wal.exists(), "-wal should be removed");
    assert!(!shm.exists(), "-shm should be removed");

    // Idempotent — missing sidecars are not an error.
    kurultai::store::remove_wal_sidecars(&path).unwrap();
}
