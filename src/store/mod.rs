use crate::types::KnowledgeAtom;
use anyhow::Result;

/// Storage backend for knowledge atoms and their embeddings.
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Insert or update a knowledge atom.
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()>;

    /// Bulk insert/update multiple atoms.
    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()>;

    /// Vector search: find atoms by embedding similarity.
    async fn vector_search(&self, query_embed: &[f32], limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>>;

    /// Full-text search over atom content.
    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>>;

    /// Delete atoms for a given source (for re-index).
    async fn delete_source(&self, source: &str) -> Result<()>;

    /// Total atom count.
    async fn count(&self) -> Result<u64>;
}

/// SQLite + sqlite-vec storage implementation.
pub struct SqliteVecStore {
    path: String,
}

impl SqliteVecStore {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait::async_trait]
impl Store for SqliteVecStore {
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()> {
        let _ = atom;
        Ok(())
    }

    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()> {
        for atom in atoms {
            self.upsert(atom).await?;
        }
        Ok(())
    }

    async fn vector_search(&self, query_embed: &[f32], limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let _ = (query_embed, limit);
        Ok(vec![])
    }

    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let _ = (query, limit);
        Ok(vec![])
    }

    async fn delete_source(&self, source: &str) -> Result<()> {
        let _ = source;
        Ok(())
    }

    async fn count(&self) -> Result<u64> {
        Ok(0)
    }
}
