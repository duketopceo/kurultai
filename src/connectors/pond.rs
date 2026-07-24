//! Pond agent-session connector (Phase 4 / #8).
//!
//! Bridges Pond's read-only SQL surface (`pond sql --format ndjson`) —
//! does not open Lance storage directly.

use crate::connectors::Connector;
use crate::error::{KurultaiError, Result};
use crate::hashutil::atom_id;
use crate::types::{KnowledgeAtom, SourceConfig};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

const DEFAULT_LIMIT: usize = 500;
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// Indexes Pond messages (user + assistant) as knowledge atoms.
pub struct PondConnector {
    source_name: String,
    pond_bin: PathBuf,
    limit: usize,
    timeout_secs: u64,
    /// Watermark: ISO timestamp string from last message.
    last_timestamp: Mutex<Option<String>>,
    /// Injectable runner for tests.
    runner: Box<dyn PondSqlRunner>,
}

trait PondSqlRunner: Send + Sync {
    fn run_sql(&self, bin: &Path, sql: &str, limit: usize, timeout_secs: u64) -> Result<String>;
}

struct ProcessPondRunner;

impl PondSqlRunner for ProcessPondRunner {
    fn run_sql(&self, bin: &Path, sql: &str, limit: usize, timeout_secs: u64) -> Result<String> {
        let mut cmd = Command::new(bin);
        cmd.args([
            "sql",
            "--format",
            "ndjson",
            "--limit",
            &limit.to_string(),
            "--timeout",
            &timeout_secs.to_string(),
            sql,
        ]);
        // Best-effort timeout via env; Command doesn't kill on timeout until wait_timeout crate —
        // we rely on pond's --timeout.
        let output = cmd.output().map_err(|e| {
            KurultaiError::connector(
                "pond",
                format!(
                    "failed to run `{}` ({e}). Install pond or set extra.pond_bin",
                    bin.display()
                ),
            )
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(KurultaiError::connector(
                "pond",
                format!("pond sql failed: {stderr}"),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

impl PondConnector {
    pub fn new() -> Self {
        Self {
            source_name: "pond".into(),
            pond_bin: PathBuf::from("pond"),
            limit: DEFAULT_LIMIT,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            last_timestamp: Mutex::new(None),
            runner: Box::new(ProcessPondRunner),
        }
    }

    #[cfg(test)]
    fn with_runner(runner: Box<dyn PondSqlRunner>) -> Self {
        let mut c = Self::new();
        c.runner = runner;
        c
    }

    fn build_sql(since: Option<&str>) -> String {
        // Prefer conversational text; skip system/tool-heavy noise.
        let base = "SELECT session_id, message_id, timestamp, role, source_agent, project, \
                    search_text, content \
                    FROM messages \
                    WHERE role IN ('user', 'assistant')";
        match since {
            Some(ts) => {
                let safe = ts.replace('\'', "''");
                format!("{base} AND timestamp > '{safe}' ORDER BY timestamp ASC")
            }
            None => format!("{base} ORDER BY timestamp ASC"),
        }
    }

    fn parse_ndjson(source_name: &str, stdout: &str) -> Result<Vec<KnowledgeAtom>> {
        let mut atoms = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let row: PondMessageRow = serde_json::from_str(line)
                .map_err(|e| KurultaiError::connector("pond", format!("ndjson decode: {e}")))?;
            if let Some(atom) = row.into_atom(source_name) {
                atoms.push(atom);
            }
        }
        Ok(atoms)
    }

    fn fetch(&self, since: Option<&str>) -> Result<Vec<KnowledgeAtom>> {
        let sql = Self::build_sql(since);
        let out = self
            .runner
            .run_sql(&self.pond_bin, &sql, self.limit, self.timeout_secs)?;
        Self::parse_ndjson(&self.source_name, &out)
    }
}

impl Default for PondConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct PondMessageRow {
    session_id: Option<String>,
    message_id: String,
    timestamp: Option<String>,
    role: Option<String>,
    source_agent: Option<String>,
    project: Option<String>,
    search_text: Option<String>,
    content: Option<String>,
}

impl PondMessageRow {
    fn into_atom(self, source_name: &str) -> Option<KnowledgeAtom> {
        let text = self
            .search_text
            .filter(|s| !s.trim().is_empty())
            .or_else(|| self.content.filter(|s| !s.trim().is_empty()))?;
        let title = text.chars().take(80).collect::<String>();
        let source_id = self.message_id.clone();
        let id = atom_id("pond", &source_id, &text);
        let updated = self
            .timestamp
            .as_ref()
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let mut metadata = HashMap::new();
        if let Some(s) = self.session_id {
            metadata.insert("session_id".into(), s);
        }
        if let Some(t) = self.timestamp.clone() {
            metadata.insert("timestamp".into(), t);
        }
        if let Some(r) = self.role.clone() {
            metadata.insert("role".into(), r);
        }
        if let Some(a) = self.source_agent.clone() {
            metadata.insert("source_agent".into(), a);
        }
        if let Some(p) = self.project {
            metadata.insert("project".into(), p);
        }

        let mut tags = vec!["pond".into()];
        if let Some(a) = self.source_agent {
            tags.push(a);
        }

        Some(KnowledgeAtom {
            id,
            source: source_name.into(),
            source_id,
            title,
            summary: text.chars().take(280).collect(),
            content: text,
            question: None,
            resolution: None,
            tags,
            source_updated_at: updated,
            indexed_at: Utc::now(),
            embedding: None,
            metadata,
        })
    }
}

#[async_trait]
impl Connector for PondConnector {
    fn name(&self) -> &str {
        "pond"
    }

    async fn init(&mut self, config: &SourceConfig) -> Result<()> {
        self.source_name = config.name.clone();
        if let Some(bin) = config.extra.get("pond_bin") {
            self.pond_bin = PathBuf::from(bin);
        }
        if let Some(limit) = config.extra.get("limit") {
            if let Ok(n) = limit.parse::<usize>() {
                self.limit = n.clamp(1, 5000);
            }
        }
        if let Some(t) = config.extra.get("timeout_secs") {
            if let Ok(n) = t.parse::<u64>() {
                self.timeout_secs = n.clamp(5, 600);
            }
        }
        // Probe binary only when PATH-style; existence of custom path is checked at fetch.
        tracing::debug!(bin = %self.pond_bin.display(), limit = self.limit, "pond connector initialized");
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<KnowledgeAtom>> {
        let since = self
            .last_timestamp
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let atoms = self.fetch(since.as_deref())?;
        if let Some(ts) = atoms
            .iter()
            .filter_map(|a| a.metadata.get("timestamp"))
            .max()
        {
            if let Ok(mut g) = self.last_timestamp.lock() {
                *g = Some(ts.clone());
            }
        }
        Ok(atoms)
    }

    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>> {
        let atoms = self.fetch(None)?;
        if let Some(ts) = atoms
            .iter()
            .filter_map(|a| a.metadata.get("timestamp"))
            .max()
        {
            if let Ok(mut g) = self.last_timestamp.lock() {
                *g = Some(ts.clone());
            }
        }
        Ok(atoms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceKind;

    struct FixtureRunner {
        body: String,
    }

    impl PondSqlRunner for FixtureRunner {
        fn run_sql(
            &self,
            _bin: &Path,
            _sql: &str,
            _limit: usize,
            _timeout_secs: u64,
        ) -> Result<String> {
            Ok(self.body.clone())
        }
    }

    #[test]
    fn parse_ndjson_fixture() {
        let line = r#"{"session_id":"s1","message_id":"s1:2","timestamp":"2026-06-13T20:08:28.390Z","role":"user","source_agent":"claude-code","project":"/tmp","search_text":"KNOWN_POND_MSG_99 how does indexing work","content":null}"#;
        let atoms = PondConnector::parse_ndjson("chats", line).unwrap();
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].content.contains("KNOWN_POND_MSG_99"));
        assert_eq!(atoms[0].source, "chats");
        assert_eq!(atoms[0].source_id, "s1:2");
    }

    #[tokio::test]
    async fn full_sync_uses_runner() {
        let line = r#"{"session_id":"s1","message_id":"s1:2","timestamp":"2026-06-13T20:08:28.390Z","role":"assistant","source_agent":"codex-cli","project":"/tmp","search_text":"answer about KNOWN_POND_MSG_99","content":null}"#;
        let mut c = PondConnector::with_runner(Box::new(FixtureRunner { body: line.into() }));
        c.init(&SourceConfig {
            name: "chats".into(),
            kind: SourceKind::Pond,
            enabled: true,
            poll_interval_secs: 60,
            extra: HashMap::new(),
        })
        .await
        .unwrap();
        let atoms = c.full_sync().await.unwrap();
        assert_eq!(atoms.len(), 1);
        assert!(atoms[0].content.contains("KNOWN_POND_MSG_99"));
    }

    #[test]
    fn skips_empty_search_text() {
        let line = r#"{"message_id":"x","role":"user","search_text":"","content":""}"#;
        let atoms = PondConnector::parse_ndjson("chats", line).unwrap();
        assert!(atoms.is_empty());
    }
}
