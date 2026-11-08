pub mod dayflow;
pub mod github;
pub mod inbox;
pub mod json;
pub mod markdown;
pub mod pond;
pub mod registry;

pub use registry::ConnectorRegistry;

use crate::error::Result;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;

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

    /// Relative paths (as stored in atom `rel_path` metadata) of every file the
    /// source currently has. `None` (default) means the connector cannot enumerate,
    /// so incremental polls never remove anything. Used to drop atoms whose
    /// backing file vanished between full syncs.
    async fn live_rel_paths(&self) -> Result<Option<std::collections::HashSet<String>>> {
        Ok(None)
    }
}
