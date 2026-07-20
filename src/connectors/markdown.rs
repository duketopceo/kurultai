use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;

/// Indexes `.md` files from any directory on disk.
///
/// Obsidian, Logseq, git wikis, etc. are just folders — no desktop app integration.
/// Config: `kind = "markdown"`, `root_path = "/path/to/notes"`.
pub struct MarkdownConnector {
    root_path: Option<String>,
}

impl MarkdownConnector {
    pub fn new() -> Self {
        Self { root_path: None }
    }

    /// `root_path` preferred; `vault_path` accepted as deprecated alias (Obsidian-era naming).
    fn resolve_root(config: &SourceConfig) -> Result<String> {
        if let Some(path) = config.extra.get("root_path") {
            return Ok(path.clone());
        }
        if let Some(path) = config.extra.get("vault_path") {
            tracing::warn!(
                source = %config.name,
                "vault_path is deprecated — use root_path for markdown sources"
            );
            return Ok(path.clone());
        }
        Err(KurultaiError::connector(
            &config.name,
            "root_path required for markdown source",
        ))
    }
}

impl Default for MarkdownConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for MarkdownConnector {
    fn name(&self) -> &str {
        "markdown"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        let root = Self::resolve_root(config)?;
        let resolved = validate_readable_path(&root, "markdown root")?;
        tracing::debug!(root = %resolved.display(), "markdown connector initialized");
        self.root_path = Some(resolved.to_string_lossy().into_owned());
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO(#31): Watch for changed .md files under root_path
        let _ = self.root_path.as_deref();
        Ok(vec![])
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        // TODO(#31): Recursively read .md files, extract frontmatter + content
        let _ = self.root_path.as_deref();
        Ok(vec![])
    }
}
