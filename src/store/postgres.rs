//! Postgres + pgvector `Store` for shared (`team` / `company`) hub atoms (HUB-2).
//!
//! Solo [`super::open_store`] stays SQLite. Personal atoms are refused (AE4).

use super::scope::PgScope;
use super::{DeviceFlow, IngestionJob, SearchFilter, Store, DEFAULT_PROJECT, MIN_EMBEDDING_NORM};
use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::hub::HubActivityStore;
use crate::memory::{classify_atom, GraphNode, MemoryTier, TierPolicy};
use crate::types::{
    normalize_soft_labels, CorpusTier, KnowledgeAtom, OntologyEntity, OntologyLink,
    OntologyProposal, TrustLane, VisibilityScope,
};
use chrono::{DateTime, Utc};
use pgvector::Vector;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Row, Transaction};

const ATOM_SELECT: &str = "id, source, source_id, title, summary, content, question, resolution, \
     tags_json, source_updated_at, indexed_at, metadata_json, trust_lane, quarantine_reason, \
     last_accessed_at, visibility, team_id, content_hash, soft_labels_json, \
     corpus_tier, visibility_labels_json";

/// `hub_device_flows` read shape — mirrors SQLite `device_flows` plus the
/// U3 connect-join columns (`team_hint`, `team_id`, `issued_key`).
const DEVICE_FLOW_SELECT: &str = "id, device_code, user_code, codename, instance_id, client_id, \
     status, created_at, expires_at, approved_at, approved_by, agent_id, token_hash, \
     team_hint, team_id, issued_key";

/// KTD1 path classification for scope resolution.
///
/// * `Request` — resolve the task-local [`PgScope`] installed by HTTP
///   middleware: member `HubPrincipal` → `kurultai.team_id`; `/api/hub/*`
///   under operator credentials → `is_admin`; no principal → fail closed
///   (company-only). With no request scope at all the caller is internal
///   (pipeline, CLI, `/ingest`, stdio MCP) → [`PgScope::INTERNAL`].
/// * `Internal` — always [`PgScope::INTERNAL`]: maintenance writes that
///   must work regardless of ambient request scope (`touch_access` on a
///   member-visible company row, merges, deletes, audit, ingestion jobs).
#[derive(Clone, Copy)]
enum PathClass {
    Request,
    Internal,
}

