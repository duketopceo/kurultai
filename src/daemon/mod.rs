//! Long-running daemon: HTTP brain API + optional background poll indexing.

use crate::connectors::ConnectorRegistry;
use crate::error::Result;
use crate::http;
use crate::mcp::BrainService;
use crate::pipeline::IndexPipeline;
use std::sync::Arc;
use std::time::Duration;

/// Clamp poll interval to at least 1 second (shared by CLI + daemon).
pub fn normalize_poll_interval_secs(secs: u64) -> u64 {
    secs.max(1)
}

/// Run HTTP serve and (optionally) an incremental poll loop until the server stops.
pub async fn run(
    brain: BrainService,
    pipeline: IndexPipeline,
    connectors: ConnectorRegistry,
    port: u16,
    poll: bool,
    poll_interval_secs: u64,
) -> Result<()> {
    let interval = Duration::from_secs(normalize_poll_interval_secs(poll_interval_secs));
    let poll_handle = if poll {
        let pipeline = Arc::new(pipeline);
        let connectors = Arc::new(connectors);
        tracing::info!(
            secs = interval.as_secs(),
            "daemon poll loop enabled (incremental index)"
        );
        Some(tokio::spawn(async move {
            poll_loop(pipeline, connectors, interval).await;
        }))
    } else {
        tracing::info!("daemon poll loop disabled (--no-poll)");
        drop(pipeline);
        drop(connectors);
        None
    };

    let serve_result = http::serve(brain, port).await;

    if let Some(handle) = poll_handle {
        handle.abort();
        let _ = handle.await;
    }

    serve_result
}

/// Immediate first poll, then sleep `interval` between cycles. Soft-fails on errors.
pub(crate) async fn poll_loop(
    pipeline: Arc<IndexPipeline>,
    connectors: Arc<ConnectorRegistry>,
    interval: Duration,
) {
    loop {
        match poll_once(pipeline.as_ref(), connectors.as_ref()).await {
            Ok(n) => {
                if n > 0 {
                    tracing::info!(sources = n, "daemon poll cycle complete");
                } else {
                    tracing::debug!("daemon poll cycle complete (no sources)");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "daemon poll cycle failed; will retry");
            }
        }
        tokio::time::sleep(interval).await;
    }
}

/// Run one incremental index across all connectors. Returns source count.
pub async fn poll_once(pipeline: &IndexPipeline, connectors: &ConnectorRegistry) -> Result<usize> {
    let stats = pipeline.index_all(connectors, false).await?;
    Ok(stats.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::{Embedder, NullEmbedder};
    use crate::error::KurultaiError;
    use crate::mcp::BrainService;
    use crate::rerank::NullReranker;
    use crate::store::{SqliteVecStore, Store};
    use crate::synthesize::ExtractiveSynthesizer;
    use crate::types::{KnowledgeAtom, SourceConfig, SourceKind};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn poll_once_indexes_markdown_fixture() {
        let root = format!("{}/tests/fixtures/vault", env!("CARGO_MANIFEST_DIR"));
        let mut connector = MarkdownConnector::new();
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::from([("root_path".into(), root)]),
            })
            .await
            .unwrap();

        let mut registry = ConnectorRegistry::new();
        registry.register("notes".into(), Box::new(connector));

        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);

        let n = poll_once(&pipeline, &registry).await.unwrap();
        assert_eq!(n, 1);
        assert!(store.count().await.unwrap() > 0);
    }

    struct CountingConnector {
        polls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Connector for CountingConnector {
        fn name(&self) -> &str {
            "count"
        }
        async fn init(&mut self, _config: &SourceConfig) -> Result<()> {
            Ok(())
        }
        async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![])
        }
        async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
            self.poll().await
        }
    }

    struct FailConnector;

    #[async_trait]
    impl Connector for FailConnector {
        fn name(&self) -> &str {
            "fail"
        }
        async fn init(&mut self, _config: &SourceConfig) -> Result<()> {
            Ok(())
        }
        async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
            Err(KurultaiError::connector("fail", "boom"))
        }
        async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
            self.poll().await
        }
    }

    fn empty_brain(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> BrainService {
        BrainService::new(
            store,
            embedder,
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        )
    }

    #[tokio::test]
    async fn poll_loop_runs_immediate_first_cycle() {
        let polls = Arc::new(AtomicUsize::new(0));
        let mut registry = ConnectorRegistry::new();
        registry.register(
            "count".into(),
            Box::new(CountingConnector {
                polls: Arc::clone(&polls),
            }),
        );
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = Arc::new(IndexPipeline::new(
            Arc::clone(&store) as Arc<dyn Store>,
            embedder,
        ));
        let connectors = Arc::new(registry);

        let handle = tokio::spawn(poll_loop(pipeline, connectors, Duration::from_secs(3600)));
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(
            polls.load(Ordering::SeqCst) >= 1,
            "expected immediate first poll before interval sleep"
        );
        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn poll_once_propagates_connector_error() {
        let mut registry = ConnectorRegistry::new();
        registry.register("fail".into(), Box::new(FailConnector));
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);
        let err = poll_once(&pipeline, &registry).await.unwrap_err();
        assert!(err.to_string().contains("boom"));
    }

    #[tokio::test]
    async fn soft_fail_poll_keeps_http_health() {
        let mut registry = ConnectorRegistry::new();
        registry.register("fail".into(), Box::new(FailConnector));
        let dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let handle =
            tokio::spawn(async move { run(brain, pipeline, registry, port, true, 1).await });
        tokio::time::sleep(Duration::from_millis(300)).await;

        let url = format!("http://127.0.0.1:{port}/health");
        let body: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
        assert_eq!(body["ok"], true);

        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn no_poll_still_serves_health() {
        let registry = ConnectorRegistry::new();
        let dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let handle =
            tokio::spawn(async move { run(brain, pipeline, registry, port, false, 1).await });
        tokio::time::sleep(Duration::from_millis(200)).await;

        let url = format!("http://127.0.0.1:{port}/health");
        let resp = reqwest::get(&url).await.unwrap();
        assert!(resp.status().is_success());

        handle.abort();
        let _ = handle.await;
    }

    #[test]
    fn normalize_poll_interval_clamps_zero() {
        assert_eq!(normalize_poll_interval_secs(0), 1);
        assert_eq!(normalize_poll_interval_secs(300), 300);
    }
}
