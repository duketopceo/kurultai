use async_trait::async_trait;
use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};

pub struct ObsidianConnector {
    vault_path: Option<String>,
}

impl ObsidianConnector {
    pub fn new() -> Self {
        Self { vault_path: None }
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
        "obsidian"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        let vault = config
            .extra
            .get("vault_path")
            .ok_or_else(|| KurultaiError::connector(&config.name, "vault_path required"))?;

        let resolved = validate_readable_path(vault, "obsidian vault")?;
        tracing::debug!(vault = %resolved.display(), "obsidian connector initialized");
        self.vault_path = Some(resolved.to_string_lossy().into_owned());
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO(#3): Watch for changed .md files in vault
        let _ = self.vault_path.as_deref();
        Ok(vec![])
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO(#3): Recursively read all .md files, extract frontmatter + content
        let _ = self.vault_path.as_deref();
        Ok(vec![])
    }
}
