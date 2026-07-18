pub mod appflowy;
pub mod obsidian;

use async_trait::async_trait;
use crate::types::{KnowledgeAtom, SourceConfig};
use anyhow::Result;

/// Trait every data source connector must implement.
#[async_trait]
pub trait Connector: Send + Sync {
    /// Name of this connector (matches source config).
    fn name(&self) -> &str;

    /// Initialize the connector with its config.
    async fn init(&mut self, config: &SourceConfig) -> Result<()>;

    /// Fetch all atoms since the last index timestamp.
    /// Returns new/changed atoms. Empty if nothing changed.
    async fn poll(&self) -> Result<Vec<KnowledgeAtom>>;

    /// Full re-index: fetch everything this source has.
    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>>;
}
