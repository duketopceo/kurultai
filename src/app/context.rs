use crate::config::{ensure_storage_parent, expand_path, load_config_with_env};
use crate::connectors::ConnectorRegistry;
use crate::embed::{Embedder, OpenRouterEmbedder};
use crate::environment::Environment;
use crate::error::{KurultaiError, Result};
use crate::pipeline::IndexPipeline;
use crate::query::{HybridQueryEngine, QueryEngine};
use crate::security::api_key_from_env_optional;
use crate::store::{migrations, SqliteVecStore, Store};
use crate::types::Config;
use std::path::Path;
use std::sync::Arc;

/// Top-level application context. Single wiring point for all subsystems.
pub struct App {
    pub config: Config,
    pub environment: Environment,
    pub store: Arc<dyn Store>,
    pub embedder: Arc<dyn Embedder>,
    pub connectors: ConnectorRegistry,
    pub pipeline: IndexPipeline,
    pub query_engine: Arc<dyn QueryEngine>,
}

impl App {
    /// Bootstrap from default config path.
    pub async fn bootstrap(env_override: Option<&str>) -> Result<Self> {
        let config = load_config_with_env(None, env_override)?;
        Self::from_config(config).await
    }

    /// Bootstrap from an explicit config file.
    pub async fn bootstrap_from(path: &Path, env_override: Option<&str>) -> Result<Self> {
        let config = load_config_with_env(Some(path), env_override)?;
        Self::from_config(config).await
    }

    async fn from_config(config: Config) -> Result<Self> {
        let environment = config.environment;
        let storage_path = expand_path(&config.storage_path)?;
        ensure_storage_parent(&storage_path)?;

        tracing::debug!(storage = %storage_path.display(), "initializing store");
        let store: Arc<dyn Store> = Arc::new(SqliteVecStore::open(
            storage_path,
            &config.embed_model,
            config.embed_dim,
        )?);

        let embedder = build_embedder(&config, environment)?;
        let connectors = ConnectorRegistry::from_config(&config).await?;
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        let query_engine: Arc<dyn QueryEngine> = Arc::new(HybridQueryEngine::new(
            Arc::clone(&store),
            Arc::clone(&embedder),
        ));

        tracing::info!(
            env = %environment,
            sources = connectors.len(),
            embedder = embedder.name(),
            dim = embedder.dim(),
            mode = ?embedder.mode(),
            "app initialized"
        );

        Ok(Self {
            config,
            environment,
            store,
            embedder,
            connectors,
            pipeline,
            query_engine,
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

fn build_embedder(config: &Config, env: Environment) -> Result<Arc<dyn Embedder>> {
    // API keys come from env only — never from config files.
    let api_key = api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"));

    let key = match api_key {
        Some(k) => k.expose().to_string(),
        None if env.requires_embed_api_key() => {
            return Err(KurultaiError::security(
                "OPENROUTER_API_KEY or KURULTAI_API_KEY is required in staging/production",
            ))
        }
        None => {
            tracing::warn!(
                env = %env,
                "no OPENROUTER_API_KEY or KURULTAI_API_KEY set — running in FTS-only mode"
            );
            String::new()
        }
    };

    let embedder: Arc<dyn Embedder> = Arc::new(OpenRouterEmbedder::new(
        key,
        config.embed_model.clone(),
        config.embed_dim,
    ));

    Ok(embedder)
}
