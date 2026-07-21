use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// Max words per heading chunk (mdvault-inspired).
const MAX_CHUNK_WORDS: usize = 400;

/// Indexes `.md` files from any directory on disk.
///
/// Obsidian, Logseq, git wikis, etc. are just folders — no desktop app integration.
/// Config: `kind = "markdown"`, `root_path = "/path/to/notes"`.
pub struct MarkdownConnector {
    /// Config source name (e.g. `notes`) — stored on each atom for delete_source.
    source_name: String,
    root_path: Option<PathBuf>,
    /// Watermark for incremental poll (mtime).
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

    fn collect_atoms(&self, since: Option<SystemTime>) -> Result<Vec<KnowledgeAtom>> {
        let root = self
            .root_path
            .as_ref()
            .ok_or_else(|| KurultaiError::connector("markdown", "not initialized"))?;

        let mut atoms = Vec::new();
        walk_md_files(root, &mut |path| {
            let meta = fs::metadata(path).map_err(|e| {
                KurultaiError::connector("markdown", format!("stat {}: {e}", path.display()))
            })?;
            let mtime = meta.modified().ok();
            if let (Some(since), Some(mtime)) = (since, mtime) {
                if mtime <= since {
                    return Ok(());
                }
            }

            let text = fs::read_to_string(path).map_err(|e| {
                KurultaiError::connector("markdown", format!("read {}: {e}", path.display()))
            })?;

            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let updated = mtime
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_else(Utc::now))
                .unwrap_or_else(Utc::now);

            atoms.extend(file_to_atoms(&self.source_name, &rel, &text, updated));
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

fn walk_md_files(root: &Path, visit: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    let entries = fs::read_dir(root).map_err(|e| {
        KurultaiError::connector("markdown", format!("read_dir {}: {e}", root.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| KurultaiError::connector("markdown", e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden dirs like .obsidian
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            walk_md_files(&path, visit)?;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("md"))
        {
            visit(&path)?;
        }
    }
    Ok(())
}

/// Parse optional YAML-ish frontmatter and body.
fn split_frontmatter(text: &str) -> (HashMap<String, String>, &str) {
    let text = text.trim_start_matches('\u{feff}');
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            let yaml = &rest[..end];
            let body = &rest[end + 5..];
            return (parse_simple_yaml(yaml), body);
        }
        if let Some(end) = rest.find("\n---") {
            // trailing --- at EOF
            if rest[end + 4..].trim().is_empty() {
                return (parse_simple_yaml(&rest[..end]), "");
            }
        }
    }
    (HashMap::new(), text)
}

fn parse_simple_yaml(yaml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_string();
            let mut val = v.trim().to_string();
            if (val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\''))
            {
                val = val[1..val.len() - 1].to_string();
            }
            // tags: [a, b] → store raw; also support tags: a, b
            map.insert(key, val);
        }
    }
    map
}

fn file_to_atoms(
    source: &str,
    rel_path: &str,
    text: &str,
    source_updated_at: DateTime<Utc>,
) -> Vec<KnowledgeAtom> {
    let (fm, body) = split_frontmatter(text);
    let file_title = fm
        .get("title")
        .cloned()
        .unwrap_or_else(|| title_from_path(rel_path));
    let tags = parse_tags(fm.get("tags").map(String::as_str));

    let chunks = chunk_markdown(body);
    let mut atoms = Vec::with_capacity(chunks.len().max(1));

    if chunks.is_empty() {
        let content = body.trim();
        if content.is_empty() {
            return atoms;
        }
        atoms.push(make_atom(
            source,
            rel_path,
            None,
            &file_title,
            content,
            &tags,
            source_updated_at,
        ));
        return atoms;
    }

    for chunk in chunks {
        let title = if chunk.heading.is_empty() {
            file_title.clone()
        } else {
            format!("{} — {}", file_title, chunk.heading)
        };
        let prefix = format!("[{rel_path} > {file_title} > {}]", chunk.heading);
        let content = if chunk.heading.is_empty() {
            chunk.body
        } else {
            format!("{prefix}\n{}", chunk.body)
        };
        atoms.push(make_atom(
            source,
            rel_path,
            Some(&chunk.heading),
            &title,
            &content,
            &tags,
            source_updated_at,
        ));
    }
    atoms
}

struct Chunk {
    heading: String,
    body: String,
}

fn chunk_markdown(body: &str) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut current_heading = String::new();
    let mut current_body = String::new();

