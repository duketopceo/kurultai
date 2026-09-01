use crate::connectors::ConnectorRegistry;
use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::quality::{apply_gate, evaluate};
use crate::store::Store;
use crate::types::{CorpusTier, SourceConfig, VisibilityScope};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Stats from a single index run.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub source: String,
    pub atoms_fetched: usize,
    pub atoms_indexed: usize,
    pub duration_ms: u128,
    pub full_sync: bool,
}

/// Orchestrates connector → embed → store flow.
pub struct IndexPipeline {
    store: Arc<dyn Store>,
    embedder: Arc<dyn Embedder>,
    /// Resolved source configs keyed by connector name, used to apply
    /// `default_corpus_tier` / `default_visibility_labels` / `default_visibility_scope` at ingest time.
    sources: HashMap<String, SourceConfig>,
}

impl IndexPipeline {
    pub fn new(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self {
            store,
            embedder,
            sources: HashMap::new(),
        }
    }

    /// Register source configs so [`IndexPipeline::index_connector`] can apply
    /// per-source `default_corpus_tier` / `default_visibility_labels` / `default_visibility_scope` defaults.
    pub fn register_sources(&mut self, configs: &[SourceConfig]) {
        for cfg in configs {
            self.sources.insert(cfg.name.clone(), cfg.clone());
        }
    }

    /// Index all registered connectors.
    pub async fn index_all(
        &self,
        registry: &ConnectorRegistry,
        full: bool,
    ) -> Result<Vec<IndexStats>> {
        let _span = tracing::info_span!("ingest_index_all", full, connectors = registry.len());
        let mut results = Vec::new();

        for (name, connector) in registry.iter() {
            let stats = self.index_connector(name, connector, full).await?;
            results.push(stats);
        }

        // Near-dupe off the write hot path — fire-and-forget after the batch.
        let store = Arc::clone(&self.store);
        let embedder = Arc::clone(&self.embedder);
        tokio::spawn(async move {
            match crate::quality::near_dupe::run_near_dupe_pass(store.as_ref(), &embedder).await {
                Ok((merges, pending)) if merges > 0 || pending > 0 => {
                    tracing::info!(merges, pending, "near-dupe pass complete");
                }
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "near-dupe pass failed"),
            }
        });

