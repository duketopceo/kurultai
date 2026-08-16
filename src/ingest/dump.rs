//! Shared dump atomizer for markdown, JSON/NDJSON, and plain text.
//!
//! Used by inbox tray, loopback `POST /ingest`, and folder markdown/json sources.
//! Stable `source_id` comes from the relative path (+ JSON record index / markdown chunk).

use crate::error::{KurultaiError, Result};
use crate::hashutil::{atom_id_from_hash, sha256_hex};
use crate::types::KnowledgeAtom;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata key for retrieval quality boost (`0.0`–`1.0` as decimal string).
pub const QUALITY_SCORE_KEY: &str = "quality_score";

/// Absolute path of an inbox tray file (used to finalize processed/failed moves).
pub const INBOX_META_PATH: &str = "_inbox_abs_path";

/// Absolute path of the inbox tray root (processed/failed live here).
pub const INBOX_META_ROOT: &str = "_inbox_root";

/// Max words per markdown heading chunk (matches historical markdown connector).
const MAX_CHUNK_WORDS: usize = 400;

/// Supported dump formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpFormat {
    Markdown,
    Json,
    Ndjson,
    PlainText,
}

/// Detect dump format from a file path extension.
pub fn detect_format(path: &Path) -> Option<DumpFormat> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("md") | Some("markdown") => Some(DumpFormat::Markdown),
        Some("json") => Some(DumpFormat::Json),
        Some("jsonl") | Some("ndjson") => Some(DumpFormat::Ndjson),
        Some("txt") | Some("text") => Some(DumpFormat::PlainText),
        _ => None,
    }
}

/// True when the path looks like a dump file we can atomize.
pub fn is_dump_file(path: &Path) -> bool {
    detect_format(path).is_some()
}

/// Cheap content richness score in `[0.0, 1.0]` for metadata + light retrieval boost.
pub fn compute_quality_score(content: &str) -> f32 {
    let trimmed = content.trim();
    let len = trimmed.chars().count();
    if len == 0 {
        return 0.0;
    }
    // Length arm: saturate around ~800 chars.
    let length_score = (len as f32 / 800.0).clamp(0.0, 1.0);
    let words = trimmed.split_whitespace().count();
    let word_score = (words as f32 / 80.0).clamp(0.0, 1.0);
    let unique: std::collections::HashSet<&str> = trimmed.split_whitespace().collect();
    let diversity = if words == 0 {
        0.0
    } else {
        unique.len() as f32 / words as f32
    };
    (0.5 * length_score + 0.3 * word_score + 0.2 * diversity).clamp(0.0, 1.0)
}

fn set_quality_score(meta: &mut HashMap<String, String>, content: &str) {
    let score = compute_quality_score(content);
    meta.insert(QUALITY_SCORE_KEY.into(), format!("{score:.4}"));
}

/// Atomize a file on disk.
pub fn atomize_path(
    source: &str,
    root: &Path,
    path: &Path,
    source_updated_at: DateTime<Utc>,
) -> Result<Vec<KnowledgeAtom>> {
    let _span = tracing::debug_span!("ingest_atomize", source, path = %path.display());
    let format = detect_format(path).ok_or_else(|| {
        KurultaiError::connector(
            source,
            format!("unsupported dump format: {}", path.display()),
        )
    })?;
    let text = fs::read_to_string(path)
        .map_err(|e| KurultaiError::connector(source, format!("read {}: {e}", path.display())))?;
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    atomize_bytes(source, &rel, text.as_bytes(), format, source_updated_at)
}

/// Atomize raw bytes with an explicit format (webhook / tests).
pub fn atomize_bytes(
    source: &str,
    rel_path: &str,
    bytes: &[u8],
    format: DumpFormat,
    source_updated_at: DateTime<Utc>,
) -> Result<Vec<KnowledgeAtom>> {
    let _span = tracing::debug_span!("ingest_atomize", source, rel_path, format = ?format);
    let text = std::str::from_utf8(bytes)
        .map_err(|e| KurultaiError::connector(source, format!("utf-8 decode {rel_path}: {e}")))?;
    match format {
        DumpFormat::Markdown => Ok(atomize_markdown(source, rel_path, text, source_updated_at)),
        DumpFormat::Json => atomize_json_array(source, rel_path, text, source_updated_at),
        DumpFormat::Ndjson => atomize_ndjson(source, rel_path, text, source_updated_at),
        DumpFormat::PlainText => Ok(atomize_plain(source, rel_path, text, source_updated_at)),
    }
}

// ── Markdown ─────────────────────────────────────────────────────────────────

