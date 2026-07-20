pub mod appflowy;
pub mod filesystem;
pub mod obsidian;

use crate::types::{KnowledgeAtom, SourceConfig, SourceKind};
use anyhow::{bail, Result};
use async_trait::async_trait;

/// Trait every data source connector must implement.
#[async_trait]
pub trait Connector: Send + Sync {
    fn name(&self) -> &str;
    async fn init(&mut self, config: &SourceConfig) -> Result<()>;
    async fn poll(&self) -> Result<Vec<KnowledgeAtom>>;
    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>>;
}

/// Build a connector for an implemented kind, or error honestly.
pub fn build_connector(config: &SourceConfig) -> Result<Box<dyn Connector>> {
    match config.kind {
        SourceKind::Filesystem => Ok(Box::new(filesystem::FilesystemConnector::new(
            config.name.clone(),
        ))),
        SourceKind::Obsidian => Ok(Box::new(obsidian::ObsidianConnector::new())),
        SourceKind::AppFlowy => bail!(
            "source '{}' kind appflowy is not implemented yet (Phase 1: filesystem/obsidian only)",
            config.name
        ),
        other => bail!(
            "source '{}' kind {} is not registered in this build",
            config.name,
            other.as_str()
        ),
    }
}
