//! Broker-local session/chat registry — SQLite, device-local only. Never the
//! knowledge store. Session keys are stored hashed (sha256); a `sess_*` key
//! authenticates proxy calls, a `(agent, chat_id)` pair is the stable chat
//! identity that survives re-boards and restarts.

use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Registry {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone)]
pub struct BoardedIdentity {
    pub session_key: String,
    pub chat_id: String,
    pub chat_name: String,
    pub agent: String,
    pub instance_id: String,
    pub device: String,
}

#[derive(Debug, Deserialize)]
pub struct BoardRequest {
    pub agent: String,
    /// Stable conversation id from the harness; falls back to sha(chat_name).
    pub chat_id: Option<String>,
    pub chat_name: Option<String>,
    pub instance_id: Option<String>,
}

impl Registry {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| KurultaiError::config(format!("broker db dir: {e}")))?;
        }
        let conn = Connection::open(path)
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("open broker db: {e}")))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS broker_chats (
               id INTEGER PRIMARY KEY,
               agent TEXT NOT NULL,
               chat_id TEXT NOT NULL,
               chat_name TEXT NOT NULL,
               instance_id TEXT NOT NULL DEFAULT '',
               created_at TEXT NOT NULL,
               UNIQUE(agent, chat_id)
             );
             CREATE TABLE IF NOT EXISTS broker_sessions (
               id INTEGER PRIMARY KEY,
               key_hash TEXT NOT NULL UNIQUE,
               chat_row INTEGER NOT NULL REFERENCES broker_chats(id),
               revoked INTEGER NOT NULL DEFAULT 0,
               created_at TEXT NOT NULL
             );",
        )
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("broker db schema: {e}")))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| KurultaiError::Other(anyhow::anyhow!("broker registry poisoned")))
    }

    /// Board an agent: upsert the chat row, mint a fresh `sess_*` key.
    /// Same `(agent, chat_id)` re-boards to the same chat identity.
    pub fn board(&self, req: &BoardRequest) -> Result<BoardedIdentity> {
        let device = crate::broker::server::hostname();
        let agent = req.agent.trim().to_string();
        if agent.is_empty() {
            return Err(KurultaiError::config("board: agent is required"));
        }
        let chat_name = req.chat_name.clone().unwrap_or_else(|| "default".into());
        let chat_id = req
            .chat_id
            .clone()
            .filter(|c| !c.trim().is_empty())
            .unwrap_or_else(|| sha256_hex(&format!("{agent}:{chat_name}"))[..16].to_string());
        let instance_id = req.instance_id.clone().unwrap_or_default();
        let session_key = format!("sess_{}", uuid::Uuid::new_v4().simple());
        let key_hash = sha256_hex(&session_key);
        let now = Utc::now().to_rfc3339();

        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO broker_chats(agent, chat_id, chat_name, instance_id, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(agent, chat_id) DO UPDATE SET chat_name=excluded.chat_name",
            params![agent, chat_id, chat_name, instance_id, now],
        )
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("board chat upsert: {e}")))?;
        let chat_row: i64 = conn
            .query_row(
                "SELECT id FROM broker_chats WHERE agent=?1 AND chat_id=?2",
                params![agent, chat_id],
                |r| r.get(0),
            )
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("board chat lookup: {e}")))?;
        conn.execute(
            "INSERT INTO broker_sessions(key_hash, chat_row, created_at) VALUES(?1, ?2, ?3)",
            params![key_hash, chat_row, now],
        )
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("board session insert: {e}")))?;

        Ok(BoardedIdentity {
            session_key,
            chat_id,
            chat_name,
            agent,
            instance_id,
            device,
        })
    }

    /// Resolve a `sess_*` key to its identity; None if unknown or revoked.
    pub fn resolve(&self, session_key: &str) -> Result<Option<BoardedIdentity>> {
        let conn = self.lock()?;
        let row = conn
            .query_row(
                "SELECT c.chat_id, c.chat_name, c.agent, c.instance_id
                 FROM broker_sessions s JOIN broker_chats c ON c.id = s.chat_row
                 WHERE s.key_hash = ?1 AND s.revoked = 0",
                params![sha256_hex(session_key)],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();
        Ok(
            row.map(|(chat_id, chat_name, agent, instance_id)| BoardedIdentity {
                session_key: session_key.to_string(),
                chat_id,
                chat_name,
                agent,
                instance_id,
                device: crate::broker::server::hostname(),
            }),
        )
    }

    /// Revoke every session key for a chat — other chats keep working.
    pub fn revoke_chat(&self, agent: &str, chat_id: &str) -> Result<u64> {
        let conn = self.lock()?;
        let n = conn
            .execute(
                "UPDATE broker_sessions SET revoked = 1
                 WHERE chat_row IN (SELECT id FROM broker_chats WHERE agent=?1 AND chat_id=?2)",
                params![agent, chat_id],
            )
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("revoke chat: {e}")))?;
        Ok(n as u64)
    }
}

/// Default broker db path — `broker.db` beside the environment's store.
pub fn default_db_path(storage_path: &str) -> PathBuf {
    let p = Path::new(storage_path);
    p.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("broker.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> (tempfile::TempDir, Registry) {
        let dir = tempfile::tempdir().unwrap();
        let reg = Registry::open(&dir.path().join("b.db")).unwrap();
        (dir, reg)
    }

    fn req(agent: &str, chat_id: Option<&str>, name: Option<&str>) -> BoardRequest {
        BoardRequest {
            agent: agent.into(),
            chat_id: chat_id.map(|s| s.to_string()),
            chat_name: name.map(|s| s.to_string()),
            instance_id: None,
        }
    }

    #[test]
    fn board_mints_and_resolves() {
        let (_d, r) = reg();
        let id = r
            .board(&req("cursor", Some("c1"), Some("chat one")))
            .unwrap();
        assert!(id.session_key.starts_with("sess_"));
        let got = r.resolve(&id.session_key).unwrap().unwrap();
        assert_eq!(got.chat_id, "c1");
        assert_eq!(got.agent, "cursor");
    }

    #[test]
    fn reboard_same_chat_new_key_same_identity() {
        let (_d, r) = reg();
        let a = r.board(&req("cursor", Some("c1"), Some("x"))).unwrap();
        let b = r.board(&req("cursor", Some("c1"), Some("x"))).unwrap();
        assert_eq!(a.chat_id, b.chat_id);
        assert_ne!(a.session_key, b.session_key);
        assert!(r.resolve(&a.session_key).unwrap().is_some());
        assert!(r.resolve(&b.session_key).unwrap().is_some());
    }

    #[test]
    fn revoke_chat_leaves_others() {
        let (_d, r) = reg();
        let a = r.board(&req("cursor", Some("c1"), None)).unwrap();
        let b = r.board(&req("cursor", Some("c2"), None)).unwrap();
        assert_eq!(r.revoke_chat("cursor", "c1").unwrap(), 1);
        assert!(r.resolve(&a.session_key).unwrap().is_none());
        assert!(r.resolve(&b.session_key).unwrap().is_some());
    }

    #[test]
    fn unknown_key_is_none() {
        let (_d, r) = reg();
        assert!(r.resolve("sess_nope").unwrap().is_none());
    }

    #[test]
    fn chat_id_derived_from_name_when_absent() {
        let (_d, r) = reg();
        let id = r.board(&req("devin", None, Some("fix the thing"))).unwrap();
        assert_eq!(id.chat_id.len(), 16);
    }
}
