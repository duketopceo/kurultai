use crate::types::KnowledgeAtom;
use anyhow::{anyhow, bail, Context, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once};
use zerocopy::IntoBytes;

static SQLITE_VEC_INIT: Once = Once::new();

/// Register sqlite-vec before opening any connection that needs vec0.
pub fn init_sqlite_vec() {
    SQLITE_VEC_INIT.call_once(|| unsafe {
        #[allow(clippy::missing_transmute_annotations)]
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    });
}

/// Storage backend for knowledge atoms and their embeddings.
pub trait Store: Send + Sync {
    fn upsert(&self, atom: &KnowledgeAtom) -> Result<()>;
    fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<KnowledgeAtom>>;
    fn get_by_source_id(&self, source: &str, source_id: &str) -> Result<Option<KnowledgeAtom>>;
    fn vector_search(&self, query_embed: &[f32], limit: usize)
        -> Result<Vec<(KnowledgeAtom, f64)>>;
    fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>>;
    fn delete_source(&self, source: &str) -> Result<()>;
    fn delete_id(&self, id: &str) -> Result<()>;
    /// Remove atoms for `source` whose `source_id` is not in `keep`.
    fn delete_orphans(&self, source: &str, keep: &[String]) -> Result<u64>;
    fn count(&self) -> Result<u64>;
    fn embed_model(&self) -> Result<Option<String>>;
    fn embed_dim(&self) -> Result<Option<usize>>;
    fn path(&self) -> &Path;
}

/// SQLite + FTS5 + sqlite-vec storage.
pub struct SqliteVecStore {
    path: PathBuf,
    conn: Mutex<Connection>,
    expected_model: String,
    expected_dim: usize,
}

