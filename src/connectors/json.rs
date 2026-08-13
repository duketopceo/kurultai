//! JSON / NDJSON file connector.
//!
//! Reads `.json` array-of-objects or `.jsonl` / `.ndjson` line-delimited JSON
//! files from a `root_path` directory.
//!
//! # Config
//! ```toml
//! [[sources]]
//! name     = "data-dump"
//! kind     = "Json"
//! enabled  = true
//! root_path = "/path/to/json-files"
//! extra    = { id_field = "url" }   # optional: use a field as stable source_id
//! # visibility = "team"             # optional source default (HUB-5); fail-closed personal
//! ```
//!
//! # Atom mapping
//! Each JSON object is mapped to a `KnowledgeAtom`:
//! - `title`  — `title` field, or `name`, or `id`, or `"<source>/<index>"`
//! - `content` — `content` field, or `body`, or `text`, or the full serialised object
//! - `tags`   — `tags` field (array or comma-separated string), or `[]
//! - `source_id` — value of `id_field` extra config (default: `"id"`), falling
//!   back to `"<rel_path>/<index>"`
//! - `visibility` — per-record override of source default (`personal`|`team`|`company`)

use crate::connectors::{source_visibility, Connector};
use crate::error::{KurultaiError, Result};
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig, VisibilityScope};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// Reads JSON / NDJSON files from a local directory tree.
pub struct JsonConnector {
    source_name: String,
    root_path: Option<PathBuf>,
    id_field: String,
    /// Source-level default; per-record `visibility` may override (HUB-5).
    visibility: VisibilityScope,
    last_poll: Mutex<Option<SystemTime>>,
}

impl JsonConnector {
    pub fn new() -> Self {
        Self {
            source_name: "json".into(),
            root_path: None,
            id_field: "id".into(),
            visibility: VisibilityScope::Personal,
            last_poll: Mutex::new(None),
        }
    }

