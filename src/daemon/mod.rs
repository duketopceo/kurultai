//! Long-running daemon: HTTP brain API + optional background poll indexing.

use crate::connectors::ConnectorRegistry;
use crate::error::Result;
use crate::http;
use crate::mcp::BrainService;
use crate::pipeline::IndexPipeline;
use std::sync::Arc;
use std::time::Duration;

/// Run HTTP serve and (optionally) an incremental poll loop until the server stops.
pub async fn run(
    brain: BrainService,
    pipeline: IndexPipeline,
    connectors: ConnectorRegistry,
    port: u16,
    poll: bool,
    poll_interval_secs: u64,
) -> Result<()> {
    let interval = Duration::from_secs(poll_interval_secs.max(1));
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
        // Drop unused pipeline/connectors when not polling.
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
async fn poll_loop(
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
    use crate::store::{SqliteVecStore, Store};
    use crate::types::{SourceConfig, SourceKind};
    use std::collections::HashMap;

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
}
