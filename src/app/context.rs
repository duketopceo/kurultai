use crate::config::{ensure_storage_parent, expand_path, load_config, load_config_from};
use crate::connectors::ConnectorRegistry;
use crate::embed::{Embedder, OpenRouterEmbedder};
use crate::error::{KurultaiError, Result};
use crate::pipeline::IndexPipeline;
use crate::security::api_key_from_env_optional;
use crate::store::{migrations, SqliteVecStore, Store};
use crate::types::Config;
use std::path::Path;
use std::sync::Arc;

/// Top-level application context. Single wiring point for all subsystems.
pub struct App {
    pub config: Config,
    pub store: Arc<dyn Store>,
    pub embedder: Arc<dyn Embedder>,
    pub connectors: ConnectorRegistry,
    pub pipeline: IndexPipeline,
}

impl App {
    /// Bootstrap from default config path.
    pub async fn bootstrap() -> Result<Self> {
        let config = load_config()?;
        Self::from_config(config).await
    }

    /// Bootstrap from an explicit config file.
    pub async fn bootstrap_from(path: &Path) -> Result<Self> {
        let config = load_config_from(path)?;
        Self::from_config(config).await
    }

    async fn from_config(config: Config) -> Result<Self> {
        let storage_path = expand_path(&config.storage_path)?;
        ensure_storage_parent(&storage_path)?;

        tracing::debug!(storage = %storage_path.display(), "initializing store");
        let store: Arc<dyn Store> = Arc::new(SqliteVecStore::open(storage_path)?);

        let embedder = build_embedder(&config)?;
        let connectors = ConnectorRegistry::from_config(&config).await?;
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));

        tracing::info!(
            sources = connectors.len(),
            embedder = embedder.name(),
            dim = embedder.dim(),
            "app initialized"
        );

        Ok(Self {
            config,
            store,
            embedder,
            connectors,
            pipeline,
        })
    }

    pub async fn atom_count(&self) -> Result<u64> {
        self.store
            .count()
            .await
            .map_err(|e| KurultaiError::Store(e.to_string()))
    }

    pub fn schema_version(&self) -> i32 {
        migrations::CURRENT_SCHEMA_VERSION
    }
}

fn build_embedder(config: &Config) -> Result<Arc<dyn Embedder>> {
    // API keys come from env only — never from config files.
    let api_key = api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"));

    if api_key.is_none() {
        tracing::warn!(
            "no OPENROUTER_API_KEY or KURULTAI_API_KEY set — embedder will return zero vectors (dev mode)"
        );
    }

    let key = api_key.map(|k| k.expose().to_string()).unwrap_or_default();

    let embedder: Arc<dyn Embedder> = Arc::new(OpenRouterEmbedder::new(
        key,
        config.embed_model.clone(),
        config.embed_dim,
    ));

    Ok(embedder)
}
