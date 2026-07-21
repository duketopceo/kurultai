pub mod migrations;

use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::types::KnowledgeAtom;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use zerocopy::AsBytes;

/// Norm below this is treated as a zero / stub vector — never written to `atoms_vec`.
const MIN_EMBEDDING_NORM: f32 = 1e-6;

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

    /// FTS ranks as `(id, score)` without hydrating full atoms.
    async fn fts_search_ids(&self, query: &str, limit: usize) -> Result<Vec<(String, f64)>>;

    /// Vector ranks as `(id, score)` without hydrating full atoms.
    async fn vector_search_ids(
        &self,
        query_embed: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f64)>>;

    /// Batch load atoms by id (order not guaranteed; missing ids omitted).
    async fn get_many(&self, ids: &[String]) -> Result<Vec<KnowledgeAtom>>;

    /// Delete atoms for a given source (for re-index).
    async fn delete_source(&self, source: &str) -> Result<()>;

    /// Total atom count.
    async fn count(&self) -> Result<u64>;

    /// Lookup by source + source_id (cite path).
    async fn get_by_source_id(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<Option<KnowledgeAtom>>;

    /// True when atom `id` already has `content_hash` and a stored vector (hash-skip re-embed).
    async fn has_fresh_embedding(&self, id: &str, content_hash: &str) -> Result<bool>;
}

/// SQLite + sqlite-vec storage implementation (#1).
pub struct SqliteVecStore {
    conn: Mutex<Connection>,
    path: PathBuf,
    embed_dim: usize,
}

