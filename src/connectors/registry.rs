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