/// Bind a [`PgScope`] to a transaction via `SELECT set_config(…, true)` —
/// `SET`/`SET LOCAL` cannot take bind parameters (KTD1), so `team_id` is
/// bound through `set_config` and `is_admin` is only ever the literal `'on'`.
/// `set_config(…, true)` is transaction-local: nothing survives commit, so
/// the next pool checkout always starts scope-free.
pub(crate) async fn apply_pg_scope(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    scope: &PgScope,
) -> Result<()> {
    match (scope.team_id.as_deref(), scope.is_admin) {
        (Some(team), true) => {
            sqlx::query(
                "SELECT set_config('kurultai.team_id', $1, true), \
                 set_config('kurultai.is_admin', 'on', true)",
            )
            .bind(team)
            .execute(&mut **tx)
            .await
        }
        (Some(team), false) => {
            sqlx::query("SELECT set_config('kurultai.team_id', $1, true)")
                .bind(team)
                .execute(&mut **tx)
                .await
        }
        (None, true) => {
            sqlx::query("SELECT set_config('kurultai.is_admin', 'on', true)")
                .execute(&mut **tx)
                .await
        }
        (None, false) => return Ok(()),
    }
    .map_err(|e| KurultaiError::Store(format!("rls scope: {e}")))?;
    Ok(())
}

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
        // KTD8: the hub runtime role (`kurultai_app`) has no DDL rights — with
        // `KURULTAI_PG_SKIP_MIGRATE=1` the pool skips the eager `migrate()`
        // entirely. Migrations then run out-of-band under
        // `KURULTAI_MIGRATE_DATABASE_URL` (`kurultai_migrate`, BYPASSRLS):
        // `kurultai hub …` CLI and `init-hub-db.sql` run without the env set,
        // so they still migrate.
        if !skip_migrate_enabled() {
            store.migrate().await?;
        }
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
                soft_labels_json TEXT NOT NULL DEFAULT '[]',
                corpus_tier TEXT NOT NULL DEFAULT 'public',
                visibility_labels_json TEXT NOT NULL DEFAULT '[]'
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

        exec_ddl(
            conn,
            "ALTER TABLE knowledge_atoms ADD COLUMN IF NOT EXISTS corpus_tier TEXT NOT NULL DEFAULT 'public'",
            "corpus_tier column",
        )
        .await?;
        exec_ddl(
            conn,
            "ALTER TABLE knowledge_atoms ADD COLUMN IF NOT EXISTS visibility_labels_json TEXT NOT NULL DEFAULT '[]'",
            "visibility_labels_json column",
        )
        .await?;
        exec_ddl(
            conn,
            "ALTER TABLE knowledge_atoms ADD COLUMN IF NOT EXISTS superseded_at TEXT",
            "superseded_at column",
        )
        .await?;
        exec_ddl(
            conn,
            "ALTER TABLE knowledge_atoms ADD COLUMN IF NOT EXISTS superseded_by TEXT",
            "superseded_by column",
        )
        .await?;
        exec_ddl(
            conn,
            "CREATE INDEX IF NOT EXISTS idx_hub_atoms_superseded ON knowledge_atoms(superseded_at)",
            "idx superseded",
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

        exec_ddl(
            conn,
            r#"
            CREATE TABLE IF NOT EXISTS hub_api_keys (
                id BIGSERIAL PRIMARY KEY,
                key_hash TEXT NOT NULL UNIQUE,
                key_prefix TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                team_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                revoked_at TEXT
            )
            "#,
            "hub_api_keys",
        )
        .await?;
        exec_ddl(
            conn,
            "CREATE INDEX IF NOT EXISTS idx_hub_api_keys_hash ON hub_api_keys(key_hash)",
            "idx hub_api_keys hash",
        )
        .await?;

        HubActivityStore::migrate_conn(conn).await?;

        // U3/KTD3 — `hub_device_flows` (connect-join). Deliberately outside
        // `PG_HUB_RLS`: `/api/device/*` + `/connect` run before any
        // principal/GUC exists (same verdict as `hub_api_keys`).
        for (sql, ctx) in super::migrations::PG_HUB_DEVICE_FLOWS {
            exec_ddl(conn, sql, ctx).await?;
        }

        // R11/KTD1 — ENABLE + FORCE row-level security and the GUC-keyed scope
        // policies (defense-in-depth under `hub_visibility_sql`). Statement
        // bodies live with the rest of the schema DDL in
        // `migrations::PG_HUB_RLS`; requires table ownership — the migrate
        // path (KTD8) is the owner, the runtime `kurultai_app` role is not.
        for (sql, ctx) in super::migrations::PG_HUB_RLS {
            exec_ddl(conn, sql, ctx).await?;
        }

        Ok(())
    }

    /// Open a transaction with the KTD1 scope applied — every PostgresStore
    /// statement runs inside one of these so transaction-local GUCs die with
    /// the tx and can never bleed into the next pool checkout (AE5).
    async fn begin_scope(&self, class: PathClass) -> Result<Transaction<'static, sqlx::Postgres>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| KurultaiError::Store(format!("begin tx: {e}")))?;
        let scope = match class {
            PathClass::Request => PgScope::current(),
            PathClass::Internal => PgScope::INTERNAL,
        };
        apply_pg_scope(&mut tx, &scope).await?;
        Ok(tx)
    }

    async fn commit_scope(tx: Transaction<'static, sqlx::Postgres>, ctx: &str) -> Result<()> {
        tx.commit()
            .await
            .map_err(|e| KurultaiError::Store(format!("commit {ctx}: {e}")))
    }

    fn hub_visibility_sql(
        filter: &SearchFilter,
        start_bind: usize,
    ) -> (String, Vec<String>, usize) {
        let mut sql = String::new();
        let mut binds = Vec::new();
        let mut idx = start_bind;
        if let Some(team) = filter.hub_team_scope() {
            sql.push_str(&format!(
                " AND (visibility = 'company' OR (visibility = 'team' AND team_id = ${idx}))"
            ));
            binds.push(team.to_string());
            idx += 1;
        }
        (sql, binds, idx)
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
        let corpus_tier_raw: String = row
            .try_get("corpus_tier")
            .unwrap_or_else(|_| "public".into());
        let visibility_labels_json: String = row
            .try_get("visibility_labels_json")
            .unwrap_or_else(|_| "[]".into());
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
            corpus_tier: CorpusTier::parse(&corpus_tier_raw),
            visibility_labels: serde_json::from_str(&visibility_labels_json).unwrap_or_default(),
            visibility: VisibilityScope::parse(&visibility_raw),
            superseded_at: row
                .try_get::<Option<String>, _>("superseded_at")
                .unwrap_or(None)
                .as_deref()
                .map(parse_dt),
            superseded_by: row.try_get("superseded_by").unwrap_or(None),
        })
    }

    fn device_flow_from_row(row: &PgRow) -> Result<DeviceFlow> {
        Ok(DeviceFlow {
            id: row
                .try_get("id")
                .map_err(|e| KurultaiError::Store(format!("flow id: {e}")))?,
            device_code: row
                .try_get("device_code")
                .map_err(|e| KurultaiError::Store(format!("flow device_code: {e}")))?,
            user_code: row
                .try_get("user_code")
                .map_err(|e| KurultaiError::Store(format!("flow user_code: {e}")))?,
            codename: row
                .try_get("codename")
                .map_err(|e| KurultaiError::Store(format!("flow codename: {e}")))?,
            instance_id: row
                .try_get("instance_id")
                .map_err(|e| KurultaiError::Store(format!("flow instance_id: {e}")))?,
            client_id: row
                .try_get("client_id")
                .map_err(|e| KurultaiError::Store(format!("flow client_id: {e}")))?,
            status: row
                .try_get("status")
                .map_err(|e| KurultaiError::Store(format!("flow status: {e}")))?,
            created_at: row
                .try_get("created_at")
                .map_err(|e| KurultaiError::Store(format!("flow created_at: {e}")))?,
            expires_at: row
                .try_get("expires_at")
                .map_err(|e| KurultaiError::Store(format!("flow expires_at: {e}")))?,
            approved_at: row.try_get("approved_at").unwrap_or(None),
            approved_by: row.try_get("approved_by").unwrap_or(None),
            agent_id: row.try_get("agent_id").unwrap_or(None),
            token_hash: row.try_get("token_hash").unwrap_or(None),
            team_hint: row.try_get("team_hint").unwrap_or(None),
            team_id: row.try_get("team_id").unwrap_or(None),
            issued_key: row.try_get("issued_key").unwrap_or(None),
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
        let visibility_labels_json = serde_json::to_string(&atom.visibility_labels)
            .map_err(|e| KurultaiError::Store(format!("visibility_labels serialize: {e}")))?;
        let corpus_tier = atom.corpus_tier.as_str();
        let content_hash = sha256_hex(&atom.content);
        let last_accessed = if atom.last_accessed_at.timestamp() == 0 {
            atom.indexed_at
        } else {
            atom.last_accessed_at
        };
        let team_id = atom.metadata.get("team_id").cloned();
        if atom.visibility == VisibilityScope::Team && team_id.is_none() {
            return Err(KurultaiError::Store(format!(
                "team-scoped atom {} missing team_id metadata",
                atom.id
            )));
        }
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
                team_id, soft_labels_json, corpus_tier, visibility_labels_json
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
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
                END,
                corpus_tier = excluded.corpus_tier,
                visibility_labels_json = excluded.visibility_labels_json
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
        .bind(corpus_tier)
        .bind(&visibility_labels_json)
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

    async fn load_by_id_tx(
        tx: &mut Transaction<'_, sqlx::Postgres>,
        id: &str,
    ) -> Result<Option<KnowledgeAtom>> {
        let sql = format!("SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("get: {e}")))?;
        row.map(|r| Self::atom_from_row(&r)).transpose()
    }

    /// `fts_search_ids` body on an already-scoped transaction (KTD1).
    async fn fts_search_ids_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(String, f64)>> {
        if limit == 0 || query.trim().is_empty() {
            return Ok(vec![]);
        }
        // Predicates in SQL so project scoping precedes LIMIT (see sqlite impl).
        let mut predicates = String::new();
        let mut extra_binds: Vec<String> = Vec::new();
        let mut bind_idx = 3usize;
        if filter.trusted_only {
            predicates.push_str(" AND trust_lane = 'trusted'");
        }
        if let Some(p) = filter.project_scope() {
            predicates.push_str(&format!(
                " AND COALESCE(CAST(metadata_json AS jsonb)->>'project_id', '{DEFAULT_PROJECT}') = ${bind_idx}"
            ));
            extra_binds.push(p.to_string());
            bind_idx += 1;
        }
        let (vis_sql, vis_binds, next_idx) = Self::hub_visibility_sql(&filter, bind_idx);
        predicates.push_str(&vis_sql);
        extra_binds.extend(vis_binds);
        bind_idx = next_idx;
        if let Some(t) = filter.as_of {
            predicates.push_str(&format!(
                " AND indexed_at <= ${bind_idx} AND (superseded_at IS NULL OR superseded_at > ${bind_idx})"
            ));
            extra_binds.push(t.to_rfc3339());
        } else if !filter.include_superseded {
            predicates.push_str(" AND superseded_at IS NULL");
        }
        let sql = format!(
            "SELECT id, ts_rank(search_tsv, plainto_tsquery('english', $1))::float8 AS score
             FROM knowledge_atoms
             WHERE search_tsv @@ plainto_tsquery('english', $1){predicates}
             ORDER BY score DESC
             LIMIT $2"
        );
        let mut q = sqlx::query(&sql).bind(query).bind(limit as i64);
        for b in &extra_binds {
            q = q.bind(b);
        }
        let rows = q
            .fetch_all(&mut **tx)
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

    /// `vector_search_ids` body on an already-scoped transaction (KTD1).
    async fn vector_search_ids_tx(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
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
        // Superseded rows excluded by default — over-fetch like the lane filter.
        let k = if filter.trusted_only || filter.as_of.is_some() || !filter.include_superseded {
            (limit.saturating_mul(3)).max(limit)
        } else {
            limit
        };
        let mut extra_binds: Vec<String> = Vec::new();
        let mut bind_idx = 3usize;
        let mut where_parts = Vec::new();
        if let Some(p) = filter.project_scope() {
            where_parts.push(format!(
                "COALESCE(CAST(a.metadata_json AS jsonb)->>'project_id', '{DEFAULT_PROJECT}') = ${bind_idx}"
            ));
            extra_binds.push(p.to_string());
            bind_idx += 1;
        }
        let (vis_sql, vis_binds, next_idx) = Self::hub_visibility_sql(&filter, bind_idx);
        if !vis_sql.is_empty() {
            where_parts.push(vis_sql.trim_start_matches(" AND ").to_string());
            extra_binds.extend(vis_binds);
        }
        bind_idx = next_idx;
        if let Some(t) = filter.as_of {
            where_parts.push(format!(
                "a.indexed_at <= ${bind_idx} AND (a.superseded_at IS NULL OR a.superseded_at > ${bind_idx})"
            ));
            extra_binds.push(t.to_rfc3339());
        } else if !filter.include_superseded {
            where_parts.push("a.superseded_at IS NULL".to_string());
        }
        let project_pred = if where_parts.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_parts.join(" AND "))
        };
        let vec = Vector::from(query_embed.to_vec());
        // L2 `<->`, score `1/(1+distance)` — same shape as sqlite-vec MATCH distance.
        let sql = format!(
            "SELECT a.id, (v.embedding <-> $1)::float8 AS distance, a.trust_lane
             FROM atoms_vec v
             JOIN knowledge_atoms a ON a.id = v.atom_id{project_pred}
             ORDER BY v.embedding <-> $1
             LIMIT $2"
        );
        let mut q = sqlx::query(&sql).bind(vec).bind(k as i64);
        for b in &extra_binds {
            q = q.bind(b);
        }
        let rows = q
            .fetch_all(&mut **tx)
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
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// `KURULTAI_PG_SKIP_MIGRATE=1` marks a runtime-role (`kurultai_app`)
/// connection — no DDL rights, so `connect` must not attempt `migrate()`
/// (KTD8). Shared by `PostgresStore::connect` and `HubKeyStore::connect`.
pub(crate) fn skip_migrate_enabled() -> bool {
    std::env::var("KURULTAI_PG_SKIP_MIGRATE")
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "on" | "yes"
            )
        })
        .unwrap_or(false)
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
        let mut tx = self.begin_scope(PathClass::Request).await?;
        Self::upsert_in_tx(&mut tx, atom, self.embed_dim).await?;
        Self::commit_scope(tx, "upsert").await
    }

    async fn upsert_batch(&self, atoms: &[KnowledgeAtom]) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        for atom in atoms {
            Self::upsert_in_tx(&mut tx, atom, self.embed_dim).await?;
        }
        Self::commit_scope(tx, "batch").await
    }

    async fn vector_search(
        &self,
        query_embed: &[f32],
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let ids = self
            .vector_search_ids_tx(&mut tx, query_embed, limit, filter)
            .await?;
        let mut out = Vec::with_capacity(ids.len());
        for (id, score) in ids {
            if let Some(atom) = Self::load_by_id_tx(&mut tx, &id).await? {
                out.push((atom, score));
            }
        }
        Self::commit_scope(tx, "vector_search").await?;
        Ok(out)
    }

    async fn fts_search(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let ids = self
            .fts_search_ids_tx(&mut tx, query, limit, filter)
            .await?;
        let mut out = Vec::with_capacity(ids.len());
        for (id, score) in ids {
            if let Some(atom) = Self::load_by_id_tx(&mut tx, &id).await? {
                out.push((atom, score));
            }
        }
        Self::commit_scope(tx, "fts_search").await?;
        Ok(out)
    }

    async fn fts_search_ids(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(String, f64)>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let out = self
            .fts_search_ids_tx(&mut tx, query, limit, filter)
            .await?;
        Self::commit_scope(tx, "fts_search_ids").await?;
        Ok(out)
    }

    async fn vector_search_ids(
        &self,
        query_embed: &[f32],
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(String, f64)>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let out = self
            .vector_search_ids_tx(&mut tx, query_embed, limit, filter)
            .await?;
        Self::commit_scope(tx, "vector_search_ids").await?;
        Ok(out)
    }

    async fn get_many(&self, ids: &[String]) -> Result<Vec<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(atom) = Self::load_by_id_tx(&mut tx, id).await? {
                out.push(atom);
            }
        }
        Self::commit_scope(tx, "get_many").await?;
        Ok(out)
    }

    async fn get(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let out = Self::load_by_id_tx(&mut tx, id).await?;
        Self::commit_scope(tx, "get").await?;
        Ok(out)
    }

    async fn delete_atom(&self, id: &str) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        sqlx::query("DELETE FROM knowledge_atoms WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("delete_atom: {e}")))?;
        Self::commit_scope(tx, "delete_atom").await
    }

    async fn mark_superseded(&self, ids: &[String], by_id: &str, ts: DateTime<Utc>) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let res = sqlx::query(
            "UPDATE knowledge_atoms SET superseded_at = $1, superseded_by = $2
             WHERE id = ANY($3) AND superseded_at IS NULL",
        )
        .bind(ts.to_rfc3339())
        .bind(by_id)
        .bind(ids)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("mark_superseded: {e}")))?;
        let n = res.rows_affected();
        Self::commit_scope(tx, "mark_superseded").await?;
        Ok(n)
    }

    async fn duplicate_content_groups(&self) -> Result<Vec<(String, Vec<String>)>> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT content_hash, id FROM knowledge_atoms
             WHERE trust_lane = 'trusted' AND superseded_at IS NULL
               AND content_hash IS NOT NULL AND content_hash != ''
               AND content_hash IN (
                   SELECT content_hash FROM knowledge_atoms
                   WHERE trust_lane = 'trusted' AND superseded_at IS NULL
                     AND content_hash IS NOT NULL AND content_hash != ''
                   GROUP BY content_hash HAVING COUNT(*) > 1
               )
             ORDER BY content_hash, indexed_at",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("dup_groups: {e}")))?;
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for (hash, id) in rows {
            match groups.last_mut() {
                Some((h, ids)) if *h == hash => ids.push(id),
                _ => groups.push((hash, vec![id])),
            }
        }
        Self::commit_scope(tx, "dup_groups").await?;
        Ok(groups)
    }

    async fn prune_stale_ontology(&self) -> Result<(u64, u64)> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let links = sqlx::query(
            "DELETE FROM ontology_links
             WHERE from_id NOT IN (SELECT id FROM ontology_entities)
                OR to_id NOT IN (SELECT id FROM ontology_entities)",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("prune links: {e}")))?
        .rows_affected();
        let entities = sqlx::query(
            "DELETE FROM ontology_entities
             WHERE atom_id IS NOT NULL
               AND atom_id NOT IN (SELECT id FROM knowledge_atoms)",
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("prune entities: {e}")))?
        .rows_affected();
        Self::commit_scope(tx, "prune_stale_ontology").await?;
        Ok((links, entities))
    }

    async fn apply_auto_merge(
        &self,
        survivor: &KnowledgeAtom,
        loser_id: &str,
        audit_detail: &serde_json::Value,
    ) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
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
        Self::commit_scope(tx, "merge").await
    }

    async fn delete_source(&self, source: &str) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        sqlx::query("DELETE FROM knowledge_atoms WHERE source = $1")
            .bind(source)
            .execute(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("delete_source: {e}")))?;
        Self::commit_scope(tx, "delete_source").await
    }

    async fn count(&self) -> Result<u64> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM knowledge_atoms")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("count: {e}")))?;
        Self::commit_scope(tx, "count").await?;
        Ok(n as u64)
    }

    async fn count_by_lane(&self, lane: TrustLane) -> Result<u64> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let (n,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM knowledge_atoms WHERE trust_lane = $1")
                .bind(lane.as_str())
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| KurultaiError::Store(format!("count_by_lane: {e}")))?;
        Self::commit_scope(tx, "count_by_lane").await?;
        Ok(n as u64)
    }

    async fn get_by_source_id(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<Option<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE source = $1 AND source_id = $2 LIMIT 1"
        );
        let row = sqlx::query(&sql)
            .bind(source)
            .bind(source_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("get_by_source_id: {e}")))?;
        let out = row.map(|r| Self::atom_from_row(&r)).transpose();
        Self::commit_scope(tx, "get_by_source_id").await?;
        out
    }

    async fn get_by_chunk_meta(
        &self,
        source: &str,
        rel_path: &str,
        chunk_index: u32,
    ) -> Result<Option<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms
             WHERE source = $1
               AND CAST(metadata_json AS jsonb)->>'rel_path' = $2
               AND (CAST(metadata_json AS jsonb)->>'chunk_index')::int = $3
             LIMIT 1"
        );
        let row = sqlx::query(&sql)
            .bind(source)
            .bind(rel_path)
            .bind(chunk_index as i32)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("get_by_chunk_meta: {e}")))?;
        let out = row.map(|r| Self::atom_from_row(&r)).transpose();
        Self::commit_scope(tx, "get_by_chunk_meta").await?;
        out
    }

    async fn has_fresh_embedding(&self, id: &str, content_hash: &str) -> Result<bool> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
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
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("has_fresh_embedding: {e}")))?;
        Self::commit_scope(tx, "has_fresh_embedding").await?;
        Ok(found.is_some())
    }

    async fn list_atoms(&self, limit: usize, filter: SearchFilter) -> Result<Vec<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let mut predicates = String::new();
        let mut extra_binds: Vec<String> = Vec::new();
        let bind_idx = 2usize;
        if filter.trusted_only {
            predicates.push_str(" WHERE trust_lane = 'trusted'");
        }
        let (vis_sql, vis_binds, _) = Self::hub_visibility_sql(&filter, bind_idx);
        if !vis_sql.is_empty() {
            if predicates.is_empty() {
                predicates.push_str(" WHERE ");
                predicates.push_str(vis_sql.trim_start_matches(" AND "));
            } else {
                predicates.push_str(&vis_sql);
            }
            extra_binds.extend(vis_binds);
        }
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms{predicates}
             ORDER BY indexed_at DESC LIMIT $1"
        );
        let mut q = sqlx::query(&sql).bind(limit as i64);
        for b in &extra_binds {
            q = q.bind(b);
        }
        let rows = q
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("list_atoms: {e}")))?;
        let out: Result<Vec<KnowledgeAtom>> = rows.iter().map(Self::atom_from_row).collect();
        let atoms = out?;
        Self::commit_scope(tx, "list_atoms").await?;
        Ok(atoms)
    }

    async fn find_atoms_by_source_id_patterns(
        &self,
        patterns: &[&str],
    ) -> Result<Vec<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let mut out = Vec::new();
        for pat in patterns {
            let sql = format!("SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE source_id LIKE $1");
            let rows = sqlx::query(&sql)
                .bind(*pat)
                .fetch_all(&mut *tx)
                .await
                .map_err(|e| KurultaiError::Store(format!("find patterns: {e}")))?;
            for row in rows {
                out.push(Self::atom_from_row(&row)?);
            }
        }
        Self::commit_scope(tx, "find_patterns").await?;
        Ok(out)
    }

    async fn find_trusted_by_content_hash(&self, content_hash: &str) -> Result<Option<String>> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM knowledge_atoms WHERE content_hash = $1 AND trust_lane = 'trusted' LIMIT 1",
        )
        .bind(content_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("find_trusted_by_content_hash: {e}")))?;
        Self::commit_scope(tx, "find_trusted_by_content_hash").await?;
        Ok(row.map(|r| r.0))
    }

    async fn set_trust_lane(
        &self,
        id: &str,
        lane: TrustLane,
        quarantine_reason: Option<&str>,
    ) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        sqlx::query(
            "UPDATE knowledge_atoms SET trust_lane = $1, quarantine_reason = $2 WHERE id = $3",
        )
        .bind(lane.as_str())
        .bind(quarantine_reason)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("set_trust_lane: {e}")))?;
        Self::commit_scope(tx, "set_trust_lane").await
    }

    async fn insert_quality_audit(
        &self,
        action: &str,
        atom_id: &str,
        actor: &str,
        detail: &serde_json::Value,
    ) -> Result<()> {
        let detail_json = detail.to_string();
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        sqlx::query(
            "INSERT INTO quality_audit (action, atom_id, actor, detail_json) VALUES ($1,$2,$3,$4)",
        )
        .bind(action)
        .bind(atom_id)
        .bind(actor)
        .bind(&detail_json)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("insert_quality_audit: {e}")))?;
        Self::commit_scope(tx, "insert_quality_audit").await
    }

    async fn insert_merge_candidate(
        &self,
        atom_a: &str,
        atom_b: &str,
        reason: &str,
    ) -> Result<bool> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
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
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("insert_merge_candidate: {e}")))?;
        let inserted = res.rows_affected() > 0;
        Self::commit_scope(tx, "insert_merge_candidate").await?;
        Ok(inserted)
    }

    async fn count_merge_candidates_pending(&self) -> Result<u64> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let (n,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM merge_candidates WHERE status = 'pending'")
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| KurultaiError::Store(format!("count_merge_candidates: {e}")))?;
        Self::commit_scope(tx, "count_merge_candidates").await?;
        Ok(n as u64)
    }

    async fn list_near_dupe_candidates(&self, limit: usize) -> Result<Vec<KnowledgeAtom>> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms
             WHERE trust_lane = 'quarantine' OR indexed_at >= $1
             ORDER BY indexed_at DESC LIMIT $2"
        );
        let cutoff = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let rows = sqlx::query(&sql)
            .bind(cutoff)
            .bind(limit as i64)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("list_near_dupe: {e}")))?;
        let atoms: Result<Vec<KnowledgeAtom>> = rows.iter().map(Self::atom_from_row).collect();
        let atoms = atoms?;
        Self::commit_scope(tx, "list_near_dupe").await?;
        Ok(atoms)
    }

    async fn touch_access(&self, id: &str) -> Result<()> {
        // Internal (KTD1): a member reading a `company` atom must still bump
        // `last_accessed_at`; the member UPDATE policy only permits own-team
        // writes, so this maintenance write runs admin-scoped.
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        sqlx::query("UPDATE knowledge_atoms SET last_accessed_at = $1 WHERE id = $2")
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("touch_access: {e}")))?;
        Self::commit_scope(tx, "touch_access").await
    }

    async fn count_by_tier(&self, policy: TierPolicy) -> Result<(u64, u64, u64)> {
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let sql = format!("SELECT {ATOM_SELECT} FROM knowledge_atoms WHERE trust_lane = 'trusted'");
        let rows = sqlx::query(&sql)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("count_by_tier: {e}")))?;
        let now = Utc::now();
        let mut hot = 0u64;
        let mut warm = 0u64;
        let mut cold = 0u64;
        for row in rows {
            let atom = Self::atom_from_row(&row)?;
            match classify_atom(&atom, now, &policy) {
                MemoryTier::Hot => hot += 1,
                MemoryTier::Warm => warm += 1,
                MemoryTier::Cold => cold += 1,
            }
        }
        Self::commit_scope(tx, "count_by_tier").await?;
        Ok((hot, warm, cold))
    }

    async fn list_graph_nodes(
        &self,
        tier: Option<MemoryTier>,
        limit: usize,
        filter: SearchFilter,
        policy: TierPolicy,
    ) -> Result<Vec<GraphNode>> {
        // Member-scoped (KTD1): the graph query has no app-level visibility
        // clause — RLS under the member scope is what keeps other teams' atom
        // ids out of the node set.
        let mut tx = self.begin_scope(PathClass::Request).await?;
        let mut conditions: Vec<&str> = Vec::new();
        if filter.trusted_only {
            conditions.push("trust_lane = 'trusted'");
        }
        if filter.source.is_some() {
            conditions.push("source = $2");
        }
        if filter.exclude_source.is_some() {
            // $2 when no source match; $3 when both present — bind order below.
            conditions.push(if filter.source.is_some() {
                "source != $3"
            } else {
                "source != $2"
            });
        }
        let where_sql = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        let sql = format!(
            "SELECT {ATOM_SELECT} FROM knowledge_atoms {where_sql}
             ORDER BY last_accessed_at DESC LIMIT $1"
        );
        let fetch_cap = if tier.is_some() {
            (limit.saturating_mul(8)).max(limit).min(50_000)
        } else {
            limit.min(50_000)
        };
        let mut q = sqlx::query(&sql).bind(fetch_cap as i64);
        if let Some(ref s) = filter.source {
            q = q.bind(s);
        }
        if let Some(ref s) = filter.exclude_source {
            q = q.bind(s);
        }
        let rows = q
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| KurultaiError::Store(format!("list_graph_nodes: {e}")))?;
        let now = Utc::now();
        let mut out = Vec::new();
        for row in rows {
            let atom = Self::atom_from_row(&row)?;
            let t = classify_atom(&atom, now, &policy);
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
        Self::commit_scope(tx, "list_graph_nodes").await?;
        Ok(out)
    }

    async fn record_ingestion_start(
        &self,
        batch_id: &str,
        source: &str,
        file_path: &str,
    ) -> Result<i64> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO ingestion_jobs (batch_id, source, file_path) VALUES ($1,$2,$3) RETURNING id",
        )
        .bind(batch_id)
        .bind(source)
        .bind(file_path)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("record_ingestion_start: {e}")))?;
        Self::commit_scope(tx, "record_ingestion_start").await?;
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
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        sqlx::query(
            "UPDATE ingestion_jobs SET status = $1, atoms_count = $2, error_message = $3, completed_at = $4 WHERE id = $5",
        )
        .bind(status)
        .bind(atoms_count)
        .bind(error_message)
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("record_ingestion_finish: {e}")))?;
        Self::commit_scope(tx, "record_ingestion_finish").await
    }

    async fn list_pending_ingestion_jobs(&self) -> Result<Vec<IngestionJob>> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let rows = sqlx::query(
            "SELECT id, batch_id, source, file_path, status, atoms_count, error_message, created_at, completed_at
             FROM ingestion_jobs WHERE status = 'pending' ORDER BY id",
        )
        .fetch_all(&mut *tx)
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
        Self::commit_scope(tx, "list_pending_ingestion_jobs").await?;
        Ok(out)
    }

    // ── Device authorization implementations (U3/KTD3) ─────────────────────
    //
    // `hub_device_flows` carries no RLS (the auth-exempt endpoints run before
    // any principal/GUC exists), so every op still runs inside a scoped
    // transaction — `PathClass::Internal`, matching ingestion jobs.

    async fn create_device_flow(
        &self,
        codename: &str,
        client_id: &str,
        instance_id: &str,
        expires_in_seconds: u64,
    ) -> Result<DeviceFlow> {
        self.create_device_flow_with_hint(
            codename,
            client_id,
            instance_id,
            None,
            expires_in_seconds,
        )
        .await
    }

    async fn create_device_flow_with_hint(
        &self,
        codename: &str,
        client_id: &str,
        instance_id: &str,
        team_hint: Option<&str>,
        expires_in_seconds: u64,
    ) -> Result<DeviceFlow> {
        let id = uuid::Uuid::new_v4().to_string();
        let device_code = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        let user_code = uuid::Uuid::new_v4().simple().to_string().to_uppercase()[..8].to_string();
        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::seconds(expires_in_seconds as i64)).to_rfc3339();

        let mut tx = self.begin_scope(PathClass::Internal).await?;
        sqlx::query(
            "INSERT INTO hub_device_flows (id, device_code, user_code, codename, instance_id, client_id, status, created_at, expires_at, team_hint) \
             VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8, $9)",
        )
        .bind(&id)
        .bind(&device_code)
        .bind(&user_code)
        .bind(codename)
        .bind(instance_id)
        .bind(client_id)
        .bind(&created_at)
        .bind(&expires_at)
        .bind(team_hint)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("create_device_flow insert: {e}")))?;
        Self::commit_scope(tx, "create_device_flow").await?;

        Ok(DeviceFlow {
            id,
            device_code,
            user_code,
            codename: codename.to_string(),
            instance_id: instance_id.to_string(),
            client_id: client_id.to_string(),
            status: "pending".to_string(),
            created_at,
            expires_at,
            approved_at: None,
            approved_by: None,
            agent_id: None,
            token_hash: None,
            team_hint: team_hint.map(str::to_string),
            team_id: None,
            issued_key: None,
        })
    }

    async fn get_device_flow_by_device_code(
        &self,
        device_code: &str,
    ) -> Result<Option<DeviceFlow>> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let row = sqlx::query(&format!(
            "SELECT {DEVICE_FLOW_SELECT} FROM hub_device_flows WHERE device_code = $1"
        ))
        .bind(device_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("get_device_flow_by_device_code: {e}")))?;
        Self::commit_scope(tx, "get_device_flow_by_device_code").await?;
        row.map(|r| Self::device_flow_from_row(&r)).transpose()
    }

    async fn get_device_flow_by_user_code(&self, user_code: &str) -> Result<Option<DeviceFlow>> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let row = sqlx::query(&format!(
            "SELECT {DEVICE_FLOW_SELECT} FROM hub_device_flows WHERE user_code = $1"
        ))
        .bind(user_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("get_device_flow_by_user_code: {e}")))?;
        Self::commit_scope(tx, "get_device_flow_by_user_code").await?;
        row.map(|r| Self::device_flow_from_row(&r)).transpose()
    }

    async fn approve_device_flow(&self, user_code: &str, approved_by: &str) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let updated = sqlx::query(
            "UPDATE hub_device_flows \
             SET status = 'approved', approved_at = $1, approved_by = $2 \
             WHERE user_code = $3 AND status = 'pending' AND expires_at > $1",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(approved_by)
        .bind(user_code)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("approve_device_flow update: {e}")))?
        .rows_affected();
        if updated == 0 {
            return Err(KurultaiError::Store(
                "device flow not found, already approved, or expired".into(),
            ));
        }
        Self::commit_scope(tx, "approve_device_flow").await
    }

    async fn approve_device_flow_hub(
        &self,
        user_code: &str,
        approved_by: &str,
        team_id: &str,
        issued_key: &str,
    ) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let updated = sqlx::query(
            "UPDATE hub_device_flows \
             SET status = 'approved', approved_at = $1, approved_by = $2, \
                 team_id = $3, issued_key = $4 \
             WHERE user_code = $5 AND status = 'pending' AND expires_at > $1",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(approved_by)
        .bind(team_id)
        .bind(issued_key)
        .bind(user_code)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("approve_device_flow_hub update: {e}")))?
        .rows_affected();
        if updated == 0 {
            return Err(KurultaiError::Store(
                "device flow not found, already approved, or expired".into(),
            ));
        }
        Self::commit_scope(tx, "approve_device_flow_hub").await
    }

    async fn deny_device_flow(&self, user_code: &str, denied_by: &str) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let updated = sqlx::query(
            "UPDATE hub_device_flows \
             SET status = 'denied', approved_at = $1, approved_by = $2 \
             WHERE user_code = $3 AND status = 'pending' AND expires_at > $1",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(denied_by)
        .bind(user_code)
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("deny_device_flow update: {e}")))?
        .rows_affected();
        if updated == 0 {
            return Err(KurultaiError::Store(
                "device flow not found, already decided, or expired".into(),
            ));
        }
        Self::commit_scope(tx, "deny_device_flow").await
    }

    async fn exchange_device_flow(
        &self,
        device_code: &str,
        token_hash: &str,
        agent_id: &str,
    ) -> Result<()> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        // `issued_key = NULL` — the minted hub key exists at most once:
        // between approve and the first successful token poll.
        let updated = sqlx::query(
            "UPDATE hub_device_flows \
             SET status = 'exchanged', token_hash = $1, agent_id = $2, issued_key = NULL \
             WHERE device_code = $3 AND status = 'approved' AND expires_at > $4",
        )
        .bind(token_hash)
        .bind(agent_id)
        .bind(device_code)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("exchange_device_flow update: {e}")))?
        .rows_affected();
        if updated == 0 {
            return Err(KurultaiError::Store(
                "device flow not approved, not found, or expired".into(),
            ));
        }
        Self::commit_scope(tx, "exchange_device_flow").await
    }

    async fn list_device_flows(&self, limit: usize) -> Result<Vec<DeviceFlow>> {
        let mut tx = self.begin_scope(PathClass::Internal).await?;
        let rows = sqlx::query(&format!(
            "SELECT {DEVICE_FLOW_SELECT} FROM hub_device_flows \
             ORDER BY CASE status WHEN 'pending' THEN 0 WHEN 'approved' THEN 1 ELSE 2 END, \
                      created_at DESC \
             LIMIT $1"
        ))
        .bind(limit as i64)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("list_device_flows: {e}")))?;
        Self::commit_scope(tx, "list_device_flows").await?;
        rows.iter().map(Self::device_flow_from_row).collect()
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

    async fn delete_ontology_entity(&self, _id: &str) -> Result<()> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn delete_ontology_link(&self, _id: &str) -> Result<()> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn insert_ontology_proposal(&self, _p: &OntologyProposal) -> Result<()> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn get_ontology_proposal(&self, _id: &str) -> Result<Option<OntologyProposal>> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn list_ontology_proposals(
        &self,
        _status: Option<&str>,
        _limit: usize,
    ) -> Result<Vec<OntologyProposal>> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }

    async fn decide_ontology_proposal(
        &self,
        _id: &str,
        _status: &str,
        _decided_by: &str,
        _decided_at: &str,
    ) -> Result<OntologyProposal> {
        Err(KurultaiError::Store("ontology not on hub store yet".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn sample_team(id: &str, title: &str, content: &str, emb: Option<Vec<f32>>) -> KnowledgeAtom {
        let mut atom = KnowledgeAtom {
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
        };
        atom.metadata.insert("team_id".into(), "team-test".into());
        atom
    }

    fn sample_scoped(
        id: &str,
        content: &str,
        visibility: VisibilityScope,
        team_id: Option<&str>,
    ) -> KnowledgeAtom {
        let mut atom = sample_team(id, id, content, None);
        atom.visibility = visibility;
        if let Some(team) = team_id {
            atom.metadata.insert("team_id".into(), team.into());
        }
        atom
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
        let pid = format!("p-ae4-{}", uuid::Uuid::new_v4());
        let mut personal = sample_team(&pid, "Personal", "must not land in hub", None);
        personal.visibility = VisibilityScope::Personal;
        let err = store.upsert(&personal).await.unwrap_err();
        assert!(
            err.to_string().contains("AE4") || err.to_string().contains("personal"),
            "{err}"
        );
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
            .fts_search("quarantine-fts-token", 10, SearchFilter::trusted(false))
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
        let _guard = crate::testutil::EnvGuard::set("KURULTAI_FEATURE_HUB", "1");
        let store = crate::store::open_hub_store(&url, 4)
            .await
            .expect("open hub");
        assert!(store.count().await.is_ok());
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

    #[tokio::test]
    async fn hub_team_filter_ae5_isolates_team_atoms() {
        let Some(store) = test_store().await else {
            return;
        };
        let token = format!("ae5-token-{}", uuid::Uuid::new_v4());
        let eng_id = format!("eng-{}", uuid::Uuid::new_v4());
        let sales_id = format!("sales-{}", uuid::Uuid::new_v4());
        let company_id = format!("company-{}", uuid::Uuid::new_v4());

        store
            .upsert(&sample_scoped(
                &eng_id,
                &format!("{token} engineering notes"),
                VisibilityScope::Team,
                Some("eng"),
            ))
            .await
            .unwrap();
        store
            .upsert(&sample_scoped(
                &sales_id,
                &format!("{token} sales notes"),
                VisibilityScope::Team,
                Some("sales"),
            ))
            .await
            .unwrap();
        store
            .upsert(&sample_scoped(
                &company_id,
                &format!("{token} company wide"),
                VisibilityScope::Company,
                None,
            ))
            .await
            .unwrap();

        let filter = SearchFilter::default().with_hub_team(Some("eng"));
        let hits = store.fts_search(&token, 10, filter).await.unwrap();
        let ids: Vec<_> = hits.iter().map(|(a, _)| a.id.as_str()).collect();
        assert!(ids.contains(&eng_id.as_str()), "eng team atom visible");
        assert!(ids.contains(&company_id.as_str()), "company atom visible");
        assert!(
            !ids.contains(&sales_id.as_str()),
            "sales team atom must not leak to eng caller (AE5)"
        );

        store.delete_atom(&eng_id).await.unwrap();
        store.delete_atom(&sales_id).await.unwrap();
        store.delete_atom(&company_id).await.unwrap();
    }
}