impl SqliteVecStore {
    /// Open (or create) the database, run migrations, register sqlite-vec, create `atoms_vec`.
    pub fn open(path: PathBuf, embed_dim: usize) -> Result<Self> {
        register_sqlite_vec();

        let conn = Connection::open(&path)
            .map_err(|e| KurultaiError::Store(format!("failed to open {}: {e}", path.display())))?;

        migrations::migrate(&conn)?;
        migrations::ensure_vec_table(&conn, embed_dim)?;

        tracing::debug!(
            path = %path.display(),
            embed_dim,
            "sqlite store opened"
        );
        Ok(Self {
            conn: Mutex::new(conn),
            path,
            embed_dim,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }

    /// Fetch one atom by id (tests + cite path).
    pub fn get_by_id(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        let conn = self.lock()?;
        load_atom_by_id(&conn, id)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| KurultaiError::Store(format!("lock poisoned: {e}")))
    }

    fn upsert_sync(conn: &Connection, atom: &KnowledgeAtom, embed_dim: usize) -> Result<()> {
        let tags_json = serde_json::to_string(&atom.tags)
            .map_err(|e| KurultaiError::Store(format!("tags serialize: {e}")))?;
        let metadata_json = serde_json::to_string(&atom.metadata)
            .map_err(|e| KurultaiError::Store(format!("metadata serialize: {e}")))?;
        let content_hash = sha256_hex(&atom.content);
        let prior_hash: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM knowledge_atoms WHERE id = ?1",
                [&atom.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("prior hash lookup: {e}")))?;
        let hash_unchanged = prior_hash.as_deref() == Some(content_hash.as_str());

        conn.execute(
            r#"
            INSERT INTO knowledge_atoms (
                id, source, source_id, title, summary, content,
                question, resolution, tags_json,
                source_updated_at, indexed_at, metadata_json, content_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                source = excluded.source,
                source_id = excluded.source_id,
                title = excluded.title,
                summary = excluded.summary,
                content = excluded.content,
                question = excluded.question,
                resolution = excluded.resolution,
                tags_json = excluded.tags_json,
                source_updated_at = excluded.source_updated_at,
                indexed_at = excluded.indexed_at,
                metadata_json = excluded.metadata_json,
                content_hash = excluded.content_hash
            "#,
            params![
                atom.id,
                atom.source,
                atom.source_id,
                atom.title,
                atom.summary,
                atom.content,
                atom.question,
                atom.resolution,
                tags_json,
                atom.source_updated_at.to_rfc3339(),
                atom.indexed_at.to_rfc3339(),
                metadata_json,
                content_hash,
            ],
        )
        .map_err(|e| KurultaiError::Store(format!("upsert atom failed: {e}")))?;

        let rowid: i64 = conn
            .query_row(
                "SELECT rowid FROM knowledge_atoms WHERE id = ?1",
                [&atom.id],
                |r| r.get(0),
            )
            .map_err(|e| KurultaiError::Store(format!("rowid lookup failed: {e}")))?;

        // FTS: delete + insert (fts5 has no reliable UPSERT by id).
        conn.execute("DELETE FROM atoms_fts WHERE id = ?1", [&atom.id])
            .map_err(|e| KurultaiError::Store(format!("fts delete failed: {e}")))?;
        conn.execute(
            "INSERT INTO atoms_fts(id, title, summary, content) VALUES (?1, ?2, ?3, ?4)",
            params![atom.id, atom.title, atom.summary, atom.content],
        )
        .map_err(|e| KurultaiError::Store(format!("fts insert failed: {e}")))?;

        // Vector: write when a new embedding is provided; preserve existing when
        // content_hash is unchanged and caller skipped re-embed (hash-skip).
        match &atom.embedding {
            Some(emb) => {
                conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", [rowid])
                    .map_err(|e| KurultaiError::Store(format!("vec delete failed: {e}")))?;
                if emb.len() != embed_dim {
                    return Err(KurultaiError::Store(format!(
                        "embedding dim {} != store embed_dim {embed_dim} for atom {}",
                        emb.len(),
                        atom.id
                    )));
                }
                if embedding_norm(emb) >= MIN_EMBEDDING_NORM {
                    conn.execute(
                        "INSERT INTO atoms_vec(rowid, embedding) VALUES (?1, ?2)",
                        params![rowid, emb.as_bytes()],
                    )
                    .map_err(|e| KurultaiError::Store(format!("vec insert failed: {e}")))?;
                } else {
                    tracing::debug!(id = %atom.id, "skipping near-zero embedding for vec index");
                }
            }
            None if !hash_unchanged => {
                conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", [rowid])
                    .map_err(|e| KurultaiError::Store(format!("vec delete failed: {e}")))?;
            }
            None => {}
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Store for SqliteVecStore {
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()> {
        let conn = self.lock()?;
        Self::upsert_sync(&conn, atom, self.embed_dim)?;
        Ok(())
    }

    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()> {
        let conn = self.lock()?;
        conn.execute_batch("BEGIN IMMEDIATE;")
            .map_err(|e| KurultaiError::Store(format!("begin batch: {e}")))?;
        let result = (|| {
            for atom in atoms {
                Self::upsert_sync(&conn, atom, self.embed_dim)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| KurultaiError::Store(format!("commit batch: {e}")))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    async fn vector_search(
        &self,
        query_embed: &[f32],
        limit: usize,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let ids = self.vector_search_ids(query_embed, limit).await?;
        hydrate_ranked(self, ids).await
    }

    async fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let ids = self.fts_search_ids(query, limit).await?;
        hydrate_ranked(self, ids).await
    }

    async fn fts_search_ids(&self, query: &str, limit: usize) -> Result<Vec<(String, f64)>> {
        if limit == 0 || query.trim().is_empty() {
            return Ok(vec![]);
        }

        let fts_query = sanitize_fts_query(query);
        if fts_query.is_empty() {
            return Ok(vec![]);
        }

        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT a.id, bm25(atoms_fts) AS score
                FROM atoms_fts
                JOIN knowledge_atoms a ON a.id = atoms_fts.id
                WHERE atoms_fts MATCH ?1
                ORDER BY score
                LIMIT ?2
                "#,
            )
            .map_err(|e| KurultaiError::Store(format!("fts_search_ids prepare: {e}")))?;

        let rows = stmt
            .query_map(params![fts_query, limit as i64], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
            })
            .map_err(|e| KurultaiError::Store(format!("fts_search_ids query: {e}")))?;

        let mut out = Vec::new();
        for row in rows {
            let (id, bm25_score) =
                row.map_err(|e| KurultaiError::Store(format!("fts_search_ids row: {e}")))?;
            let score = 1.0 / (1.0 + bm25_score.abs());
            out.push((id, score));
        }
        Ok(out)
    }

    async fn vector_search_ids(
        &self,
        query_embed: &[f32],
        limit: usize,
    ) -> Result<Vec<(String, f64)>> {
        if limit == 0 {
            return Ok(vec![]);
        }
        if query_embed.len() != self.embed_dim {
            return Err(KurultaiError::Store(format!(
                "query embed dim {} != store embed_dim {}",
                query_embed.len(),
                self.embed_dim
            )));
        }
        if embedding_norm(query_embed) < MIN_EMBEDDING_NORM {
            return Ok(vec![]);
        }

        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT a.id, v.distance
                FROM atoms_vec v
                JOIN knowledge_atoms a ON a.rowid = v.rowid
                WHERE v.embedding MATCH ?1 AND k = ?2
                ORDER BY v.distance
                "#,
            )
            .map_err(|e| KurultaiError::Store(format!("vector_search_ids prepare: {e}")))?;

        let rows = stmt
            .query_map(params![query_embed.as_bytes(), limit as i64], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
            })
            .map_err(|e| KurultaiError::Store(format!("vector_search_ids query: {e}")))?;

        let mut out = Vec::new();
        for row in rows {
            let (id, distance) =
                row.map_err(|e| KurultaiError::Store(format!("vector_search_ids row: {e}")))?;
            let score = 1.0 / (1.0 + distance);
            out.push((id, score));
        }
        Ok(out)
    }

    async fn get_many(&self, ids: &[String]) -> Result<Vec<KnowledgeAtom>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let conn = self.lock()?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(atom) = load_atom_by_id(&conn, id)? {
                out.push(atom);
            }
        }
        Ok(out)
    }

