use crate::connectors::ConnectorRegistry;
use crate::embed::{EmbedMode, Embedder};
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

        if full && fetched > 0 {
            self.store
                .delete_source(source_name)
                .await
                .map_err(|e| KurultaiError::Store(format!("delete_source failed: {e}")))?;
        }

        let mut enriched: Vec<crate::types::KnowledgeAtom> = Vec::with_capacity(atoms.len());
        let mut seen_source_ids: Vec<String> = Vec::with_capacity(atoms.len());

        for mut atom in atoms {
            seen_source_ids.push(atom.source_id.clone());

            if self
                .store
                .content_hash_unchanged(&atom.id, &atom.content_hash)
                .await
                .map_err(|e| KurultaiError::Store(format!("content_hash check failed: {e}")))?
            {
                tracing::trace!(id = %atom.id, "content hash unchanged; skip");
                continue;
            }

            if self.embedder.mode() == EmbedMode::Full && atom.embedding.is_none() {
                let text = format!("{}\n{}", atom.title, atom.content);
                match self.embedder.embed(&text).await {
                    Ok(embedding) => {
                        atom.embedding = Some(embedding);
                    }
                    Err(e) => {
                        tracing::warn!(id = %atom.id, error = %e, "embed failed; indexing without embedding");
                        // FTS-only degrade for this atom.
                    }
                }
            }

            enriched.push(atom);
        }

        if !enriched.is_empty() {
            self.store
                .upsert_batch(&enriched)
                .await
                .map_err(|e| KurultaiError::Store(format!("upsert_batch failed: {e}")))?;
        }

        if !full && !seen_source_ids.is_empty() {
            let keep: Vec<&str> = seen_source_ids.iter().map(String::as_str).collect();
            self.store
                .delete_source_ids_not_in(source_name, &keep)
                .await
                .map_err(|e| KurultaiError::Store(format!("orphan cleanup failed: {e}")))?;
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
