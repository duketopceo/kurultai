use kurultai::embed::{reject_zero_vector, Embedder, FtsOnlyEmbedder, OpenRouterEmbedder};

#[test]
fn reject_zero_and_empty() {
    assert!(reject_zero_vector(&[]).is_err());
    assert!(reject_zero_vector(&[0.0, 0.0, 0.0]).is_err());
    assert!(reject_zero_vector(&[0.0, 0.1]).is_ok());
}

#[tokio::test]
async fn fts_only_mode_without_key() {
    let e = FtsOnlyEmbedder::new(8);
    assert_eq!(e.name(), "fts-only");
    assert!(e.embed("query").await.is_err());
}

#[test]
fn openrouter_constructs() {
    let e = OpenRouterEmbedder::new(
        "sk-test".into(),
        "openai/text-embedding-3-small".into(),
        1536,
    );
    assert_eq!(e.dim(), 1536);
}
