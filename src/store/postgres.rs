//! Postgres + pgvector `Store` for shared (`team` / `company`) hub atoms (HUB-2).
//!
//! Solo [`super::open_store`] stays SQLite. Personal atoms are refused (AE4).

use super::{IngestionJob, SearchFilter, Store, MIN_EMBEDDING_NORM};
use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::memory::{classify, GraphNode, MemoryTier, TierPolicy};
use crate::types::{
    normalize_soft_labels, CorpusTier, KnowledgeAtom, OntologyEntity, OntologyLink, TrustLane,
    VisibilityScope,
};
use chrono::{DateTime, Utc};
use pgvector::Vector;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Row, Transaction};

const ATOM_SELECT: &str = "id, source, source_id, title, summary, content, question, resolution, \
     tags_json, source_updated_at, indexed_at, metadata_json, trust_lane, quarantine_reason, \
     last_accessed_at, visibility, team_id, content_hash, soft_labels_json";

pub struct PostgresStore {
    pool: PgPool,
    embed_dim: usize,
}

impl PostgresStore {
    pub async fn connect(database_url: &str, embed_dim: usize) -> Result<Self> {
        if embed_dim == 0 {
            return Err(KurultaiError::Store(
                "postgres store embed_dim must be > 0".into(),
            ));
        }
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await
            .map_err(|e| KurultaiError::Store(format!("postgres connect: {e}")))?;
        let store = Self { pool, embed_dim };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> Result<()> {
        // Concurrent `CREATE … IF NOT EXISTS` still races on catalog unique indexes.
        // Serialize the whole migration on one connection.
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| KurultaiError::Store(format!("acquire for migrate: {e}")))?;
        sqlx::query("SELECT pg_advisory_lock(87231401)")
            .execute(&mut *conn)
            .await
            .map_err(|e| KurultaiError::Store(format!("advisory lock: {e}")))?;
        let result = self.migrate_locked(&mut conn).await;
        let _ = sqlx::query("SELECT pg_advisory_unlock(87231401)")
            .execute(&mut *conn)
            .await;
        result
    }

    async fn migrate_locked(&self, conn: &mut sqlx::postgres::PgConnection) -> Result<()> {
        exec_ddl(
            conn,
            "CREATE EXTENSION IF NOT EXISTS vector",
            "create extension vector",
        )
        .await?;

        exec_ddl(
            conn,
            r#"
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
                metadata_json TEXT NOT NULL DEFAULT '{}',
                content_hash TEXT NOT NULL DEFAULT '',
                trust_lane TEXT NOT NULL DEFAULT 'trusted',
                quarantine_reason TEXT,
                last_accessed_at TEXT NOT NULL DEFAULT '',
                visibility TEXT NOT NULL CHECK (visibility IN ('team', 'company')),
                team_id TEXT,
                soft_labels_json TEXT NOT NULL DEFAULT '[]'
            )
            "#,
            "create knowledge_atoms",
        )
        .await?;

        exec_ddl(
            conn,
            "CREATE INDEX IF NOT EXISTS idx_hub_atoms_source ON knowledge_atoms(source)",
            "idx source",
        )
        .await?;
        exec_ddl(
            conn,
            "CREATE INDEX IF NOT EXISTS idx_hub_atoms_visibility ON knowledge_atoms(visibility)",
            "idx visibility",
        )
        .await?;
        exec_ddl(
            conn,
            "CREATE INDEX IF NOT EXISTS idx_hub_atoms_hash_trusted ON knowledge_atoms(content_hash) WHERE trust_lane = 'trusted'",
            "idx hash",
        )
        .await?;

        exec_ddl(
            conn,
            r#"
            ALTER TABLE knowledge_atoms
            ADD COLUMN IF NOT EXISTS search_tsv tsvector
            GENERATED ALWAYS AS (
                to_tsvector('english', coalesce(title,'') || ' ' || coalesce(summary,'') || ' ' || coalesce(content,''))
            ) STORED
            "#,
            "search_tsv column",
        )
        .await?;
        exec_ddl(
            conn,
            "CREATE INDEX IF NOT EXISTS idx_hub_atoms_fts ON knowledge_atoms USING GIN (search_tsv)",
            "fts gin",
        )
        .await?;

