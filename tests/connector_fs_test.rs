use kurultai::connectors::filesystem::FilesystemConnector;
use kurultai::connectors::obsidian::ObsidianConnector;
use kurultai::connectors::Connector;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::types::{SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

fn vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

#[tokio::test]
async fn indexes_three_fixture_files() {
    let mut fs = FilesystemConnector::new("notes");
    let mut extra = HashMap::new();
    extra.insert("path".into(), vault().display().to_string());
    let cfg = SourceConfig {
        name: "notes".into(),
        kind: SourceKind::Filesystem,
        enabled: true,
        poll_interval_secs: 60,
        extra,
    };
    fs.init(&cfg).await.unwrap();
    let atoms = fs.full_sync().await.unwrap();
    assert_eq!(atoms.len(), 3);
    assert!(atoms
        .iter()
        .any(|a| a.content.contains("zebra-migration-alpha")));
}

#[tokio::test]
async fn edit_one_file_changes_one_hash() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("v");
    std::fs::create_dir_all(&vault).unwrap();
    let f1 = vault.join("a.md");
    let f2 = vault.join("b.md");
    std::fs::write(&f1, "alpha one").unwrap();
    std::fs::write(&f2, "beta two").unwrap();

    let mut fs = FilesystemConnector::new("notes");
    let mut extra = HashMap::new();
    extra.insert("path".into(), vault.display().to_string());
    let cfg = SourceConfig {
        name: "notes".into(),
        kind: SourceKind::Filesystem,
        enabled: true,
        poll_interval_secs: 60,
        extra,
    };
    fs.init(&cfg).await.unwrap();
    let first = fs.full_sync().await.unwrap();
    let h1 = first
        .iter()
        .find(|a| a.source_id == "a.md")
        .unwrap()
        .content_hash
        .clone();

    std::fs::write(&f1, "alpha one edited").unwrap();
    let second = fs.full_sync().await.unwrap();
    let a2 = second.iter().find(|a| a.source_id == "a.md").unwrap();
    let b2 = second.iter().find(|a| a.source_id == "b.md").unwrap();
    assert_ne!(a2.content_hash, h1);
    assert_eq!(
        b2.content_hash,
        first
            .iter()
            .find(|a| a.source_id == "b.md")
            .unwrap()
            .content_hash
    );

    // Store: only re-upsert changed — simulate app logic
    let store = SqliteVecStore::open(dir.path().join("s.db"), "m", 4).unwrap();
    for a in &first {
        store.upsert(a).unwrap();
    }
    let mut changed = 0u64;
    for a in &second {
        if let Some(ex) = store.get_by_source_id(&a.source, &a.source_id).unwrap() {
            if ex.content_hash == a.content_hash {
                continue;
            }
            store.delete_id(&ex.id).unwrap();
        }
        store.upsert(a).unwrap();
        changed += 1;
    }
    assert_eq!(changed, 1);
    assert_eq!(store.count().unwrap(), 2);
}

#[tokio::test]
async fn obsidian_aliases_vault_path() {
    let mut obs = ObsidianConnector::new();
    let mut extra = HashMap::new();
    extra.insert("vault_path".into(), vault().display().to_string());
    let cfg = SourceConfig {
        name: "obs".into(),
        kind: SourceKind::Obsidian,
        enabled: true,
        poll_interval_secs: 60,
        extra,
    };
    obs.init(&cfg).await.unwrap();
    let atoms = obs.full_sync().await.unwrap();
    assert_eq!(atoms.len(), 3);
}

/// Orphan policy: after full sync, atoms for deleted files are removed via delete_orphans.
#[tokio::test]
async fn orphan_policy_deletes_removed_files() {
    let dir = tempdir().unwrap();
    let vault = dir.path().join("v");
    std::fs::create_dir_all(&vault).unwrap();
    std::fs::write(vault.join("keep.md"), "keep").unwrap();
    std::fs::write(vault.join("gone.md"), "gone").unwrap();

    let mut fs = FilesystemConnector::new("notes");
    let mut extra = HashMap::new();
    extra.insert("path".into(), vault.display().to_string());
    let cfg = SourceConfig {
        name: "notes".into(),
        kind: SourceKind::Filesystem,
        enabled: true,
        poll_interval_secs: 60,
        extra,
    };
    fs.init(&cfg).await.unwrap();
    let store = SqliteVecStore::open(dir.path().join("s.db"), "m", 4).unwrap();
    let atoms = fs.full_sync().await.unwrap();
    for a in &atoms {
        store.upsert(a).unwrap();
    }
    assert_eq!(store.count().unwrap(), 2);

    std::fs::remove_file(vault.join("gone.md")).unwrap();
    let atoms2 = fs.full_sync().await.unwrap();
    let keep: Vec<_> = atoms2.iter().map(|a| a.source_id.clone()).collect();
    for a in &atoms2 {
        if let Some(ex) = store.get_by_source_id(&a.source, &a.source_id).unwrap() {
            if ex.content_hash != a.content_hash {
                store.delete_id(&ex.id).unwrap();
                store.upsert(a).unwrap();
            }
        } else {
            store.upsert(a).unwrap();
        }
    }
    let n = store.delete_orphans("notes", &keep).unwrap();
    assert_eq!(n, 1);
    assert_eq!(store.count().unwrap(), 1);
}
