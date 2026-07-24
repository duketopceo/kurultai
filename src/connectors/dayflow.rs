//! Dayflow activity journal connector (Phase 4 / #21).
//!
//! Read-only SQLite over `timeline_cards` — never writes the Dayflow DB.

use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::hashutil::atom_id;
use crate::security::validate_readable_path;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const DEFAULT_REL: &str = "Library/Application Support/Dayflow/chunks.sqlite";

/// Indexes Dayflow `timeline_cards` as knowledge atoms.
pub struct DayflowConnector {
    source_name: String,
    db_path: Option<PathBuf>,
    /// Incremental watermark: max `start_ts` seen (unix seconds).
    last_start_ts: Mutex<Option<i64>>,
}

impl DayflowConnector {
    pub fn new() -> Self {
        Self {
            source_name: "dayflow".into(),
            db_path: None,
            last_start_ts: Mutex::new(None),
        }
    }

    fn resolve_db_path(config: &SourceConfig) -> Result<String> {
        if let Some(p) = config
            .extra
            .get("db_path")
            .or_else(|| config.extra.get("data_path"))
        {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                return Err(KurultaiError::connector(
                    &config.name,
                    "db_path/data_path must be non-empty",
                ));
            }
            // If data_path is a directory, append chunks.sqlite.
            let path = Path::new(trimmed);
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("sqlite") || e.eq_ignore_ascii_case("db"))
            {
                return Ok(trimmed.to_string());
            }
            return Ok(path.join("chunks.sqlite").to_string_lossy().into_owned());
        }

        let home = dirs::home_dir().ok_or_else(|| {
            KurultaiError::connector(&config.name, "cannot resolve home directory for Dayflow")
        })?;
        Ok(home.join(DEFAULT_REL).to_string_lossy().into_owned())
    }

    fn open_ro(path: &Path) -> Result<Connection> {
        let uri = format!("file:{}?mode=ro", path.display());
        Connection::open_with_flags(
            &uri,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|e| {
            KurultaiError::connector("dayflow", format!("open read-only {}: {e}", path.display()))
        })
    }

    fn fetch_cards(&self, since_ts: Option<i64>) -> Result<Vec<KnowledgeAtom>> {
        let path = self
            .db_path
            .as_ref()
            .ok_or_else(|| KurultaiError::connector("dayflow", "not initialized"))?;
        let conn = Self::open_ro(path)?;

        let sql = if since_ts.is_some() {
            "SELECT id, day, title, summary, detailed_summary, category, subcategory, \
             start_ts, end_ts, metadata \
             FROM timeline_cards \
             WHERE COALESCE(is_deleted, 0) = 0 AND start_ts IS NOT NULL AND start_ts > ?1 \
             ORDER BY start_ts ASC"
        } else {
            "SELECT id, day, title, summary, detailed_summary, category, subcategory, \
             start_ts, end_ts, metadata \
             FROM timeline_cards \
             WHERE COALESCE(is_deleted, 0) = 0 \
             ORDER BY start_ts ASC"
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| KurultaiError::connector("dayflow", format!("prepare: {e}")))?;

        let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<KnowledgeAtom> {
            let id: i64 = row.get(0)?;
            let day: String = row.get(1)?;
            let title: String = row.get(2)?;
            let summary: Option<String> = row.get(3)?;
            let detailed: Option<String> = row.get(4)?;
            let category: String = row.get(5)?;
            let subcategory: Option<String> = row.get(6)?;
            let start_ts: Option<i64> = row.get(7)?;
            let end_ts: Option<i64> = row.get(8)?;
            let metadata_json: Option<String> = row.get(9)?;

            let body = detailed
                .filter(|s| !s.trim().is_empty())
                .or_else(|| summary.clone().filter(|s| !s.trim().is_empty()))
                .unwrap_or_else(|| title.clone());

            let source_id = id.to_string();
            let content = body;
            let atom_id = atom_id("dayflow", &source_id, &content);

            let mut tags = vec![category.clone()];
            if let Some(sub) = subcategory.clone().filter(|s| !s.is_empty()) {
                tags.push(sub);
            }

            let updated = start_ts
                .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
                .unwrap_or_else(Utc::now);

            let mut metadata = HashMap::new();
            metadata.insert("day".into(), day);
            metadata.insert("category".into(), category);
            if let Some(sub) = subcategory {
                metadata.insert("subcategory".into(), sub);
            }
            if let Some(ts) = start_ts {
                metadata.insert("start_ts".into(), ts.to_string());
            }
            if let Some(ts) = end_ts {
                metadata.insert("end_ts".into(), ts.to_string());
            }
            if let Some(m) = metadata_json {
                metadata.insert("dayflow_metadata".into(), m);
            }

            Ok(KnowledgeAtom {
                id: atom_id,
                source: self.source_name.clone(),
                source_id,
                title,
                summary: summary.unwrap_or_default().chars().take(280).collect(),
                content,
                question: None,
                resolution: None,
                tags,
                source_updated_at: updated,
                indexed_at: Utc::now(),
                embedding: None,
                metadata,
            })
        };

        let atoms: Vec<KnowledgeAtom> = if let Some(since) = since_ts {
            let rows = stmt
                .query_map([since], map_row)
                .map_err(|e| KurultaiError::connector("dayflow", format!("query: {e}")))?;
            rows.collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| KurultaiError::connector("dayflow", format!("row: {e}")))?
        } else {
            let rows = stmt
                .query_map([], map_row)
                .map_err(|e| KurultaiError::connector("dayflow", format!("query: {e}")))?;
            rows.collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| KurultaiError::connector("dayflow", format!("row: {e}")))?
        };

        Ok(atoms)
    }
}