    async fn delete_source(&self, source: &str) -> Result<()> {
        let conn = self.lock()?;
        conn.execute_batch("BEGIN IMMEDIATE;")
            .map_err(|e| KurultaiError::Store(format!("begin delete_source: {e}")))?;

        let result = (|| {
            // Collect rowids + ids first
            let mut stmt = conn
                .prepare("SELECT rowid, id FROM knowledge_atoms WHERE source = ?1")
                .map_err(|e| KurultaiError::Store(format!("delete_source select: {e}")))?;
            let pairs: Vec<(i64, String)> = stmt
                .query_map([source], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| KurultaiError::Store(format!("delete_source map: {e}")))?
                .collect::<std::result::Result<_, _>>()
                .map_err(|e| KurultaiError::Store(format!("delete_source collect: {e}")))?;

            for (rowid, id) in &pairs {
                conn.execute("DELETE FROM atoms_fts WHERE id = ?1", [id])
                    .map_err(|e| KurultaiError::Store(format!("delete fts: {e}")))?;
                conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", [rowid])
                    .map_err(|e| KurultaiError::Store(format!("delete vec: {e}")))?;
            }

            conn.execute("DELETE FROM knowledge_atoms WHERE source = ?1", [source])
                .map_err(|e| KurultaiError::Store(format!("delete_source failed: {e}")))?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| KurultaiError::Store(format!("commit delete_source: {e}")))?;
                tracing::debug!(source, "deleted atoms for source");
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    async fn count(&self) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_atoms", [], |row| row.get(0))
            .map_err(|e| KurultaiError::Store(format!("count failed: {e}")))?;
        Ok(count as u64)
    }

    async fn get_by_source_id(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<Option<KnowledgeAtom>> {
        let conn = self.lock()?;
        load_atom_by_source_id(&conn, source, source_id)
    }

    async fn has_fresh_embedding(&self, id: &str, content_hash: &str) -> Result<bool> {
        let conn = self.lock()?;
        let found: Option<i64> = conn
            .query_row(
                r#"
                SELECT 1
                FROM knowledge_atoms a
                JOIN atoms_vec v ON v.rowid = a.rowid
                WHERE a.id = ?1 AND a.content_hash = ?2
                LIMIT 1
                "#,
                params![id, content_hash],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("has_fresh_embedding: {e}")))?;
        Ok(found.is_some())
    }
}

/// Hydrate ranked `(id, score)` pairs into atoms, skipping missing ids.
async fn hydrate_ranked(
    store: &SqliteVecStore,
    ranked: Vec<(String, f64)>,
) -> Result<Vec<(KnowledgeAtom, f64)>> {
    if ranked.is_empty() {
        return Ok(vec![]);
    }
    let ids: Vec<String> = ranked.iter().map(|(id, _)| id.clone()).collect();
    let atoms = store.get_many(&ids).await?;
    let by_id: HashMap<String, KnowledgeAtom> =
        atoms.into_iter().map(|a| (a.id.clone(), a)).collect();
    Ok(ranked
        .into_iter()
        .filter_map(|(id, score)| by_id.get(&id).cloned().map(|atom| (atom, score)))
        .collect())
}

/// Register sqlite-vec once per process (safe to call repeatedly).
fn register_sqlite_vec() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
        tracing::debug!("sqlite-vec extension registered");
    });
}

