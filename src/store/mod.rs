pub mod migrations;

use crate::error::{KurultaiError, Result};
use crate::types::KnowledgeAtom;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
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

    /// Delete atoms from a source whose source_id is not in the keep list.
    async fn delete_source_ids_not_in(&self, source: &str, keep_ids: &[&str]) -> Result<u64>;

    /// Total atom count.
    async fn count(&self) -> Result<u64>;

    /// Check whether an atom with the same id and content_hash already exists.
    async fn content_hash_unchanged(&self, id: &str, content_hash: &str) -> Result<bool>;
}

/// SQLite + FTS5 + stored-vector brute-force storage implementation.
///
/// Phase 1: FTS5 is the primary search path. Embeddings are stored as BLOBs and
/// vector search is a brute-force cosine scan over the stored rows. This avoids
/// adding a native vector extension dependency while still satisfying the Phase 1
/// "non-zero embeddings upsert" acceptance criterion.
pub struct SqliteVecStore {
    conn: Mutex<Connection>,
    path: PathBuf,
    embed_dim: usize,
}

impl SqliteVecStore {
    /// Open (or create) the database and run migrations. Validates the stored
    /// embedding-model fingerprint against the supplied config.
    pub fn open(path: PathBuf, embed_model: &str, embed_dim: usize) -> Result<Self> {
        let conn = Connection::open(&path)
            .map_err(|e| KurultaiError::Store(format!("failed to open {}: {e}", path.display())))?;

        migrations::migrate(&conn)?;

        let store = Self {
            conn: Mutex::new(conn),
            path,
            embed_dim,
        };
        store.ensure_fingerprint(embed_model, embed_dim)?;

        tracing::debug!(path = %store.path.display(), dim = embed_dim, "sqlite store opened");
        Ok(store)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn ensure_fingerprint(&self, model: &str, dim: usize) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;

        let stored_model: Option<String> = conn
            .query_row(
                "SELECT value FROM store_meta WHERE key = 'embed_model'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("fingerprint read failed: {e}")))?;

        let stored_dim: Option<String> = conn
            .query_row(
                "SELECT value FROM store_meta WHERE key = 'embed_dim'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("fingerprint read failed: {e}")))?;

        match (stored_model, stored_dim) {
            (Some(m), Some(d)) => {
                let stored_dim_parsed: usize = d.parse().map_err(|_| {
                    KurultaiError::Store(format!("stored embed_dim '{d}' is not a number"))
                })?;
                if m != model || stored_dim_parsed != dim {
                    return Err(KurultaiError::Store(format!(
                        "embedding fingerprint mismatch: store has {m}/{stored_dim_parsed}, config has {model}/{dim}"
                    )));
                }
            }
            _ => {
                conn.execute(
                    "INSERT OR REPLACE INTO store_meta (key, value) VALUES (?1, ?2)",
                    ["embed_model", model],
                )
                .map_err(|e| KurultaiError::Store(format!("fingerprint write failed: {e}")))?;
                conn.execute(
                    "INSERT OR REPLACE INTO store_meta (key, value) VALUES (?1, ?2)",
                    ["embed_dim", &dim.to_string()],
                )
                .map_err(|e| KurultaiError::Store(format!("fingerprint write failed: {e}")))?;
            }
        }

        Ok(())
    }

    fn atom_from_row(row: &Row<'_>) -> std::result::Result<KnowledgeAtom, rusqlite::Error> {
        let embedding_blob: Option<Vec<u8>> = row.get("embedding")?;
        let embedding = embedding_blob.map(|bytes| {
            bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect()
        });

        Ok(KnowledgeAtom {
            id: row.get("id")?,
            source: row.get("source")?,
            source_id: row.get("source_id")?,
            title: row.get("title")?,
            summary: row.get("summary")?,
            content: row.get("content")?,
            question: row.get("question")?,
            resolution: row.get("resolution")?,
            tags: serde_json::from_str(&row.get::<_, String>("tags_json")?).unwrap_or_default(),
            source_updated_at: row
                .get::<_, String>("source_updated_at")?
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now()),
            indexed_at: row
                .get::<_, String>("indexed_at")?
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now()),
            metadata: serde_json::from_str(&row.get::<_, String>("metadata_json")?)
                .unwrap_or_default(),
            embedding,
            content_hash: row.get("content_hash")?,
            source_uri: row.get("source_uri")?,
            provenance: row.get("provenance")?,
        })
    }

    fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
        embedding.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn is_zero_vector(v: &[f32]) -> bool {
        v.iter().all(|x| *x == 0.0)
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0f64;
        let mut a_norm = 0.0f64;
        let mut b_norm = 0.0f64;
        for i in 0..a.len() {
            let ai = a[i] as f64;
            let bi = b[i] as f64;
            dot += ai * bi;
            a_norm += ai * ai;
            b_norm += bi * bi;
        }
        if a_norm == 0.0 || b_norm == 0.0 {
            return 0.0;
        }
        dot / (a_norm.sqrt() * b_norm.sqrt())
    }
}

#[async_trait::async_trait]
impl Store for SqliteVecStore {
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()> {
        if atom
            .embedding
            .as_ref()
            .map(|e| Self::is_zero_vector(e))
            .unwrap_or(false)
        {
            return Err(KurultaiError::Store(
                "refusing to upsert atom with zero vector".into(),
            ));
        }

        if let Some(ref embedding) = atom.embedding {
            if embedding.len() != self.embed_dim {
                return Err(KurultaiError::Store(format!(
                    "embedding dimension mismatch: atom has {} dims, store expects {}",
                    embedding.len(),
                    self.embed_dim
                )));
            }
        }

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;
        let tx = conn
            .transaction()
            .map_err(|e| KurultaiError::Store(format!("upsert tx begin failed: {e}")))?;

        tx.execute("DELETE FROM knowledge_atoms WHERE id = ?1", [&atom.id])
            .map_err(|e| KurultaiError::Store(format!("upsert delete failed: {e}")))?;
        tx.execute("DELETE FROM knowledge_atoms_fts WHERE id = ?1", [&atom.id])
            .map_err(|e| KurultaiError::Store(format!("upsert fts delete failed: {e}")))?;

        tx.execute(
            "INSERT INTO knowledge_atoms (
                id, source, source_id, title, summary, content, question, resolution,
                tags_json, source_updated_at, indexed_at, metadata_json, content_hash,
                source_uri, provenance, embedding
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16
            )",
            params![
                atom.id,
                atom.source,
                atom.source_id,
                atom.title,
                atom.summary,
                atom.content,
                atom.question.as_ref(),
                atom.resolution.as_ref(),
                serde_json::to_string(&atom.tags).unwrap_or_else(|_| "[]".into()),
                atom.source_updated_at.to_rfc3339(),
                atom.indexed_at.to_rfc3339(),
                serde_json::to_string(&atom.metadata).unwrap_or_else(|_| "{}".into()),
                atom.content_hash,
                atom.source_uri.as_ref(),
                atom.provenance.as_ref(),
                atom.embedding
                    .as_ref()
                    .map(|e| Self::serialize_embedding(e)),
            ],
        )
        .map_err(|e| KurultaiError::Store(format!("upsert insert failed: {e}")))?;

        tx.execute(
            "INSERT INTO knowledge_atoms_fts (id, title, content, summary) VALUES (?1, ?2, ?3, ?4)",
            params![atom.id, atom.title, atom.content, atom.summary],
        )
        .map_err(|e| KurultaiError::Store(format!("upsert fts insert failed: {e}")))?;

        tx.commit()
            .map_err(|e| KurultaiError::Store(format!("upsert tx commit failed: {e}")))?;
        tracing::trace!(id = %atom.id, source = %atom.source, "upsert complete");
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
        if query_embed.len() != self.embed_dim {
            return Err(KurultaiError::Store(format!(
                "query embedding dimension mismatch: {} vs {}",
                query_embed.len(),
                self.embed_dim
            )));
        }

        if Self::is_zero_vector(query_embed) {
            return Ok(vec![]);
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, source, source_id, title, summary, content, question, resolution,
                    tags_json, source_updated_at, indexed_at, metadata_json, content_hash,
                    source_uri, provenance, embedding
                 FROM knowledge_atoms
                 WHERE embedding IS NOT NULL",
            )
            .map_err(|e| KurultaiError::Store(format!("vector_search prepare failed: {e}")))?;

        let rows = stmt
            .query_map([], Self::atom_from_row)
            .map_err(|e| KurultaiError::Store(format!("vector_search query failed: {e}")))?;