impl Default for DayflowConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connector for DayflowConnector {
    fn name(&self) -> &str {
        "dayflow"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        self.source_name = config.name.clone();
        let raw = Self::resolve_db_path(config)?;
        let path = validate_readable_path(&raw, "dayflow db")?;
        tracing::debug!(db = %path.display(), "dayflow connector initialized");
        self.db_path = Some(path);
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        let since = *self.last_start_ts.lock().unwrap_or_else(|e| e.into_inner());
        let atoms = self.fetch_cards(since)?;
        if let Some(max_ts) = atoms
            .iter()
            .filter_map(|a| a.metadata.get("start_ts"))
            .filter_map(|s| s.parse::<i64>().ok())
            .max()
        {
            if let Ok(mut guard) = self.last_start_ts.lock() {
                *guard = Some(max_ts);
            }
        }
        Ok(atoms)
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        let atoms = self.fetch_cards(None)?;
        if let Some(max_ts) = atoms
            .iter()
            .filter_map(|a| a.metadata.get("start_ts"))
            .filter_map(|s| s.parse::<i64>().ok())
            .max()
        {
            if let Ok(mut guard) = self.last_start_ts.lock() {
                *guard = Some(max_ts);
            }
        }
        Ok(atoms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceKind;
    use std::fs;

    fn fixture_db() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-dayflow-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("chunks.sqlite");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE timeline_cards (
                id INTEGER PRIMARY KEY,
                batch_id INTEGER,
                start TEXT NOT NULL,
                end TEXT NOT NULL,
                start_ts INTEGER,
                end_ts INTEGER,
                day DATE NOT NULL,
                title TEXT NOT NULL,
                summary TEXT,
                category TEXT NOT NULL,
                subcategory TEXT,
                detailed_summary TEXT,
                metadata TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_deleted INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO timeline_cards
              (id, start, end, start_ts, end_ts, day, title, summary, category, subcategory, detailed_summary)
            VALUES
              (1, '9:00 AM', '10:00 AM', 1700000000, 1700003600, '2023-11-14',
               'KNOWN_DAYFLOW_CARD_42', 'short summary', 'Work', 'Software',
               'Debugged Rust CI pipeline with fixture phrase KNOWN_DAYFLOW_CARD_42');",
        )
        .unwrap();
        path
    }

    #[tokio::test]
    async fn full_sync_indexes_fixture_card() {
        let path = fixture_db();
        let mut c = DayflowConnector::new();
        let mut extra = HashMap::new();
        extra.insert("db_path".into(), path.to_string_lossy().into_owned());
        c.init(&SourceConfig {
            name: "activity".into(),
            kind: SourceKind::Dayflow,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();
        let atoms = c.full_sync().await.unwrap();
        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].title, "KNOWN_DAYFLOW_CARD_42");
        assert!(atoms[0].content.contains("KNOWN_DAYFLOW_CARD_42"));
        assert_eq!(atoms[0].source, "activity");
    }

    #[tokio::test]
    async fn missing_db_errors_clearly() {
        let mut c = DayflowConnector::new();
        let mut extra = HashMap::new();
        extra.insert(
            "db_path".into(),
            "/tmp/kurultai-definitely-missing-dayflow.sqlite".into(),
        );
        let err = c
            .init(&SourceConfig {
                name: "activity".into(),
                kind: SourceKind::Dayflow,
                enabled: true,
                poll_interval_secs: 60,
                extra,
            })
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("dayflow")
                || err.to_string().contains("path")
                || err.to_string().contains("exist")
                || err.to_string().contains("No such")
                || err.to_string().contains("not")
        );
    }

    #[tokio::test]
    async fn poll_skips_already_seen_ts() {
        let path = fixture_db();
        let mut c = DayflowConnector::new();
        let mut extra = HashMap::new();
        extra.insert("db_path".into(), path.to_string_lossy().into_owned());
        c.init(&SourceConfig {
            name: "activity".into(),
            kind: SourceKind::Dayflow,
            enabled: true,
            poll_interval_secs: 60,
            extra,
        })
        .await
        .unwrap();
        assert_eq!(c.full_sync().await.unwrap().len(), 1);
        assert!(c.poll().await.unwrap().is_empty());
    }
}
