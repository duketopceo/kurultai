//! Long-running daemon: HTTP brain API + optional background poll / notify watch.

use crate::connectors::ConnectorRegistry;
use crate::error::Result;
use crate::http;
use crate::mcp::BrainService;
use crate::pipeline::IndexPipeline;
use crate::security::validate_readable_path;
use crate::types::{SourceConfig, SourceKind};
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Debounce window before a notify burst triggers one incremental index.
pub const WATCH_DEBOUNCE: Duration = Duration::from_millis(300);

/// Delay before re-arming notify after the watch task ends unexpectedly.
const WATCH_REARM_DELAY: Duration = Duration::from_secs(2);

/// Background-index options for [`run`].
#[derive(Debug, Clone)]
pub struct DaemonOptions {
    pub port: u16,
    pub poll: bool,
    pub poll_interval_secs: u64,
    pub watch: bool,
    pub watch_roots: Vec<PathBuf>,
}

/// Clamp poll interval to at least 1 second (shared by CLI + daemon).
pub fn normalize_poll_interval_secs(secs: u64) -> u64 {
    secs.max(1)
}

/// Collect existing directories to watch from enabled markdown/github sources.
///
/// Uses `root_path`, falling back to deprecated markdown `vault_path`.
/// Paths go through [`validate_readable_path`] (canonicalize + traversal checks).
/// Skips rejected/missing/non-dir paths. Deduplicates on canonical form.
pub fn watch_roots_from_sources(sources: &[SourceConfig]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for src in sources {
        if !src.enabled {
            continue;
        }
        if !matches!(src.kind, SourceKind::Markdown | SourceKind::GitHub) {
            continue;
        }
        let path = src
            .extra
            .get("root_path")
            .or_else(|| src.extra.get("vault_path"));
        let Some(path) = path else {
            continue;
        };
        let Ok(resolved) = validate_readable_path(path, &src.name) else {
            tracing::warn!(
                source = %src.name,
                path,
                "skipping watch root (failed path validation)"
            );
            continue;
        };
        if !resolved.is_dir() {
            tracing::warn!(
                source = %src.name,
                path = %resolved.display(),
                "skipping watch root (not a directory)"
            );
            continue;
        }
        if !roots.iter().any(|r| r == &resolved) {
            roots.push(resolved);
        }
    }
    roots
}

/// Aborts background tasks if `run` is cancelled before explicit shutdown.
struct AbortOnDrop(Vec<JoinHandle<()>>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        for handle in self.0.drain(..) {
            handle.abort();
        }
    }
}

/// Serve the brain HTTP API with optional incremental poll and notify watch loops.
///
/// When `opts.poll` is true, spawns a background task that calls [`poll_once`]
/// immediately, then every `poll_interval_secs` (clamped to ≥1s). When
/// `opts.watch` is true and `watch_roots` is non-empty, spawns a notify watcher
/// that debounces filesystem events and triggers the same incremental index.
/// Poll/watch share a single-flight lock. Errors soft-fail and do not stop HTTP.
/// Aborting this future cancels background tasks via [`AbortOnDrop`].
pub async fn run(
    brain: BrainService,
    pipeline: IndexPipeline,
    connectors: ConnectorRegistry,
    opts: DaemonOptions,
) -> Result<()> {
    let interval = Duration::from_secs(normalize_poll_interval_secs(opts.poll_interval_secs));
    let pipeline = Arc::new(pipeline);
    let connectors = Arc::new(connectors);
    let flight = Arc::new(Mutex::new(()));
    let mut bg = AbortOnDrop(Vec::new());

    if opts.poll {
        tracing::info!(
            secs = interval.as_secs(),
            "daemon poll loop enabled (incremental index)"
        );
        let pipeline = Arc::clone(&pipeline);
        let connectors = Arc::clone(&connectors);
        let flight = Arc::clone(&flight);
        bg.0.push(tokio::spawn(async move {
            poll_loop(pipeline, connectors, interval, flight).await;
        }));
    } else {
        tracing::info!("daemon poll loop disabled (--no-poll)");
    }

    if opts.watch {
        if opts.watch_roots.is_empty() {
            tracing::info!("daemon watch enabled but no watchable roots; skipping");
        } else {
            tracing::info!(
                roots = opts.watch_roots.len(),
                "daemon notify watch enabled (debounced incremental index)"
            );
            let pipeline = Arc::clone(&pipeline);
            let connectors = Arc::clone(&connectors);
            let flight = Arc::clone(&flight);
            let watch_roots = opts.watch_roots;
            bg.0.push(tokio::spawn(async move {
                watch_loop(pipeline, connectors, watch_roots, flight).await;
            }));
        }
    } else {
        tracing::info!("daemon notify watch disabled (--no-watch)");
    }

    let serve_result = http::serve(brain, opts.port).await;

    for handle in bg.0.drain(..) {
        handle.abort();
        let _ = handle.await;
    }

    serve_result
}

