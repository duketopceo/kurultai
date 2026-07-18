pub mod migrations;

use crate::error::{KurultaiError, Result};
use crate::types::KnowledgeAtom;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Storage backend for knowledge atoms and their embeddings.
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Insert or update a knowledge atom.
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()>;

    /// Bulk insert/update multiple atoms.
    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()>;

    /// Vector search: find atoms by embedding similarity.
    async fn vector_search(
        &self,
        query_embed: &[f32],
        limit: usize,
    ) -> Result<Vec<(KnowledgeAtom, f64)>>;

    /// Full-text search over atom content.
    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>>;

    /// Delete atoms for a given source (for re-index).
    async fn delete_source(&self, source: &str) -> Result<()>;

    /// Total atom count.
    async fn count(&self) -> Result<u64>;
}

/// SQLite + sqlite-vec storage implementation.
///
/// Vector search and FTS are stubs until #1 lands; migrations and CRUD skeleton are real.
pub struct SqliteVecStore {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl SqliteVecStore {
    /// Open (or create) the database and run migrations.
    pub fn open(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)
            .map_err(|e| KurultaiError::Store(format!("failed to open {}: {e}", path.display())))?;

        migrations::migrate(&conn)?;

        tracing::debug!(path = %path.display(), "sqlite store opened");
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[async_trait::async_trait]
impl Store for SqliteVecStore {
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()> {
        let _ = atom;
        // TODO(#1): real upsert with embedding blob
        tracing::trace!(id = %atom.id, source = %atom.source, "upsert stub");
        Ok(())
    }

    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()> {
        for atom in atoms {
            self.upsert(atom).await?;
        }
        Ok(())
    }

    async fn vector_search(
        &self,
        query_embed: &[f32],
        limit: usize,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let _ = (query_embed, limit);
        Ok(vec![])
    }

    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let _ = (query, limit);
        Ok(vec![])
    }

    async fn delete_source(&self, source: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;
        conn.execute("DELETE FROM knowledge_atoms WHERE source = ?1", [source])
            .map_err(|e| KurultaiError::Store(format!("delete_source failed: {e}")))?;
        tracing::debug!(source, "deleted atoms for source");
        Ok(())
    }

    async fn count(&self) -> Result<u64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_atoms", [], |row| row.get(0))
            .map_err(|e| KurultaiError::Store(format!("count failed: {e}")))?;
        Ok(count as u64)
    }
}