    fn collect_atoms(&self, since: Option<SystemTime>) -> Result<Vec<KnowledgeAtom>> {
        let root = self
            .root_path
            .as_ref()
            .ok_or_else(|| KurultaiError::connector(&self.source_name, "not initialized"))?;

        let mut atoms = Vec::new();
        walk_json_files(root, &mut |path| {
            let meta = fs::metadata(path).map_err(|e| {
                KurultaiError::connector(&self.source_name, format!("stat {}: {e}", path.display()))
            })?;
            let mtime = meta.modified().ok();
            if let (Some(since), Some(mtime)) = (since, mtime) {
                if mtime <= since {
                    return Ok(());
                }
            }

            let rel = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            let records = parse_json_file(path, &self.source_name)?;
            let file_atoms = records_to_atoms(
                &self.source_name,
                &rel,
                &self.id_field,
                self.visibility,
                records,
            );
            atoms.extend(file_atoms);
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
        self.visibility = source_visibility(config);

        let root = config.extra.get("root_path").ok_or_else(|| {
            KurultaiError::connector(&config.name, "root_path required for json source")
        })?;
        let resolved = validate_readable_path(root, "json root")?;
        tracing::debug!(
            root = %resolved.display(),
            visibility = self.visibility.as_str(),
            "json connector initialized"
        );
        self.root_path = Some(resolved);

        if let Some(id_field) = config.extra.get("id_field") {
            self.id_field = id_field.clone();
        }

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

// ── File traversal ───────────────────────────────────────────────────────────

fn walk_json_files(root: &Path, visit: &mut dyn FnMut(&Path) -> Result<()>) -> Result<()> {
    let entries = fs::read_dir(root).map_err(|e| {
        KurultaiError::connector("json", format!("read_dir {}: {e}", root.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| KurultaiError::connector("json", e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue; // skip hidden dirs
            }
            walk_json_files(&path, visit)?;
        } else if is_json_file(&path) {
            visit(&path)?;
        }
    }
    Ok(())
}

fn is_json_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "json" | "jsonl" | "ndjson"
        ),
        None => false,
    }
}

// ── Parsing ──────────────────────────────────────────────────────────────────

/// Parse a single file into a list of JSON objects (records).
///
/// `.json` files must contain a top-level JSON array of objects.  
/// `.jsonl` / `.ndjson` files contain one JSON object per line (blank lines
/// and comment lines starting with `//` are silently skipped).
fn parse_json_file(path: &Path, source_name: &str) -> Result<Vec<Value>> {
    let text = fs::read_to_string(path).map_err(|e| {
        KurultaiError::connector(source_name, format!("read {}: {e}", path.display()))
    })?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "json" {
        // Expect a JSON array at the top level.
        let v: Value = serde_json::from_str(&text).map_err(|e| {
            KurultaiError::connector(
                source_name,
                format!("malformed JSON in {}: {e}", path.display()),
            )
        })?;
        match v {
            Value::Array(arr) => {
                // Each element must be an object.
                for (i, item) in arr.iter().enumerate() {
                    if !item.is_object() {
                        return Err(KurultaiError::connector(
                            source_name,
                            format!("{}: element [{}] is not a JSON object", path.display(), i),
                        ));
                    }
                }
                Ok(arr)
            }
            _ => Err(KurultaiError::connector(
                source_name,
                format!(
                    "{}: expected a JSON array at top level, got {}",
                    path.display(),
                    v.type_name()
                ),
            )),
        }
    } else {
        // NDJSON / JSONL: one object per line.
        let mut records = Vec::new();
        for (line_no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let v: Value = serde_json::from_str(line).map_err(|e| {
                KurultaiError::connector(
                    source_name,
                    format!(
                        "malformed JSON in {} line {}: {e}",
                        path.display(),
                        line_no + 1
                    ),
                )
            })?;
            if !v.is_object() {
                return Err(KurultaiError::connector(
                    source_name,
                    format!(
                        "{} line {}: expected JSON object, got {}",
                        path.display(),
                        line_no + 1,
                        v.type_name()
                    ),
                ));
            }
            records.push(v);
        }
        Ok(records)
    }
}

// ── Atom mapping ─────────────────────────────────────────────────────────────

/// Map a slice of JSON objects (from one file) to `KnowledgeAtom`s.
fn records_to_atoms(
    source: &str,
    rel_path: &str,
    id_field: &str,
    default_visibility: VisibilityScope,
    records: Vec<Value>,
) -> Vec<KnowledgeAtom> {
    let mut out = Vec::with_capacity(records.len());
    for (i, record) in records.into_iter().enumerate() {
        let obj = match record.as_object() {
            Some(o) => o,
            None => continue,
        };

        let source_id = obj
            .get(id_field)
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("{rel_path}/{i}"));

        let title = obj
            .get("title")
            .or_else(|| obj.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("{source}/{i}"));

        let content = obj
            .get("content")
            .or_else(|| obj.get("body"))
            .or_else(|| obj.get("text"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| serde_json::to_string(obj).unwrap_or_else(|_| String::from("{}")));

        let tags = extract_tags(obj);
        let visibility = obj
            .get("visibility")
            .and_then(Value::as_str)
            .map(VisibilityScope::parse)
            .unwrap_or(default_visibility);

        // Collect any remaining scalar fields as atom metadata.
        let metadata: HashMap<String, String> = obj
            .iter()
            .filter(|(k, v)| {
                !matches!(
                    k.as_str(),
                    "title" | "name" | "content" | "body" | "text" | "tags" | "visibility"
                ) && v.is_string()
            })
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
            .collect();

        let content_hash = crate::hashutil::sha256_hex(&content);
        let id = crate::hashutil::atom_id_from_hash(source, &source_id, &content_hash);
        let summary: String = content.chars().take(280).collect();

        out.push(KnowledgeAtom {
            id,
            source: source.to_string(),
            source_id,
            title,
            summary,
            content,
            question: None,
            resolution: None,
            tags,
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: None,
            metadata,
            visibility,
            ..Default::default()
        });
    }
    out
}

fn extract_tags(obj: &serde_json::Map<String, Value>) -> Vec<String> {
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

/// Extension method to get a human-readable type name for errors.
trait JsonTypeName {
    fn type_name(&self) -> &'static str;
}

impl JsonTypeName for Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{content}").unwrap();
        path
    }

    #[test]
    fn parse_json_array_of_objects() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"[
          {"id": "a1", "title": "First", "content": "Hello world", "tags": ["rust","test"]},
          {"id": "a2", "title": "Second", "content": "Foo bar"}
        ]"#;
        let path = write_temp_file(dir.path(), "data.json", content);
        let records = parse_json_file(&path, "json").unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["title"], "First");
    }

    #[test]
    fn parse_ndjson_records() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{"id": "x1", "title": "Line1", "content": "abc"}
{"id": "x2", "title": "Line2", "content": "def"}
"#;
        let path = write_temp_file(dir.path(), "data.ndjson", content);
        let records = parse_json_file(&path, "json").unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn parse_jsonl_skips_blank_lines() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"{"id": "y1", "content": "first"}

