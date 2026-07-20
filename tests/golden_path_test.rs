use kurultai::app::Runtime;
use kurultai::types::{Config, KurultaiEnv, SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn fixture_vault_index_fts_search() {
    let dir = tempdir().unwrap();
    let vault = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
    let mut extra = HashMap::new();
    extra.insert("path".into(), vault.display().to_string());

    let config = Config {
        env: KurultaiEnv::Dev,
        sources: vec![SourceConfig {
            name: "fixture".into(),
            kind: SourceKind::Filesystem,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        }],
        storage_path: dir.path().join("store.db").display().to_string(),
        embed_model: "test-model".into(),
        embed_dim: 8,
        reranker_model: None,
        poll_interval_secs: 60,
        openrouter_api_key_env: "KURULTAI_TEST_NO_KEY".into(),
    };

    // Ensure FTS-only (no key)
    std::env::remove_var("KURULTAI_TEST_NO_KEY");

    let rt = Runtime::open(config).unwrap();
    let report = rt.index(true).await.unwrap();
    assert_eq!(report.upserted, 3);
    assert_eq!(rt.store.count().unwrap(), 3);

    let hits = rt.query.search("zebra-migration-alpha", 5).await.unwrap();
    assert!(
        !hits.is_empty(),
        "expected FTS hit for zebra-migration-alpha"
    );
    assert!(hits[0].matched_by.iter().any(|m| m == "fts"));
    assert!(hits[0].atom.content.contains("zebra-migration-alpha"));
}
