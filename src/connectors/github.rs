//! Local filesystem code connector (`kind = "github"`).
//!
//! Indexes a checkout on disk (Pace-Server, luke-agents, etc.). No GitHub API —
//! name matches the Phase 4 roadmap “GitHub/Code” source.

use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::hashutil::{atom_id_from_hash, sha256_hex};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

const MAX_CHUNK_WORDS: usize = 400;
const DEFAULT_MAX_FILE_BYTES: u64 = 256 * 1024;

const DEFAULT_EXTENSIONS: &[&str] = &[
    "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "c", "cpp", "h", "hpp", "rb", "sh", "md",
    "toml", "yaml", "yml", "json",
];

const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    "vendor",
    "__pycache__",
    ".venv",
    "venv",
    ".cargo",
];

/// Indexes source files from a local repository tree.
///
/// Config: `kind = "github"`, `root_path = "/path/to/checkout"`.
/// Optional: `extensions` (comma list), `max_file_bytes`.
pub struct GitHubConnector {
    source_name: String,
    root_path: Option<PathBuf>,
    extensions: HashSet<String>,
    max_file_bytes: u64,
    last_poll: Mutex<Option<SystemTime>>,
}

impl GitHubConnector {
    /// Create a connector with default extensions and file-size limit.
    /// Call [`Connector::init`] with `root_path` before syncing.
    pub fn new() -> Self {
        Self {
            source_name: "github".into(),
            root_path: None,
            extensions: DEFAULT_EXTENSIONS
                .iter()
                .map(|e| (*e).to_string())
                .collect(),
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            last_poll: Mutex::new(None),
        }
    }

    fn resolve_root(config: &SourceConfig) -> Result<String> {
        config.extra.get("root_path").cloned().ok_or_else(|| {
            KurultaiError::connector(&config.name, "root_path required for github source")
        })
    }

    fn parse_extensions(raw: Option<&String>) -> HashSet<String> {
        match raw {
            Some(s) if !s.trim().is_empty() => s
                .split(',')
                .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase())
                .filter(|e| !e.is_empty())
                .collect(),
            _ => DEFAULT_EXTENSIONS
                .iter()
                .map(|e| (*e).to_string())
                .collect(),
        }
    }

    async fn sync_atoms(&self, since: Option<SystemTime>) -> Result<Vec<KnowledgeAtom>> {
        let scan_start = SystemTime::now();
        // Coarse FS mtimes (1s) can round edits to at/before scan_start; overlap re-scan.
        let since = since.map(|t| t.checked_sub(Duration::from_secs(2)).unwrap_or(t));
        let source_name = self.source_name.clone();
        let root = self
            .root_path
            .clone()
            .ok_or_else(|| KurultaiError::connector("github", "not initialized"))?;
        let extensions = self.extensions.clone();
        let max_file_bytes = self.max_file_bytes;

        let atoms = tokio::task::spawn_blocking(move || {
            collect_atoms(source_name, root, extensions, max_file_bytes, since)
        })
        .await
        .map_err(|e| KurultaiError::connector("github", format!("spawn_blocking: {e}")))??;

        *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector("github", format!("lock: {e}")))? =
            Some(scan_start);
        Ok(atoms)
    }
}

impl Default for GitHubConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for GitHubConnector {
    fn name(&self) -> &str {
        "github"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        self.source_name = config.name.clone();
        let root = Self::resolve_root(config)?;
        let resolved = validate_readable_path(&root, "github root")?;
        self.extensions = Self::parse_extensions(config.extra.get("extensions"));
        self.max_file_bytes = config
            .extra
            .get("max_file_bytes")
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_FILE_BYTES);
        tracing::debug!(root = %resolved.display(), "github connector initialized");
        self.root_path = Some(resolved);
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        let since = *self
            .last_poll
            .lock()
            .map_err(|e| KurultaiError::connector("github", format!("lock: {e}")))?;
        self.sync_atoms(since).await
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        self.sync_atoms(None).await
    }
}