    for line in body.lines() {
        let is_heading = line.starts_with("## ") || line.starts_with("### ");
        if is_heading {
            flush_chunk(&mut chunks, &current_heading, &mut current_body);
            current_heading = line.trim_start_matches('#').trim().to_string();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    flush_chunk(&mut chunks, &current_heading, &mut current_body);

    // Split oversized chunks by word count
    let mut out = Vec::new();
    for chunk in chunks {
        for piece in split_by_words(&chunk.body, MAX_CHUNK_WORDS) {
            if piece.trim().is_empty() {
                continue;
            }
            out.push(Chunk {
                heading: chunk.heading.clone(),
                body: piece,
            });
        }
    }
    out
}

fn flush_chunk(chunks: &mut Vec<Chunk>, heading: &str, body: &mut String) {
    let trimmed = body.trim().to_string();
    if !trimmed.is_empty() || !heading.is_empty() {
        chunks.push(Chunk {
            heading: heading.to_string(),
            body: trimmed,
        });
    }
    body.clear();
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
    heading: Option<&str>,
    title: &str,
    content: &str,
    tags: &[String],
    source_updated_at: DateTime<Utc>,
) -> KnowledgeAtom {
    let source_id = match heading {
        Some(h) if !h.is_empty() => format!("{rel_path}#{h}"),
        _ => rel_path.to_string(),
    };
    let hash = sha256_hex(content);
    let id = sha256_hex(&format!("{source}\0{source_id}\0{hash}"));
    let summary: String = content.chars().take(280).collect();

    KnowledgeAtom {
        id,
        source: source.to_string(),
        source_id,
        title: title.to_string(),
        summary,
        content: content.to_string(),
        question: None,
        resolution: None,
        tags: tags.to_vec(),
        source_updated_at,
        indexed_at: Utc::now(),
        embedding: None,
        metadata: HashMap::from([("content_hash".into(), hash)]),
    }
}

fn title_from_path(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(rel)
        .replace(['-', '_'], " ")
}

fn parse_tags(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw else {
        return vec![];
    };
    let raw = raw.trim().trim_start_matches('[').trim_end_matches(']');
    raw.split(',')
        .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let bytes = hasher.finalize();
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in &bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn frontmatter_and_chunks() {
        let text = r#"---
title: Deploy Guide
tags: [ops, k8s]
---
Intro paragraph.

## Database migration
Run the database migration scripts carefully.

## Rollback
How to rollback a bad deploy.
"#;
        let atoms = file_to_atoms("notes", "ops/deploy.md", text, Utc::now());
        assert!(atoms.len() >= 2);
        assert!(atoms
            .iter()
            .any(|a| a.content.contains("database migration")));
        assert!(atoms.iter().all(|a| a.source == "notes"));
        assert!(atoms.iter().any(|a| a.tags.contains(&"ops".into())));
    }

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
            "---\ntitle: Fixture Note\n---\n\n## Section\nKNOWN_PHRASE_KURULTAI_42 appears here.\n"
        )
        .unwrap();

        let mut connector = MarkdownConnector::new();
        let mut extra = HashMap::new();
        extra.insert("root_path".into(), dir.to_string_lossy().into_owned());
        let config = SourceConfig {
            name: "notes".into(),
            kind: crate::types::SourceKind::Markdown,
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
}