        let vec_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS atoms_vec (
                atom_id TEXT PRIMARY KEY REFERENCES knowledge_atoms(id) ON DELETE CASCADE,
                embedding vector({embed_dim}) NOT NULL
            )
            "#,
            embed_dim = self.embed_dim
        );
        exec_ddl(conn, &vec_sql, "create atoms_vec").await?;

        let (vec_type,): (String,) = sqlx::query_as(
            "SELECT format_type(atttypid, atttypmod)
             FROM pg_attribute
             WHERE attrelid = 'atoms_vec'::regclass AND attname = 'embedding'",
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| KurultaiError::Store(format!("atoms_vec typmod: {e}")))?;
        let expected = format!("vector({})", self.embed_dim);
        if vec_type != expected {
            return Err(KurultaiError::Store(format!(
                "atoms_vec embedding is {vec_type}, expected {expected} (recreate the hub database)"
            )));
        }

        exec_ddl(
            conn,
            r#"
            CREATE TABLE IF NOT EXISTS quality_audit (
                id BIGSERIAL PRIMARY KEY,
                ts TEXT NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
                action TEXT NOT NULL,
                atom_id TEXT NOT NULL,
                actor TEXT NOT NULL,
                detail_json TEXT NOT NULL DEFAULT '{}'
            )
            "#,
            "quality_audit",
        )
        .await?;

        exec_ddl(
            conn,
            r#"
            CREATE TABLE IF NOT EXISTS merge_candidates (
                id BIGSERIAL PRIMARY KEY,
                atom_a TEXT NOT NULL,
                atom_b TEXT NOT NULL,
                reason TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
                UNIQUE(atom_a, atom_b)
            )
            "#,
            "merge_candidates",
        )
        .await?;

        exec_ddl(
            conn,
            r#"
            CREATE TABLE IF NOT EXISTS ingestion_jobs (
                id BIGSERIAL PRIMARY KEY,
                batch_id TEXT NOT NULL,
                source TEXT NOT NULL,
                file_path TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                atoms_count BIGINT,
                error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
                completed_at TEXT
            )
            "#,
            "ingestion_jobs",
        )
        .await?;

        Ok(())
    }

    fn reject_personal(atom: &KnowledgeAtom) -> Result<()> {
        if atom.visibility == VisibilityScope::Personal {
            return Err(KurultaiError::Store(format!(
                "personal atom {} cannot be written to hub postgres (AE4)",
                atom.id
            )));
        }
        Ok(())
    }

    fn embedding_norm(v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    fn atom_from_row(row: &PgRow) -> Result<KnowledgeAtom> {
        let tags_json: String = row
            .try_get("tags_json")
            .map_err(|e| KurultaiError::Store(format!("tags_json: {e}")))?;
        let metadata_json: String = row
            .try_get("metadata_json")
            .map_err(|e| KurultaiError::Store(format!("metadata_json: {e}")))?;
        let soft_json: String = row
            .try_get("soft_labels_json")
            .unwrap_or_else(|_| "[]".into());
        let source_updated_at: String = row
            .try_get("source_updated_at")
            .map_err(|e| KurultaiError::Store(format!("source_updated_at: {e}")))?;
        let indexed_at: String = row
            .try_get("indexed_at")
            .map_err(|e| KurultaiError::Store(format!("indexed_at: {e}")))?;
        let last_accessed_raw: String = row.try_get("last_accessed_at").unwrap_or_default();
        let trust_lane: String = row
            .try_get("trust_lane")
            .map_err(|e| KurultaiError::Store(format!("trust_lane: {e}")))?;
        let visibility_raw: String = row.try_get("visibility").unwrap_or_else(|_| "team".into());
        let indexed = parse_dt(&indexed_at);
        let last_accessed_at = if last_accessed_raw.is_empty() {
            indexed
        } else {
            parse_dt(&last_accessed_raw)
        };
        Ok(KnowledgeAtom {
            id: row
                .try_get("id")
                .map_err(|e| KurultaiError::Store(format!("id: {e}")))?,
            source: row
                .try_get("source")
                .map_err(|e| KurultaiError::Store(format!("source: {e}")))?,
            source_id: row
                .try_get("source_id")
                .map_err(|e| KurultaiError::Store(format!("source_id: {e}")))?,
            title: row
                .try_get("title")
                .map_err(|e| KurultaiError::Store(format!("title: {e}")))?,
            summary: row
                .try_get("summary")
                .map_err(|e| KurultaiError::Store(format!("summary: {e}")))?,
            content: row
                .try_get("content")
                .map_err(|e| KurultaiError::Store(format!("content: {e}")))?,
            question: row.try_get("question").ok(),
            resolution: row.try_get("resolution").ok(),
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            source_updated_at: parse_dt(&source_updated_at),
            indexed_at: indexed,
            last_accessed_at,
            embedding: None,
            metadata: serde_json::from_str(&metadata_json).unwrap_or_default(),
            trust_lane: TrustLane::parse(&trust_lane),
            quarantine_reason: row.try_get("quarantine_reason").ok(),
            soft_labels: serde_json::from_str(&soft_json).unwrap_or_default(),
            corpus_tier: CorpusTier::Public,
            visibility_labels: Vec::new(),
            visibility: VisibilityScope::parse(&visibility_raw),
        })
    }

    async fn upsert_in_tx(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        atom: &KnowledgeAtom,
        embed_dim: usize,
    ) -> Result<()> {
        Self::reject_personal(atom)?;
        let tags_json = serde_json::to_string(&atom.tags)
            .map_err(|e| KurultaiError::Store(format!("tags serialize: {e}")))?;
        let metadata_json = serde_json::to_string(&atom.metadata)
            .map_err(|e| KurultaiError::Store(format!("metadata serialize: {e}")))?;
        let soft_json = serde_json::to_string(&normalize_soft_labels(&atom.soft_labels))
            .map_err(|e| KurultaiError::Store(format!("soft_labels serialize: {e}")))?;
        let content_hash = sha256_hex(&atom.content);
        let last_accessed = if atom.last_accessed_at.timestamp() == 0 {
            atom.indexed_at
        } else {
            atom.last_accessed_at
        };
        let team_id = atom.metadata.get("team_id").cloned();
        let prior_hash: Option<(String,)> =
            sqlx::query_as("SELECT content_hash FROM knowledge_atoms WHERE id = $1")
                .bind(&atom.id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| KurultaiError::Store(format!("prior hash lookup: {e}")))?;
        let hash_unchanged =
            prior_hash.as_ref().map(|(h,)| h.as_str()) == Some(content_hash.as_str());

        sqlx::query(
            r#"
            INSERT INTO knowledge_atoms (
                id, source, source_id, title, summary, content,
                question, resolution, tags_json,
                source_updated_at, indexed_at, metadata_json, content_hash,
                trust_lane, quarantine_reason, last_accessed_at, visibility,
                team_id, soft_labels_json
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            ON CONFLICT (id) DO UPDATE SET
                source = excluded.source,
                source_id = excluded.source_id,
                title = excluded.title,
                summary = excluded.summary,
                content = excluded.content,
                question = excluded.question,
                resolution = excluded.resolution,
                tags_json = excluded.tags_json,
                source_updated_at = excluded.source_updated_at,
                indexed_at = CASE
                    WHEN knowledge_atoms.content_hash = excluded.content_hash
                    THEN knowledge_atoms.indexed_at
                    ELSE excluded.indexed_at
                END,
                metadata_json = excluded.metadata_json,
                content_hash = excluded.content_hash,
                trust_lane = excluded.trust_lane,
                quarantine_reason = excluded.quarantine_reason,
                visibility = excluded.visibility,
                team_id = excluded.team_id,
                soft_labels_json = CASE
                    WHEN excluded.soft_labels_json = '[]' THEN knowledge_atoms.soft_labels_json
                    ELSE excluded.soft_labels_json
                END
            "#,
        )
        .bind(&atom.id)
        .bind(&atom.source)
        .bind(&atom.source_id)
        .bind(&atom.title)
        .bind(&atom.summary)
        .bind(&atom.content)
        .bind(&atom.question)
        .bind(&atom.resolution)
        .bind(&tags_json)
        .bind(atom.source_updated_at.to_rfc3339())
        .bind(atom.indexed_at.to_rfc3339())
        .bind(&metadata_json)
        .bind(&content_hash)
        .bind(atom.trust_lane.as_str())
        .bind(&atom.quarantine_reason)
        .bind(last_accessed.to_rfc3339())
        .bind(atom.visibility.as_str())
        .bind(&team_id)
        .bind(&soft_json)
        .execute(&mut **tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("upsert atom: {e}")))?;

        // Vector: write when a new embedding is provided; preserve existing when
        // content_hash is unchanged and caller skipped re-embed (hash-skip).
        // Quarantine always clears vec (same KTD7 as SQLite).
        if atom.trust_lane == TrustLane::Quarantine {
            sqlx::query("DELETE FROM atoms_vec WHERE atom_id = $1")
                .bind(&atom.id)
                .execute(&mut **tx)
                .await
                .map_err(|e| KurultaiError::Store(format!("clear vec: {e}")))?;
        } else {
            match atom.embedding.as_ref() {
                Some(emb) => {
                    sqlx::query("DELETE FROM atoms_vec WHERE atom_id = $1")
                        .bind(&atom.id)
                        .execute(&mut **tx)
                        .await
                        .map_err(|e| KurultaiError::Store(format!("clear vec: {e}")))?;
                    if emb.len() != embed_dim {
                        return Err(KurultaiError::Store(format!(
                            "embedding dim {} != store embed_dim {embed_dim}",
                            emb.len()
                        )));
                    }
                    if Self::embedding_norm(emb) >= MIN_EMBEDDING_NORM {
                        let vec = Vector::from(emb.clone());
                        sqlx::query("INSERT INTO atoms_vec (atom_id, embedding) VALUES ($1, $2)")
                            .bind(&atom.id)
                            .bind(vec)
                            .execute(&mut **tx)
                            .await
                            .map_err(|e| KurultaiError::Store(format!("insert vec: {e}")))?;
                    }
                }
                None if !hash_unchanged => {
                    sqlx::query("DELETE FROM atoms_vec WHERE atom_id = $1")
                        .bind(&atom.id)
                        .execute(&mut **tx)
                        .await
                        .map_err(|e| KurultaiError::Store(format!("clear vec: {e}")))?;
                }
                None => {}
            }
        }
        Ok(())
    }

    async fn load_by_id(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        let sql = format!("SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("get: {e}")))?;
        row.map(|r| Self::atom_from_row(&r)).transpose()
    }
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

async fn exec_ddl(conn: &mut sqlx::postgres::PgConnection, sql: &str, ctx: &str) -> Result<()> {
    match sqlx::query(sql).execute(&mut *conn).await {
        Ok(_) => Ok(()),
        Err(e) if is_benign_catalog_race(&e) => Ok(()),
        Err(e) => Err(KurultaiError::Store(format!("{ctx}: {e}"))),
    }
}

fn is_benign_catalog_race(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db) => {
            matches!(db.code().as_deref(), Some("23505" | "42710"))
        }
        _ => {
            let msg = err.to_string();
            msg.contains("already exists")
                || msg.contains("pg_extension_name_index")
                || msg.contains("pg_type_typname_nsp_index")
                || msg.contains("pg_class_relname_nsp_index")
        }
    }
}