async fn run_poll_cycle(
    pipeline: &IndexPipeline,
    connectors: &ConnectorRegistry,
    flight: &Mutex<()>,
    label: &'static str,
) {
    let _guard = flight.lock().await;
    match poll_once(pipeline, connectors).await {
        Ok(n) => {
            if n > 0 {
                tracing::info!(connectors = n, label, "index cycle complete");
            } else {
                tracing::debug!(label, "index cycle complete (no connectors)");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, label, "index cycle failed; will retry");
        }
    }
}

/// Immediate first poll, then sleep `interval` between cycles. Soft-fails on errors.
pub(crate) async fn poll_loop(
    pipeline: Arc<IndexPipeline>,
    connectors: Arc<ConnectorRegistry>,
    interval: Duration,
    flight: Arc<Mutex<()>>,
) {
    loop {
        run_poll_cycle(
            pipeline.as_ref(),
            connectors.as_ref(),
            flight.as_ref(),
            "daemon poll",
        )
        .await;
        tokio::time::sleep(interval).await;
    }
}

/// Watch `roots` with notify; debounce bursts then incremental index. Soft-fails.
///
/// Re-arms after watcher setup failures or channel close so a deleted/recreated
/// root (or transient inotify limit) does not permanently disable freshness.
pub(crate) async fn watch_loop(
    pipeline: Arc<IndexPipeline>,
    connectors: Arc<ConnectorRegistry>,
    roots: Vec<PathBuf>,
    flight: Arc<Mutex<()>>,
) {
    loop {
        watch_session(
            Arc::clone(&pipeline),
            Arc::clone(&connectors),
            &roots,
            Arc::clone(&flight),
        )
        .await;
        tracing::warn!(
            delay_secs = WATCH_REARM_DELAY.as_secs(),
            "daemon notify watch ended; re-arming"
        );
        tokio::time::sleep(WATCH_REARM_DELAY).await;
    }
}

async fn watch_session(
    pipeline: Arc<IndexPipeline>,
    connectors: Arc<ConnectorRegistry>,
    roots: &[PathBuf],
    flight: Arc<Mutex<()>>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = match notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    }) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!(error = %e, "daemon notify watcher failed to start");
            return;
        }
    };

    let mut watching = 0usize;
    for root in roots {
        match watcher.watch(root.as_path(), RecursiveMode::Recursive) {
            Ok(()) => {
                watching += 1;
                tracing::info!(path = %root.display(), "watching source root");
            }
            Err(e) => {
                tracing::warn!(path = %root.display(), error = %e, "failed to watch root");
            }
        }
    }
    if watching == 0 {
        tracing::warn!("daemon notify watch: no roots registered");
        return;
    }

    // Keep watcher alive for the duration of this session.
    let _watcher = watcher;

    loop {
        match rx.recv().await {
            Some(Ok(_event)) => {}
            Some(Err(e)) => {
                tracing::warn!(error = %e, "daemon notify event error");
                continue;
            }
            None => return,
        }

        let deadline = Instant::now() + WATCH_DEBOUNCE;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Some(Ok(_))) => {}
                Ok(Some(Err(e))) => {
                    tracing::warn!(error = %e, "daemon notify event error");
                }
                Ok(None) => return,
                Err(_) => break,
            }
        }
        while rx.try_recv().is_ok() {}

        run_poll_cycle(
            pipeline.as_ref(),
            connectors.as_ref(),
            flight.as_ref(),
            "daemon watch",
        )
        .await;
    }
}

