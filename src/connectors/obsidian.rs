use async_trait::async_trait;
use crate::connectors::Connector;
use crate::types::{KnowledgeAtom, SourceConfig};
use anyhow::Result;

pub struct ObsidianConnector {
    vault_path: Option<String>,
}

impl ObsidianConnector {
    pub fn new() -> Self {
        Self { vault_path: None }
    }
}

#[async_trait]
impl Connector for ObsidianConnector {
    fn name(&self) -> &str { "obsidian" }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        self.vault_path = config.extra.get("vault_path").cloned();
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO: Watch for changed .md files in vault
        Ok(vec![])
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO: Recursively read all .md files, extract frontmatter + content
        Ok(vec![])
    }
}