#[async_trait::async_trait]
impl Store for PostgresStore {
    async fn upsert(&self, atom: &KnowledgeAtom) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| KurultaiError::Store(format!("begin upsert: {e}")))?;
        Self::upsert_in_tx(&mut tx, atom, self.embed_dim).await?;
        tx.commit()
            .await
            .map_err(|e| KurultaiError::Store(format!("commit upsert: {e}")))
    }

    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| KurultaiError::Store(format!("begin batch: {e}")))?;
        for atom in atoms {
            Self::upsert_in_tx(&mut tx, atom, self.embed_dim).await?;
        }
        tx.commit()
            .await
            .map_err(|e| KurultaiError::Store(format!("commit batch: {e}")))
    }

    async fn vector_search(
        &self,
        query_embed: &[f32],
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let ids = self.vector_search_ids(query_embed, limit, filter).await?;
        let mut out = Vec::with_capacity(ids.len());
        for (id, score) in ids {
            if let Some(atom) = self.load_by_id(&id).await? {
                out.push((atom, score));
            }
        }
        Ok(out)
    }

    async fn fts_search(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let ids = self.fts_search_ids(query, limit, filter).await?;
        let mut out = Vec::with_capacity(ids.len());
        for (id, score) in ids {
            if let Some(atom) = self.load_by_id(&id).await? {
                out.push((atom, score));
            }
        }
        Ok(out)
    }

    async fn fts_search_ids(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(String, f64)>> {
        if limit == 0 || query.trim().is_empty() {
            return Ok(vec![]);
        }
        let sql = if filter.trusted_only {
            "SELECT id, ts_rank(search_tsv, plainto_tsquery('english', $1))::float8 AS score
             FROM knowledge_atoms
             WHERE search_tsv @@ plainto_tsquery('english', $1)
               AND trust_lane = 'trusted'
             ORDER BY score DESC
             LIMIT $2"
        } else {
            "SELECT id, ts_rank(search_tsv, plainto_tsquery('english', $1))::float8 AS score
             FROM knowledge_atoms
             WHERE search_tsv @@ plainto_tsquery('english', $1)
             ORDER BY score DESC
             LIMIT $2"
        };
        let rows = sqlx::query(sql)
            .bind(query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("fts_search_ids: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| KurultaiError::Store(e.to_string()))?;
            let score: f64 = row
                .try_get("score")
                .map_err(|e| KurultaiError::Store(e.to_string()))?;
            out.push((id, score));
        }
        Ok(out)
    }

    async fn vector_search_ids(
        &self,
        query_embed: &[f32],
        limit: usize,
        filter: SearchFilter,
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
        if Self::embedding_norm(query_embed) < MIN_EMBEDDING_NORM {
            return Ok(vec![]);
        }
        let k = if filter.trusted_only {
            (limit.saturating_mul(3)).max(limit)
        } else {
            limit
        };
        let vec = Vector::from(query_embed.to_vec());
        // L2 `<->`, score `1/(1+distance)` — same shape as sqlite-vec MATCH distance.
        let rows = sqlx::query(
            "SELECT a.id, (v.embedding <-> $1)::float8 AS distance, a.trust_lane
             FROM atoms_vec v
             JOIN knowledge_atoms a ON a.id = v.atom_id
             ORDER BY v.embedding <-> $1
             LIMIT $2",
        )
        .bind(vec)
        .bind(k as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("vector_search_ids: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| KurultaiError::Store(e.to_string()))?;
            let distance: f64 = row
                .try_get("distance")
                .map_err(|e| KurultaiError::Store(e.to_string()))?;
            let lane: String = row
                .try_get("trust_lane")
                .map_err(|e| KurultaiError::Store(e.to_string()))?;
            if filter.trusted_only && lane != "trusted" {
                continue;
            }
            out.push((id, 1.0 / (1.0 + distance)));
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    async fn get_many(&self, ids: &[String]) -> Result<Vec<KnowledgeAtom>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(atom) = self.load_by_id(id).await? {
                out.push(atom);
            }
        }
        Ok(out)
    }

    async fn get(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        self.load_by_id(id).await
    }

    async fn delete_atom(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM knowledge_atoms WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("delete_atom: {e}")))?;
        Ok(())
    }

    async fn apply_auto_merge(
        &self,
        survivor: &KnowledgeAtom,
        loser_id: &str,
        audit_detail: &serde_json::Value,
    ) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| KurultaiError::Store(format!("begin merge: {e}")))?;
        Self::upsert_in_tx(&mut tx, survivor, self.embed_dim).await?;
        sqlx::query("DELETE FROM knowledge_atoms WHERE id = $1")
            .bind(loser_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("merge delete loser: {e}")))?;
        let detail = audit_detail.to_string();
        sqlx::query(
            "INSERT INTO quality_audit (action, atom_id, actor, detail_json) VALUES ($1,$2,$3,$4)",
        )
        .bind("auto_merge")
        .bind(&survivor.id)
        .bind("near_dupe")
        .bind(&detail)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("merge audit: {e}")))?;
        tx.commit()
            .await
            .map_err(|e| KurultaiError::Store(format!("commit merge: {e}")))
    }

    async fn delete_source(&self, source: &str) -> Result<()> {
        sqlx::query("DELETE FROM knowledge_atoms WHERE source = $1")
            .bind(source)
            .execute(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("delete_source: {e}")))?;
        Ok(())
    }

    async fn count(&self) -> Result<u64> {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM knowledge_atoms")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("count: {e}")))?;
        Ok(n as u64)
    }

    async fn count_by_lane(&self, lane: TrustLane) -> Result<u64> {
        let (n,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM knowledge_atoms WHERE trust_lane = $1")
                .bind(lane.as_str())
                .fetch_one(&self.pool)
                .await
                .map_err(|e| KurultaiError::Store(format!("count_by_lane: {e}")))?;
        Ok(n as u64)
    }

    async fn get_by_source_id(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<Option<KnowledgeAtom>> {
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE source = $1 AND source_id = $2 LIMIT 1"
        );
        let row = sqlx::query(&sql)
            .bind(source)
            .bind(source_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("get_by_source_id: {e}")))?;
        row.map(|r| Self::atom_from_row(&r)).transpose()
    }

    async fn get_by_chunk_meta(
        &self,
        source: &str,
        rel_path: &str,
        chunk_index: u32,
    ) -> Result<Option<KnowledgeAtom>> {
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms
             WHERE source = $1
               AND metadata_json::jsonb->>'rel_path' = $2
               AND (metadata_json::jsonb->>'chunk_index')::int = $3
             LIMIT 1"
        );
        let row = sqlx::query(&sql)
            .bind(source)
            .bind(rel_path)
            .bind(chunk_index as i32)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("get_by_chunk_meta: {e}")))?;
        row.map(|r| Self::atom_from_row(&r)).transpose()
    }

    async fn has_fresh_embedding(&self, id: &str, content_hash: &str) -> Result<bool> {
        let found: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT 1
            FROM knowledge_atoms a
            JOIN atoms_vec v ON v.atom_id = a.id
            WHERE a.id = $1 AND a.content_hash = $2
            LIMIT 1
            "#,
        )
        .bind(id)
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("has_fresh_embedding: {e}")))?;
        Ok(found.is_some())
    }

    async fn list_atoms(&self, limit: usize, filter: SearchFilter) -> Result<Vec<KnowledgeAtom>> {
        let sql = if filter.trusted_only {
            format!(
                "SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE trust_lane = 'trusted'
                 ORDER BY indexed_at DESC LIMIT $1"
            )
        } else {
            format!("SELECT {ATOM_SELECT} FROM knowledge_atoms ORDER BY indexed_at DESC LIMIT $1")
        };
        let rows = sqlx::query(&sql)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("list_atoms: {e}")))?;
        rows.iter().map(Self::atom_from_row).collect()
    }

    async fn find_atoms_by_source_id_patterns(
        &self,
        patterns: &[&str],
    ) -> Result<Vec<KnowledgeAtom>> {
        let mut out = Vec::new();
        for pat in patterns {
            let sql = format!("SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE source_id LIKE $1");
            let rows = sqlx::query(&sql)
                .bind(*pat)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| KurultaiError::Store(format!("find patterns: {e}")))?;
            for row in rows {
                out.push(Self::atom_from_row(&row)?);
            }
        }
        Ok(out)
    }

    async fn find_trusted_by_content_hash(&self, content_hash: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM knowledge_atoms WHERE content_hash = $1 AND trust_lane = 'trusted' LIMIT 1",
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("find_trusted_by_content_hash: {e}")))?;
        Ok(row.map(|r| r.0))
    }

    async fn set_trust_lane(
        &self,
        id: &str,
        lane: TrustLane,
        quarantine_reason: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE knowledge_atoms SET trust_lane = $1, quarantine_reason = $2 WHERE id = $3",
        )
        .bind(lane.as_str())
        .bind(quarantine_reason)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("set_trust_lane: {e}")))?;
        Ok(())
    }

    async fn insert_quality_audit(
        &self,
        action: &str,
        atom_id: &str,
        actor: &str,
        detail: &serde_json::Value,
    ) -> Result<()> {
        let detail_json = detail.to_string();
        sqlx::query(
            "INSERT INTO quality_audit (action, atom_id, actor, detail_json) VALUES ($1,$2,$3,$4)",
        )
        .bind(action)
        .bind(atom_id)
        .bind(actor)
        .bind(&detail_json)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("insert_quality_audit: {e}")))?;
        Ok(())
    }

    async fn insert_merge_candidate(
        &self,
        atom_a: &str,
        atom_b: &str,
        reason: &str,
    ) -> Result<bool> {
        let res = sqlx::query(
            r#"
            INSERT INTO merge_candidates (atom_a, atom_b, reason)
            VALUES ($1,$2,$3)
            ON CONFLICT (atom_a, atom_b) DO NOTHING
            "#,
        )
        .bind(atom_a)
        .bind(atom_b)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("insert_merge_candidate: {e}")))?;
        Ok(res.rows_affected() > 0)
    }

    async fn count_merge_candidates_pending(&self) -> Result<u64> {
        let (n,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM merge_candidates WHERE status = 'pending'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| KurultaiError::Store(format!("count_merge_candidates: {e}")))?;
        Ok(n as u64)
    }

    async fn list_near_dupe_candidates(&self, limit: usize) -> Result<Vec<KnowledgeAtom>> {
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms
             WHERE trust_lane = 'quarantine' OR indexed_at >= $1
             ORDER BY indexed_at DESC LIMIT $2"
        );
        let cutoff = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let rows = sqlx::query(&sql)
            .bind(cutoff)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("list_near_dupe: {e}")))?;
        rows.iter().map(Self::atom_from_row).collect()
    }

    async fn touch_access(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE knowledge_atoms SET last_accessed_at = $1 WHERE id = $2")
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("touch_access: {e}")))?;
        Ok(())
    }

    async fn count_by_tier(&self, policy: TierPolicy) -> Result<(u64, u64, u64)> {
        let sql = format!("SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE trust_lane = 'trusted'");
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("count_by_tier: {e}")))?;
        let now = Utc::now();
        let mut hot = 0u64;
        let mut warm = 0u64;
        let mut cold = 0u64;
        for row in rows {
            let atom = Self::atom_from_row(&row)?;
            match classify(atom.indexed_at, atom.last_accessed_at, now, policy) {
                MemoryTier::Hot => hot += 1,
                MemoryTier::Warm => warm += 1,
                MemoryTier::Cold => cold += 1,
            }
        }
        Ok((hot, warm, cold))
    }

    async fn list_graph_nodes(
        &self,
        tier: Option<MemoryTier>,
        limit: usize,
        filter: SearchFilter,
        policy: TierPolicy,
    ) -> Result<Vec<GraphNode>> {
        let sql = if filter.trusted_only {
            format!(
                "SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE trust_lane = 'trusted'
                 ORDER BY last_accessed_at DESC LIMIT $1"
            )
        } else {
            format!(
                "SELECT {ATOM_SELECT} FROM knowledge_atoms ORDER BY last_accessed_at DESC LIMIT $1"
            )
        };
        let fetch_cap = if tier.is_some() {
            (limit.saturating_mul(8)).max(limit).min(50_000)
        } else {
            limit.min(50_000)
        };
        let rows = sqlx::query(&sql)
            .bind(fetch_cap as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("list_graph_nodes: {e}")))?;
        let now = Utc::now();
        let mut out = Vec::new();
        for row in rows {
            let atom = Self::atom_from_row(&row)?;
            let t = classify(atom.indexed_at, atom.last_accessed_at, now, policy);
            if let Some(want) = tier {
                if t != want {
                    continue;
                }
            }
            out.push(GraphNode::from_atom(&atom, t, t == MemoryTier::Hot));
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    async fn record_ingestion_start(
        &self,
        batch_id: &str,
        source: &str,
        file_path: &str,
    ) -> Result<i64> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO ingestion_jobs (batch_id, source, file_path) VALUES ($1,$2,$3) RETURNING id",
        )
        .bind(batch_id)
        .bind(source)
        .bind(file_path)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("record_ingestion_start: {e}")))?;
        Ok(id)
    }

    async fn record_ingestion_finish(
        &self,
        job_id: i64,
        atoms_count: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let status = if error_message.is_some() {
            "failed"
        } else {
            "completed"
        };
        sqlx::query(
            "UPDATE ingestion_jobs SET status = $1, atoms_count = $2, error_message = $3, completed_at = $4 WHERE id = $5",
        )
        .bind(status)
        .bind(atoms_count)
        .bind(error_message)
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("record_ingestion_finish: {e}")))?;
        Ok(())
    }

    async fn list_pending_ingestion_jobs(&self) -> Result<Vec<IngestionJob>> {
        let rows = sqlx::query(
            "SELECT id, batch_id, source, file_path, status, atoms_count, error_message, created_at, completed_at
             FROM ingestion_jobs WHERE status = 'pending' ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("list_pending_ingestion_jobs: {e}")))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(IngestionJob {
                id: row.try_get::<i64, _>("id").unwrap_or(0),
                batch_id: row.try_get("batch_id").unwrap_or_default(),
                source: row.try_get("source").unwrap_or_default(),
                file_path: row.try_get("file_path").unwrap_or_default(),
                status: row.try_get("status").unwrap_or_default(),
                atoms_count: row.try_get("atoms_count").ok(),
                error_message: row.try_get("error_message").ok(),
                created_at: row.try_get("created_at").unwrap_or_default(),
                completed_at: row.try_get("completed_at").ok(),
            });
        }
        Ok(out)
    }

    async fn upsert_ontology_entity(&self, _e: &OntologyEntity) -> Result<()> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn get_ontology_entity(&self, _id: &str) -> Result<Option<OntologyEntity>> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn list_ontology_entities(&self, _limit: usize) -> Result<Vec<OntologyEntity>> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn upsert_ontology_link(&self, _l: &OntologyLink) -> Result<()> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn list_ontology_links(&self, _endpoint: Option<&str>) -> Result<Vec<OntologyLink>> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn sample_team(id: &str, title: &str, content: &str, emb: Option<Vec<f32>>) -> KnowledgeAtom {
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
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: emb,
            visibility: VisibilityScope::Team,
            ..Default::default()
        }
    }

    async fn test_store() -> Option<PostgresStore> {
        match std::env::var("KURULTAI_TEST_DATABASE_URL") {
            Ok(url) if !url.is_empty() => Some(
                PostgresStore::connect(&url, 4)
                    .await
                    .expect("connect postgres"),
            ),
            _ if std::env::var("CI").is_ok() => {
                panic!(
                    "KURULTAI_TEST_DATABASE_URL must be set in CI for --features postgres tests"
                );
            }
            _ => None,
        }
    }

    #[tokio::test]
    async fn personal_upsert_is_rejected_and_writes_nothing() {
        let Some(store) = test_store().await else {
            return;
        };
        let before = store.count().await.unwrap();
        let pid = format!("p-ae4-{}", uuid::Uuid::new_v4());
        let mut personal = sample_team(&pid, "Personal", "must not land in hub", None);
        personal.visibility = VisibilityScope::Personal;
        let err = store.upsert(&personal).await.unwrap_err();
        assert!(
            err.to_string().contains("AE4") || err.to_string().contains("personal"),
            "{err}"
        );
        assert_eq!(store.count().await.unwrap(), before);
        assert!(store.get(&pid).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn team_round_trip_fts_and_vector() {
        let Some(store) = test_store().await else {
            return;
        };
        let id = format!("hub-{}", uuid::Uuid::new_v4());
        let atom = sample_team(
            &id,
            "Hub welcome",
            "unique-hub-fts-token shared notes",
            Some(vec![0.1, 0.2, 0.3, 0.4]),
        );
        store.upsert(&atom).await.unwrap();
        let loaded = store.get(&id).await.unwrap().unwrap();
        assert_eq!(loaded.visibility, VisibilityScope::Team);
        assert_eq!(loaded.title, "Hub welcome");

        let fts = store
            .fts_search("unique-hub-fts-token", 10, SearchFilter::default())
            .await
            .unwrap();
        assert!(
            fts.iter().any(|(a, _)| a.id == id),
            "fts missed atom: {:?}",
            fts.iter().map(|(a, _)| &a.id).collect::<Vec<_>>()
        );

        let knn = store
            .vector_search(&[0.1, 0.2, 0.3, 0.4], 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(knn.iter().any(|(a, _)| a.id == id));

        store.delete_atom(&id).await.unwrap();
    }

    #[tokio::test]
    async fn zero_vector_not_indexed() {
        let Some(store) = test_store().await else {
            return;
        };
        let id = format!("hub-zero-{}", uuid::Uuid::new_v4());
        let atom = sample_team(&id, "Zero vec", "zero vector body text", Some(vec![0.0; 4]));
        store.upsert(&atom).await.unwrap();
        assert!(!store
            .has_fresh_embedding(&id, &sha256_hex(&atom.content))
            .await
            .unwrap());
        store.delete_atom(&id).await.unwrap();
    }

    #[tokio::test]
    async fn company_round_trip() {
        let Some(store) = test_store().await else {
            return;
        };
        let id = format!("hub-co-{}", uuid::Uuid::new_v4());
        let mut atom = sample_team(&id, "Company memo", "company-wide shared atom", None);
        atom.visibility = VisibilityScope::Company;
        store.upsert(&atom).await.unwrap();
        let loaded = store.get(&id).await.unwrap().unwrap();
        assert_eq!(loaded.visibility, VisibilityScope::Company);
        store.delete_atom(&id).await.unwrap();
    }

    #[tokio::test]
    async fn trusted_only_skips_quarantine_fts() {
        let Some(store) = test_store().await else {
            return;
        };
        let id = format!("hub-q-{}", uuid::Uuid::new_v4());
        let mut atom = sample_team(
            &id,
            "Quarantine secret",
            "quarantine-fts-token must not leak",
            None,
        );
        atom.trust_lane = TrustLane::Quarantine;
        store.upsert(&atom).await.unwrap();
        let trusted = store
            .fts_search("quarantine-fts-token", 10, SearchFilter::default())
            .await
            .unwrap();
        assert!(!trusted.iter().any(|(a, _)| a.id == id));
        let all = store
            .fts_search(
                "quarantine-fts-token",
                10,
                SearchFilter {
                    trusted_only: false,
                },
            )
            .await
            .unwrap();
        assert!(all.iter().any(|(a, _)| a.id == id));
        store.delete_atom(&id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_source_removes_atoms() {
        let Some(store) = test_store().await else {
            return;
        };
        let src = format!("src-{}", uuid::Uuid::new_v4());
        let id = format!("hub-src-{}", uuid::Uuid::new_v4());
        let mut atom = sample_team(&id, "Source atom", "delete-source body", None);
        atom.source = src.clone();
        store.upsert(&atom).await.unwrap();
        store.delete_source(&src).await.unwrap();
        assert!(store.get(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn apply_auto_merge_keeps_survivor_and_drops_loser() {
        let Some(store) = test_store().await else {
            return;
        };
        let survivor_id = format!("hub-surv-{}", uuid::Uuid::new_v4());
        let loser_id = format!("hub-lose-{}", uuid::Uuid::new_v4());
        let survivor = sample_team(&survivor_id, "Survivor", "keep this atom", None);
        let loser = sample_team(&loser_id, "Loser", "drop this atom", None);
        store.upsert(&survivor).await.unwrap();
        store.upsert(&loser).await.unwrap();
        store
            .apply_auto_merge(&survivor, &loser_id, &serde_json::json!({"reason": "test"}))
            .await
            .unwrap();
        assert!(store.get(&survivor_id).await.unwrap().is_some());
        assert!(store.get(&loser_id).await.unwrap().is_none());
        store.delete_atom(&survivor_id).await.unwrap();
    }

    #[tokio::test]
    async fn hash_skip_preserves_vector_when_embedding_omitted() {
        let Some(store) = test_store().await else {
            return;
        };
        let id = format!("hub-hash-{}", uuid::Uuid::new_v4());
        let mut atom = sample_team(
            &id,
            "Hash skip",
            "stable hub content",
            Some(vec![0.5, 0.5, 0.5, 0.5]),
        );
        store.upsert(&atom).await.unwrap();
        let hash = sha256_hex(&atom.content);
        assert!(store.has_fresh_embedding(&id, &hash).await.unwrap());
        atom.embedding = None;
        store.upsert(&atom).await.unwrap();
        assert!(store.has_fresh_embedding(&id, &hash).await.unwrap());
        store.delete_atom(&id).await.unwrap();
    }

    #[tokio::test]
    async fn open_hub_store_connects_when_flag_and_url_set() {
        let Some(_) = test_store().await else {
            return;
        };
        let url = std::env::var("KURULTAI_TEST_DATABASE_URL").unwrap();
        let prev = std::env::var("KURULTAI_FEATURE_HUB").ok();
        std::env::set_var("KURULTAI_FEATURE_HUB", "1");
        let store = crate::store::open_hub_store(&url, 4)
            .await
            .expect("open hub");
        assert!(store.count().await.is_ok());
        match prev {
            Some(v) => std::env::set_var("KURULTAI_FEATURE_HUB", v),
            None => std::env::remove_var("KURULTAI_FEATURE_HUB"),
        }
    }

    #[tokio::test]
    async fn ingestion_job_round_trip() {
        let Some(store) = test_store().await else {
            return;
        };
        let batch = format!("batch-{}", uuid::Uuid::new_v4());
        let id = store
            .record_ingestion_start(&batch, "markdown", "/tmp/x.md")
            .await
            .unwrap();
        let pending = store.list_pending_ingestion_jobs().await.unwrap();
        assert!(pending.iter().any(|j| j.id == id));
        store
            .record_ingestion_finish(id, Some(1), None)
            .await
            .unwrap();
        let pending = store.list_pending_ingestion_jobs().await.unwrap();
        assert!(!pending.iter().any(|j| j.id == id));
    }
}
