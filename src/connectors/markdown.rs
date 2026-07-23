use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Indexes `.md` files from any directory on disk.
///
/// Obsidian, Logseq, git wikis, etc. are just folders — no desktop app integration.
/// Config: `kind = "markdown"`, `root_path = "/path/to/notes"`.
pub struct MarkdownConnector {
    root_path: Option<String>,
    source_name: String,
}

impl MarkdownConnector {
    pub fn new() -> Self {
        Self {
            root_path: None,
            source_name: "markdown".into(),
        }
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

    fn root(&self) -> Result<&Path> {
        self.root_path
            .as_ref()
            .map(Path::new)
            .ok_or_else(|| KurultaiError::connector("markdown", "connector not initialized"))
    }

    fn content_hash(content: &str) -> String {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn extract_title(content: &str, fallback: &str) -> String {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                return title.trim().to_string();
            }
        }
        fallback.to_string()
    }

    fn atom_from_file(&self, root: &Path, path: &Path, source_name: &str) -> Result<KnowledgeAtom> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            KurultaiError::connector(source_name, format!("read {}: {e}", path.display()))
        })?;

        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let source_id = relative.replace(std::path::MAIN_SEPARATOR, "/");
        let id = format!("{source_name}:{source_id}");
        let source_uri = Some(format!(
            "file://{}",
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .display()
        ));

        let fallback = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&source_id)
            .to_string();
        let title = Self::extract_title(&content, &fallback);
        let content_hash = Self::content_hash(&content);

        let modified = path
            .metadata()
            .and_then(|m| m.modified())
            .map(DateTime::from)
            .unwrap_or_else(|_| Utc::now());

        let mut metadata = HashMap::new();
        metadata.insert("relative_path".into(), source_id.clone());

        Ok(KnowledgeAtom {
            id,
            source: source_name.to_string(),
            source_id,
            title,
            summary: String::new(),
            content,
            question: None,
            resolution: None,
            tags: vec![],
            source_updated_at: modified,
            indexed_at: Utc::now(),
            metadata,
            embedding: None,
            content_hash,
            source_uri,
            provenance: Some("kurultai markdown connector".into()),
        })
    }

    fn read_all(&self) -> Result<Vec<KnowledgeAtom>> {
        let root = self.root()?;
        let source_name = &self.source_name;
        let mut atoms = Vec::new();

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("md"))
                        .unwrap_or(false)
            })
        {
            let path = entry.path();
            match self.atom_from_file(root, path, source_name) {
                Ok(atom) => atoms.push(atom),
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "skipping markdown file");
                }
            }
        }

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
        let root = Self::resolve_root(config)?;
        let resolved = validate_readable_path(&root, "markdown root")?;
        tracing::debug!(root = %resolved.display(), "markdown connector initialized");
        self.root_path = Some(resolved.to_string_lossy().into_owned());
        self.source_name = config.name.clone();
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        // Phase 1: poll does a full read. The pipeline handles content-hash
        // skip and orphan cleanup. Incremental file-watching is deferred.
        self.read_all()
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        self.read_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_vault() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let mut file1 = std::fs::File::create(root.join("notes.md")).unwrap();
        write!(file1, "# Hello World\n\nThis is a test note.").unwrap();

        let nested = root.join("sub");
        std::fs::create_dir(&nested).unwrap();
        let mut file2 = std::fs::File::create(nested.join("deep.md")).unwrap();
        write!(file2, "# Nested\n\nUnique phrase for search.").unwrap();

        tmp
    }

    #[tokio::test]
    async fn reads_markdown_files() {
        let vault = make_vault();
        let mut connector = MarkdownConnector::new();
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: crate::types::SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra: {
                    let mut m = HashMap::new();
                    m.insert(
                        "root_path".into(),
                        vault.path().to_string_lossy().into_owned(),
                    );
                    m
                },
            })
            .await
            .unwrap();

        let atoms = connector.full_sync().await.unwrap();
        assert_eq!(atoms.len(), 2);

        let titles: Vec<&str> = atoms.iter().map(|a| a.title.as_str()).collect();
        assert!(titles.contains(&"Hello World"));
        assert!(titles.contains(&"Nested"));

        let unique = atoms
            .iter()
            .find(|a| a.content.contains("Unique phrase"))
            .unwrap();
        assert_eq!(unique.source_id, "sub/deep.md");
        assert!(!unique.content_hash.is_empty());
    }

    #[tokio::test]
    async fn content_hash_changes_with_content() {
        let vault = make_vault();
        let mut connector = MarkdownConnector::new();
        connector
            .init(&SourceConfig {
                name: "notes".into(),
                kind: crate::types::SourceKind::Markdown,
                enabled: true,
                poll_interval_secs: 60,
                extra: {
                    let mut m = HashMap::new();
                    m.insert(
                        "root_path".into(),
                        vault.path().to_string_lossy().into_owned(),
                    );
                    m
                },
            })
            .await
            .unwrap();

        let first = connector.full_sync().await.unwrap();
        let original_hash = first
            .iter()
            .find(|a| a.source_id == "notes.md")
            .unwrap()
            .content_hash
            .clone();

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(vault.path().join("notes.md"))
            .unwrap();
        write!(file, "# Changed\n\nDifferent content.").unwrap();
        drop(file);

        let second = connector.full_sync().await.unwrap();
        let new_hash = second
            .iter()
            .find(|a| a.source_id == "notes.md")
            .unwrap()
            .content_hash
            .clone();

        assert_ne!(original_hash, new_hash);
    }
}
