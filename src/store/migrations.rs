use crate::error::{KurultaiError, Result};
use rusqlite::Connection;

/// Bump when schema changes. Migrations run in order on store open.
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

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

/// Run pending migrations. Called once when the store opens.
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

    tracing::info!(from = current, to = CURRENT_SCHEMA_VERSION, "running migrations");

    if current < 1 {
        conn.execute_batch(MIGRATION_001)
            .map_err(|e| KurultaiError::Store(format!("migration 001 failed: {e}")))?;
        conn.execute(
            "INSERT INTO schema_migrations (version) VALUES (?1)",
            [1],
        )
        .map_err(|e| KurultaiError::Store(format!("migration 001 record failed: {e}")))?;
    }

    tracing::info!(version = CURRENT_SCHEMA_VERSION, "migrations complete");
    Ok(())
}
