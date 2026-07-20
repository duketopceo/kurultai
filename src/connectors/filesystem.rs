use crate::connectors::Connector;
use crate::types::{KnowledgeAtom, SourceConfig};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Index a directory tree of Markdown files.
///
/// **Orphan policy (full_sync):** callers should pass returned atoms to the store,
/// then call `Store::delete_orphans(source, keep_source_ids)` so deleted files
/// are removed from the hot store. Incremental `poll` does not delete orphans.
pub struct FilesystemConnector {
    name: String,
    root: Option<PathBuf>,
}

impl FilesystemConnector {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            root: None,
        }
    }

    pub fn with_root(name: impl Into<String>, root: PathBuf) -> Self {
        Self {
            name: name.into(),
            root: Some(root),
        }
    }

    fn root(&self) -> Result<&Path> {
        self.root
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("filesystem connector not initialized"))
    }

    pub fn atom_from_file(source: &str, root: &Path, path: &Path) -> Result<KnowledgeAtom> {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let content = String::from_utf8_lossy(&bytes).to_string();
        let content_hash = hex::encode(Sha256::digest(&bytes));
        // Content-addressed: changes when file bytes change; orphans cleaned on full sync.
        let id = atom_id(source, &rel, &content_hash);
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| rel.clone());
        let meta = std::fs::metadata(path)?;
        let source_updated_at = meta
            .modified()
            .ok()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);
        let uri = format!("file://{}", path.display());
        Ok(KnowledgeAtom {
            id,
            source: source.into(),
            source_id: rel,
            source_uri: Some(uri),
            title,
            summary: String::new(),
            content,
            question: None,
            resolution: None,
            tags: vec![],
            provenance: Some(format!("filesystem:{}", root.display())),
            source_updated_at,
            indexed_at: Utc::now(),
            content_hash,
            embedding: None,
            metadata: Default::default(),
        })
    }

    fn collect_md(&self) -> Result<Vec<KnowledgeAtom>> {
        let root = self.root()?;
        if !root.is_dir() {
            bail!("filesystem root is not a directory: {}", root.display());
        }
        let mut atoms = Vec::new();
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !ext.eq_ignore_ascii_case("md") {
                continue;
            }
            atoms.push(Self::atom_from_file(&self.name, root, path)?);
        }
        Ok(atoms)
    }
}

pub fn atom_id(source: &str, source_id: &str, content_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update(b"\0");
    hasher.update(source_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(content_hash.as_bytes());
    hex::encode(hasher.finalize())
}

#[async_trait]
impl Connector for FilesystemConnector {
    fn name(&self) -> &str {
        &self.name
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        let path = config
            .extra
            .get("path")
            .or_else(|| config.extra.get("vault_path"))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("filesystem source requires extra.path"))?;
        let expanded = crate::config::expand_path(&path)?;
        self.root = Some(expanded);
        self.name = config.name.clone();
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        // Phase 1: poll == full scan; hash upsert skips unchanged content at store layer.
        self.collect_md()
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        self.collect_md()
    }
}
