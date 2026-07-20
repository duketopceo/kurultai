use crate::connectors::filesystem::FilesystemConnector;
use crate::connectors::Connector;
use crate::types::{KnowledgeAtom, SourceConfig};
use anyhow::Result;
use async_trait::async_trait;

/// Obsidian vault = filesystem connector with `vault_path` alias.
pub struct ObsidianConnector {
    inner: FilesystemConnector,
}

impl ObsidianConnector {
    pub fn new() -> Self {
        Self {
            inner: FilesystemConnector::new("obsidian"),
        }
    }
}

impl Default for ObsidianConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for ObsidianConnector {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        let mut mapped = config.clone();
        if let Some(vp) = mapped.extra.get("vault_path").cloned() {
            mapped.extra.insert("path".into(), vp);
        }
        if !mapped.extra.contains_key("path") {
            anyhow::bail!("obsidian source requires extra.vault_path (or path)");
        }
        self.inner.init(&mapped).await
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        self.inner.poll().await
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        self.inner.full_sync().await
    }
}
