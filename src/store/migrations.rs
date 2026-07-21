use crate::error::{KurultaiError, Result};
use rusqlite::Connection;

/// Bump when schema changes. Migrations run in order on store open.
pub const CURRENT_SCHEMA_VERSION: i32 = 2;

const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS knowledge_atoms (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    source_id TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    question TEXT,
    resolution TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    source_updated_at TEXT NOT NULL,
    indexed_at TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_atoms_source ON knowledge_atoms(source);
CREATE INDEX IF NOT EXISTS idx_atoms_source_id ON knowledge_atoms(source, source_id);
"#;

const MIGRATION_002: &str = r#"
ALTER TABLE knowledge_atoms ADD COLUMN content_hash TEXT NOT NULL DEFAULT '';

CREATE TABLE IF NOT EXISTS store_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS atoms_fts USING fts5(
    id UNINDEXED,
    title,
    summary,
    content,
    tokenize = 'porter unicode61'
);

CREATE INDEX IF NOT EXISTS idx_atoms_content_hash ON knowledge_atoms(content_hash);
"#;

/// Run pending migrations. Called once when the store opens (before vec0 setup).
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| KurultaiError::Store(format!("failed to init schema_migrations: {e}")))?;

    let current: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current >= CURRENT_SCHEMA_VERSION {
        tracing::debug!(current, "schema up to date");
        return Ok(());
    }

    tracing::info!(
        from = current,
        to = CURRENT_SCHEMA_VERSION,
        "running migrations"
    );

    if current < 1 {
        conn.execute_batch(MIGRATION_001)
            .map_err(|e| KurultaiError::Store(format!("migration 001 failed: {e}")))?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [1])
            .map_err(|e| KurultaiError::Store(format!("migration 001 record failed: {e}")))?;
    }

    if current < 2 {
        // ALTER may fail if column already exists on a partially migrated DB — ignore duplicate.
        match conn.execute_batch(MIGRATION_002) {
            Ok(()) => {}
            Err(e) if e.to_string().contains("duplicate column") => {
                // content_hash already present; ensure remaining objects exist
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS store_meta (
                        key TEXT PRIMARY KEY,
                        value TEXT NOT NULL
                    );
                    CREATE VIRTUAL TABLE IF NOT EXISTS atoms_fts USING fts5(
                        id UNINDEXED,
                        title,
                        summary,
                        content,
                        tokenize = 'porter unicode61'
                    );
                    "#,
                )
                .map_err(|e2| {
                    KurultaiError::Store(format!("migration 002 recovery failed: {e2}"))
                })?;
            }
            Err(e) => {
                return Err(KurultaiError::Store(format!("migration 002 failed: {e}")));
            }
        }
        conn.execute("INSERT INTO schema_migrations (version) VALUES (?1)", [2])
            .map_err(|e| KurultaiError::Store(format!("migration 002 record failed: {e}")))?;
    }

    tracing::info!(version = CURRENT_SCHEMA_VERSION, "migrations complete");
    Ok(())
}

/// Ensure `atoms_vec` exists for `embed_dim`. Refuses mixed dimensions when data exists.
pub fn ensure_vec_table(conn: &Connection, embed_dim: usize) -> Result<()> {
    if embed_dim == 0 {
        return Err(KurultaiError::Store(
            "embed_dim must be > 0 to create atoms_vec".into(),
        ));
    }

    let existing = meta_get(conn, "embed_dim")?;
    if let Some(prev) = existing {
        let prev_dim: usize = prev.parse().map_err(|_| {
            KurultaiError::Store(format!("invalid store_meta.embed_dim value: {prev}"))
        })?;
        if prev_dim != embed_dim {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM knowledge_atoms", [], |r| r.get(0))
                .unwrap_or(0);
            if count > 0 {
                return Err(KurultaiError::Store(format!(
                    "embed_dim mismatch: store has {prev_dim}, config wants {embed_dim}. \
                     Re-index with --rebuild-vectors (or delete the store) before changing dimensions."
                )));
            }
            // Empty store — allow dim change by dropping vec table.
            conn.execute_batch("DROP TABLE IF EXISTS atoms_vec;")
                .map_err(|e| KurultaiError::Store(format!("drop atoms_vec failed: {e}")))?;
        }
    }

    let sql = format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS atoms_vec USING vec0(embedding float[{embed_dim}])"
    );
    conn.execute_batch(&sql)
        .map_err(|e| KurultaiError::Store(format!("create atoms_vec failed: {e}")))?;

    meta_set(conn, "embed_dim", &embed_dim.to_string())?;
    Ok(())
}

pub fn meta_get(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn
        .prepare("SELECT value FROM store_meta WHERE key = ?1")
        .map_err(|e| KurultaiError::Store(format!("meta_get prepare: {e}")))?;
    let mut rows = stmt
        .query([key])
        .map_err(|e| KurultaiError::Store(format!("meta_get query: {e}")))?;
    if let Some(row) = rows
        .next()
        .map_err(|e| KurultaiError::Store(format!("meta_get next: {e}")))?
    {
        Ok(Some(row.get(0).map_err(|e| {
            KurultaiError::Store(format!("meta_get get: {e}"))
        })?))
    } else {
        Ok(None)
    }
}

pub fn meta_set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO store_meta(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )
    .map_err(|e| KurultaiError::Store(format!("meta_set failed: {e}")))?;
    Ok(())
}