fn atomize_markdown(
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
    // YAML frontmatter tags take priority; fall back to dedicated hashtag-line
    // tags scanned from the body when no frontmatter tags are present.
    let mut tags = parse_tags(fm.get("tags").map(String::as_str));
    if tags.is_empty() {
        tags = parse_hashtag_line_tags(body);
    }

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
            0,
            1,
        ));
        return atoms;
    }

    let chunk_count = chunks.len();
    for (chunk_index, chunk) in chunks.into_iter().enumerate() {
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
            chunk_index as u32,
            chunk_count as u32,
        ));
    }
    atoms
}

fn split_frontmatter(text: &str) -> (HashMap<String, String>, &str) {
    let text = text.trim_start_matches('\u{feff}');
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            let yaml = &rest[..end];
            let body = &rest[end + 5..];
            return (parse_simple_yaml(yaml), body);
        }
        if let Some(end) = rest.find("\n---") {
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
            map.insert(key, val);
        }
    }
    map
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

/// Scan markdown body for dedicated hashtag lines — lines composed entirely of
/// `#word` tokens (regex `^(\s*#\w+)+\s*$`). These are tag lines, NOT headings:
/// a heading like `# IT Doc` has a space after `#`, while a tag line like
/// `#ops #deploy` has no space between `#` and the word. Returns the deduped tag
/// set across all matching lines (tag word without the leading `#`).
fn parse_hashtag_line_tags(body: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in body.lines() {
        if let Some(line_tags) = parse_hashtag_line(line) {
            for tag in line_tags {
                if seen.insert(tag.clone()) {
                    tags.push(tag);
                }
            }
        }
    }
    tags
}

/// Parse a single line as a hashtag tag line. Returns `Some(tags)` when the line
/// is composed entirely of `#word` tokens (each `#` immediately followed by ≥1
/// word char), else `None`. A bare `#` or a heading (`# Heading`, `## Sub`)
/// does not match because the `#` is not immediately followed by a word char.
fn parse_hashtag_line(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut tags = Vec::new();
    for token in trimmed.split_whitespace() {
        let word = token.strip_prefix('#').unwrap_or(token);
        // Token must start with '#' (strip_prefix yielded the rest) and the rest
        // must be non-empty and all word chars (\w = [A-Za-z0-9_]).
        if word.len() == token.len()
            || word.is_empty()
            || !word.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return None;
        }
        tags.push(word.to_string());
    }
    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
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

// ── JSON / NDJSON ────────────────────────────────────────────────────────────

fn atomize_json_array(
    source: &str,
    rel_path: &str,
    text: &str,
    source_updated_at: DateTime<Utc>,
) -> Result<Vec<KnowledgeAtom>> {
    let v: Value = serde_json::from_str(text).map_err(|e| {
        KurultaiError::connector(source, format!("malformed JSON in {rel_path}: {e}"))
    })?;
    match v {
        Value::Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                if !item.is_object() {
                    return Err(KurultaiError::connector(
                        source,
                        format!("{rel_path}: element [{i}] is not a JSON object"),
                    ));
                }
            }
            Ok(records_to_atoms(source, rel_path, arr, source_updated_at))
        }
        Value::Object(obj) => Ok(records_to_atoms(
            source,
            rel_path,
            vec![Value::Object(obj)],
            source_updated_at,
        )),
        _ => Err(KurultaiError::connector(
            source,
            format!("{rel_path}: expected a JSON array or object at top level"),
        )),
    }
}

fn atomize_ndjson(
    source: &str,
    rel_path: &str,
    text: &str,
    source_updated_at: DateTime<Utc>,
) -> Result<Vec<KnowledgeAtom>> {
    let mut records = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        let v: Value = serde_json::from_str(line).map_err(|e| {
            KurultaiError::connector(
                source,
                format!("malformed JSON in {rel_path} line {}: {e}", line_no + 1),
            )
        })?;
        if !v.is_object() {
            return Err(KurultaiError::connector(
                source,
                format!(
                    "{rel_path} line {}: expected JSON object, got {}",
                    line_no + 1,
                    json_type_name(&v)
                ),
            ));
        }
        records.push(v);
    }
    Ok(records_to_atoms(
        source,
        rel_path,
        records,
        source_updated_at,
    ))
}

