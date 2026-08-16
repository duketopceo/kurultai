//! Admin key store — mints and revokes scoped access tokens for HTTP/MCP API consumers.
//!
//! Independent of `src/store` (the knowledge-atom store) and of `SearchFilter` — those are
//! owned by a parallel PR (`feat/mesh-schema`) adding `mesh_ids`/`mesh_scope`/`max_tier` to the
//! query path. This module only mints/stores/revokes claims records; a later PR wires a
//! resolved [`KeyClaims`] into that query path as the caller's grant.
//!
//! Tokens are never stored or logged in plaintext: only a SHA-256 hash (via
//! [`crate::hashutil::sha256_hex`], the same helper already used for atom hashing and hub API
//! key comparison in `src/http/auth.rs`) is persisted. The raw token is generated, printed once
//! to the operator's stdout, and dropped.

use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

/// Ceiling on what a key's grants may reach into. Mirrors the `max_tier` shape landing in
/// `feat/mesh-schema`'s `SearchFilter` so claims composed here plug straight in later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxTier {
    Public,
    Private,
}

impl MaxTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            MaxTier::Public => "public",
            MaxTier::Private => "private",
        }
    }

    pub fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "public" => Ok(MaxTier::Public),
            "private" => Ok(MaxTier::Private),
            other => Err(KurultaiError::config(format!(
                "--max-tier must be \"public\" or \"private\", got {other:?}"
            ))),
        }
    }
}

impl std::fmt::Display for MaxTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Claims minted for a single issued key. Shaped to carry a mesh grant list, a tier ceiling,
/// and an allowed tool list so a future PR can compose these with `SearchFilter`'s
/// `mesh_ids`/`mesh_scope`/`max_tier` fields without reshaping either side.
#[derive(Debug, Clone)]
pub struct KeyClaims {
    pub name: String,
    pub mesh: Vec<String>,
    pub max_tier: MaxTier,
    pub tools: Vec<String>,
    /// Free-text policy shown to the connecting agent at connect time. NOT enforced in code —
    /// purely advisory / informational. Any actual restriction must come from `mesh`,
    /// `max_tier`, and `tools`.
    pub rules_doc: Option<String>,
}

/// A stored key record as returned by `list`/`resolve` — never includes the token or its hash.
#[derive(Debug, Clone)]
pub struct KeyRecord {
    pub name: String,
    pub mesh: Vec<String>,
    pub max_tier: MaxTier,
    pub tools: Vec<String>,
    pub rules_doc: Option<String>,
    pub active: bool,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

pub struct AdminKeyStore {
    conn: Mutex<Connection>,
}

impl AdminKeyStore {
    /// Open (or create) the admin-keys database and ensure its schema exists. Deliberately a
    /// separate SQLite file from the knowledge-atom store (`store.db`) so this table can't
    /// collide with migrations owned by `src/store/migrations.rs`.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .map_err(|e| KurultaiError::Store(format!("failed to open {}: {e}", path.display())))?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS admin_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                token_hash TEXT NOT NULL UNIQUE,
                mesh_json TEXT NOT NULL DEFAULT '[]',
                max_tier TEXT NOT NULL,
                tools_json TEXT NOT NULL DEFAULT '[]',
                rules_doc TEXT,
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                revoked_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_admin_keys_hash ON admin_keys(token_hash);
            "#,
        )
        .map_err(|e| KurultaiError::Store(format!("admin_keys schema: {e}")))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| KurultaiError::Store("admin_keys connection lock poisoned".into()))
    }

    /// Generate a random token, hash it, and store the hash + claims. Returns the *raw* token —
    /// the only time it ever exists as a return value; it is never written to disk or logged.
    pub fn issue(&self, claims: &KeyClaims) -> Result<String> {
        let token = generate_token();
        let hash = sha256_hex(&token);
        let mesh_json = serde_json::to_string(&claims.mesh)
            .map_err(|e| KurultaiError::Store(format!("serialize mesh: {e}")))?;
        let tools_json = serde_json::to_string(&claims.tools)
            .map_err(|e| KurultaiError::Store(format!("serialize tools: {e}")))?;

        let conn = self.lock()?;
        let inserted = conn.execute(
            "INSERT INTO admin_keys (name, token_hash, mesh_json, max_tier, tools_json, rules_doc, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
            params![
                claims.name,
                hash,
                mesh_json,
                claims.max_tier.as_str(),
                tools_json,
                claims.rules_doc,
            ],
        );
        match inserted {
            Ok(_) => Ok(token),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(KurultaiError::config(format!(
                    "a key named '{}' already exists (revoke or pick a new name)",
                    claims.name
                )))
            }
            Err(e) => Err(KurultaiError::Store(format!("insert admin key: {e}"))),
        }
    }

    /// Soft-delete: marks the row inactive and stamps `revoked_at`, keeping it for audit.
    /// Returns `true` if a row was found and revoked, `false` if no key with that name exists.
    pub fn revoke_by_name(&self, name: &str) -> Result<bool> {
        let conn = self.lock()?;
        let updated = conn
            .execute(
                "UPDATE admin_keys SET active = 0, revoked_at = datetime('now')
                 WHERE name = ?1 AND active = 1",
                params![name],
            )
            .map_err(|e| KurultaiError::Store(format!("revoke admin key: {e}")))?;
        if updated > 0 {
            return Ok(true);
        }
        // Distinguish "already revoked" from "never existed" for a clearer CLI message.
        let exists: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM admin_keys WHERE name = ?1",
                params![name],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("revoke lookup: {e}")))?;
        let _ = exists; // existence only changes the CLI message, not the return value
        Ok(false)
    }

    /// All keys (active and revoked), never including the token or its hash.
    pub fn list(&self) -> Result<Vec<KeyRecord>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "SELECT name, mesh_json, max_tier, tools_json, rules_doc, active, created_at, revoked_at
                 FROM admin_keys ORDER BY created_at ASC",
            )
            .map_err(|e| KurultaiError::Store(format!("list admin keys: {e}")))?;
        let rows = stmt
            .query_map([], row_to_record)
            .map_err(|e| KurultaiError::Store(format!("list admin keys: {e}")))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| KurultaiError::Store(format!("list admin keys: {e}")))?);
        }
        Ok(out)
    }

    /// Resolve a record by the SHA-256 hash of a presented token. Returns the record regardless
    /// of `active` state (callers check `.active`) so revoked keys resolve to "inactive" rather
    /// than silently vanishing — useful for audit and for the revoke test below.
    pub fn resolve_by_hash(&self, token_hash: &str) -> Result<Option<KeyRecord>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT name, mesh_json, max_tier, tools_json, rules_doc, active, created_at, revoked_at
             FROM admin_keys WHERE token_hash = ?1",
            params![token_hash],
            row_to_record,
        )
        .optional()
        .map_err(|e| KurultaiError::Store(format!("resolve admin key: {e}")))
    }
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<KeyRecord> {
    let mesh_json: String = row.get(1)?;
    let max_tier_raw: String = row.get(2)?;
    let tools_json: String = row.get(3)?;
    let active: i64 = row.get(5)?;
    Ok(KeyRecord {
        name: row.get(0)?,
        mesh: serde_json::from_str(&mesh_json).unwrap_or_default(),
        max_tier: MaxTier::parse(&max_tier_raw).unwrap_or(MaxTier::Public),
        tools: serde_json::from_str(&tools_json).unwrap_or_default(),
        rules_doc: row.get(4)?,
        active: active != 0,
        created_at: row.get(6)?,
        revoked_at: row.get(7)?,
    })
}

