use crate::connectors::Connector;
use crate::error::Result;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;

pub struct AppFlowyConnector;

impl AppFlowyConnector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppFlowyConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for AppFlowyConnector {
    fn name(&self) -> &str {
        "appflowy"
    }

    async fn init(&mut self, _config: &SourceConfig) -> Result<()> {
        // TODO: Connect to AppFlowy via its REST API or MCP
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO: Fetch new/changed pages from AppFlowy
        Ok(vec![])
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO: Full re-index of all AppFlowy pages
        Ok(vec![])
    }
}
