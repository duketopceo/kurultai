//! Golden-path integration test: fixture vault -> index --full -> FTS search hit.
//!
//! This is the Phase 1 exit criterion. It runs end-to-end through the App,
//! using the default FTS-only embedder so no API key is required.

use std::io::Write;
use tempfile::TempDir;

fn make_fixture_vault() -> TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let mut f1 = std::fs::File::create(root.join("README.md")).unwrap();
    write!(
        f1,
        "# Kurultai Golden Path\n\nThis vault is used for the golden-path integration test.\n"
    )
    .unwrap();

    let nested = root.join("docs");
    std::fs::create_dir(&nested).unwrap();
    let mut f2 = std::fs::File::create(nested.join("unique.md")).unwrap();
    write!(
        f2,
        "# Unique Phrase\n\nThe quick brown kurultai jumps over the lazy index.\n"
    )
    .unwrap();

    tmp
}

fn make_config(vault: &TempDir, storage: &TempDir) -> std::path::PathBuf {
    let config_path = storage.path().join("config.toml");
    let storage_path = storage.path().join("store.db");
    let vault_path = vault.path().to_string_lossy();

    let contents = format!(
        r#"
environment = "dev"

[sources.notes]
kind = "markdown"
enabled = true
poll_interval_secs = 60
root_path = "{vault_path}"

[storage]
path = "{storage_path}"

[embed]
model = "fixed-test"
dimension = 8
"#,
        storage_path = storage_path.display(),
    );

    std::fs::write(&config_path, contents).unwrap();
    config_path
}

#[tokio::test]
async fn golden_path_index_and_search() {
    let vault = make_fixture_vault();
    let storage = tempfile::tempdir().unwrap();
    let config_path = make_config(&vault, &storage);

    let app = kurultai::app::App::bootstrap_from(&config_path, Some("dev"))
        .await
        .expect("app should bootstrap with fixture vault");

    assert_eq!(app.connectors.len(), 1);
    assert_eq!(app.connectors.names(), vec!["notes"]);

    let stats = app
        .pipeline
        .index_all(&app.connectors, true)
        .await
        .expect("index should succeed");
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].atoms_fetched, 2);
    assert_eq!(stats[0].atoms_indexed, 2);

    let count = app.atom_count().await.expect("count should succeed");
    assert_eq!(count, 2);

    let results = app
        .query_engine
        .search("kurultai jumps", 10)
        .await
        .expect("search should succeed");
    assert!(!results.is_empty(), "expected FTS hit for 'kurultai jumps'");
    let hit = &results[0];
    assert!(hit.atom.content.contains("kurultai"));
    assert!(hit.matched_by.contains(&"fts".to_string()));

    // Edit a file and re-index without --full; the changed atom should update.
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(vault.path().join("docs/unique.md"))
        .unwrap();
    write!(f, "# Updated\n\nThe quick brown kurultai now runs fast.\n").unwrap();
    drop(f);

    let stats = app
        .pipeline
        .index_all(&app.connectors, false)
        .await
        .expect("incremental index should succeed");
    assert_eq!(stats[0].atoms_fetched, 2);
    assert_eq!(stats[0].atoms_indexed, 1); // only the changed file

    // Delete a file and run full sync; orphan should be removed.
    std::fs::remove_file(vault.path().join("docs/unique.md")).unwrap();
    let stats = app
        .pipeline
        .index_all(&app.connectors, true)
        .await
        .expect("full sync after delete should succeed");
    assert_eq!(stats[0].atoms_fetched, 1);
    assert_eq!(stats[0].atoms_indexed, 1);

    let count = app.atom_count().await.expect("count should succeed");
    assert_eq!(count, 1);
}