        Ok(results)
    }

    /// Index a single connector by name.
    pub async fn index_connector(
        &self,
        source_name: &str,
        connector: &dyn crate::connectors::Connector,
        full: bool,
    ) -> Result<IndexStats> {
        let started = Instant::now();
        tracing::info!(source = %source_name, full, "index started");

        let atoms = if full {
            connector.full_sync().await.map_err(|e| {
                KurultaiError::connector(source_name, format!("full_sync failed: {e}"))
            })?
        } else {
            connector
                .poll()
                .await
                .map_err(|e| KurultaiError::connector(source_name, format!("poll failed: {e}")))?
        };

        let fetched = atoms.len();
        tracing::debug!(source = %source_name, atoms = fetched, "connector returned atoms");

        if full {
            self.store
                .delete_source(source_name)
                .await
                .map_err(|e| KurultaiError::Store(format!("delete_source failed: {e}")))?;
        }

        let mut enriched = atoms;

        // Apply per-source corpus_tier / visibility_labels / visibility scope
        // defaults from the SourceConfig (only when the atom hasn't already set
        // them — i.e. the atom is still at the default Public tier with no
        // labels and default Personal scope). Frontmatter or connector-supplied
        // values win; this only fills in blanks.
        if let Some(cfg) = self.sources.get(source_name) {
            let default_tier = cfg.default_corpus_tier();
            let default_labels = cfg.default_visibility_labels();
            let default_scope = cfg.default_visibility_scope();
            if default_tier != CorpusTier::Public
                || !default_labels.is_empty()
                || default_scope != VisibilityScope::Personal
            {
                for atom in &mut enriched {
                    if atom.corpus_tier == CorpusTier::Public {
                        atom.corpus_tier = default_tier;
                    }
                    if atom.visibility_labels.is_empty() {
                        atom.visibility_labels = default_labels.clone();
                    }
                    if atom.visibility == VisibilityScope::Personal {
                        atom.visibility = default_scope;
                    }
                }
            }
        }

        // Gate each atom before embed (KTD7: don't pay embed on junk / quarantine).
        // Track in-batch content hashes so exact dupes in the same connector batch
        // quarantine even before commit.
        let mut batch_seen_hashes = std::collections::HashSet::new();
        for atom in &mut enriched {
            let hash = sha256_hex(&atom.content);
            let outcome = if batch_seen_hashes.contains(&hash) {
                crate::quality::GateOutcome::Quarantine {
                    reason: format!("exact_duplicate:batch:{hash}"),
                }
            } else {
                evaluate(self.store.as_ref(), atom).await?
            };
            batch_seen_hashes.insert(hash);
            apply_gate(atom, outcome);
            if atom.trust_lane == crate::types::TrustLane::Quarantine {
                atom.embedding = None;
            }
        }

        let mut skipped_embed = 0usize;
        if self.embedder.is_live() {
            // Embed trusted atoms only. Hash-skip: unchanged content_hash + existing
            // vector → leave embedding None so upsert preserves the stored vec row.
            let mut pending_idx = Vec::new();
            let mut pending_texts = Vec::new();
            for (i, atom) in enriched.iter().enumerate() {
                if atom.trust_lane != crate::types::TrustLane::Trusted {
                    continue;
                }
                if atom.embedding.is_some() {
                    continue;
                }
                let hash = sha256_hex(&atom.content);
                if self.store.has_fresh_embedding(&atom.id, &hash).await? {
                    skipped_embed += 1;
                    continue;
                }
                pending_idx.push(i);
                pending_texts.push(format!("{}\n{}", atom.title, atom.content));
            }
            if !pending_texts.is_empty() {
                let refs: Vec<&str> = pending_texts.iter().map(String::as_str).collect();
                let vectors = self.embedder.embed_batch(&refs).await.map_err(|e| {
                    KurultaiError::Embed(format!("batch embed failed for {source_name}: {e}"))
                })?;
                for (i, emb) in pending_idx.into_iter().zip(vectors) {
                    enriched[i].embedding = Some(emb);
                }
            }
            if skipped_embed > 0 {
                tracing::debug!(
                    source = %source_name,
                    skipped_embed,
                    "hash-skip re-embed for unchanged atoms"
                );
            }
        } else {
            tracing::debug!(
                source = %source_name,
                "embedder not live — indexing FTS-only (no vectors)"
            );
        }

        if !enriched.is_empty() {
            self.store
                .upsert_batch(&enriched)
                .await
                .map_err(|e| KurultaiError::Store(format!("upsert_batch failed: {e}")))?;
        }

        // Inbox tray finalization (trusted → processed/, quarantine → failed/).
        if let Err(e) = crate::connectors::inbox::finalize_inbox_batch(&enriched) {
            tracing::warn!(source = %source_name, error = %e, "inbox tray finalize failed");
        }

        let duration_ms = started.elapsed().as_millis();
        tracing::info!(
            source = %source_name,
            fetched,
            indexed = enriched.len(),
            duration_ms,
            "index complete"
        );

        Ok(IndexStats {
            source: source_name.to_string(),
            atoms_fetched: fetched,
            atoms_indexed: enriched.len(),
            duration_ms,
            full_sync: full,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::markdown::MarkdownConnector;
    use crate::connectors::Connector;
    use crate::embed::NullEmbedder;
    use crate::store::{SearchFilter, SqliteVecStore};
    use crate::types::{SourceConfig, SourceKind, VisibilityScope};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn index_fixture_vault_fts_hit() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        assert!(
            fixture.is_dir(),
            "missing fixture vault at {}",
            fixture.display()
        );

        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-pipe-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        let config = SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        };
        connector.init(&config).await.unwrap();

        let stats = pipeline
            .index_connector("notes", &connector, true)
            .await
            .unwrap();
        assert!(stats.atoms_indexed > 0);

        let hits = store
            .fts_search("KNOWN_PHRASE_KURULTAI_42", 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(!hits.is_empty(), "expected FTS hit on golden phrase");
        assert_eq!(hits[0].0.source, "notes");
    }

    #[tokio::test]
    async fn index_pipeline_applies_visibility_scope_from_source() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        assert!(
            fixture.is_dir(),
            "missing fixture vault at {}",
            fixture.display()
        );

        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-pipe-scope-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(NullEmbedder::new(4));
        let mut pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);

        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        extra.insert("default_visibility_scope".into(), "team".into());
        let config = SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        };
        pipeline.register_sources(std::slice::from_ref(&config));

        let mut connector = MarkdownConnector::new();
        connector.init(&config).await.unwrap();

        let stats = pipeline
            .index_connector("notes", &connector, true)
            .await
            .unwrap();
        assert!(stats.atoms_indexed > 0);

        let hits = store
            .fts_search("KNOWN_PHRASE_KURULTAI_42", 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(!hits.is_empty(), "expected FTS hit on golden phrase");
        for (atom, _score) in hits {
            assert_eq!(atom.visibility, VisibilityScope::Team);
        }
    }

    /// Counts embed_batch invocations for hash-skip verification.
    struct CountingEmbedder {
        dim: usize,
        calls: std::sync::Mutex<usize>,
    }

    impl CountingEmbedder {
        fn new(dim: usize) -> Self {
            Self {
                dim,
                calls: std::sync::Mutex::new(0),
            }
        }
        fn calls(&self) -> usize {
            *self.calls.lock().unwrap()
        }
    }

    #[async_trait::async_trait]
    impl Embedder for CountingEmbedder {
        fn name(&self) -> &str {
            "counting"
        }
        fn dim(&self) -> usize {
            self.dim
        }
        fn is_live(&self) -> bool {
            true
        }
        async fn embed(&self, text: &str) -> Result<Vec<f32>> {
            let mut batch = self.embed_batch(&[text]).await?;
            Ok(batch.pop().unwrap())
        }
        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            *self.calls.lock().unwrap() += 1;
            Ok(texts
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let mut v = vec![0.0f32; self.dim];
                    v[0] = 0.1 + (i as f32) * 0.01;
                    v
                })
                .collect())
        }
    }

    #[tokio::test]
    async fn hash_skip_avoids_second_embed_batch() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-hashskip-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder = Arc::new(CountingEmbedder::new(4));
        let pipeline = IndexPipeline::new(
            Arc::clone(&store) as Arc<dyn Store>,
            Arc::clone(&embedder) as Arc<dyn Embedder>,
        );

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap();

        pipeline
            .index_connector("notes", &connector, true)
            .await
            .unwrap();
        let first_calls = embedder.calls();
        assert!(first_calls >= 1);

        // Incremental re-index of unchanged vault — must not call embed_batch again.
        pipeline
            .index_connector("notes", &connector, false)
            .await
            .unwrap();
        assert_eq!(
            embedder.calls(),
            first_calls,
            "unchanged content must hash-skip re-embed"
        );
    }

    #[tokio::test]
    async fn live_embedder_writes_vectors_searchable() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault");
        let db_dir = std::env::temp_dir().join(format!(
            "kurultai-live-vec-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&db_dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(db_dir.join("store.db"), 4).unwrap());
        let embedder: Arc<dyn Embedder> = Arc::new(CountingEmbedder::new(4));
        assert!(embedder.is_live());
        let pipeline = IndexPipeline::new(Arc::clone(&store) as Arc<dyn Store>, embedder);

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), fixture.to_string_lossy().into_owned());
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap();

        pipeline
            .index_connector("notes", &connector, true)
            .await
            .unwrap();

        let q = vec![0.1f32, 0.0, 0.0, 0.0];
        let hits = store
            .vector_search(&q, 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(
            !hits.is_empty(),
            "live embedder must write searchable atoms_vec rows"
        );
    }
}
