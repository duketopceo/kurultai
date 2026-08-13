//! JSON folder connector with dump format parity (json / ndjson / md / txt).
//!
//! # Config
//! ```toml
//! [sources.data]
//! kind = "json"
//! root_path = "/path/to/json-files"
//! ```
//!
//! Use **one source per mixed folder** (do not also point a `markdown` source at the same root).
//! Stable `source_id` is relative path (+ record index for JSON) via the shared dump atomizer.

use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::ingest::dump;
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// Reads dump files from a local directory tree (JSON-primary, format-parity dumps).
pub struct JsonConnector {
    source_name: String,
    root_path: Option<PathBuf>,
    last_poll: Mutex<Option<SystemTime>>,
}

impl JsonConnector {
    pub fn new() -> Self {
        Self {
            source_name: "json".into(),
            root_path: None,
            last_poll: Mutex::new(None),
        }
    }

    fn collect_atoms(&self, since: Option<SystemTime>) -> Result<Vec<KnowledgeAtom>> {
        let root = self
            .root_path
            .as_ref()
            .ok_or_else(|| KurultaiError::connector(&self.source_name, "not initialized"))?;

        let mut atoms = Vec::new();
        dump::walk_dump_files(root, &[], &mut |path| {
            let meta = std::fs::metadata(path).map_err(|e| {
                KurultaiError::connector(
                    &self.source_name,
                    format!("stat {}: {e}", path.display()),
                )
            })?;
            let mtime = meta.modified().ok();
            if let (Some(since), Some(mtime)) = (since, mtime) {
                if mtime <= since {
                    return Ok(());
                }
            }

            let updated = mtime
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_else(Utc::now))
                .unwrap_or_else(Utc::now);

            atoms.extend(dump::atomize_path(
                &self.source_name,
                root,
                path,
                updated,
            )?);
            Ok(())
        })?;

        Ok(atoms)
    }
}

impl Default for JsonConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for JsonConnector {
    fn name(&self) -> &str {
        "json"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        self.source_name = config.name.clone();

        let root = config.extra.get("root_path").ok_or_else(|| {
            KurultaiError::connector(&config.name, "root_path required for json source")
        })?;
        let resolved = validate_readable_path(root, "json root")?;
        tracing::debug!(root = %resolved.display(), "json connector initialized");
        self.root_path = Some(resolved);
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        let since = *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector(&self.source_name, format!("lock: {e}")))?;
        let atoms = self.collect_atoms(since)?;
        *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector(&self.source_name, format!("lock: {e}")))? =
            Some(SystemTime::now());
        Ok(atoms)
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        let atoms = self.collect_atoms(None)?;
        *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector(&self.source_name, format!("lock: {e}")))? =
            Some(SystemTime::now());
        Ok(atoms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceKind;
    use std::fs;

    #[tokio::test]
    async fn full_sync_indexes_json_array_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"[
          {"id": "k1", "title": "Alpha", "content": "FIXTURE_JSON_KNOWN_PHRASE_42 with enough detail for quality.", "tags": ["alpha"]},
          {"id": "k2", "title": "Beta",  "content": "Another record with enough operational detail here."}
        ]"#;
        fs::write(dir.path().join("fixture.json"), content).unwrap();

        let mut connector = JsonConnector::new();
        let config = SourceConfig {
            name: "json-test".into(),
            kind: SourceKind::Json,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        };
        connector.init(&config).await.unwrap();
        let atoms = connector.full_sync().await.unwrap();

        assert_eq!(atoms.len(), 2);
        assert!(atoms
            .iter()
            .any(|a| a.content.contains("FIXTURE_JSON_KNOWN_PHRASE_42")));
        assert!(atoms.iter().any(|a| a.tags.contains(&"alpha".to_string())));
        assert!(atoms.iter().all(|a| a.source == "json-test"));
        // Path-stable source_id (KTD2).
        assert!(atoms.iter().any(|a| a.source_id.ends_with("/0")));
    }

    #[tokio::test]
    async fn full_sync_indexes_ndjson_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let content = [
            r#"{"uid": "n1", "title": "NDJSON First", "content": "NDJSON_KNOWN_42 with enough detail for the quality gate.", "tags": ["ndjson"]}"#,
            r#"{"uid": "n2", "title": "NDJSON Second", "content": "other body with enough detail for quality gate here", "tags": ["ndjson"]}"#,
        ]
        .join("\n");
        fs::write(dir.path().join("fixture.ndjson"), &content).unwrap();

        let mut connector = JsonConnector::new();
        let config = SourceConfig {
            name: "ndjson-test".into(),
            kind: SourceKind::Json,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        };
        connector.init(&config).await.unwrap();
        let atoms = connector.full_sync().await.unwrap();

        assert_eq!(atoms.len(), 2);
        assert!(atoms.iter().any(|a| a.content.contains("NDJSON_KNOWN_42")));
    }

    #[tokio::test]
    async fn format_parity_reads_markdown() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("note.md"),
            "---\ntags: [md]\n---\n\nMarkdown dump via json source with enough detail for parity.\n",
        )
        .unwrap();

        let mut connector = JsonConnector::new();
        connector
            .init(&SourceConfig {
                name: "data".into(),
                kind: SourceKind::Json,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::from([(
                    "root_path".into(),
                    dir.path().to_string_lossy().into_owned(),
                )]),
            })
            .await
            .unwrap();
        let atoms = connector.full_sync().await.unwrap();
        assert!(atoms.iter().any(|a| a.content.contains("Markdown dump")));
    }
}