fn embedding_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Build a safe FTS5 MATCH query from free text (AND of quoted tokens).
fn sanitize_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|t| t.chars().any(|c| c.is_alphanumeric()))
        .map(|t| {
            let cleaned: String = t
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            format!("\"{cleaned}\"")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_atom_by_id(conn: &Connection, id: &str) -> Result<Option<KnowledgeAtom>> {
    conn.query_row(
        r#"
        SELECT id, source, source_id, title, summary, content,
               question, resolution, tags_json,
               source_updated_at, indexed_at, metadata_json
        FROM knowledge_atoms WHERE id = ?1
        "#,
        [id],
        row_to_atom,
    )
    .optional()
    .map_err(|e| KurultaiError::Store(format!("load_atom_by_id: {e}")))
}

fn load_atom_by_source_id(
    conn: &Connection,
    source: &str,
    source_id: &str,
) -> Result<Option<KnowledgeAtom>> {
    conn.query_row(
        r#"
        SELECT id, source, source_id, title, summary, content,
               question, resolution, tags_json,
               source_updated_at, indexed_at, metadata_json
        FROM knowledge_atoms WHERE source = ?1 AND source_id = ?2
        LIMIT 1
        "#,
        params![source, source_id],
        row_to_atom,
    )
    .optional()
    .map_err(|e| KurultaiError::Store(format!("load_atom_by_source_id: {e}")))
}

fn row_to_atom(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeAtom> {
    let tags_json: String = row.get(8)?;
    let metadata_json: String = row.get(11)?;
    let source_updated_at: String = row.get(9)?;
    let indexed_at: String = row.get(10)?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let metadata: HashMap<String, String> =
        serde_json::from_str(&metadata_json).unwrap_or_default();

    Ok(KnowledgeAtom {
        id: row.get(0)?,
        source: row.get(1)?,
        source_id: row.get(2)?,
        title: row.get(3)?,
        summary: row.get(4)?,
        content: row.get(5)?,
        question: row.get(6)?,
        resolution: row.get(7)?,
        tags,
        source_updated_at: parse_dt(&source_updated_at),
        indexed_at: parse_dt(&indexed_at),
        embedding: None, // not loaded on read path by default (token budget)
        metadata,
    })
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn sample_atom(id: &str, title: &str, content: &str, emb: Option<Vec<f32>>) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "markdown".into(),
            source_id: format!("/{id}.md"),
            title: title.into(),
            summary: content.chars().take(80).collect(),
            content: content.into(),
            question: None,
            resolution: None,
            tags: vec!["test".into()],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: emb,
            metadata: HashMap::new(),
        }
    }

    fn temp_store(dim: usize) -> SqliteVecStore {
        let dir = std::env::temp_dir().join(format!(
            "kurultai-store-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        SqliteVecStore::open(dir.join("store.db"), dim).unwrap()
    }

    #[tokio::test]
    async fn upsert_and_get_by_id() {
        let store = temp_store(4);
        let atom = sample_atom(
            "a1",
            "Hello",
            "unique phrase alpha",
            Some(vec![0.1, 0.2, 0.3, 0.4]),
        );
        store.upsert(&atom).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 1);
        let loaded = store.get_by_id("a1").unwrap().expect("atom present");
        assert_eq!(loaded.title, "Hello");
        assert_eq!(loaded.content, "unique phrase alpha");
    }

    #[tokio::test]
    async fn upsert_updates_existing() {
        let store = temp_store(4);
        let mut atom = sample_atom("a1", "V1", "content one", Some(vec![1.0, 0.0, 0.0, 0.0]));
        store.upsert(&atom).await.unwrap();
        atom.title = "V2".into();
        atom.content = "content two".into();
        store.upsert(&atom).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 1);
        let loaded = store.get_by_id("a1").unwrap().unwrap();
        assert_eq!(loaded.title, "V2");
        assert_eq!(loaded.content, "content two");
    }

    #[tokio::test]
    async fn fts_search_matches_keywords() {
        let store = temp_store(4);
        store
            .upsert(&sample_atom(
                "a1",
                "Migration Guide",
                "how to run database migration scripts",
                None,
            ))
            .await
            .unwrap();
        store
            .upsert(&sample_atom("a2", "Cooking", "how to boil pasta", None))
            .await
            .unwrap();

        let hits = store.fts_search("database migration", 10).await.unwrap();
        assert!(!hits.is_empty(), "expected FTS hit");
        assert_eq!(hits[0].0.id, "a1");
    }

    #[tokio::test]
    async fn fts_search_ids_then_get_many() {
        let store = temp_store(4);
        store
            .upsert(&sample_atom(
                "a1",
                "Migration Guide",
                "how to run database migration scripts",
                None,
            ))
            .await
            .unwrap();
        let ranks = store
            .fts_search_ids("database migration", 10)
            .await
            .unwrap();
        assert_eq!(ranks[0].0, "a1");
        let atoms = store
            .get_many(&ranks.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>())
            .await
            .unwrap();
        assert_eq!(atoms[0].title, "Migration Guide");
        let missing = store.get_many(&["no-such-id".into()]).await.unwrap();
        assert!(missing.is_empty());
    }

    #[tokio::test]
    async fn vector_search_nearest_neighbors() {
        let store = temp_store(4);
        store
            .upsert(&sample_atom(
                "near",
                "Near",
                "near vec",
                Some(vec![0.9, 0.9, 0.9, 0.9]),
            ))
            .await
            .unwrap();
        store
            .upsert(&sample_atom(
                "far",
                "Far",
                "far vec",
                Some(vec![0.0, 0.0, 0.0, 0.1]),
            ))
            .await
            .unwrap();

        let hits = store
            .vector_search(&[0.85, 0.85, 0.85, 0.85], 2)
            .await
            .unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0.id, "near");
    }

    #[tokio::test]
    async fn hash_skip_preserves_vector_when_embedding_omitted() {
        let store = temp_store(4);
        let atom = sample_atom(
            "keep",
            "Keep",
            "stable content",
            Some(vec![0.5, 0.5, 0.5, 0.5]),
        );
        store.upsert(&atom).await.unwrap();
        let hash = sha256_hex(&atom.content);
        assert!(store.has_fresh_embedding("keep", &hash).await.unwrap());

        // Re-upsert same content without embedding — vec must remain searchable.
        let mut again = atom.clone();
        again.embedding = None;
        again.title = "Keep (retitled)".into();
        store.upsert(&again).await.unwrap();
        assert!(store.has_fresh_embedding("keep", &hash).await.unwrap());
        let hits = store.vector_search(&[0.5, 0.5, 0.5, 0.5], 1).await.unwrap();
        assert_eq!(hits[0].0.id, "keep");
        assert_eq!(hits[0].0.title, "Keep (retitled)");
    }

    #[tokio::test]
    async fn changed_content_without_embedding_drops_stale_vector() {
        let store = temp_store(4);
        let atom = sample_atom("stale", "Stale", "old body", Some(vec![0.7, 0.1, 0.1, 0.1]));
        store.upsert(&atom).await.unwrap();
        assert!(store
            .has_fresh_embedding("stale", &sha256_hex("old body"))
            .await
            .unwrap());

        let mut changed = atom;
        changed.content = "new body".into();
        changed.embedding = None;
        store.upsert(&changed).await.unwrap();

        assert!(!store
            .has_fresh_embedding("stale", &sha256_hex("old body"))
            .await
            .unwrap());
        assert!(!store
            .has_fresh_embedding("stale", &sha256_hex("new body"))
            .await
            .unwrap());
        let hits = store.vector_search(&[0.7, 0.1, 0.1, 0.1], 5).await.unwrap();
        assert!(hits.is_empty(), "stale vector must be removed");
    }

    #[tokio::test]
    async fn zero_vector_not_indexed_in_vec() {
        let store = temp_store(4);
        store
            .upsert(&sample_atom(
                "z",
                "Zero",
                "has zero embed",
                Some(vec![0.0, 0.0, 0.0, 0.0]),
            ))
            .await
            .unwrap();
        let hits = store.vector_search(&[0.1, 0.1, 0.1, 0.1], 5).await.unwrap();
        assert!(
            hits.is_empty(),
            "zero vectors must not appear in vec search"
        );
        // Still in FTS / count
        assert_eq!(store.count().await.unwrap(), 1);
        assert!(!store.fts_search("zero embed", 5).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn delete_source_removes_atoms() {
        let store = temp_store(4);
        store
            .upsert(&sample_atom(
                "a1",
                "T",
                "delete me please",
                Some(vec![0.2, 0.2, 0.2, 0.2]),
            ))
            .await
            .unwrap();
        store.delete_source("markdown").await.unwrap();
        assert_eq!(store.count().await.unwrap(), 0);
        assert!(store.fts_search("delete", 5).await.unwrap().is_empty());
        assert!(store
            .vector_search(&[0.2, 0.2, 0.2, 0.2], 5)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn upsert_batch_transactional() {
        let store = temp_store(4);
        let atoms = vec![
            sample_atom("b1", "B1", "batch one", Some(vec![0.1; 4])),
            sample_atom("b2", "B2", "batch two", Some(vec![0.2; 4])),
        ];
        store.upsert_batch(&atoms).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 2);
    }
}