impl SqliteVecStore {
    /// Open (or create) a store. Applies schema-at-open; fingerprints embed contract.
    pub fn open(path: impl AsRef<Path>, embed_model: &str, embed_dim: usize) -> Result<Self> {
        if embed_dim == 0 {
            bail!("embed_dim must be > 0");
        }
        init_sqlite_vec();
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create store parent {}", parent.display()))?;
        }
        let conn =
            Connection::open(&path).with_context(|| format!("open sqlite {}", path.display()))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::apply_schema(&conn)?;
        Self::ensure_meta(&conn, embed_model, embed_dim)?;
        Self::ensure_vec_table(&conn, embed_dim)?;
        Ok(Self {
            path,
            conn: Mutex::new(conn),
            expected_model: embed_model.to_string(),
            expected_dim: embed_dim,
        })
    }

    fn apply_schema(conn: &Connection) -> Result<()> {
        let schema = include_str!("schema.sql");
        conn.execute_batch(schema)?;
        Ok(())
    }

    fn ensure_meta(conn: &Connection, embed_model: &str, embed_dim: usize) -> Result<()> {
        let existing_model: Option<String> = conn
            .query_row(
                "SELECT value FROM store_meta WHERE key = 'embed_model'",
                [],
                |r| r.get(0),
            )
            .ok();
        let existing_dim: Option<String> = conn
            .query_row(
                "SELECT value FROM store_meta WHERE key = 'embed_dim'",
                [],
                |r| r.get(0),
            )
            .ok();

        match (existing_model, existing_dim) {
            (None, None) => {
                conn.execute(
                    "INSERT INTO store_meta(key, value) VALUES ('embed_model', ?1)",
                    params![embed_model],
                )?;
                conn.execute(
                    "INSERT INTO store_meta(key, value) VALUES ('embed_dim', ?1)",
                    params![embed_dim.to_string()],
                )?;
            }
            (Some(m), Some(d)) => {
                let d: usize = d
                    .parse()
                    .map_err(|_| anyhow!("corrupt store_meta.embed_dim: {d}"))?;
                if m != embed_model {
                    bail!(
                        "store embed_model mismatch: db={m} config={embed_model} (refuse mixed models)"
                    );
                }
                if d != embed_dim {
                    bail!(
                        "store embed_dim mismatch: db={d} config={embed_dim} (refuse mixed dims)"
                    );
                }
            }
            _ => bail!("store_meta incomplete: both embed_model and embed_dim required"),
        }
        Ok(())
    }

    fn ensure_vec_table(conn: &Connection, embed_dim: usize) -> Result<()> {
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='atoms_vec'",
            [],
            |r| r.get(0),
        )?;
        if !exists {
            let sql =
                format!("CREATE VIRTUAL TABLE atoms_vec USING vec0(embedding float[{embed_dim}])");
            conn.execute_batch(&sql)?;
        }
        Ok(())
    }

    fn reject_zero_vector(embedding: &[f32]) -> Result<()> {
        if embedding.is_empty() {
            bail!("refuse empty embedding vector");
        }
        if embedding.iter().all(|v| *v == 0.0) {
            bail!("refuse zero-vector embedding upsert");
        }
        Ok(())
    }

    fn row_to_atom(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeAtom> {
        let tags_json: String = row.get("tags")?;
        let meta_json: String = row.get("metadata")?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let metadata: HashMap<String, String> =
            serde_json::from_str(&meta_json).unwrap_or_default();
        let source_updated_at =
            chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("source_updated_at")?)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
        let indexed_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>("indexed_at")?)
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        Ok(KnowledgeAtom {
            id: row.get("id")?,
            source: row.get("source")?,
            source_id: row.get("source_id")?,
            source_uri: row.get("source_uri")?,
            title: row.get("title")?,
            summary: row.get("summary")?,
            content: row.get("content")?,
            question: row.get("question")?,
            resolution: row.get("resolution")?,
            tags,
            provenance: row.get("provenance")?,
            source_updated_at,
            indexed_at,
            content_hash: row.get("content_hash")?,
            embedding: None,
            metadata,
        })
    }

    fn upsert_inner(conn: &Connection, atom: &KnowledgeAtom, expected_dim: usize) -> Result<()> {
        if let Some(ref emb) = atom.embedding {
            Self::reject_zero_vector(emb)?;
            if emb.len() != expected_dim {
                bail!(
                    "embedding dim mismatch: got {} expected {}",
                    emb.len(),
                    expected_dim
                );
            }
        }

        let tags = serde_json::to_string(&atom.tags)?;
        let metadata = serde_json::to_string(&atom.metadata)?;
        let has_embedding = atom.embedding.is_some() as i64;

        // Preserve rowid across content updates by looking up existing id.
        let existing_rowid: Option<i64> = conn
            .query_row(
                "SELECT rowid FROM atoms WHERE id = ?1",
                params![&atom.id],
                |r| r.get(0),
            )
            .ok();

        if let Some(rowid) = existing_rowid {
            conn.execute(
                "UPDATE atoms SET source=?1, source_id=?2, source_uri=?3, title=?4, summary=?5,
                 content=?6, question=?7, resolution=?8, tags=?9, provenance=?10,
                 source_updated_at=?11, indexed_at=?12, content_hash=?13, metadata=?14,
                 has_embedding=?15 WHERE rowid=?16",
                params![
                    atom.source,
                    atom.source_id,
                    atom.source_uri,
                    atom.title,
                    atom.summary,
                    atom.content,
                    atom.question,
                    atom.resolution,
                    tags,
                    atom.provenance,
                    atom.source_updated_at.to_rfc3339(),
                    atom.indexed_at.to_rfc3339(),
                    atom.content_hash,
                    metadata,
                    has_embedding,
                    rowid,
                ],
            )?;
            if let Some(ref emb) = atom.embedding {
                conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", params![rowid])?;
                conn.execute(
                    "INSERT INTO atoms_vec(rowid, embedding) VALUES (?1, ?2)",
                    params![rowid, emb.as_bytes()],
                )?;
            }
        } else {
            conn.execute(
                "INSERT INTO atoms (
                    id, source, source_id, source_uri, title, summary, content, question, resolution,
                    tags, provenance, source_updated_at, indexed_at, content_hash, metadata, has_embedding
                ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                params![
                    atom.id,
                    atom.source,
                    atom.source_id,
                    atom.source_uri,
                    atom.title,
                    atom.summary,
                    atom.content,
                    atom.question,
                    atom.resolution,
                    tags,
                    atom.provenance,
                    atom.source_updated_at.to_rfc3339(),
                    atom.indexed_at.to_rfc3339(),
                    atom.content_hash,
                    metadata,
                    has_embedding,
                ],
            )?;
            if let Some(ref emb) = atom.embedding {
                let rowid: i64 = conn.query_row(
                    "SELECT rowid FROM atoms WHERE id = ?1",
                    params![&atom.id],
                    |r| r.get(0),
                )?;
                conn.execute(
                    "INSERT INTO atoms_vec(rowid, embedding) VALUES (?1, ?2)",
                    params![rowid, emb.as_bytes()],
                )?;
            }
        }
        Ok(())
    }
}