/// Default path for the admin-keys DB: sibling of the knowledge-atom store, so both live under
/// the same configured storage directory without touching `src/store`'s schema.
pub fn default_admin_keys_path(storage_path: &Path) -> PathBuf {
    storage_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("admin_keys.db")
}

/// High-entropy random bearer token. Uses `uuid::Uuid::new_v4` (OS CSPRNG-backed) rather than
/// pulling in a `rand` dependency the repo doesn't already have; three concatenated v4 UUIDs
/// give ~366 bits of raw randomness, comfortably enough for a bearer credential.
fn generate_token() -> String {
    format!(
        "kk_{}{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn open_test_store() -> (tempfile::TempDir, AdminKeyStore) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("admin_keys.db");
        let store = AdminKeyStore::open(&path).unwrap();
        (dir, store)
    }

    fn claims(name: &str) -> KeyClaims {
        KeyClaims {
            name: name.to_string(),
            mesh: vec!["bartlett".into(), "ops".into()],
            max_tier: MaxTier::Private,
            tools: vec!["search".into(), "ask".into()],
            rules_doc: Some("be nice".into()),
        }
    }

    #[test]
    fn issue_stores_hash_not_raw_token() {
        let (dir, store) = open_test_store();
        let token = store.issue(&claims("agent-1")).unwrap();
        assert!(token.starts_with("kk_"));

        // Resolvable by hash of the raw token.
        let hash = sha256_hex(&token);
        let rec = store.resolve_by_hash(&hash).unwrap().expect("record found");
        assert_eq!(rec.name, "agent-1");
        assert!(rec.active);
        assert_eq!(rec.mesh, vec!["bartlett".to_string(), "ops".to_string()]);
        assert_eq!(rec.max_tier, MaxTier::Private);

        // The raw token itself must never appear anywhere in the DB file's bytes.
        let db_path = dir.path().join("admin_keys.db");
        let bytes = std::fs::read(&db_path).unwrap();
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(
            !haystack.contains(&token),
            "raw token must not be persisted anywhere in the store"
        );
    }

    #[test]
    fn list_never_exposes_token_or_hash() {
        let (_dir, store) = open_test_store();
        store.issue(&claims("agent-2")).unwrap();
        let records = store.list().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "agent-2");
        assert!(records[0].active);
    }

    #[test]
    fn revoke_marks_inactive_and_old_hash_reports_inactive() {
        let (_dir, store) = open_test_store();
        let token = store.issue(&claims("agent-3")).unwrap();
        let hash = sha256_hex(&token);

        assert!(store.resolve_by_hash(&hash).unwrap().unwrap().active);

        let revoked = store.revoke_by_name("agent-3").unwrap();
        assert!(revoked);

        // Row still exists (soft delete / audit trail) but reports inactive.
        let rec = store.resolve_by_hash(&hash).unwrap().expect("row kept for audit");
        assert!(!rec.active);
        assert!(rec.revoked_at.is_some());

        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert!(!listed[0].active);
    }

    #[test]
    fn revoke_unknown_name_returns_false() {
        let (_dir, store) = open_test_store();
        assert!(!store.revoke_by_name("nope").unwrap());
    }

    #[test]
    fn revoking_twice_is_idempotent_false_second_time() {
        let (_dir, store) = open_test_store();
        store.issue(&claims("agent-4")).unwrap();
        assert!(store.revoke_by_name("agent-4").unwrap());
        // Already revoked: no active row to flip, so this reports false (not re-revoked).
        assert!(!store.revoke_by_name("agent-4").unwrap());
    }

    #[test]
    fn duplicate_name_rejected() {
        let (_dir, store) = open_test_store();
        store.issue(&claims("dup")).unwrap();
        let err = store.issue(&claims("dup")).unwrap_err();
        assert!(format!("{err}").contains("already exists"));
    }
}