fn records_to_atoms(
    source: &str,
    rel_path: &str,
    records: Vec<Value>,
    source_updated_at: DateTime<Utc>,
) -> Vec<KnowledgeAtom> {
    let mut out = Vec::with_capacity(records.len());
    for (i, record) in records.into_iter().enumerate() {
        let obj = match record.as_object() {
            Some(o) => o,
            None => continue,
        };

        // KTD2: stable source_id from relative path + record index.
        let source_id = format!("{rel_path}/{i}");

        let title = obj
            .get("title")
            .or_else(|| obj.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| title_from_path(rel_path));

        let content = obj
            .get("content")
            .or_else(|| obj.get("body"))
            .or_else(|| obj.get("text"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| serde_json::to_string(obj).unwrap_or_else(|_| String::from("{}")));

        let tags = extract_json_tags(obj);

        let mut metadata: HashMap<String, String> = obj
            .iter()
            .filter(|(k, v)| {
                !matches!(
                    k.as_str(),
                    "title" | "name" | "content" | "body" | "text" | "tags"
                ) && v.is_string()
            })
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
            .collect();
        metadata.insert("rel_path".into(), rel_path.to_string());
        metadata.insert("record_index".into(), i.to_string());
        if let Some(id) = obj.get("id").and_then(Value::as_str) {
            metadata.insert("external_id".into(), id.to_string());
        }

        out.push(finish_atom(
            source,
            &source_id,
            &title,
            &content,
            tags,
            metadata,
            source_updated_at,
        ));
    }
    out
}

fn extract_json_tags(obj: &serde_json::Map<String, Value>) -> Vec<String> {
    let Some(raw) = obj.get("tags") else {
        return vec![];
    };
    match raw {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect(),
        Value::String(s) => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => vec![],
    }
}

fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// ── Plain text ───────────────────────────────────────────────────────────────

fn atomize_plain(
    source: &str,
    rel_path: &str,
    text: &str,
    source_updated_at: DateTime<Utc>,
) -> Vec<KnowledgeAtom> {
    let content = text.trim();
    if content.is_empty() {
        return vec![];
    }
    let title = title_from_path(rel_path);
    let metadata = HashMap::from([("rel_path".into(), rel_path.to_string())]);
    // Plain dumps have no tags — gate will quarantine as untagged unless caller adds tags.
    vec![finish_atom(
        source,
        rel_path,
        &title,
        content,
        vec![],
        metadata,
        source_updated_at,
    )]
}

// ── Atom builders ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn make_atom(
    source: &str,
    rel_path: &str,
    heading: Option<&str>,
    title: &str,
    content: &str,
    tags: &[String],
    source_updated_at: DateTime<Utc>,
    chunk_index: u32,
    chunk_count: u32,
) -> KnowledgeAtom {
    let source_id = match heading {
        Some(h) if !h.is_empty() => format!("{rel_path}#{h}#c{chunk_index}"),
        _ if chunk_count > 1 => format!("{rel_path}#c{chunk_index}"),
        _ => rel_path.to_string(),
    };
    let mut metadata = HashMap::from([
        ("rel_path".into(), rel_path.to_string()),
        ("chunk_index".into(), chunk_index.to_string()),
        ("chunk_count".into(), chunk_count.to_string()),
    ]);
    if let Some(h) = heading.filter(|h| !h.is_empty()) {
        metadata.insert("heading".into(), h.to_string());
    }
    finish_atom(
        source,
        &source_id,
        title,
        content,
        tags.to_vec(),
        metadata,
        source_updated_at,
    )
}

fn finish_atom(
    source: &str,
    source_id: &str,
    title: &str,
    content: &str,
    tags: Vec<String>,
    mut metadata: HashMap<String, String>,
    source_updated_at: DateTime<Utc>,
) -> KnowledgeAtom {
    let hash = sha256_hex(content);
    metadata.insert("content_hash".into(), hash.clone());
    set_quality_score(&mut metadata, content);
    let id = atom_id_from_hash(source, source_id, &hash);
    let summary: String = content.chars().take(280).collect();
    KnowledgeAtom {
        id,
        source: source.to_string(),
        source_id: source_id.to_string(),
        title: title.to_string(),
        summary,
        content: content.to_string(),
        question: None,
        resolution: None,
        tags,
        soft_labels: vec![],
        source_updated_at,
        indexed_at: Utc::now(),
        embedding: None,
        metadata,
        ..Default::default()
    }
}

fn title_from_path(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(rel)
        .replace(['-', '_'], " ")
}