impl Store for SqliteVecStore {
    fn upsert(&self, atom: &KnowledgeAtom) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        Self::upsert_inner(&conn, atom, self.expected_dim)
    }

    fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        let tx = conn.unchecked_transaction()?;
        for atom in atoms {
            Self::upsert_inner(&tx, atom, self.expected_dim)?;
        }
        tx.commit()?;
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, source_id, source_uri, title, summary, content, question, resolution,
                    tags, provenance, source_updated_at, indexed_at, content_hash, metadata
             FROM atoms WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_atom(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_by_source_id(&self, source: &str, source_id: &str) -> Result<Option<KnowledgeAtom>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, source_id, source_uri, title, summary, content, question, resolution,
                    tags, provenance, source_updated_at, indexed_at, content_hash, metadata
             FROM atoms WHERE source = ?1 AND source_id = ?2",
        )?;
        let mut rows = stmt.query(params![source, source_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_atom(row)?))
        } else {
            Ok(None)
        }
    }

    fn vector_search(
        &self,
        query_embed: &[f32],
        limit: usize,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        Self::reject_zero_vector(query_embed)?;
        if query_embed.len() != self.expected_dim {
            bail!(
                "query embedding dim mismatch: got {} expected {}",
                query_embed.len(),
                self.expected_dim
            );
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        let mut stmt = conn.prepare(
            "SELECT a.id, a.source, a.source_id, a.source_uri, a.title, a.summary, a.content,
                    a.question, a.resolution, a.tags, a.provenance, a.source_updated_at,
                    a.indexed_at, a.content_hash, a.metadata, v.distance
             FROM atoms_vec v
             JOIN atoms a ON a.rowid = v.rowid
             WHERE v.embedding MATCH ?1 AND k = ?2
             ORDER BY v.distance",
        )?;
        let mut rows = stmt.query(params![query_embed.as_bytes(), limit as i64])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let mut atom = Self::row_to_atom(row)?;
            // distance from MATCH; convert to similarity-ish score
            let distance: f64 = row.get("distance")?;
            let score = 1.0 / (1.0 + distance);
            let _ = &mut atom;
            out.push((atom, score));
        }
        Ok(out)
    }

    fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        // Escape FTS special chars loosely: quote the whole query as a phrase if needed.
        let fts_query = sanitize_fts_query(query);
        let mut stmt = conn.prepare(
            "SELECT a.id, a.source, a.source_id, a.source_uri, a.title, a.summary, a.content,
                    a.question, a.resolution, a.tags, a.provenance, a.source_updated_at,
                    a.indexed_at, a.content_hash, a.metadata,
                    bm25(atoms_fts) AS rank
             FROM atoms_fts
             JOIN atoms a ON a.rowid = atoms_fts.rowid
             WHERE atoms_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let mut rows = stmt.query(params![fts_query, limit as i64])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let atom = Self::row_to_atom(row)?;
            let rank: f64 = row.get("rank")?;
            // bm25: lower is better; invert for score
            let score = 1.0 / (1.0 + rank.abs());
            out.push((atom, score));
        }
        Ok(out)
    }

    fn delete_source(&self, source: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        conn.execute(
            "DELETE FROM atoms_vec WHERE rowid IN (SELECT rowid FROM atoms WHERE source = ?1)",
            params![source],
        )?;
        conn.execute("DELETE FROM atoms WHERE source = ?1", params![source])?;
        Ok(())
    }

    fn delete_id(&self, id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        conn.execute(
            "DELETE FROM atoms_vec WHERE rowid IN (SELECT rowid FROM atoms WHERE id = ?1)",
            params![id],
        )?;
        conn.execute("DELETE FROM atoms WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn delete_orphans(&self, source: &str, keep: &[String]) -> Result<u64> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        let mut placeholders = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        values.push(Box::new(source.to_string()));
        for (i, id) in keep.iter().enumerate() {
            placeholders.push(format!("?{}", i + 2));
            values.push(Box::new(id.clone()));
        }
        let sql = if keep.is_empty() {
            "DELETE FROM atoms_vec WHERE rowid IN (SELECT rowid FROM atoms WHERE source = ?1)"
                .to_string()
        } else {
            format!(
                "DELETE FROM atoms_vec WHERE rowid IN (
                    SELECT rowid FROM atoms WHERE source = ?1 AND source_id NOT IN ({})
                )",
                placeholders.join(",")
            )
        };
        let params_ref: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
        if keep.is_empty() {
            conn.execute(
                "DELETE FROM atoms_vec WHERE rowid IN (SELECT rowid FROM atoms WHERE source = ?1)",
                params![source],
            )?;
            let n = conn.execute("DELETE FROM atoms WHERE source = ?1", params![source])?;
            return Ok(n as u64);
        }
        conn.execute(&sql, params_ref.as_slice())?;
        let del_sql = format!(
            "DELETE FROM atoms WHERE source = ?1 AND source_id NOT IN ({})",
            placeholders.join(",")
        );
        let params_ref: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
        let n = conn.execute(&del_sql, params_ref.as_slice())?;
        Ok(n as u64)
    }

    fn count(&self) -> Result<u64> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| anyhow!("store lock poisoned"))?;
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM atoms", [], |r| r.get(0))?;
        Ok(n as u64)
    }

    fn embed_model(&self) -> Result<Option<String>> {
        Ok(Some(self.expected_model.clone()))
    }

    fn embed_dim(&self) -> Result<Option<usize>> {
        Ok(Some(self.expected_dim))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

fn sanitize_fts_query(query: &str) -> String {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return "\"\"".into();
    }
    // Prefer AND of tokens. Strip FTS operators (incl. '-') so "zebra-migration"
    // does not parse as `zebra NOT migration`.
    let tokens: Vec<String> = trimmed
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .map(|t| {
            t.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        format!("\"{}\"", trimmed.replace('"', ""))
    } else {
        tokens.join(" AND ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    fn sample_atom(id: &str, content: &str, emb: Option<Vec<f32>>) -> KnowledgeAtom {
        KnowledgeAtom {
            id: id.into(),
            source: "test".into(),
            source_id: id.into(),
            source_uri: Some(format!("file://{id}")),
            title: format!("Title {id}"),
            summary: String::new(),
            content: content.into(),
            question: None,
            resolution: None,
            tags: vec!["tag1".into()],
            provenance: Some("unit-test".into()),
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            content_hash: format!("hash-{id}"),
            embedding: emb,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn open_creates_schema() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.db");
        let store = SqliteVecStore::open(&path, "test-model", 4).unwrap();
        assert_eq!(store.count().unwrap(), 0);
        assert_eq!(store.embed_dim().unwrap(), Some(4));
    }

    #[test]
    fn upsert_fts_roundtrip() {
        let dir = tempdir().unwrap();
        let store = SqliteVecStore::open(dir.path().join("t.db"), "m", 4).unwrap();
        store
            .upsert(&sample_atom("a1", "unique phrase zebra-migration", None))
            .unwrap();
        let hits = store.fts_search("zebra-migration", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0.id, "a1");
        assert!(hits[0].0.provenance.as_deref() == Some("unit-test"));
    }

    #[test]
    fn refuse_zero_vector() {
        let dir = tempdir().unwrap();
        let store = SqliteVecStore::open(dir.path().join("t.db"), "m", 4).unwrap();
        let err = store
            .upsert(&sample_atom("z", "x", Some(vec![0.0; 4])))
            .unwrap_err();
        assert!(err.to_string().contains("zero-vector"));
    }

    #[test]
    fn meta_fingerprint_rejects_mixed_dim() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.db");
        let _ = SqliteVecStore::open(&path, "m", 4).unwrap();
        let err = match SqliteVecStore::open(&path, "m", 8) {
            Ok(_) => panic!("expected dim mismatch"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("embed_dim mismatch"));
    }

    #[test]
    fn vector_search_works() {
        let dir = tempdir().unwrap();
        let store = SqliteVecStore::open(dir.path().join("t.db"), "m", 4).unwrap();
        let emb = vec![0.1, 0.2, 0.3, 0.4];
        store
            .upsert(&sample_atom("v1", "vector doc", Some(emb.clone())))
            .unwrap();
        let hits = store.vector_search(&emb, 5).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0.id, "v1");
    }
}
