use crate::config::{ensure_storage_parent, expand_path, load_config_with_env};
use crate::connectors::ConnectorRegistry;
use crate::embed::{
    resolve_local_model, EmbedBackend, Embedder, HttpEmbedder, NullEmbedder,
    DEFAULT_LOCAL_EMBED_URL,
};
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
        let storage_path = expand_path(&config.storage_path)?;
        ensure_storage_parent(&storage_path)?;

        tracing::debug!(
            storage = %storage_path.display(),
            embed_dim = config.embed_dim,
            "initializing store"
        );
        let store: Arc<dyn Store> = Arc::new(SqliteVecStore::open(storage_path, config.embed_dim)?);

        let embedder = build_embedder(&config, environment)?;
        let reranker = build_reranker(&config);
        let synthesizer = synthesizer_from_env(None);
        let connectors = ConnectorRegistry::from_config(&config).await?;
        let pipeline = IndexPipeline::new(Arc::clone(&store), Arc::clone(&embedder));

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

/// Select embedder from config + env. `KURULTAI_EMBED_BACKEND` overrides file backend.
pub(crate) fn build_embedder(config: &Config, env: Environment) -> Result<Arc<dyn Embedder>> {
    // API keys come from env only — never from config files.
    let api_key = api_key_from_env_optional("OPENROUTER_API_KEY")
        .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"))
        .map(|k| k.expose().to_string());

    let backend_raw = std::env::var("KURULTAI_EMBED_BACKEND")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| config.embed_backend.clone());

    let local_url = std::env::var("KURULTAI_LOCAL_EMBED_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if config.local_embed_url.trim().is_empty() {
                DEFAULT_LOCAL_EMBED_URL.into()
            } else {
                config.local_embed_url.clone()
            }
        });

    select_embedder(config, env, &backend_raw, api_key.as_deref(), &local_url)
}

pub(crate) fn select_embedder(
    config: &Config,
    env: Environment,
    backend_raw: &str,
    api_key: Option<&str>,
    local_url: &str,
) -> Result<Arc<dyn Embedder>> {
    let backend = EmbedBackend::parse(backend_raw)?;

    match backend {
        EmbedBackend::Null => {
            tracing::info!(env = %env, "embed.backend=null — FTS-only (NullEmbedder)");
            Ok(Arc::new(NullEmbedder::new(config.embed_dim)))
        }
        EmbedBackend::OpenRouter => {
            let Some(key) = api_key else {
                return Err(KurultaiError::config(
                    "embed.backend=openrouter requires OPENROUTER_API_KEY or KURULTAI_API_KEY",
                ));
            };
            Ok(Arc::new(HttpEmbedder::openrouter(
                key.to_string(),
                config.embed_model.clone(),
                config.embed_dim,
            )))
        }
        EmbedBackend::Local => {
            let model = resolve_local_model(&config.embed_model);
            tracing::info!(
                env = %env,
                url = %local_url,
                model = %model,
                dim = config.embed_dim,
                "embed.backend=local — OpenAI-compatible HTTP embedder"
            );
            Ok(Arc::new(HttpEmbedder::local(
                local_url.to_string(),
                api_key.map(str::to_string),
                model,
                config.embed_dim,
            )))
        }
        EmbedBackend::Auto => match api_key {
            Some(key) => Ok(Arc::new(HttpEmbedder::openrouter(
                key.to_string(),
                config.embed_model.clone(),
                config.embed_dim,
            ))),
            None => {
                tracing::warn!(
                    env = %env,
                    "no OPENROUTER_API_KEY or KURULTAI_API_KEY — FTS-only mode (NullEmbedder); set embed.backend=local for Ollama/TEI"
                );
                Ok(Arc::new(NullEmbedder::new(config.embed_dim)))
            }
        },
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
    use crate::embed::DEFAULT_LOCAL_EMBED_URL;

    fn sample_config(backend: &str) -> Config {
        Config {
            environment: Environment::Dev,
            sources: vec![],
            storage_path: "/tmp/kurultai-embed-test.db".into(),
            embed_model: "openai/text-embedding-3-large".into(),
            embed_dim: 768,
            embed_backend: backend.into(),
            local_embed_url: DEFAULT_LOCAL_EMBED_URL.into(),
            reranker_model: None,
            poll_interval_secs: 300,
        }
    }

    #[test]
    fn auto_without_key_is_null() {
        let e = select_embedder(
            &sample_config("auto"),
            Environment::Dev,
            "auto",
            None,
            DEFAULT_LOCAL_EMBED_URL,
        )
        .unwrap();
        assert!(!e.is_live());
        assert_eq!(e.name(), "none");
    }

    #[test]
    fn openrouter_without_key_errors() {
        let result = select_embedder(
            &sample_config("openrouter"),
            Environment::Dev,
            "openrouter",
            None,
            DEFAULT_LOCAL_EMBED_URL,
        );
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(err.contains("openrouter"), "{err}");
    }

    #[test]
    fn local_is_live_without_cloud_key() {
        let e = select_embedder(
            &sample_config("local"),
            Environment::Dev,
            "local",
            None,
            DEFAULT_LOCAL_EMBED_URL,
        )
        .unwrap();
        assert!(e.is_live());
        assert_eq!(e.name(), "nomic-embed-text");
        assert_eq!(e.dim(), 768);
    }

    #[test]
    fn auto_with_key_is_openrouter_model() {
        let e = select_embedder(
            &sample_config("auto"),
            Environment::Dev,
            "auto",
            Some("sk-test"),
            DEFAULT_LOCAL_EMBED_URL,
        )
        .unwrap();
        assert!(e.is_live());
        assert_eq!(e.name(), "openai/text-embedding-3-large");
    }
}
