//! Inbox tray connector — dump files in, successes to `processed/`, failures to `failed/`.
//!
//! Tray finalization runs after the index pipeline gates and upserts (see
//! [`finalize_inbox_batch`]). Parse failures are moved immediately during collect.

use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::ingest::dump::{self, INBOX_META_PATH, INBOX_META_ROOT};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig, TrustLane};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

const PROCESSED: &str = "processed";
const FAILED: &str = "failed";

/// Local dump inbox with processed/failed trays.
pub struct InboxConnector {
    source_name: String,
    root_path: Option<PathBuf>,
    last_poll: Mutex<Option<SystemTime>>,
}

impl InboxConnector {
    pub fn new() -> Self {
        Self {
            source_name: "inbox".into(),
            root_path: None,
            last_poll: Mutex::new(None),
        }
    }

    fn collect_atoms(&self, since: Option<SystemTime>) -> Result<Vec<KnowledgeAtom>> {
        let root = self
            .root_path
            .as_ref()
            .ok_or_else(|| KurultaiError::connector(&self.source_name, "not initialized"))?;

        ensure_tray_dirs(root)?;

        let mut atoms = Vec::new();
        dump::walk_dump_files(root, &[PROCESSED, FAILED], &mut |path| {
            let meta = fs::metadata(path).map_err(|e| {
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

            match dump::atomize_path(&self.source_name, root, path, updated) {
                Ok(mut file_atoms) => {
                    let abs = path.to_string_lossy().into_owned();
                    let root_s = root.to_string_lossy().into_owned();
                    for a in &mut file_atoms {
                        a.metadata.insert(INBOX_META_PATH.into(), abs.clone());
                        a.metadata.insert(INBOX_META_ROOT.into(), root_s.clone());
                    }
                    atoms.extend(file_atoms);
                    Ok(())
                }
                Err(e) => {
                    // Parse / format failure → failed/ + reason; no atoms stored for this file.
                    let _ = move_to_failed(root, path, &e.to_string());
                    Ok(())
                }
            }
        })?;

        Ok(atoms)
    }
}

impl Default for InboxConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for InboxConnector {
    fn name(&self) -> &str {
        "inbox"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        self.source_name = config.name.clone();
        let root = config.extra.get("root_path").ok_or_else(|| {
            KurultaiError::connector(&config.name, "root_path required for inbox source")
        })?;
        let resolved = validate_readable_path(root, "inbox root")?;
        ensure_tray_dirs(&resolved)?;
        tracing::debug!(root = %resolved.display(), "inbox connector initialized");
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

fn ensure_tray_dirs(root: &Path) -> Result<()> {
    for name in [PROCESSED, FAILED] {
        let p = root.join(name);
        if !p.exists() {
            fs::create_dir_all(&p).map_err(|e| {
                KurultaiError::connector("inbox", format!("mkdir {}: {e}", p.display()))
            })?;
        }
    }
    Ok(())
}

fn unique_dest(dir: &Path, file_name: &str) -> PathBuf {
    let dest = dir.join(file_name);
    if !dest.exists() {
        return dest;
    }
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    for i in 1..10_000 {
        let candidate = if ext.is_empty() {
            dir.join(format!("{stem}.{i}"))
        } else {
            dir.join(format!("{stem}.{i}.{ext}"))
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!(
        "{stem}.{}.{}",
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        ext
    ))
}

fn move_to_processed(root: &Path, abs_path: &Path) -> Result<()> {
    let processed = root.join(PROCESSED);
    fs::create_dir_all(&processed).map_err(|e| {
        KurultaiError::connector("inbox", format!("mkdir processed: {e}"))
    })?;
    let name = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("dump");
    let dest = unique_dest(&processed, name);
    fs::rename(abs_path, &dest).map_err(|e| {
        KurultaiError::connector(
            "inbox",
            format!("move {} → {}: {e}", abs_path.display(), dest.display()),
        )
    })?;
    Ok(())
}

fn move_to_failed(root: &Path, abs_path: &Path, reason: &str) -> Result<()> {
    let failed = root.join(FAILED);
    fs::create_dir_all(&failed)
        .map_err(|e| KurultaiError::connector("inbox", format!("mkdir failed: {e}")))?;
    let name = abs_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("dump");
    let dest = unique_dest(&failed, name);
    fs::rename(abs_path, &dest).map_err(|e| {
        KurultaiError::connector(
            "inbox",
            format!("move {} → {}: {e}", abs_path.display(), dest.display()),
        )
    })?;
    // Sidecar: `{moved-filename}.reason.txt`
    let sidecar = PathBuf::from(format!("{}.reason.txt", dest.to_string_lossy()));
    fs::write(&sidecar, reason).map_err(|e| {
        KurultaiError::connector("inbox", format!("write reason {}: {e}", sidecar.display()))
    })?;
    Ok(())
}

/// After gate+upsert, move inbox files to processed/ or failed/ based on atom outcomes.
///
/// A file is `processed/` only when every atom from that file is trusted.
/// Quarantine (including untagged / low_quality) → `failed/` + reason sidecar; atoms remain stored.
pub fn finalize_inbox_batch(atoms: &[KnowledgeAtom]) -> Result<()> {
    use std::collections::HashMap;
    let mut by_path: HashMap<String, (PathBuf, Vec<&KnowledgeAtom>)> = HashMap::new();
    for a in atoms {
        let Some(p) = a.metadata.get(INBOX_META_PATH) else {
            continue;
        };
        let root = a
            .metadata
            .get(INBOX_META_ROOT)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(p)
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf()
            });
        by_path
            .entry(p.clone())
            .or_insert_with(|| (root, Vec::new()))
            .1
            .push(a);
    }

    for (abs, (root, group)) in by_path {
        let path = PathBuf::from(&abs);
        if !path.exists() {
            // Already moved (e.g. re-index) — skip.
            continue;
        }
        let all_trusted = group.iter().all(|a| a.trust_lane == TrustLane::Trusted);
        if all_trusted {
            move_to_processed(&root, &path)?;
        } else {
            let reason = group
                .iter()
                .find_map(|a| a.quarantine_reason.as_deref())
                .unwrap_or("quarantine");
            move_to_failed(&root, &path, reason)?;
        }
    }
    Ok(())
}

/// Count pending dump files and failed entries for status surfaces.
pub fn inbox_tray_counts(root: &Path) -> (u64, u64) {
    let mut pending = 0u64;
    let _ = dump::walk_dump_files(root, &[PROCESSED, FAILED], &mut |_| {
        pending += 1;
        Ok(())
    });
    let failed = root
        .join(FAILED)
        .read_dir()
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| {
                    let p = e.path();
                    p.is_file()
                        && dump::is_dump_file(&p)
                })
                .count() as u64
        })
        .unwrap_or(0);
    (pending, failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceKind;
    use std::collections::HashMap;
    use std::io::Write;

    #[tokio::test]
    async fn parse_fail_moves_to_failed() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = fs::File::create(dir.path().join("bad.json")).unwrap();
        write!(f, "not json{{{{").unwrap();

        let mut connector = InboxConnector::new();
        connector
            .init(&SourceConfig {
                name: "inbox".into(),
                kind: SourceKind::Inbox,
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
        assert!(atoms.is_empty());
        assert!(!dir.path().join("bad.json").exists());
        let failed_dir = dir.path().join(FAILED);
        assert!(failed_dir
            .read_dir()
            .unwrap()
            .any(|e| e.unwrap().file_name().to_string_lossy().starts_with("bad")));
    }

    #[tokio::test]
    async fn collect_attaches_inbox_meta_path() {
        let dir = tempfile::tempdir().unwrap();
        let body = "---\ntags: [ops]\n---\n\nDetailed inbox dump about cluster rollout verification steps.\n";
        fs::write(dir.path().join("note.md"), body).unwrap();

        let mut connector = InboxConnector::new();
        connector
            .init(&SourceConfig {
                name: "tray".into(),
                kind: SourceKind::Inbox,
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
        assert!(!atoms.is_empty());
        assert!(atoms.iter().all(|a| a.metadata.contains_key(INBOX_META_PATH)));
    }
}