{"id": "y2", "content": "second"}
"#;
        let path = write_temp_file(dir.path(), "data.jsonl", content);
        let records = parse_json_file(&path, "json").unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn parse_malformed_json_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_file(dir.path(), "bad.json", "not json at all {{");
        let err = parse_json_file(&path, "json").unwrap_err();
        assert!(err.to_string().contains("malformed JSON"));
    }

    #[test]
    fn parse_non_array_json_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_file(dir.path(), "scalar.json", "{\"key\":\"val\"}");
        // A single object (not wrapped in array) should fail.
        let err = parse_json_file(&path, "json").unwrap_err();
        assert!(err.to_string().contains("expected a JSON array"));
    }

    #[test]
    fn records_to_atoms_maps_fields_correctly() {
        let records: Vec<Value> = serde_json::from_str(
            r#"[{"id": "doc1", "title": "My Doc", "content": "The quick brown fox", "tags": ["rust"]}]"#,
        )
        .unwrap();
        let atoms = records_to_atoms(
            "test-src",
            "docs/data.json",
            "id",
            VisibilityScope::Personal,
            records,
        );
        assert_eq!(atoms.len(), 1);
        let a = &atoms[0];
        assert_eq!(a.source_id, "doc1");
        assert_eq!(a.title, "My Doc");
        assert!(a.content.contains("quick brown fox"));
        assert!(a.tags.contains(&"rust".to_string()));
        assert_eq!(a.source, "test-src");
        assert_eq!(a.visibility, VisibilityScope::Personal);
    }

    #[test]
    fn records_to_atoms_falls_back_for_missing_fields() {
        let records: Vec<Value> =
            serde_json::from_str(r#"[{"description": "no standard fields"}]"#).unwrap();
        let atoms = records_to_atoms("src", "f.json", "id", VisibilityScope::Personal, records);
        assert_eq!(atoms.len(), 1);
        // Source_id falls back to "f.json/0"
        assert_eq!(atoms[0].source_id, "f.json/0");
        // Title falls back to "src/0"
        assert_eq!(atoms[0].title, "src/0");
        // Content is the full serialized object
        assert!(atoms[0].content.contains("description"));
    }

    #[test]
    fn records_to_atoms_uses_source_default_visibility() {
        let records: Vec<Value> =
            serde_json::from_str(r#"[{"id": "t1", "content": "shared ops note"}]"#).unwrap();
        let atoms = records_to_atoms("src", "f.json", "id", VisibilityScope::Team, records);
        assert_eq!(atoms[0].visibility, VisibilityScope::Team);
    }

    #[test]
    fn records_to_atoms_per_record_visibility_overrides_default() {
        let records: Vec<Value> = serde_json::from_str(
            r#"[
              {"id": "a", "content": "dm", "visibility": "personal"},
              {"id": "b", "content": "channel", "visibility": "team"},
              {"id": "c", "content": "all-hands", "visibility": "company"}
            ]"#,
        )
        .unwrap();
        let atoms = records_to_atoms("src", "f.json", "id", VisibilityScope::Company, records);
        assert_eq!(atoms[0].visibility, VisibilityScope::Personal);
        assert_eq!(atoms[1].visibility, VisibilityScope::Team);
        assert_eq!(atoms[2].visibility, VisibilityScope::Company);
    }

    #[test]
    fn records_to_atoms_unknown_visibility_fail_closed() {
        let records: Vec<Value> = serde_json::from_str(
            r#"[{"id": "x", "content": "x", "visibility": "everyone"}]"#,
        )
        .unwrap();
        let atoms = records_to_atoms("src", "f.json", "id", VisibilityScope::Team, records);
        assert_eq!(atoms[0].visibility, VisibilityScope::Personal);
    }

    #[test]
    fn extract_tags_from_csv_string() {
        let v: Value = serde_json::from_str(r#"{"tags": "rust, async, tokio"}"#).unwrap();
        let tags = extract_tags(v.as_object().unwrap());
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"async".to_string()));
        assert!(tags.contains(&"tokio".to_string()));
    }

    #[tokio::test]
    async fn full_sync_indexes_json_array_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"[
          {"id": "k1", "title": "Alpha", "content": "FIXTURE_JSON_KNOWN_PHRASE_42", "tags": ["alpha"]},
          {"id": "k2", "title": "Beta",  "content": "Another record"}
        ]"#;
        fs::write(dir.path().join("fixture.json"), content).unwrap();

        let mut connector = JsonConnector::new();
        let config = SourceConfig {
            name: "json-test".into(),
            kind: crate::types::SourceKind::Json,
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
        assert!(atoms
            .iter()
            .all(|a| a.visibility == VisibilityScope::Personal));
    }

    #[tokio::test]
    async fn full_sync_applies_source_visibility_config() {
        let dir = tempfile::tempdir().unwrap();
        let content = r#"[{"id": "k1", "title": "Team", "content": "TEAM_VIS_42"}]"#;
        fs::write(dir.path().join("fixture.json"), content).unwrap();

        let mut connector = JsonConnector::new();
        let config = SourceConfig {
            name: "json-team".into(),
            kind: crate::types::SourceKind::Json,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([
                (
                    "root_path".into(),
                    dir.path().to_string_lossy().into_owned(),
                ),
                ("visibility".into(), "team".into()),
            ]),
        };
        connector.init(&config).await.unwrap();
        let atoms = connector.full_sync().await.unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].visibility, VisibilityScope::Team);
    }

    #[tokio::test]
    async fn full_sync_indexes_ndjson_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let content = [
            r#"{"uid": "n1", "title": "NDJSON First", "content": "NDJSON_KNOWN_42", "tags": ["ndjson"]}"#,
            r#"{"uid": "n2", "title": "NDJSON Second", "content": "other", "tags": ["ndjson"]}"#,
        ]
        .join("\n");
        fs::write(dir.path().join("fixture.ndjson"), &content).unwrap();

        let mut connector = JsonConnector::new();
        let config = SourceConfig {
            name: "ndjson-test".into(),
            kind: crate::types::SourceKind::Json,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::from([
                (
                    "root_path".into(),
                    dir.path().to_string_lossy().into_owned(),
                ),
                ("id_field".into(), "uid".into()),
            ]),
        };
        connector.init(&config).await.unwrap();
        let atoms = connector.full_sync().await.unwrap();

        assert_eq!(atoms.len(), 2);
        assert!(atoms.iter().any(|a| a.content.contains("NDJSON_KNOWN_42")));
        // Stable source_id from id_field
        assert!(atoms.iter().any(|a| a.source_id == "n1"));
    }
}