fn collect_atoms(
    source_name: String,
    root: PathBuf,
    extensions: HashSet<String>,
    max_file_bytes: u64,
    since: Option<SystemTime>,
) -> Result<Vec<KnowledgeAtom>> {
    let mut by_source_id: HashMap<String, KnowledgeAtom> = HashMap::new();
    walk_code_files(&root, &extensions, &mut |path| {
        let Some((mut file, meta)) = open_regular_nofollow(path)? else {
            return Ok(());
        };
        let len = meta.len();
        if len > max_file_bytes {
            tracing::debug!(
                path = %path.display(),
                len,
                max = max_file_bytes,
                "github: skip oversized file"
            );
            return Ok(());
        }
        let mtime = meta.modified().ok();
        if let (Some(since), Some(mtime)) = (since, mtime) {
            if mtime <= since {
                return Ok(());
            }
        }

        let mut bytes = Vec::with_capacity(len as usize);
        file.read_to_end(&mut bytes).map_err(|e| {
            KurultaiError::connector("github", format!("read {}: {e}", path.display()))
        })?;
        let text = match String::from_utf8(bytes) {
            Ok(t) => t,
            Err(_) => {
                tracing::debug!(path = %path.display(), "github: skip non-utf8 file");
                return Ok(());
            }
        };

        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        let updated = mtime
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_else(Utc::now))
            .unwrap_or_else(Utc::now);

        for atom in file_to_atoms(&source_name, &rel, &text, updated) {
            by_source_id.insert(atom.source_id.clone(), atom);
        }
        Ok(())
    })?;

    Ok(by_source_id.into_values().collect())
}

/// Open a regular file without following symlinks. Returns `Ok(None)` to skip.
fn open_regular_nofollow(path: &Path) -> Result<Option<(File, std::fs::Metadata)>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = match fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
        {
            Ok(f) => f,
            Err(e) if e.raw_os_error() == Some(libc::ELOOP) => {
                tracing::debug!(path = %path.display(), "github: skip symlink");
                return Ok(None);
            }
            Err(e) => {
                return Err(KurultaiError::connector(
                    "github",
                    format!("open {}: {e}", path.display()),
                ));
            }
        };
        let meta = file.metadata().map_err(|e| {
            KurultaiError::connector("github", format!("fstat {}: {e}", path.display()))
        })?;
        if !meta.is_file() {
            return Ok(None);
        }
        Ok(Some((file, meta)))
    }
    #[cfg(not(unix))]
    {
        let meta = fs::symlink_metadata(path).map_err(|e| {
            KurultaiError::connector("github", format!("lstat {}: {e}", path.display()))
        })?;
        if meta.file_type().is_symlink() || !meta.is_file() {
            return Ok(None);
        }
        let file = File::open(path).map_err(|e| {
            KurultaiError::connector("github", format!("open {}: {e}", path.display()))
        })?;
        Ok(Some((file, meta)))
    }
}

fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.') || SKIP_DIRS.contains(&name)
}

