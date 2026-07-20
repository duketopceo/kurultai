use crate::connectors::Connector;
use crate::types::{KnowledgeAtom, SourceConfig};
use anyhow::{bail, Result};
use async_trait::async_trait;

/// AppFlowy connector — honest stub until filesystem path is proven (#4 after #31).
pub struct AppFlowyConnector;

#[async_trait]
impl Connector for AppFlowyConnector {
    fn name(&self) -> &str {
        "appflowy"
    }

    async fn init(&mut self, _config: &SourceConfig) -> Result<()> {
        bail!("appflowy connector not implemented in Phase 1 (use filesystem/obsidian)")
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        bail!("appflowy connector not implemented")
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        bail!("appflowy connector not implemented")
    }
}