/// Run one incremental index (`full = false`) across all registered connectors.
///
/// Returns `Ok(connector_count)` where `connector_count` is the number of
/// connectors processed by [`IndexPipeline::index_all`]. Propagates the first
/// connector/pipeline error (callers that soft-fail should catch it).
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
    use std::fs;
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

    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn opts(port: u16, poll: bool, watch: bool, watch_roots: Vec<PathBuf>) -> DaemonOptions {
        DaemonOptions {
            port,
            poll,
            poll_interval_secs: if poll { 1 } else { 60 },
            watch,
            watch_roots,
        }
    }

    async fn wait_healthy(port: u16) {
        let url = format!("http://127.0.0.1:{port}/health");
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Ok(resp) = reqwest::get(&url).await {
                if resp.status().is_success() {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("/health not ready on port {port}");
    }

    async fn wait_polls(polls: &AtomicUsize, min: usize, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if polls.load(Ordering::SeqCst) >= min {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!(
            "expected >= {min} polls within {:?}, got {}",
            timeout,
            polls.load(Ordering::SeqCst)
        );
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
        let flight = Arc::new(Mutex::new(()));

        let handle = tokio::spawn(poll_loop(
            pipeline,
            connectors,
            Duration::from_secs(1),
            flight,
        ));
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            polls.load(Ordering::SeqCst),
            1,
            "expected exactly one immediate poll before interval elapses"
        );
        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert!(
            polls.load(Ordering::SeqCst) >= 2,
            "expected a second poll after the 1s interval"
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
        let port = free_port();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, true, false, Vec::new()),
            )
            .await
        });
        wait_healthy(port).await;

        let url = format!("http://127.0.0.1:{port}/health");
        for _ in 0..2 {
            tokio::time::sleep(Duration::from_millis(400)).await;
            let body: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
            assert_eq!(body["ok"], true);
        }

        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn aborting_run_stops_further_polls() {
        let polls = Arc::new(AtomicUsize::new(0));
        let mut registry = ConnectorRegistry::new();
        registry.register(
            "count".into(),
            Box::new(CountingConnector {
                polls: Arc::clone(&polls),
            }),
        );
        let dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);
        let port = free_port();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, true, false, Vec::new()),
            )
            .await
        });
        wait_polls(&polls, 1, Duration::from_secs(2)).await;
        let after_first = polls.load(Ordering::SeqCst);

        handle.abort();
        let _ = handle.await;
        let frozen = polls.load(Ordering::SeqCst);
        assert!(frozen >= after_first);
        tokio::time::sleep(Duration::from_millis(1500)).await;
        assert_eq!(
            polls.load(Ordering::SeqCst),
            frozen,
            "poll loop must not continue after run is aborted"
        );
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
        let port = free_port();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, false, false, Vec::new()),
            )
            .await
        });
        wait_healthy(port).await;

        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn notify_write_triggers_incremental_poll() {
        let watch_dir = tempfile::tempdir().unwrap();
        assert!(watch_dir.path().is_dir());

        let polls = Arc::new(AtomicUsize::new(0));
        let mut registry = ConnectorRegistry::new();
        registry.register(
            "count".into(),
            Box::new(CountingConnector {
                polls: Arc::clone(&polls),
            }),
        );
        let store_dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(store_dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);
        let port = free_port();
        let root = watch_dir.path().canonicalize().unwrap();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, false, true, vec![root]),
            )
            .await
        });
        wait_healthy(port).await;
        assert_eq!(
            polls.load(Ordering::SeqCst),
            0,
            "watch must not index until a filesystem event"
        );

        fs::write(watch_dir.path().join("note.md"), "hello watch").unwrap();
        wait_polls(&polls, 1, WATCH_DEBOUNCE + Duration::from_secs(2)).await;

        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn aborting_run_stops_further_watch_polls() {
        let watch_dir = tempfile::tempdir().unwrap();
        let polls = Arc::new(AtomicUsize::new(0));
        let mut registry = ConnectorRegistry::new();
        registry.register(
            "count".into(),
            Box::new(CountingConnector {
                polls: Arc::clone(&polls),
            }),
        );
        let store_dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(store_dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);
        let port = free_port();
        let root = watch_dir.path().canonicalize().unwrap();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, false, true, vec![root]),
            )
            .await
        });
        wait_healthy(port).await;
        fs::write(watch_dir.path().join("a.md"), "one").unwrap();
        wait_polls(&polls, 1, WATCH_DEBOUNCE + Duration::from_secs(2)).await;

        handle.abort();
        let _ = handle.await;
        let frozen = polls.load(Ordering::SeqCst);
        fs::write(watch_dir.path().join("b.md"), "two").unwrap();
        tokio::time::sleep(WATCH_DEBOUNCE + Duration::from_millis(800)).await;
        assert_eq!(
            polls.load(Ordering::SeqCst),
            frozen,
            "watch loop must not continue after run is aborted"
        );
    }

    #[tokio::test]
    async fn notify_burst_keeps_http_and_soft_indexes() {
        let watch_dir = tempfile::tempdir().unwrap();
        let polls = Arc::new(AtomicUsize::new(0));
        let mut registry = ConnectorRegistry::new();
        registry.register(
            "count".into(),
            Box::new(CountingConnector {
                polls: Arc::clone(&polls),
            }),
        );
        let store_dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(store_dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);
        let port = free_port();
        let root = watch_dir.path().canonicalize().unwrap();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, false, true, vec![root]),
            )
            .await
        });
        wait_healthy(port).await;

        for i in 0..20 {
            fs::write(watch_dir.path().join(format!("n{i}.md")), format!("x{i}")).unwrap();
        }
        wait_polls(&polls, 1, WATCH_DEBOUNCE + Duration::from_secs(3)).await;
        wait_healthy(port).await;

        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test]
    async fn no_watch_still_serves_health() {
        let registry = ConnectorRegistry::new();
        let dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn Store> =
            Arc::new(SqliteVecStore::open(dir.path().join("s.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let brain = empty_brain(store, embedder);
        let port = free_port();
        let watch_dir = tempfile::tempdir().unwrap();

        let handle = tokio::spawn(async move {
            run(
                brain,
                pipeline,
                registry,
                opts(port, false, false, vec![watch_dir.path().to_path_buf()]),
            )
            .await
        });
        wait_healthy(port).await;

        handle.abort();
        let _ = handle.await;
    }

    #[test]
    fn normalize_poll_interval_clamps_zero() {
        assert_eq!(normalize_poll_interval_secs(0), 1);
        assert_eq!(normalize_poll_interval_secs(300), 300);
    }

    #[test]
    fn watch_roots_from_enabled_markdown_and_github() {
        let dir = tempfile::tempdir().unwrap();
        let md = dir.path().join("notes");
        let gh = dir.path().join("code");
        let vault = dir.path().join("vault");
        fs::create_dir_all(&md).unwrap();
        fs::create_dir_all(&gh).unwrap();
        fs::create_dir_all(&vault).unwrap();
        let md = md.canonicalize().unwrap();
        let gh = gh.canonicalize().unwrap();
        let vault = vault.canonicalize().unwrap();
        let missing = dir.path().join("gone");

        let sources = vec![
            SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::from([("root_path".into(), md.to_string_lossy().into())]),
            },
            SourceConfig {
                name: "code".into(),
                kind: SourceKind::GitHub,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::from([("root_path".into(), gh.to_string_lossy().into())]),
            },
            SourceConfig {
                name: "legacy".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::from([("vault_path".into(), vault.to_string_lossy().into())]),
            },
            SourceConfig {
                name: "pond".into(),
                kind: SourceKind::Pond,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::new(),
            },
            SourceConfig {
                name: "missing".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::from([("root_path".into(), missing.to_string_lossy().into())]),
            },
            SourceConfig {
                name: "off".into(),
                kind: SourceKind::Markdown,
                enabled: false,
                poll_interval_secs: 60,
                extra: HashMap::from([("root_path".into(), md.to_string_lossy().into())]),
            },
        ];

        let roots = watch_roots_from_sources(&sources);
        assert_eq!(roots.len(), 3);
        assert!(roots.contains(&md));
        assert!(roots.contains(&gh));
        assert!(roots.contains(&vault));
    }
}
