use kurultai::app::Runtime;
use kurultai::mcp::tool_names;
use kurultai::types::{Config, KurultaiEnv, SourceConfig, SourceKind};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn tools_list_has_four() {
    let names = tool_names();
    assert_eq!(names.len(), 4);
    assert!(names.contains(&"search"));
    assert!(names.contains(&"read_atom"));
    assert!(names.contains(&"status"));
    assert!(names.contains(&"reindex"));
}

#[tokio::test]
async fn search_returns_citation_shaped_payload() {
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
        openrouter_api_key_env: "KURULTAI_MCP_SMOKE_NO_KEY".into(),
    };
    std::env::remove_var("KURULTAI_MCP_SMOKE_NO_KEY");
    let rt = Runtime::open(config).unwrap();
    rt.index(true).await.unwrap();
    let hits = rt.query.search("zebra-migration-alpha", 5).await.unwrap();
    assert!(!hits.is_empty());
    let top = &hits[0];
    assert!(!top.atom.id.is_empty());
    assert!(!top.atom.title.is_empty());
    assert!(top.atom.source_uri.is_some());
    assert!(top.matched_by.iter().any(|m| m == "fts"));
}