/// Walk a directory for dump files, skipping hidden dirs and optional exclude names
/// (e.g. `processed`, `failed` for inbox).
pub fn walk_dump_files(
    root: &Path,
    exclude_dir_names: &[&str],
    visit: &mut dyn FnMut(&Path) -> Result<()>,
) -> Result<()> {
    let entries = fs::read_dir(root).map_err(|e| {
        KurultaiError::connector("dump", format!("read_dir {}: {e}", root.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| KurultaiError::connector("dump", e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if name.starts_with('.') || exclude_dir_names.contains(&name) {
                continue;
            }
            walk_dump_files(&path, exclude_dir_names, visit)?;
        } else if is_dump_file(&path) {
            visit(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{content}").unwrap();
        path
    }

    #[test]
    fn detect_formats() {
        assert_eq!(detect_format(Path::new("a.md")), Some(DumpFormat::Markdown));
        assert_eq!(detect_format(Path::new("a.json")), Some(DumpFormat::Json));
        assert_eq!(
            detect_format(Path::new("a.ndjson")),
            Some(DumpFormat::Ndjson)
        );
        assert_eq!(
            detect_format(Path::new("a.txt")),
            Some(DumpFormat::PlainText)
        );
        assert_eq!(detect_format(Path::new("a.bin")), None);
    }

    #[test]
    fn markdown_tagged_dump_stable_source_id() {
        let text = r#"---
title: Deploy Guide
tags: [ops, k8s]
---
Intro paragraph with enough content to be useful for operators.

## Database migration
Run the database migration scripts carefully and verify checksums.
"#;
        let atoms = atomize_bytes(
            "notes",
            "ops/deploy.md",
            text.as_bytes(),
            DumpFormat::Markdown,
            Utc::now(),
        )
        .unwrap();
        assert!(atoms.len() >= 2);
        assert!(atoms.iter().all(|a| a.tags.contains(&"ops".into())));
        assert!(atoms
            .iter()
            .all(|a| a.metadata.contains_key(QUALITY_SCORE_KEY)));
        assert!(atoms.iter().any(|a| a.source_id.contains("ops/deploy.md")));
        // Re-atomize must produce the same ids (stable source_id + content hash).
        let again = atomize_bytes(
            "notes",
            "ops/deploy.md",
            text.as_bytes(),
            DumpFormat::Markdown,
            Utc::now(),
        )
        .unwrap();
        let ids: Vec<_> = atoms.iter().map(|a| &a.id).collect();
        let ids2: Vec<_> = again.iter().map(|a| &a.id).collect();
        assert_eq!(ids, ids2);
    }

    #[test]
    fn json_record_index_source_id() {
        let text = r#"[{"id":"a1","title":"First","content":"Hello world unique content here","tags":["rust"]}]"#;
        let atoms = atomize_bytes(
            "data",
            "docs/data.json",
            text.as_bytes(),
            DumpFormat::Json,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].source_id, "docs/data.json/0");
        assert_eq!(
            atoms[0].metadata.get("external_id").map(String::as_str),
            Some("a1")
        );
    }

    #[test]
    fn plain_text_untagged() {
        let atoms = atomize_bytes(
            "inbox",
            "note.txt",
            b"plain dump body with enough characters for a note",
            DumpFormat::PlainText,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].tags.is_empty());
        assert_eq!(atoms[0].source_id, "note.txt");
    }

    #[test]
    fn quality_score_increases_with_length() {
        let short = compute_quality_score("short");
        let long = compute_quality_score(
            "This is a longer piece of knowledge content with many distinct words about deployments, migrations, and rollback procedures for the cluster.",
        );
        assert!(long > short);
        assert!((0.0..=1.0).contains(&long));
    }

    #[test]
    fn atomize_path_reads_file() {
        let dir = tempfile::tempdir().unwrap();
        write_temp(
            dir.path(),
            "x.md",
            "---\ntags: [t]\n---\n\nBody with enough text for a dump atom here.\n",
        );
        let atoms = atomize_path("src", dir.path(), &dir.path().join("x.md"), Utc::now()).unwrap();
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].tags.contains(&"t".into()));
    }

    #[test]
    fn ndjson_and_single_object_json() {
        let nd =
            "{\"title\":\"A\",\"content\":\"ndjson body content for atom\",\"tags\":[\"n\"]}\n";
        let atoms = atomize_bytes(
            "s",
            "f.ndjson",
            nd.as_bytes(),
            DumpFormat::Ndjson,
            Utc::now(),
        )
        .unwrap();
        assert_eq!(atoms.len(), 1);

        let obj = r#"{"title":"B","content":"single object json body content","tags":["j"]}"#;
        let atoms =
            atomize_bytes("s", "f.json", obj.as_bytes(), DumpFormat::Json, Utc::now()).unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].source_id, "f.json/0");
    }
}