fn walk_code_files(
    root: &Path,
    extensions: &HashSet<String>,
    visit: &mut dyn FnMut(&Path) -> Result<()>,
) -> Result<()> {
    let entries = fs::read_dir(root).map_err(|e| {
        KurultaiError::connector("github", format!("read_dir {}: {e}", root.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| KurultaiError::connector("github", e.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| KurultaiError::connector("github", e.to_string()))?;
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if should_skip_dir(name) {
                continue;
            }
            walk_code_files(&path, extensions, visit)?;
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| extensions.contains(&e.to_ascii_lowercase()))
        {
            visit(&path)?;
        }
    }
    Ok(())
}

fn file_to_atoms(
    source: &str,
    rel_path: &str,
    text: &str,
    source_updated_at: DateTime<Utc>,
) -> Vec<KnowledgeAtom> {
    let content = text.trim();
    if content.is_empty() {
        return Vec::new();
    }
    let pieces = split_by_words(content, MAX_CHUNK_WORDS);
    let chunk_count = pieces.len() as u32;
    pieces
        .into_iter()
        .enumerate()
        .map(|(i, body)| {
            let title = if chunk_count > 1 {
                format!("{rel_path}#c{i}")
            } else {
                rel_path.to_string()
            };
            let content = format!("[{rel_path}]\n{body}");
            make_atom(
                source,
                rel_path,
                &title,
                &content,
                source_updated_at,
                i as u32,
                chunk_count,
            )
        })
        .collect()
}

fn split_by_words(text: &str, max_words: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= max_words {
        return vec![text.trim().to_string()];
    }
    words.chunks(max_words).map(|c| c.join(" ")).collect()
}

fn make_atom(
    source: &str,
    rel_path: &str,
    title: &str,
    content: &str,
    source_updated_at: DateTime<Utc>,
    chunk_index: u32,
    chunk_count: u32,
) -> KnowledgeAtom {
    let source_id = if chunk_count > 1 {
        format!("{rel_path}#c{chunk_index}")
    } else {
        rel_path.to_string()
    };
    let hash = sha256_hex(content);
    let id = atom_id_from_hash(source, &source_id, &hash);
    let summary: String = content.chars().take(280).collect();
    let ext = Path::new(rel_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    KnowledgeAtom {
        id,
        source: source.to_string(),
        source_id,
        title: title.to_string(),
        summary,
        content: content.to_string(),
        question: None,
        resolution: None,
        tags: if ext.is_empty() {
            vec!["code".into()]
        } else {
            vec!["code".into(), ext]
        },
        source_updated_at,
        indexed_at: Utc::now(),
        embedding: None,
        metadata: HashMap::from([
            ("content_hash".into(), hash),
            ("rel_path".into(), rel_path.to_string()),
            ("chunk_index".into(), chunk_index.to_string()),
            ("chunk_count".into(), chunk_count.to_string()),
        ]),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceKind;
    use std::io::Write;

    fn fixture_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        let mut f = fs::File::create(src.join("lib.rs")).unwrap();
        writeln!(f, "// KNOWN_GITHUB_PHRASE_42\npub fn hello() {{}}").unwrap();

        // Should be skipped
        let nm = dir.path().join("node_modules").join("pkg");
        fs::create_dir_all(&nm).unwrap();
        let mut junk = fs::File::create(nm.join("index.js")).unwrap();
        writeln!(junk, "SHOULD_NOT_INDEX_NODE_MODULES").unwrap();

        dir
    }

    #[tokio::test]
    async fn full_sync_indexes_fixture_rs() {
        let dir = fixture_repo();
        let mut c = GitHubConnector::new();
        c.init(&SourceConfig {
            name: "pace".into(),
            kind: SourceKind::GitHub,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();

        let atoms = c.full_sync().await.unwrap();
        assert!(!atoms.is_empty());
        assert!(atoms.iter().all(|a| a.source == "pace"));
        assert!(atoms
            .iter()
            .any(|a| a.content.contains("KNOWN_GITHUB_PHRASE_42")));
        assert!(!atoms
            .iter()
            .any(|a| a.content.contains("SHOULD_NOT_INDEX_NODE_MODULES")));
    }

    #[tokio::test]
    async fn init_requires_root_path() {
        let mut c = GitHubConnector::new();
        let err = c
            .init(&SourceConfig {
                name: "gh".into(),
                kind: SourceKind::GitHub,
                enabled: true,
                poll_interval_secs: 60,
                extra: HashMap::new(),
            })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("root_path"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn skips_symlinks() {
        use std::os::unix::fs::symlink;
        let dir = fixture_repo();
        let target = dir.path().join("src/lib.rs");
        let link = dir.path().join("evil.rs");
        symlink(&target, &link).unwrap();
        let mut c = GitHubConnector::new();
        c.init(&SourceConfig {
            name: "pace".into(),
            kind: SourceKind::GitHub,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();
        let atoms = c.full_sync().await.unwrap();
        assert!(
            !atoms.iter().any(|a| {
                a.metadata.get("rel_path").is_some_and(|p| p == "evil.rs")
                    || a.source_id == "evil.rs"
                    || a.source_id.starts_with("evil.rs#")
            }),
            "symlink evil.rs must not produce atoms"
        );
    }

    #[tokio::test]
    async fn poll_skips_unchanged() {
        let dir = fixture_repo();
        let mut c = GitHubConnector::new();
        c.init(&SourceConfig {
            name: "pace".into(),
            kind: SourceKind::GitHub,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([(
                "root_path".into(),
                dir.path().to_string_lossy().into_owned(),
            )]),
        })
        .await
        .unwrap();
        let first = c.full_sync().await.unwrap();
        assert!(!first.is_empty());
        // Coarse-mtime overlap re-emits until watermark is ≥2s past file mtimes.
        tokio::time::sleep(Duration::from_secs(3)).await;
        let _advance = c.poll().await.unwrap();
        let quiet = c.poll().await.unwrap();
        assert!(quiet.is_empty());
    }
}