        let mut scored: Vec<(KnowledgeAtom, f64)> = Vec::new();
        for row in rows {
            let atom =
                row.map_err(|e| KurultaiError::Store(format!("vector_search row failed: {e}")))?;
            if let Some(ref embedding) = atom.embedding {
                let score = Self::cosine_similarity(query_embed, embedding);
                if score > 0.0 {
                    scored.push((atom, score));
                }
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored)
    }

    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;

        let sql = "SELECT a.id, a.source, a.source_id, a.title, a.summary, a.content, a.question,
                a.resolution, a.tags_json, a.source_updated_at, a.indexed_at, a.metadata_json,
                a.content_hash, a.source_uri, a.provenance, a.embedding,
                rank FROM knowledge_atoms_fts fts
             JOIN knowledge_atoms a ON a.id = fts.id
             WHERE knowledge_atoms_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2";

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| KurultaiError::Store(format!("fts_search prepare failed: {e}")))?;

        let rows = stmt
            .query_map(params![query, limit as i64], |row| {
                let atom = Self::atom_from_row(row)?;
                // rank is lower-is-better; convert to a score in (0,1]
                let rank: f64 = row.get("rank")?;
                let score = 1.0 / (1.0 + rank.abs());
                Ok((atom, score))
            })
            .map_err(|e| KurultaiError::Store(format!("fts_search query failed: {e}")))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(
                row.map_err(|e| KurultaiError::Store(format!("fts_search row failed: {e}")))?,
            );
        }
        Ok(results)
    }

    async fn delete_source(&self, source: &str) -> Result<()> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;
        let tx = conn
            .transaction()
            .map_err(|e| KurultaiError::Store(format!("delete_source tx begin failed: {e}")))?;

        tx.execute(
            "DELETE FROM knowledge_atoms_fts WHERE id IN (SELECT id FROM knowledge_atoms WHERE source = ?1)",
            [source],
        )
        .map_err(|e| KurultaiError::Store(format!("delete_source fts failed: {e}")))?;
        tx.execute("DELETE FROM knowledge_atoms WHERE source = ?1", [source])
            .map_err(|e| KurultaiError::Store(format!("delete_source failed: {e}")))?;
        tx.commit()
            .map_err(|e| KurultaiError::Store(format!("delete_source tx commit failed: {e}")))?;
        tracing::debug!(source, "deleted atoms for source");
        Ok(())
    }

    async fn delete_source_ids_not_in(&self, source: &str, keep_ids: &[&str]) -> Result<u64> {
        if keep_ids.is_empty() {
            self.delete_source(source).await?;
            return Ok(0);
        }

        let placeholders: Vec<String> = keep_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM knowledge_atoms_fts WHERE id IN (
                SELECT id FROM knowledge_atoms WHERE source = ?1 AND source_id NOT IN ({})
             )",
            placeholders.join(",")
        );
        let main_sql = format!(
            "DELETE FROM knowledge_atoms WHERE source = ?1 AND source_id NOT IN ({})",
            placeholders.join(",")
        );

        let mut conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;
        let tx = conn.transaction().map_err(|e| {
            KurultaiError::Store(format!("delete_source_ids_not_in tx begin failed: {e}"))
        })?;

        let mut fts_params: Vec<&dyn rusqlite::ToSql> = vec![&source];
        for id in keep_ids {
            fts_params.push(id);
        }
        tx.execute(&sql, &*fts_params).map_err(|e| {
            KurultaiError::Store(format!("delete_source_ids_not_in fts failed: {e}"))
        })?;

        let mut main_params: Vec<&dyn rusqlite::ToSql> = vec![&source];
        for id in keep_ids {
            main_params.push(id);
        }
        let deleted = tx
            .execute(&main_sql, &*main_params)
            .map_err(|e| KurultaiError::Store(format!("delete_source_ids_not_in failed: {e}")))?;

        tx.commit().map_err(|e| {
            KurultaiError::Store(format!("delete_source_ids_not_in tx commit failed: {e}"))
        })?;
        tracing::debug!(source, deleted, "deleted stale atoms for source");
        Ok(deleted as u64)
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

    async fn content_hash_unchanged(&self, id: &str, content_hash: &str) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))?;
        let existing: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM knowledge_atoms WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("content_hash check failed: {e}")))?;
        Ok(existing.map(|h| h == content_hash).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::KnowledgeAtom;
    use chrono::Utc;
    use std::collections::HashMap;

    struct TestStore {
        #[allow(dead_code)]
        tmp: tempfile::TempDir,
        store: SqliteVecStore,
    }

    impl TestStore {
        fn new(model: &str, dim: usize) -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let store = SqliteVecStore::open(tmp.path().join("store.db"), model, dim).unwrap();
            Self { tmp, store }
        }
    }

    fn atom(id: &str, content: &str) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "test".into(),
            source_id: id.into(),
            title: "title".into(),
            summary: "summary".into(),
            content: content.into(),
            question: None,
            resolution: None,
            tags: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            metadata: HashMap::new(),
            embedding: None,
            content_hash: format!("hash-{content}"),
            source_uri: None,
            provenance: None,
        }
    }

    fn atom_with_embed(id: &str, content: &str, embedding: Vec<f32>) -> KnowledgeAtom {
        let mut a = atom(id, content);
        a.embedding = Some(embedding);
        a
    }

    #[tokio::test]
    async fn fresh_db_migrates_and_counts_zero() {
        let ts = TestStore::new("model-a", 4);
        assert_eq!(ts.store.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn upsert_and_fts_search() {
        let ts = TestStore::new("model-a", 4);
        let mut a = atom("1", "the quick brown fox");
        a.title = "animals".into();
        ts.store.upsert(&a).await.unwrap();
        assert_eq!(ts.store.count().await.unwrap(), 1);

        let hits = ts.store.fts_search("fox", 10).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0.id, "1");
    }

    #[tokio::test]
    async fn zero_vector_upsert_rejected() {
        let ts = TestStore::new("model-a", 4);
        let a = atom_with_embed("1", "x", vec![0.0, 0.0, 0.0, 0.0]);
        assert!(ts.store.upsert(&a).await.is_err());
    }

    #[tokio::test]
    async fn dimension_mismatch_upsert_rejected() {
        let ts = TestStore::new("model-a", 4);
        let a = atom_with_embed("1", "x", vec![1.0, 2.0, 3.0]);
        assert!(ts.store.upsert(&a).await.is_err());
    }

    #[tokio::test]
    async fn fingerprint_mismatch_on_open_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("store.db");
        {
            let store = SqliteVecStore::open(path.clone(), "model-a", 4).unwrap();
            store.upsert(&atom("1", "x")).await.unwrap();
        }
        let result = SqliteVecStore::open(path, "model-b", 4);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn vector_search_orders_by_cosine() {
        let ts = TestStore::new("model-a", 3);
        let a = atom_with_embed("a", "alpha", vec![1.0, 0.0, 0.0]);
        let b = atom_with_embed("b", "beta", vec![0.0, 1.0, 0.0]);
        let c = atom_with_embed("c", "gamma", vec![1.0, 1.0, 0.0]);
        ts.store.upsert_batch(&[a, b, c]).await.unwrap();

        let query = vec![1.0, 0.0, 0.0];
        let hits = ts.store.vector_search(&query, 2).await.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0.id, "a");
    }

    #[tokio::test]
    async fn delete_source_removes_atoms_and_fts() {
        let ts = TestStore::new("model-a", 4);
        ts.store.upsert(&atom("1", "foo")).await.unwrap();
        ts.store.delete_source("test").await.unwrap();
        assert_eq!(ts.store.count().await.unwrap(), 0);
        assert!(ts.store.fts_search("foo", 10).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn delete_source_ids_not_in_cleans_orphans() {
        let ts = TestStore::new("model-a", 4);
        ts.store.upsert(&atom("1", "keep")).await.unwrap();
        ts.store.upsert(&atom("2", "drop")).await.unwrap();
        ts.store
            .delete_source_ids_not_in("test", &["1"])
            .await
            .unwrap();
        assert_eq!(ts.store.count().await.unwrap(), 1);
        assert!(ts.store.fts_search("drop", 10).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn content_hash_unchanged_detects_same_content() {
        let ts = TestStore::new("model-a", 4);
        let a = atom("1", "stable");
        ts.store.upsert(&a).await.unwrap();
        assert!(ts
            .store
            .content_hash_unchanged("1", "hash-stable")
            .await
            .unwrap());
        assert!(!ts
            .store
            .content_hash_unchanged("1", "hash-changed")
            .await
            .unwrap());
    }
}
