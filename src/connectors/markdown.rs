//! Markdown folder connector with dump format parity (md / json / ndjson / txt).

use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::ingest::dump;
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// Indexes dump files from any directory on disk (markdown-primary, format-parity dumps).
///
/// Config: `kind = "markdown"`, `root_path = "/path/to/notes"`.
/// Accepts `.md`, `.json`/`.jsonl`/`.ndjson`, and `.txt` via the shared dump atomizer.
/// Use **one source per mixed folder** (do not also point a `json` source at the same root).
pub struct MarkdownConnector {
    source_name: String,
    root_path: Option<PathBuf>,
    last_poll: Mutex<Option<SystemTime>>,
}

impl MarkdownConnector {
    pub fn new() -> Self {
        Self {
            source_name: "markdown".into(),
            root_path: None,
            last_poll: Mutex::new(None),
        }
    }

    /// `root_path` preferred; `vault_path` accepted as deprecated alias.
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

    fn collect_atoms(&self, since: Option<SystemTime>) -> Result<Vec<KnowledgeAtom>> {
        let root = self
            .root_path
            .as_ref()
            .ok_or_else(|| KurultaiError::connector("markdown", "not initialized"))?;

        let mut atoms = Vec::new();
        dump::walk_dump_files(root, &[], &mut |path| {
            let meta = std::fs::metadata(path).map_err(|e| {
                KurultaiError::connector("markdown", format!("stat {}: {e}", path.display()))
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

            atoms.extend(dump::atomize_path(&self.source_name, root, path, updated)?);
            Ok(())
        })?;

        Ok(atoms)
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
        self.source_name = config.name.clone();
        let root = Self::resolve_root(config)?;
        let resolved = validate_readable_path(&root, "markdown root")?;
        tracing::debug!(root = %resolved.display(), "markdown connector initialized");
        self.root_path = Some(resolved);
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        let since = *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector("markdown", format!("lock: {e}")))?;
        let atoms = self.collect_atoms(since)?;
        *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector("markdown", format!("lock: {e}")))? =
            Some(SystemTime::now());
        Ok(atoms)
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        let atoms = self.collect_atoms(None)?;
        *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector("markdown", format!("lock: {e}")))? =
            Some(SystemTime::now());
        Ok(atoms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceKind;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;

    #[tokio::test]
    async fn full_sync_indexes_fixture_files() {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-md-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(dir.join("sub")).unwrap();
        let mut f = fs::File::create(dir.join("sub/note.md")).unwrap();
        writeln!(
            f,
            "---\ntitle: Fixture Note\ntags: [fixture]\n---\n\n## Section\nKNOWN_PHRASE_KURULTAI_42 appears here with enough detail for the quality gate.\n"
        )
        .unwrap();

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), dir.to_string_lossy().into_owned());
        let config = SourceConfig {
            name: "notes".into(),
            kind: SourceKind::Markdown,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        };
        connector.init(&config).await.unwrap();
        let atoms = connector.full_sync().await.unwrap();
        assert!(!atoms.is_empty());
        assert!(atoms
            .iter()
            .any(|a| a.content.contains("KNOWN_PHRASE_KURULTAI_42")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn format_parity_reads_json_and_txt() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("a.json"),
            r#"[{"title":"J","content":"JSON dump body with operational detail for parity test case.","tags":["j"]}]"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("b.txt"),
            "plain text dump body with operational detail for parity test case.\n",
        )
        .unwrap();

        let mut connector = MarkdownConnector::new();
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: SourceKind::Markdown,
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
        assert!(atoms.iter().any(|a| a.content.contains("JSON dump body")));
        assert!(atoms.iter().any(|a| a.content.contains("plain text dump")));
    }
}
