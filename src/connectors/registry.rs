use crate::connectors::appflowy::AppFlowyConnector;
use crate::connectors::markdown::MarkdownConnector;
use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::types::{Config, SourceConfig, SourceKind};
use std::collections::HashMap;

/// Factory + registry for source connectors.
///
/// New connectors register here — callers never match on `SourceKind` directly.
pub struct ConnectorRegistry {
    connectors: HashMap<String, Box<dyn Connector>>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        Self {
            connectors: HashMap::new(),
        }
    }

    /// Build connectors from config, initializing each enabled source.
    pub async fn from_config(config: &Config) -> Result<Self> {
        let mut registry = Self::new();

        for source in config.sources.iter().filter(|s| s.enabled) {
            let mut connector = build_connector(&source.kind)?;
            connector
                .init(source)
                .await
                .map_err(|e| KurultaiError::connector(&source.name, e.to_string()))?;
            registry.register(source.name.clone(), connector);
            tracing::info!(source = %source.name, kind = ?source.kind, "connector registered");
        }

        Ok(registry)
    }

    pub fn register(&mut self, name: String, connector: Box<dyn Connector>) {
        self.connectors.insert(name, connector);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Connector> {
        self.connectors.get(name).map(|c| c.as_ref())
    }

    pub fn names(&self) -> Vec<&str> {
        self.connectors.keys().map(String::as_str).collect()
    }

    pub fn len(&self) -> usize {
        self.connectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.connectors.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn Connector)> {
        self.connectors
            .iter()
            .map(|(name, connector)| (name.as_str(), connector.as_ref()))
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn build_connector(kind: &SourceKind) -> Result<Box<dyn Connector>> {
    let connector: Box<dyn Connector> = match kind {
        SourceKind::AppFlowy => Box::new(AppFlowyConnector::new()),
        SourceKind::Markdown => Box::new(MarkdownConnector::new()),
        SourceKind::Pond | SourceKind::TechTracker | SourceKind::GitHub => {
            return Err(KurultaiError::connector(
                format!("{kind:?}"),
                "connector not implemented yet",
            ));
        }
        SourceKind::Custom(name) => {
            return Err(KurultaiError::connector(name, "unknown custom connector"));
        }
    };
    Ok(connector)
}

/// Resolve a source config by name from the top-level config.
pub fn source_config<'a>(config: &'a Config, name: &str) -> Result<&'a SourceConfig> {
    config
        .sources
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| KurultaiError::config(format!("unknown source: {name}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::types::Config;
    use std::collections::HashMap;

    #[tokio::test]
    async fn from_config_registers_enabled_markdown_only() {
        let config = Config {
            environment: Environment::Dev,
            sources: vec![
                SourceConfig {
                    name: "notes".into(),
                    kind: SourceKind::Markdown,
                    enabled: true,
                    poll_interval_secs: 60,
                    extra: HashMap::from([(
                        "root_path".into(),
                        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/fixtures/vault",
                    )]),
                },
                SourceConfig {
                    name: "disabled".into(),
                    kind: SourceKind::Markdown,
                    enabled: false,
                    poll_interval_secs: 60,
                    extra: HashMap::from([("root_path".into(), "/tmp".into())]),
                },
            ],
            storage_path: "/tmp/kurultai-test.db".into(),
            embed_model: "openai/text-embedding-3-large".into(),
            embed_dim: 4,
            reranker_model: None,
            synthesis_model: None,
            poll_interval_secs: 300,
        };

        let registry = ConnectorRegistry::from_config(&config).await.unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.get("notes").is_some());
        assert!(registry.get("disabled").is_none());
    }

    #[tokio::test]
    async fn from_config_rejects_unimplemented_kinds() {
        let config = Config {
            environment: Environment::Dev,
            sources: vec![SourceConfig {
                name: "gh".into(),
                kind: SourceKind::GitHub,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::new(),
            }],
            storage_path: "/tmp/kurultai-test.db".into(),
            embed_model: "m".into(),
            embed_dim: 4,
            reranker_model: None,
            synthesis_model: None,
            poll_interval_secs: 300,
        };
        match ConnectorRegistry::from_config(&config).await {
            Ok(_) => panic!("expected unimplemented connector error"),
            Err(err) => assert!(err.to_string().contains("not implemented")),
        }
    }
}
