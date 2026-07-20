use kurultai::config::{self, default_init_config, write_config};
use kurultai::types::{KurultaiEnv, SourceKind};
use tempfile::tempdir;

#[test]
fn init_writes_config_shape() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let cfg = default_init_config(KurultaiEnv::Dev, Some(dir.path().join("notes")));
    write_config(&cfg, &path).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("filesystem") || text.contains("Filesystem"));
    let loaded: kurultai::types::Config = toml::from_str(&text).unwrap();
    assert_eq!(loaded.embed_dim, 1536);
    assert!(!loaded.sources.is_empty());
}

#[test]
fn env_isolates_default_store_paths() {
    let dev = config::default_storage_path(KurultaiEnv::Dev).unwrap();
    let staging = config::default_storage_path(KurultaiEnv::Staging).unwrap();
    let prod = config::default_storage_path(KurultaiEnv::Prod).unwrap();
    assert!(dev.to_string_lossy().contains("/dev/"));
    assert!(staging.to_string_lossy().contains("/staging/"));
    assert!(prod.to_string_lossy().contains("/prod/"));
    assert_ne!(dev, staging);
    assert_ne!(staging, prod);
}

#[test]
fn registered_kinds_honest() {
    let reg = SourceKind::registered();
    assert!(reg.contains(&SourceKind::Filesystem));
    assert!(reg.contains(&SourceKind::Obsidian));
    assert!(!reg.contains(&SourceKind::AppFlowy));
    assert!(!SourceKind::AppFlowy.is_implemented());
}
