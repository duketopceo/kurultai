pub mod appflowy;
pub mod dayflow;
pub mod github;
pub mod json;
pub mod markdown;
pub mod pond;
pub mod registry;

pub use registry::ConnectorRegistry;

use crate::error::Result;
use crate::types::{KnowledgeAtom, SourceConfig, VisibilityScope};
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
}

/// Resolve source-level visibility from config (`visibility` in TOML / `extra`).
///
/// Fail-closed to [`VisibilityScope::Personal`] when unset or unrecognized (HUB-5 / R9).
/// Scope is tagged at ingest; nothing reclassifies after the fact.
pub fn source_visibility(config: &SourceConfig) -> VisibilityScope {
    config
        .extra
        .get("visibility")
        .map(|s| VisibilityScope::parse(s.trim()))
        .unwrap_or(VisibilityScope::Personal)
}

/// Stamp every atom with a visibility scope (source default / explicit override).
pub fn apply_visibility(atoms: &mut [KnowledgeAtom], scope: VisibilityScope) {
    for atom in atoms {
        atom.visibility = scope;
    }
}

#[cfg(test)]
mod visibility_tests {
    use super::*;
    use crate::types::SourceKind;
    use std::collections::HashMap;

    fn cfg(extra: HashMap<String, String>) -> SourceConfig {
        SourceConfig {
            name: "t".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        }
    }

    #[test]
    fn missing_visibility_defaults_personal() {
        assert_eq!(source_visibility(&cfg(HashMap::new())), VisibilityScope::Personal);
    }

    #[test]
    fn parses_team_and_company() {
        assert_eq!(
            source_visibility(&cfg(HashMap::from([("visibility".into(), "team".into())]))),
            VisibilityScope::Team
        );
        assert_eq!(
            source_visibility(&cfg(HashMap::from([(
                "visibility".into(),
                "company".into()
            )]))),
            VisibilityScope::Company
        );
    }

    #[test]
    fn unknown_visibility_fail_closed() {
        assert_eq!(
            source_visibility(&cfg(HashMap::from([(
                "visibility".into(),
                "secret".into()
            )]))),
            VisibilityScope::Personal
        );
    }

    #[test]
    fn apply_visibility_stamps_all() {
        let mut atoms = vec![KnowledgeAtom::default(), KnowledgeAtom::default()];
        apply_visibility(&mut atoms, VisibilityScope::Team);
        assert!(atoms.iter().all(|a| a.visibility == VisibilityScope::Team));
    }
}
