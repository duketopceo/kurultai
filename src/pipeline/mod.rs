use crate::connectors::ConnectorRegistry;
use crate::embed::Embedder;
use crate::error::{KurultaiError, Result};
use crate::store::Store;
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
}

impl IndexPipeline {
    pub fn new(store: Arc<dyn Store>, embedder: Arc<dyn Embedder>) -> Self {
        Self { store, embedder }
    }

    /// Index all registered connectors.
    pub async fn index_all(
        &self,
        registry: &ConnectorRegistry,
        full: bool,
    ) -> Result<Vec<IndexStats>> {
        let mut results = Vec::new();

        for (name, connector) in registry.iter() {
            let stats = self.index_connector(name, connector, full).await?;
            results.push(stats);
        }

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

        if self.embedder.is_live() {
            // Collect texts that need embeddings, batch call, assign back.
            let mut pending_idx = Vec::new();
            let mut pending_texts = Vec::new();
            for (i, atom) in enriched.iter().enumerate() {
                if atom.embedding.is_none() {
                    pending_idx.push(i);
                    pending_texts.push(format!("{}\n{}", atom.title, atom.content));
                }
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
    use crate::store::SqliteVecStore;
    use crate::types::{SourceConfig, SourceKind};
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
            .fts_search("KNOWN_PHRASE_KURULTAI_42", 5)
            .await
            .unwrap();
        assert!(!hits.is_empty(), "expected FTS hit on golden phrase");
        assert_eq!(hits[0].0.source, "notes");
    }
}
