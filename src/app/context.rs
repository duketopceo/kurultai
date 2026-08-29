use crate::config::{ensure_storage_parent, expand_path, load_config_with_env};
use crate::connectors::ConnectorRegistry;
use crate::embed::{Embedder, NullEmbedder, OpenRouterEmbedder};
use crate::environment::Environment;
use crate::error::{KurultaiError, Result};
use crate::pipeline::IndexPipeline;
use crate::rerank::{NullReranker, OpenRouterReranker, Reranker};
use crate::security::api_key_from_env_optional;
use crate::store::{migrations, SqliteVecStore, Store};
use crate::synthesize::{synthesizer_from_env, Synthesizer};
use crate::types::Config;
use std::path::Path;
use std::sync::Arc;

/// Top-level application context. Single wiring point for all subsystems.
pub struct App {
    pub config: Config,
    pub environment: Environment,
    pub store: Arc<dyn Store>,
    pub embedder: Arc<dyn Embedder>,
    pub reranker: Arc<dyn Reranker>,
    pub synthesizer: Arc<dyn Synthesizer>,
    pub connectors: ConnectorRegistry,
    pub pipeline: IndexPipeline,
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

        let store: Arc<dyn Store> = if crate::features::enabled("hub") {
            let url = crate::store::database_url_from_env().ok_or_else(|| {
                KurultaiError::config(
                    "KURULTAI_FEATURE_HUB=1 requires DATABASE_URL or KURULTAI_DATABASE_URL",
                )
            })?;
            tracing::info!(
                embed_dim = config.embed_dim,
                "initializing hub Postgres store"
            );
            crate::store::open_hub_store(&url, config.embed_dim).await?
        } else {
            let storage_path = expand_path(&config.storage_path)?;
            ensure_storage_parent(&storage_path)?;
            tracing::debug!(
                storage = %storage_path.display(),
                embed_dim = config.embed_dim,
                "initializing store"
            );
            Arc::new(SqliteVecStore::open(storage_path, config.embed_dim)?)
        };

        let embedder = build_embedder(&config, environment)?;
        let reranker = build_reranker(&config);
        let synthesizer = synthesizer_from_env(None);
        let connectors = ConnectorRegistry::from_config(&config).await?;
        let mut pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));
        pipeline.register_sources(&config.sources);

        tracing::info!(
            env = %environment,
            sources = connectors.len(),
            embedder = embedder.name(),
            reranker = reranker.name(),
            synthesizer = synthesizer.name(),
            dim = embedder.dim(),
            "app initialized"
        );

        Ok(Self {
            config,
            environment,
            store,
            embedder,
            reranker,
            synthesizer,
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

pub fn build_embedder(config: &Config, env: Environment) -> Result<Arc<dyn Embedder>> {
    // API keys come from env only — never from config files.
    let api_key = api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"));

    match api_key {
        Some(key) => {
            let embedder: Arc<dyn Embedder> = Arc::new(OpenRouterEmbedder::new(
                key.expose().to_string(),
                config.embed_model.clone(),
                config.embed_dim,
            ));
            Ok(embedder)
        }
        None if wants_local_embed(config) => build_local_embedder(config),
        None => {
            tracing::warn!(
                env = %env,
                "no OPENROUTER_API_KEY or KURULTAI_API_KEY — FTS-only mode (NullEmbedder)"
            );
            Ok(Arc::new(NullEmbedder::new(config.embed_dim)))
        }
    }
}

fn wants_local_embed(config: &Config) -> bool {
    config
        .embed_backend
        .as_deref()
        .is_some_and(|b| b.eq_ignore_ascii_case("local"))
}

#[cfg(feature = "local-embed")]
fn resolve_local_model(config: &Config) -> &str {
    if config.embed_model.starts_with("openai/") || config.embed_model.contains("text-embedding") {
        "AllMiniLML6V2"
    } else {
        config.embed_model.as_str()
    }
}

fn build_local_embedder(config: &Config) -> Result<Arc<dyn Embedder>> {
    #[cfg(feature = "local-embed")]
    {
        let model = resolve_local_model(config);
        tracing::info!(model, dim = config.embed_dim, "using local ONNX embedder");
        let local = crate::embed::LocalEmbedder::try_new(model, config.embed_dim)?;
        Ok(Arc::new(local))
    }
    #[cfg(not(feature = "local-embed"))]
    {
        let _ = config;
        Err(KurultaiError::config(
            "embed.backend = \"local\" requires building with --features local-embed",
        ))
    }
}

fn build_reranker(config: &Config) -> Arc<dyn Reranker> {
    let Some(model) = config
        .reranker_model
        .as_ref()
        .filter(|m| !m.trim().is_empty())
    else {
        return Arc::new(NullReranker::new());
    };
    let api_key = api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"));
    match api_key {
        Some(key) => Arc::new(OpenRouterReranker::new(
            key.expose().to_string(),
            model.clone(),
        )),
        None => {
            tracing::warn!("reranker_model set but no API key — rerank disabled");
            Arc::new(NullReranker::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Config;

    fn sample_config(backend: Option<&str>) -> Config {
        Config {
            environment: Environment::Dev,
            sources: vec![],
            storage_path: "/tmp/kurultai-embed-test.db".into(),
            embed_model: "AllMiniLML6V2".into(),
            embed_dim: 384,
            embed_backend: backend.map(str::to_string),
            reranker_model: None,
            poll_interval_secs: 300,
            nightly_full_sync_hour: None,
            inactivity_threshold_hours: None,
            mcp_http_secret: None,
            banner: crate::art::BannerMode::Auto,
        }
    }

    #[test]
    fn wants_local_only_when_backend_local() {
        assert!(!wants_local_embed(&sample_config(None)));
        assert!(wants_local_embed(&sample_config(Some("local"))));
        assert!(wants_local_embed(&sample_config(Some("LOCAL"))));
        assert!(!wants_local_embed(&sample_config(Some("openrouter"))));
    }

    #[test]
    fn local_backend_without_feature_errors() {
        #[cfg(not(feature = "local-embed"))]
        {
            let res = build_local_embedder(&sample_config(Some("local")));
            let err = match res {
                Ok(_) => panic!("expected error without local-embed feature"),
                Err(e) => e,
            };
            assert!(err.to_string().contains("local-embed"), "got {err}");
        }
    }

    #[test]
    fn null_when_no_key_and_no_local() {
        let cfg = sample_config(None);
        assert!(!wants_local_embed(&cfg));
        let e = NullEmbedder::new(cfg.embed_dim);
        assert!(!e.is_live());
    }

    #[tokio::test]
    async fn hub_flag_without_database_url_is_config_error() {
        let key = "KURULTAI_FEATURE_HUB";
        let prev = std::env::var(key).ok();
        let prev_db = std::env::var("DATABASE_URL").ok();
        let prev_kdb = std::env::var("KURULTAI_DATABASE_URL").ok();
        std::env::set_var(key, "1");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("KURULTAI_DATABASE_URL");
        let err = match App::from_config(sample_config(None)).await {
            Ok(_) => panic!("expected config error without DATABASE_URL"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(msg.contains("DATABASE_URL"), "{msg}");
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
        match prev_db {
            Some(v) => std::env::set_var("DATABASE_URL", v),
            None => std::env::remove_var("DATABASE_URL"),
        }
        match prev_kdb {
            Some(v) => std::env::set_var("KURULTAI_DATABASE_URL", v),
            None => std::env::remove_var("KURULTAI_DATABASE_URL"),
        }
    }
}
