pub mod migrations;
#[cfg(feature = "postgres")]
pub mod postgres;

use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::memory::{classify, GraphNode, MemoryTier, TierPolicy};
use crate::types::{
    normalize_soft_labels, CorpusTier, KnowledgeAtom, OntologyEntity, OntologyLink,
    OntologyLinkType, SoftLabel, TrustLane, VisibilityScope,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zerocopy::AsBytes;

/// Norm below this is treated as a zero / stub vector — never written to `atoms_vec`.
pub(crate) const MIN_EMBEDDING_NORM: f32 = 1e-6;

/// Columns loaded when hydrating a full `KnowledgeAtom` from the SQLite store.
const ATOM_COLUMNS: &str = "id, source, source_id, title, summary, content, question, resolution, \
     tags_json, source_updated_at, indexed_at, metadata_json, trust_lane, quarantine_reason, \
     last_accessed_at, visibility, corpus_tier, visibility_labels_json";

/// Retrieval filter — default skips quarantine and spans every project.
///
/// `project` is namespacing, not isolation: it keeps one Claude Code session's
/// recall from being polluted by another session's ingest on the same box. Any
/// local process running as the same unix user can still pass any project
/// string. See `docs/PROJECT_SCOPING.md`.
#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub trusted_only: bool,
    /// When set, only atoms whose `metadata.project_id` equals this value match.
    /// Atoms with no `project_id` are treated as `"default"`.
    pub project: Option<String>,
    /// Hub AE5: when set, only `company` atoms or `team` atoms matching this team_id.
    pub hub_team_id: Option<String>,
}

impl Default for SearchFilter {
    fn default() -> Self {
        Self {
            trusted_only: true,
            project: None,
            hub_team_id: None,
        }
    }
}

pub use crate::project::{normalize_project, DEFAULT_PROJECT};

/// vec0 KNN takes no WHERE predicates, so project scoping is applied in Rust.
/// Widen `k` when scoped so other sessions' atoms cannot crowd out our own.
const VECTOR_PROJECT_OVERFETCH: usize = 8;
const VECTOR_K_CAP: usize = 2_000;

impl SearchFilter {
    /// Trust-lane-only filter (no project scoping) — the historical constructor.
    pub fn trusted(trusted_only: bool) -> Self {
        Self {
            trusted_only,
            project: None,
            hub_team_id: None,
        }
    }

    /// Scope to a project namespace. Empty / whitespace input clears the scope.
    /// The value is normalized so read scoping matches write tagging exactly.
    pub fn with_project(mut self, project: Option<&str>) -> Self {
        self.project = project
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(normalize_project);
        self
    }

    /// Hub AE5 team isolation: company atoms plus team atoms for this team_id.
    pub fn with_hub_team(mut self, team_id: Option<&str>) -> Self {
        self.hub_team_id = team_id
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_string);
        self
    }

    /// Project scope as a SQL bind value, if any.
    pub fn project_scope(&self) -> Option<&str> {
        self.project.as_deref()
    }

    /// Hub team scope as a SQL bind value, if any.
    pub fn hub_team_scope(&self) -> Option<&str> {
        self.hub_team_id.as_deref()
    }
}

/// A row from the `ingestion_jobs` staging table.
#[derive(Debug, Clone)]
pub struct IngestionJob {
    pub id: i64,
    pub batch_id: String,
    pub source: String,
    pub file_path: String,
    pub status: String,
    pub atoms_count: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

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
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>>;

    /// Full-text search over atom content.
    async fn fts_search(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>>;

    /// FTS ranks as `(id, score)` without hydrating full atoms.
    async fn fts_search_ids(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(String, f64)>>;

    /// Vector ranks as `(id, score)` without hydrating full atoms.
    async fn vector_search_ids(
        &self,
        query_embed: &[f32],
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(String, f64)>>;

    /// Batch load atoms by id (order not guaranteed; missing ids omitted).
    async fn get_many(&self, ids: &[String]) -> Result<Vec<KnowledgeAtom>>;

    /// Load one atom by id (any lane).
    async fn get(&self, id: &str) -> Result<Option<KnowledgeAtom>>;

    /// Delete a single atom (and its FTS/vec rows).
    async fn delete_atom(&self, id: &str) -> Result<()>;

    /// Atomic auto-merge: upsert survivor, delete loser (fts+vec+row), insert quality_audit
    /// in one `BEGIN IMMEDIATE` transaction.
    async fn apply_auto_merge(
        &self,
        survivor: &KnowledgeAtom,
        loser_id: &str,
        audit_detail: &serde_json::Value,
    ) -> Result<()>;

    /// Delete atoms for a given source (for re-index).
    async fn delete_source(&self, source: &str) -> Result<()>;

    /// Total atom count.
    async fn count(&self) -> Result<u64>;

    /// Count atoms in a trust lane.
    async fn count_by_lane(&self, lane: TrustLane) -> Result<u64>;

    /// Lookup by source + source_id (cite path).
    async fn get_by_source_id(
        &self,
        source: &str,
        source_id: &str,
    ) -> Result<Option<KnowledgeAtom>>;

    /// Markdown neighbor by `rel_path` + `chunk_index` metadata (same source).
    async fn get_by_chunk_meta(
        &self,
        source: &str,
        rel_path: &str,
        chunk_index: u32,
    ) -> Result<Option<KnowledgeAtom>>;

    /// True when atom `id` already has `content_hash` and a stored vector (hash-skip re-embed).
    async fn has_fresh_embedding(&self, id: &str, content_hash: &str) -> Result<bool>;

    /// Return up to `limit` atoms ordered newest-first.
    async fn list_atoms(&self, limit: usize, filter: SearchFilter) -> Result<Vec<KnowledgeAtom>>;

    /// Return atoms whose `source_id` matches any of the given SQL LIKE patterns.
    async fn find_atoms_by_source_id_patterns(
        &self,
        patterns: &[&str],
    ) -> Result<Vec<KnowledgeAtom>>;

    /// First trusted atom id with this content hash (exact-dupe gate).
    async fn find_trusted_by_content_hash(&self, content_hash: &str) -> Result<Option<String>>;

    /// Update trust lane + optional quarantine reason.
    async fn set_trust_lane(
        &self,
        id: &str,
        lane: TrustLane,
        quarantine_reason: Option<&str>,
    ) -> Result<()>;

    /// Append a quality_audit row.
    async fn insert_quality_audit(
        &self,
        action: &str,
        atom_id: &str,
        actor: &str,
        detail: &serde_json::Value,
    ) -> Result<()>;

    /// Insert merge candidate if not already pending; returns true when inserted.
    async fn insert_merge_candidate(
        &self,
        atom_a: &str,
        atom_b: &str,
        reason: &str,
    ) -> Result<bool>;

    /// Pending merge_candidates count.
    async fn count_merge_candidates_pending(&self) -> Result<u64>;

    /// Quarantine + recently indexed atoms for near-dupe scan.
    async fn list_near_dupe_candidates(&self, limit: usize) -> Result<Vec<KnowledgeAtom>>;

    /// Bump `last_accessed_at` (search / cite / UI focus).
    async fn touch_access(&self, id: &str) -> Result<()>;

    /// Count atoms by derived memory tier (hot / warm / cold).
    async fn count_by_tier(&self, policy: TierPolicy) -> Result<(u64, u64, u64)>;

    /// Graph stubs ordered by access freshness; optional tier filter.
    async fn list_graph_nodes(
        &self,
        tier: Option<MemoryTier>,
        limit: usize,
        filter: SearchFilter,
        policy: TierPolicy,
    ) -> Result<Vec<GraphNode>>;

    // ── Ingestion staging ────────────────────────────────────────────────────

    /// Record the start of an ingestion job; returns the new job `id`.
    async fn record_ingestion_start(
        &self,
        batch_id: &str,
        source: &str,
        file_path: &str,
    ) -> Result<i64>;

    /// Mark an ingestion job as completed (success) or failed.
    ///
    /// When `error_message` is `Some`, status is set to `'failed'`;
    /// otherwise status is set to `'completed'` and `atoms_count` is recorded.
    async fn record_ingestion_finish(
        &self,
        job_id: i64,
        atoms_count: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<()>;

    /// Return all ingestion jobs with `status = 'pending'`.
    async fn list_pending_ingestion_jobs(&self) -> Result<Vec<IngestionJob>>;

    /// Insert or replace an ontology entity (O1).
    async fn upsert_ontology_entity(&self, e: &OntologyEntity) -> Result<()>;

    async fn get_ontology_entity(&self, id: &str) -> Result<Option<OntologyEntity>>;

    async fn list_ontology_entities(&self, limit: usize) -> Result<Vec<OntologyEntity>>;

    /// Insert or update a typed ontology link. Duplicate `(from,to,rel)` updates confidence/actor.
    async fn upsert_ontology_link(&self, l: &OntologyLink) -> Result<()>;

    /// All links, or those incident on an entity id / atom id.
    async fn list_ontology_links(&self, endpoint: Option<&str>) -> Result<Vec<OntologyLink>>;
}

/// SQLite + sqlite-vec storage implementation (#1).
/// Open the default store from a loaded config (convenience helper for CLI commands).
pub async fn open_store(config: &crate::types::Config) -> Result<Arc<dyn Store>> {
    let path = crate::config::expand_path(&config.storage_path)?;
    Ok(Arc::new(SqliteVecStore::open(path, config.embed_dim)?) as Arc<dyn Store>)
}

/// Shared-tier hub store. Never selected by [`open_store`] (solo stays SQLite).
///
/// Requires `--features postgres` **and** `KURULTAI_FEATURE_HUB=1`.
pub fn database_url_from_env() -> Option<String> {
    for key in ["KURULTAI_DATABASE_URL", "DATABASE_URL"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

pub async fn open_hub_store(database_url: &str, embed_dim: usize) -> Result<Arc<dyn Store>> {
    if !crate::features::enabled("hub") {
        return Err(KurultaiError::Store(
            "hub store requires KURULTAI_FEATURE_HUB=1 (v0.5.0 flag, default off)".into(),
        ));
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (database_url, embed_dim);
        Err(KurultaiError::Store(
            "hub store requires `cargo build --features postgres`".into(),
        ))
    }
    #[cfg(feature = "postgres")]
    {
        Ok(
            Arc::new(postgres::PostgresStore::connect(database_url, embed_dim).await?)
                as Arc<dyn Store>,
        )
    }
}

/// How long a connection waits on a lock held by another process before
/// returning `SQLITE_BUSY`. Sized for many concurrent MCP stdio processes on
/// one box sharing a single `store.db`.
pub const BUSY_TIMEOUT_MS: u32 = 5_000;

/// Env override for [`BUSY_TIMEOUT_MS`], in milliseconds.
pub const BUSY_TIMEOUT_ENV: &str = "KURULTAI_SQLITE_BUSY_TIMEOUT_MS";

fn busy_timeout_ms() -> u32 {
    std::env::var(BUSY_TIMEOUT_ENV)
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(BUSY_TIMEOUT_MS)
}

/// Put a freshly opened connection into a state safe for concurrent
/// multi-process access.
///
/// Kurultai's shared deployment shape is N independent OS processes (one
/// `kurultai mcp` per agent session, plus `kurultai index` runs) each opening
/// the same `store.db`. The in-process `Mutex<Connection>` does nothing across
/// process boundaries, so the database itself has to be configured for it:
///
/// * `journal_mode = WAL` — readers no longer block on the writer. Under the
///   default `DELETE` journal a single writer takes an exclusive lock over the
///   whole file, so every concurrent `search` fails for the duration of an
///   index run. WAL is persisted in the database header, so this only has to
///   take effect once, but setting it on every open is cheap and correct.
/// * `busy_timeout` — per-connection, must be set every open. Without it the
///   losing process gets `SQLITE_BUSY` immediately instead of waiting.
/// * `synchronous = NORMAL` — per-connection, the standard companion to WAL.
///
/// `journal_mode` is a *query* pragma: `execute_batch` silently swallows a
/// failed switch, so the result is read back and verified. A failure to reach
/// WAL is a warning, not an error — a store on a filesystem that cannot do WAL
/// (e.g. a network mount) should still open for single-process use.
fn configure_multiprocess(conn: &Connection, path: &std::path::Path) -> Result<()> {
    let timeout = busy_timeout_ms();

    // busy_timeout first, so the journal_mode switch below can itself wait on
    // another process that currently holds the lock.
    conn.pragma_update(None, "busy_timeout", timeout)
        .map_err(|e| KurultaiError::Store(format!("set busy_timeout: {e}")))?;

    let mode: String = conn
        .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))
        .map_err(|e| KurultaiError::Store(format!("set journal_mode=WAL: {e}")))?;

    if !mode.eq_ignore_ascii_case("wal") {
        tracing::warn!(
            path = %path.display(),
            journal_mode = %mode,
            "sqlite did not switch to WAL — concurrent readers will block on writes"
        );
    }

    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| KurultaiError::Store(format!("set synchronous=NORMAL: {e}")))?;

    tracing::debug!(
        path = %path.display(),
        journal_mode = %mode,
        busy_timeout_ms = timeout,
        "sqlite configured for multi-process access"
    );
    Ok(())
}

/// Sidecar files SQLite creates alongside `store.db` in WAL mode.
///
/// These must be removed whenever `store.db` is replaced wholesale by a file
/// copy, or SQLite will apply the old WAL over the new database.
pub fn wal_sidecar_paths(db: &std::path::Path) -> [PathBuf; 2] {
    let name = db.file_name().unwrap_or_default().to_os_string();
    let mut wal = name.clone();
    wal.push("-wal");
    let mut shm = name;
    shm.push("-shm");
    let parent = db.parent().unwrap_or_else(|| std::path::Path::new("."));
    [parent.join(wal), parent.join(shm)]
}

/// Remove stale `-wal` / `-shm` sidecars next to `db`. Missing files are fine.
pub fn remove_wal_sidecars(db: &std::path::Path) -> Result<()> {
    for side in wal_sidecar_paths(db) {
        match std::fs::remove_file(&side) {
            Ok(()) => tracing::debug!(path = %side.display(), "removed stale sqlite sidecar"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(KurultaiError::Store(format!(
                    "remove {}: {e}",
                    side.display()
                )))
            }
        }
    }
    Ok(())
}

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

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| KurultaiError::Store(format!("enable foreign_keys: {e}")))?;

        configure_multiprocess(&conn, &path)?;

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

    /// Read an integer pragma off this connection (e.g. `busy_timeout`).
    ///
    /// Exposed so deployment/acceptance checks can assert on the connection
    /// state set up by [`configure_multiprocess`] rather than trusting it.
    pub fn pragma_i64(&self, name: &str) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
            .map_err(|e| KurultaiError::Store(format!("read pragma {name}: {e}")))
    }

    /// The journal mode this database is actually in — `"wal"` when
    /// multi-process access is safe.
    pub fn journal_mode(&self) -> Result<String> {
        let conn = self.lock()?;
        conn.query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .map_err(|e| KurultaiError::Store(format!("read pragma journal_mode: {e}")))
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

    /// Return up to `limit` atoms ordered by indexed_at DESC (newest first).
    pub fn list_atoms_sync(
        &self,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<KnowledgeAtom>> {
        let conn = self.lock()?;
        let sql = if filter.trusted_only {
            format!(
                "SELECT {} FROM knowledge_atoms WHERE trust_lane = 'trusted' \
                 ORDER BY indexed_at DESC LIMIT ?1",
                ATOM_COLUMNS
            )
        } else {
            format!(
                "SELECT {} FROM knowledge_atoms ORDER BY indexed_at DESC LIMIT ?1",
                ATOM_COLUMNS
            )
        };
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| KurultaiError::Store(format!("list_atoms prepare: {e}")))?;
        let atoms = stmt
            .query_map([limit as i64], row_to_atom)
            .map_err(|e| KurultaiError::Store(format!("list_atoms query: {e}")))?;
        let mut atoms = atoms
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KurultaiError::Store(format!("list_atoms collect: {e}")))?;
        attach_soft_labels(&conn, &mut atoms)?;
        Ok(atoms)
    }

    /// Keyset-paginated atom list with optional embeddings (`ORDER BY id`, `id > after_id`).
    ///
    /// Used by export/import combine walks — avoids `OFFSET` scan cost on large stores.
    pub fn list_atoms_page_sync(
        &self,
        after_id: Option<&str>,
        limit: usize,
        filter: SearchFilter,
        with_embeddings: bool,
    ) -> Result<Vec<KnowledgeAtom>> {
        let conn = self.lock()?;
        let select = if with_embeddings {
            format!(
                "SELECT {}, atoms_vec.embedding FROM knowledge_atoms \
                 LEFT JOIN atoms_vec ON atoms_vec.rowid = knowledge_atoms.rowid",
                ATOM_COLUMNS
            )
        } else {
            format!("SELECT {} FROM knowledge_atoms", ATOM_COLUMNS)
        };
        let trusted = if filter.trusted_only {
            "trust_lane = 'trusted'"
        } else {
            "1=1"
        };

        let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<KnowledgeAtom> {
            let mut atom = row_to_atom(row)?;
            if with_embeddings {
                // ATOM_COLUMNS ends at index 17 (`visibility_labels_json`); embedding is the next select.
                const EMBEDDING_COL: usize = 18;
                let blob: Option<Vec<u8>> = row.get(EMBEDDING_COL)?;
                if let Some(bytes) = blob {
                    atom.embedding = Some(embedding_f32s_from_blob(&bytes).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            EMBEDDING_COL,
                            rusqlite::types::Type::Blob,
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                e.to_string(),
                            )),
                        )
                    })?);
                }
            }
            Ok(atom)
        };

        let atoms = match after_id {
            Some(id) => {
                let sql = format!("{select} WHERE {trusted} AND id > ?1 ORDER BY id LIMIT ?2");
                let mut stmt = conn
                    .prepare(&sql)
                    .map_err(|e| KurultaiError::Store(format!("list_atoms_page prepare: {e}")))?;
                let rows = stmt
                    .query_map(params![id, limit as i64], map_row)
                    .map_err(|e| KurultaiError::Store(format!("list_atoms_page query: {e}")))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
            }
            None => {
                let sql = format!("{select} WHERE {trusted} ORDER BY id LIMIT ?1");
                let mut stmt = conn
                    .prepare(&sql)
                    .map_err(|e| KurultaiError::Store(format!("list_atoms_page prepare: {e}")))?;
                let rows = stmt
                    .query_map(params![limit as i64], map_row)
                    .map_err(|e| KurultaiError::Store(format!("list_atoms_page query: {e}")))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
            }
        }
        .map_err(|e| KurultaiError::Store(format!("list_atoms_page collect: {e}")))?;
        let mut atoms = atoms;
        attach_soft_labels(&conn, &mut atoms)?;
        Ok(atoms)
    }

    /// Load the sqlite-vec embedding for an atom id, if present.
    pub fn load_embedding_sync(&self, id: &str) -> Result<Option<Vec<f32>>> {
        let conn = self.lock()?;
        let rowid: Option<i64> = conn
            .query_row(
                "SELECT rowid FROM knowledge_atoms WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("rowid lookup: {e}")))?;
        let Some(rowid) = rowid else {
            return Ok(None);
        };
        let blob: Option<Vec<u8>> = conn
            .query_row(
                "SELECT embedding FROM atoms_vec WHERE rowid = ?1",
                [rowid],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| KurultaiError::Store(format!("embedding load: {e}")))?;
        let Some(bytes) = blob else {
            return Ok(None);
        };
        Ok(Some(embedding_f32s_from_blob(&bytes)?))
    }

    /// Atom count (sync).
    pub fn count_sync(&self) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_atoms", [], |row| row.get(0))
            .map_err(|e| KurultaiError::Store(format!("count failed: {e}")))?;
        Ok(count as u64)
    }

    /// Online backup of this store to `dst` (safe while the connection is open).
    pub fn backup_to_path(&self, dst: &std::path::Path) -> Result<()> {
        let src = self.lock()?;
        src.backup(rusqlite::DatabaseName::Main, dst, None)
            .map_err(|e| {
                KurultaiError::Store(format!("sqlite backup to {}: {e}", dst.display()))
            })?;
        Ok(())
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

        let trust_lane = atom.trust_lane.as_str();
        let quarantine_reason = atom.quarantine_reason.as_deref();
        let visibility = atom.visibility.as_str();
        let corpus_tier = atom.corpus_tier.as_str();
        let visibility_labels_json = serde_json::to_string(&atom.visibility_labels)
            .map_err(|e| KurultaiError::Store(format!("visibility_labels serialize: {e}")))?;
        let last_accessed = if atom.last_accessed_at.timestamp() == 0 {
            atom.indexed_at
        } else {
            atom.last_accessed_at
        };
        conn.execute(
            r#"
            INSERT INTO knowledge_atoms (
                id, source, source_id, title, summary, content,
                question, resolution, tags_json,
                source_updated_at, indexed_at, metadata_json, content_hash,
                trust_lane, quarantine_reason, last_accessed_at, visibility,
                corpus_tier, visibility_labels_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
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
                corpus_tier = excluded.corpus_tier,
                visibility_labels_json = excluded.visibility_labels_json
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
                trust_lane,
                quarantine_reason,
                last_accessed.to_rfc3339(),
                visibility,
                corpus_tier,
                visibility_labels_json,
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
        // Quarantine always clears vec (KTD7 — don't pollute atoms_vec with junk).
        if atom.trust_lane == TrustLane::Quarantine {
            conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", [rowid])
                .map_err(|e| KurultaiError::Store(format!("vec delete failed: {e}")))?;
        } else {
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
        }

        // Soft labels: replace when caller provided any; preserve existing when empty
        // so re-index / hash-skip paths do not wipe distillation scores.
        if !atom.soft_labels.is_empty() {
            replace_soft_labels(conn, &atom.id, &atom.soft_labels)?;
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
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let ids = self.vector_search_ids(query_embed, limit, filter).await?;
        hydrate_ranked(self, ids).await
    }

    async fn fts_search(
        &self,
        query: &str,
        limit: usize,
        filter: SearchFilter,
    ) -> Result<Vec<(KnowledgeAtom, f64)>> {
        let ids = self.fts_search_ids(query, limit, filter).await?;
        hydrate_ranked(self, ids).await
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

        let fts_query = sanitize_fts_query(query);
        if fts_query.is_empty() {
            return Ok(vec![]);
        }

        let conn = self.lock()?;
        // Predicates live in SQL so project scoping happens BEFORE LIMIT — an
        // in-memory retain() after truncation silently drops matching atoms once
        // several sessions share one store.
        let mut predicates = String::new();
        if filter.trusted_only {
            predicates.push_str(" AND a.trust_lane = 'trusted'");
        }
        let project = filter.project_scope();
        if project.is_some() {
            predicates.push_str(&format!(
                " AND COALESCE(json_extract(a.metadata_json, '$.project_id'), '{DEFAULT_PROJECT}') = ?3"
            ));
        }
        let sql = format!(
            r#"
                SELECT a.id, bm25(atoms_fts) AS score
                FROM atoms_fts
                JOIN knowledge_atoms a ON a.id = atoms_fts.id
                WHERE atoms_fts MATCH ?1{predicates}
                ORDER BY score
                LIMIT ?2
                "#
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| KurultaiError::Store(format!("fts_search_ids prepare: {e}")))?;

        let row_map = |r: &rusqlite::Row<'_>| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?));
        let rows = match project {
            Some(p) => stmt.query_map(params![fts_query, limit as i64, p], row_map),
            None => stmt.query_map(params![fts_query, limit as i64], row_map),
        }
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
        if embedding_norm(query_embed) < MIN_EMBEDDING_NORM {
            return Ok(vec![]);
        }

        // Over-fetch when filtering trusted so k-nearest still fills after lane filter.
        // vec0 KNN cannot take arbitrary WHERE predicates, so lane and project are
        // applied in Rust — over-fetch harder when project-scoped so a session's own
        // atoms are not crowded out of the k window by other sessions on the box.
        let mut k = if filter.trusted_only {
            (limit.saturating_mul(3)).max(limit)
        } else {
            limit
        };
        let project = filter.project_scope();
        if project.is_some() {
            k = k.saturating_mul(VECTOR_PROJECT_OVERFETCH).min(VECTOR_K_CAP);
        }

        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT a.id, v.distance, a.trust_lane,
                       COALESCE(json_extract(a.metadata_json, '$.project_id'), 'default')
                FROM atoms_vec v
                JOIN knowledge_atoms a ON a.rowid = v.rowid
                WHERE v.embedding MATCH ?1 AND k = ?2
                ORDER BY v.distance
                "#,
            )
            .map_err(|e| KurultaiError::Store(format!("vector_search_ids prepare: {e}")))?;

        let rows = stmt
            .query_map(params![query_embed.as_bytes(), k as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, f64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| KurultaiError::Store(format!("vector_search_ids query: {e}")))?;

        let mut out = Vec::new();
        for row in rows {
            let (id, distance, lane, atom_project) =
                row.map_err(|e| KurultaiError::Store(format!("vector_search_ids row: {e}")))?;
            if filter.trusted_only && lane != "trusted" {
                continue;
            }
            if project.is_some_and(|p| p != atom_project) {
                continue;
            }
            let score = 1.0 / (1.0 + distance);
            out.push((id, score));
            if out.len() >= limit {
                break;
            }
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

            // Clean up merge_candidates referencing deleted atoms.
            for (_, id) in &pairs {
                conn.execute(
                    "DELETE FROM merge_candidates WHERE atom_a = ?1 OR atom_b = ?1",
                    [id],
                )
                .map_err(|e| KurultaiError::Store(format!("delete merge_candidates: {e}")))?;
            }
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

    async fn get_by_chunk_meta(
        &self,
        source: &str,
        rel_path: &str,
        chunk_index: u32,
    ) -> Result<Option<KnowledgeAtom>> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {} FROM knowledge_atoms
             WHERE source = ?1
               AND json_extract(metadata_json, '$.rel_path') = ?2
               AND CAST(json_extract(metadata_json, '$.chunk_index') AS INTEGER) = ?3
             LIMIT 1",
            ATOM_COLUMNS
        );
        conn.query_row(
            &sql,
            params![source, rel_path, chunk_index as i64],
            row_to_atom,
        )
        .optional()
        .map_err(|e| KurultaiError::Store(format!("get_by_chunk_meta: {e}")))
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

    async fn list_atoms(&self, limit: usize, filter: SearchFilter) -> Result<Vec<KnowledgeAtom>> {
        self.list_atoms_sync(limit, filter)
    }

    async fn find_atoms_by_source_id_patterns(
        &self,
        patterns: &[&str],
    ) -> Result<Vec<KnowledgeAtom>> {
        if patterns.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.lock()?;
        let placeholders = patterns
            .iter()
            .enumerate()
            .map(|(i, _)| format!("source_id LIKE ?{}", i + 1))
            .collect::<Vec<_>>()
            .join(" OR ");
        let sql = format!(
            "SELECT {} FROM knowledge_atoms WHERE {}",
            ATOM_COLUMNS, placeholders
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| {
            KurultaiError::Store(format!("find_atoms_by_source_id_patterns prepare: {e}"))
        })?;
        let params: Vec<&dyn rusqlite::ToSql> =
            patterns.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let atoms = stmt
            .query_map(params.as_slice(), row_to_atom)
            .map_err(|e| {
                KurultaiError::Store(format!("find_atoms_by_source_id_patterns query: {e}"))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                KurultaiError::Store(format!("find_atoms_by_source_id_patterns collect: {e}"))
            })?;
        Ok(atoms)
    }

    async fn get(&self, id: &str) -> Result<Option<KnowledgeAtom>> {
        let conn = self.lock()?;
        load_atom_by_id(&conn, id)
    }

    async fn delete_atom(&self, id: &str) -> Result<()> {
        let conn = self.lock()?;
        conn.execute_batch("BEGIN IMMEDIATE;")
            .map_err(|e| KurultaiError::Store(format!("begin delete_atom: {e}")))?;

        let result = (|| {
            let rowid: Option<i64> = conn
                .query_row(
                    "SELECT rowid FROM knowledge_atoms WHERE id = ?1",
                    [id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| KurultaiError::Store(format!("delete_atom rowid: {e}")))?;
            let Some(rowid) = rowid else {
                return Ok(());
            };
            conn.execute("DELETE FROM atoms_fts WHERE id = ?1", [id])
                .map_err(|e| KurultaiError::Store(format!("delete_atom fts: {e}")))?;
            conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", [rowid])
                .map_err(|e| KurultaiError::Store(format!("delete_atom vec: {e}")))?;
            conn.execute("DELETE FROM knowledge_atoms WHERE id = ?1", [id])
                .map_err(|e| KurultaiError::Store(format!("delete_atom: {e}")))?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| KurultaiError::Store(format!("commit delete_atom: {e}")))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    async fn apply_auto_merge(
        &self,
        survivor: &KnowledgeAtom,
        loser_id: &str,
        audit_detail: &serde_json::Value,
    ) -> Result<()> {
        let conn = self.lock()?;
        conn.execute_batch("BEGIN IMMEDIATE;")
            .map_err(|e| KurultaiError::Store(format!("begin apply_auto_merge: {e}")))?;

        let result = (|| {
            Self::upsert_sync(&conn, survivor, self.embed_dim)?;

            let rowid: Option<i64> = conn
                .query_row(
                    "SELECT rowid FROM knowledge_atoms WHERE id = ?1",
                    [loser_id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| KurultaiError::Store(format!("apply_auto_merge rowid: {e}")))?;
            if let Some(rowid) = rowid {
                conn.execute("DELETE FROM atoms_fts WHERE id = ?1", [loser_id])
                    .map_err(|e| KurultaiError::Store(format!("apply_auto_merge fts: {e}")))?;
                conn.execute("DELETE FROM atoms_vec WHERE rowid = ?1", [rowid])
                    .map_err(|e| KurultaiError::Store(format!("apply_auto_merge vec: {e}")))?;
                conn.execute("DELETE FROM knowledge_atoms WHERE id = ?1", [loser_id])
                    .map_err(|e| KurultaiError::Store(format!("apply_auto_merge delete: {e}")))?;
            }

            let detail_json = audit_detail.to_string();
            conn.execute(
                "INSERT INTO quality_audit (action, atom_id, actor, detail_json) VALUES (?1, ?2, ?3, ?4)",
                params!["auto_merge", survivor.id.as_str(), "near_dupe", detail_json],
            )
            .map_err(|e| KurultaiError::Store(format!("apply_auto_merge audit: {e}")))?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT;")
                    .map_err(|e| KurultaiError::Store(format!("commit apply_auto_merge: {e}")))?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }

    async fn count_by_lane(&self, lane: TrustLane) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_atoms WHERE trust_lane = ?1",
                [lane.as_str()],
                |row| row.get(0),
            )
            .map_err(|e| KurultaiError::Store(format!("count_by_lane: {e}")))?;
        Ok(count as u64)
    }

    async fn find_trusted_by_content_hash(&self, content_hash: &str) -> Result<Option<String>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id FROM knowledge_atoms WHERE content_hash = ?1 AND trust_lane = 'trusted' LIMIT 1",
            [content_hash],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| KurultaiError::Store(format!("find_trusted_by_content_hash: {e}")))
    }

    async fn set_trust_lane(
        &self,
        id: &str,
        lane: TrustLane,
        quarantine_reason: Option<&str>,
    ) -> Result<()> {
        let conn = self.lock()?;
        let n = conn
            .execute(
                "UPDATE knowledge_atoms SET trust_lane = ?1, quarantine_reason = ?2 WHERE id = ?3",
                params![lane.as_str(), quarantine_reason, id],
            )
            .map_err(|e| KurultaiError::Store(format!("set_trust_lane: {e}")))?;
        if n == 0 {
            return Err(KurultaiError::Store(format!(
                "set_trust_lane: atom not found: {id}"
            )));
        }
        Ok(())
    }

    async fn insert_quality_audit(
        &self,
        action: &str,
        atom_id: &str,
        actor: &str,
        detail: &serde_json::Value,
    ) -> Result<()> {
        let conn = self.lock()?;
        let detail_json = detail.to_string();
        conn.execute(
            "INSERT INTO quality_audit (action, atom_id, actor, detail_json) VALUES (?1, ?2, ?3, ?4)",
            params![action, atom_id, actor, detail_json],
        )
        .map_err(|e| KurultaiError::Store(format!("insert_quality_audit: {e}")))?;
        Ok(())
    }

    async fn insert_merge_candidate(
        &self,
        atom_a: &str,
        atom_b: &str,
        reason: &str,
    ) -> Result<bool> {
        let (a, b) = if atom_a <= atom_b {
            (atom_a, atom_b)
        } else {
            (atom_b, atom_a)
        };
        let conn = self.lock()?;
        let n = conn
            .execute(
                r#"
                INSERT OR IGNORE INTO merge_candidates (atom_a, atom_b, reason, status)
                VALUES (?1, ?2, ?3, 'pending')
                "#,
                params![a, b, reason],
            )
            .map_err(|e| KurultaiError::Store(format!("insert_merge_candidate: {e}")))?;
        Ok(n > 0)
    }

    async fn count_merge_candidates_pending(&self) -> Result<u64> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM merge_candidates WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| KurultaiError::Store(format!("count_merge_candidates_pending: {e}")))?;
        Ok(count as u64)
    }

    async fn list_near_dupe_candidates(&self, limit: usize) -> Result<Vec<KnowledgeAtom>> {
        let conn = self.lock()?;
        // Sargable recency: compare indexed_at (RFC3339) to a UTC cutoff string.
        // Quarantine branch is intentionally unbounded in time — near-dupe must see all quarantine.
        let cutoff = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let sql = format!(
            "SELECT {} FROM knowledge_atoms
             WHERE trust_lane = 'quarantine'
                OR indexed_at >= ?1
             ORDER BY indexed_at DESC
             LIMIT ?2",
            ATOM_COLUMNS
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| KurultaiError::Store(format!("list_near_dupe_candidates prepare: {e}")))?;
        let atoms = stmt
            .query_map(params![cutoff, limit as i64], row_to_atom)
            .map_err(|e| KurultaiError::Store(format!("list_near_dupe_candidates query: {e}")))?;
        atoms
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KurultaiError::Store(format!("list_near_dupe_candidates collect: {e}")))
    }

    async fn touch_access(&self, id: &str) -> Result<()> {
        let conn = self.lock()?;
        let now = Utc::now().to_rfc3339();
        let n = conn
            .execute(
                "UPDATE knowledge_atoms SET last_accessed_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| KurultaiError::Store(format!("touch_access: {e}")))?;
        if n == 0 {
            return Err(KurultaiError::Store(format!("atom not found: {id}")));
        }
        Ok(())
    }

    async fn count_by_tier(&self, policy: TierPolicy) -> Result<(u64, u64, u64)> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {} FROM knowledge_atoms WHERE trust_lane = 'trusted'",
            ATOM_COLUMNS
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| KurultaiError::Store(format!("count_by_tier prepare: {e}")))?;
        let rows = stmt
            .query_map([], row_to_atom)
            .map_err(|e| KurultaiError::Store(format!("count_by_tier query: {e}")))?;
        let now = Utc::now();
        let mut hot = 0u64;
        let mut warm = 0u64;
        let mut cold = 0u64;
        for row in rows {
            let atom = row.map_err(|e| KurultaiError::Store(format!("count_by_tier row: {e}")))?;
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
        let conn = self.lock()?;
        let sql = if filter.trusted_only {
            format!(
                "SELECT {} FROM knowledge_atoms WHERE trust_lane = 'trusted'
                 ORDER BY last_accessed_at DESC LIMIT ?1",
                ATOM_COLUMNS
            )
        } else {
            format!(
                "SELECT {} FROM knowledge_atoms ORDER BY last_accessed_at DESC LIMIT ?1",
                ATOM_COLUMNS
            )
        };
        // Over-fetch then filter by tier so hot/warm/cold slices stay accurate.
        let fetch_cap = if tier.is_some() {
            (limit.saturating_mul(8)).max(limit).min(50_000)
        } else {
            limit.min(50_000)
        };
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| KurultaiError::Store(format!("list_graph_nodes prepare: {e}")))?;
        let rows = stmt
            .query_map(params![fetch_cap as i64], row_to_atom)
            .map_err(|e| KurultaiError::Store(format!("list_graph_nodes query: {e}")))?;
        let now = Utc::now();
        let mut out = Vec::with_capacity(limit.min(1024));
        for row in rows {
            let atom =
                row.map_err(|e| KurultaiError::Store(format!("list_graph_nodes row: {e}")))?;
            let t = classify(atom.indexed_at, atom.last_accessed_at, now, policy);
            if let Some(want) = tier {
                if t != want {
                    continue;
                }
            }
            let include_summary = t == MemoryTier::Hot;
            out.push(GraphNode::from_atom(&atom, t, include_summary));
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    // ── Ingestion staging ────────────────────────────────────────────────

    async fn record_ingestion_start(
        &self,
        batch_id: &str,
        source: &str,
        file_path: &str,
    ) -> Result<i64> {
        let conn = self.lock()?;
        Self::record_ingestion_start_sync(&conn, batch_id, source, file_path)
    }

    async fn record_ingestion_finish(
        &self,
        job_id: i64,
        atoms_count: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let conn = self.lock()?;
        Self::record_ingestion_finish_sync(&conn, job_id, atoms_count, error_message)
    }

    async fn list_pending_ingestion_jobs(&self) -> Result<Vec<IngestionJob>> {
        let conn = self.lock()?;
        Self::list_pending_ingestion_jobs_sync(&conn)
    }

    async fn upsert_ontology_entity(&self, e: &OntologyEntity) -> Result<()> {
        let conn = self.lock()?;
        let attrs = match &e.attributes {
            serde_json::Value::Object(_) => e.attributes.to_string(),
            _ => "{}".into(),
        };
        conn.execute(
            r#"
            INSERT INTO ontology_entities (id, kind, name, atom_id, attributes_json)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                name = excluded.name,
                atom_id = excluded.atom_id,
                attributes_json = excluded.attributes_json
            "#,
            params![e.id, e.kind, e.name, e.atom_id, attrs],
        )
        .map_err(|e| KurultaiError::Store(format!("upsert_ontology_entity: {e}")))?;
        Ok(())
    }

    async fn get_ontology_entity(&self, id: &str) -> Result<Option<OntologyEntity>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, name, atom_id, attributes_json FROM ontology_entities WHERE id = ?1",
            )
            .map_err(|e| KurultaiError::Store(format!("get_ontology_entity prepare: {e}")))?;
        stmt.query_row([id], row_to_ontology_entity)
            .optional()
            .map_err(|e| KurultaiError::Store(format!("get_ontology_entity: {e}")))
    }

    async fn list_ontology_entities(&self, limit: usize) -> Result<Vec<OntologyEntity>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, name, atom_id, attributes_json FROM ontology_entities ORDER BY id LIMIT ?1",
            )
            .map_err(|e| KurultaiError::Store(format!("list_ontology_entities prepare: {e}")))?;
        let rows = stmt
            .query_map([limit as i64], row_to_ontology_entity)
            .map_err(|e| KurultaiError::Store(format!("list_ontology_entities query: {e}")))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KurultaiError::Store(format!("list_ontology_entities collect: {e}")))
    }

    async fn upsert_ontology_link(&self, l: &OntologyLink) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            r#"
            INSERT INTO ontology_links (id, from_id, to_id, rel, confidence, status, actor)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(from_id, to_id, rel) DO UPDATE SET
                confidence = excluded.confidence,
                actor = excluded.actor,
                status = excluded.status
            "#,
            params![
                l.id,
                l.from_id,
                l.to_id,
                l.rel.as_str(),
                l.confidence as f64,
                l.status,
                l.actor
            ],
        )
        .map_err(|e| KurultaiError::Store(format!("upsert_ontology_link: {e}")))?;
        Ok(())
    }

    async fn list_ontology_links(&self, endpoint: Option<&str>) -> Result<Vec<OntologyLink>> {
        let conn = self.lock()?;
        let mut out = Vec::new();
        match endpoint {
            None => {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, from_id, to_id, rel, confidence, status, actor FROM ontology_links ORDER BY id",
                    )
                    .map_err(|e| KurultaiError::Store(format!("list_ontology_links prepare: {e}")))?;
                let rows = stmt
                    .query_map([], row_to_ontology_link_raw)
                    .map_err(|e| KurultaiError::Store(format!("list_ontology_links query: {e}")))?;
                for row in rows {
                    let raw = row.map_err(|e| {
                        KurultaiError::Store(format!("list_ontology_links row: {e}"))
                    })?;
                    if let Some(link) = parse_ontology_link(raw) {
                        out.push(link);
                    }
                }
            }
            Some(id) => {
                let mut stmt = conn
                    .prepare(
                        r#"
                        SELECT l.id, l.from_id, l.to_id, l.rel, l.confidence, l.status, l.actor
                        FROM ontology_links l
                        LEFT JOIN ontology_entities e_from ON e_from.id = l.from_id
                        LEFT JOIN ontology_entities e_to ON e_to.id = l.to_id
                        WHERE l.from_id = ?1 OR l.to_id = ?1
                           OR e_from.atom_id = ?1 OR e_to.atom_id = ?1
                        ORDER BY l.id
                        "#,
                    )
                    .map_err(|e| {
                        KurultaiError::Store(format!("list_ontology_links endpoint prepare: {e}"))
                    })?;
                let rows = stmt
                    .query_map([id], row_to_ontology_link_raw)
                    .map_err(|e| {
                        KurultaiError::Store(format!("list_ontology_links endpoint query: {e}"))
                    })?;
                for row in rows {
                    let raw = row.map_err(|e| {
                        KurultaiError::Store(format!("list_ontology_links endpoint row: {e}"))
                    })?;
                    if let Some(link) = parse_ontology_link(raw) {
                        out.push(link);
                    }
                }
            }
        }
        Ok(out)
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

fn row_to_ontology_entity(row: &rusqlite::Row<'_>) -> rusqlite::Result<OntologyEntity> {
    let attrs_raw: String = row.get(4)?;
    let attributes = serde_json::from_str(&attrs_raw).unwrap_or_else(|_| serde_json::json!({}));
    Ok(OntologyEntity {
        id: row.get(0)?,
        kind: row.get(1)?,
        name: row.get(2)?,
        atom_id: row.get(3)?,
        attributes,
    })
}

struct OntologyLinkRaw {
    id: String,
    from_id: String,
    to_id: String,
    rel: String,
    confidence: f64,
    status: String,
    actor: String,
}

fn row_to_ontology_link_raw(row: &rusqlite::Row<'_>) -> rusqlite::Result<OntologyLinkRaw> {
    Ok(OntologyLinkRaw {
        id: row.get(0)?,
        from_id: row.get(1)?,
        to_id: row.get(2)?,
        rel: row.get(3)?,
        confidence: row.get(4)?,
        status: row.get(5)?,
        actor: row.get(6)?,
    })
}

fn parse_ontology_link(raw: OntologyLinkRaw) -> Option<OntologyLink> {
    let rel = match OntologyLinkType::parse(&raw.rel) {
        Some(rel) => rel,
        None => {
            tracing::warn!(rel = %raw.rel, id = %raw.id, "skipping ontology link with unknown rel");
            return None;
        }
    };
    Some(OntologyLink {
        id: raw.id,
        from_id: raw.from_id,
        to_id: raw.to_id,
        rel,
        confidence: raw.confidence as f32,
        status: raw.status,
        actor: raw.actor,
    })
}

/// Build a safe FTS5 MATCH query from free text (AND of quoted tokens).
fn sanitize_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|t| t.chars().any(|c| c.is_alphanumeric()))
        .filter_map(|t| {
            let cleaned: String = t
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if STOPWORDS.contains(&cleaned.to_ascii_lowercase().as_str()) {
                return None;
            }
            Some(format!("\"{cleaned}\""))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "been", "by", "for", "from", "has", "have", "he",
    "in", "is", "it", "its", "of", "on", "or", "that", "the", "to", "was", "what", "with", "who",
    "how", "why", "where", "when", "does", "do", "did", "can", "could", "would", "should", "will",
    "shall", "this", "these", "those", "i", "you", "we", "they",
];

fn load_atom_by_id(conn: &Connection, id: &str) -> Result<Option<KnowledgeAtom>> {
    let sql = format!("SELECT {} FROM knowledge_atoms WHERE id = ?1", ATOM_COLUMNS);
    let mut atom = conn
        .query_row(&sql, [id], row_to_atom)
        .optional()
        .map_err(|e| KurultaiError::Store(format!("load_atom_by_id: {e}")))?;
    if let Some(ref mut a) = atom {
        a.soft_labels = load_soft_labels(conn, &a.id)?;
    }
    Ok(atom)
}

fn load_atom_by_source_id(
    conn: &Connection,
    source: &str,
    source_id: &str,
) -> Result<Option<KnowledgeAtom>> {
    let sql = format!(
        "SELECT {} FROM knowledge_atoms WHERE source = ?1 AND source_id = ?2 LIMIT 1",
        ATOM_COLUMNS
    );
    let mut atom = conn
        .query_row(&sql, params![source, source_id], row_to_atom)
        .optional()
        .map_err(|e| KurultaiError::Store(format!("load_atom_by_source_id: {e}")))?;
    if let Some(ref mut a) = atom {
        a.soft_labels = load_soft_labels(conn, &a.id)?;
    }
    Ok(atom)
}

fn load_soft_labels(conn: &Connection, atom_id: &str) -> Result<Vec<SoftLabel>> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT v.id, v.name, s.score, v.aliases_json
            FROM atom_soft_labels s
            JOIN label_vocab v ON v.id = s.label_id
            WHERE s.atom_id = ?1
            ORDER BY s.score DESC, v.name COLLATE NOCASE
            "#,
        )
        .map_err(|e| KurultaiError::Store(format!("load_soft_labels prepare: {e}")))?;
    let rows = stmt
        .query_map([atom_id], |row| {
            let aliases_json: String = row.get(3)?;
            let aliases: Vec<String> = serde_json::from_str(&aliases_json).unwrap_or_default();
            Ok(SoftLabel {
                label_id: row.get(0)?,
                name: row.get(1)?,
                score: row.get::<_, f64>(2)? as f32,
                aliases,
            })
        })
        .map_err(|e| KurultaiError::Store(format!("load_soft_labels query: {e}")))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| KurultaiError::Store(format!("load_soft_labels collect: {e}")))
}

fn attach_soft_labels(conn: &Connection, atoms: &mut [KnowledgeAtom]) -> Result<()> {
    for atom in atoms.iter_mut() {
        atom.soft_labels = load_soft_labels(conn, &atom.id)?;
    }
    Ok(())
}

fn ensure_label_id(conn: &Connection, name: &str, aliases: &[String]) -> Result<i64> {
    let name = name.trim();
    if name.is_empty() {
        return Err(KurultaiError::Store("soft label name empty".into()));
    }
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM label_vocab WHERE name = ?1 COLLATE NOCASE",
            [name],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| KurultaiError::Store(format!("label_vocab lookup: {e}")))?;
    if let Some(id) = existing {
        if !aliases.is_empty() {
            let aliases_json = serde_json::to_string(aliases)
                .map_err(|e| KurultaiError::Store(format!("aliases serialize: {e}")))?;
            conn.execute(
                "UPDATE label_vocab SET aliases_json = ?1 WHERE id = ?2",
                params![aliases_json, id],
            )
            .map_err(|e| KurultaiError::Store(format!("label_vocab aliases update: {e}")))?;
        }
        return Ok(id);
    }
    let aliases_json = serde_json::to_string(aliases)
        .map_err(|e| KurultaiError::Store(format!("aliases serialize: {e}")))?;
    conn.execute(
        "INSERT INTO label_vocab (name, aliases_json) VALUES (?1, ?2)",
        params![name, aliases_json],
    )
    .map_err(|e| KurultaiError::Store(format!("label_vocab insert: {e}")))?;
    Ok(conn.last_insert_rowid())
}

fn replace_soft_labels(conn: &Connection, atom_id: &str, labels: &[SoftLabel]) -> Result<()> {
    let normalized = normalize_soft_labels(labels);
    conn.execute("DELETE FROM atom_soft_labels WHERE atom_id = ?1", [atom_id])
        .map_err(|e| KurultaiError::Store(format!("soft_labels delete: {e}")))?;
    for label in &normalized {
        let id = ensure_label_id(conn, &label.name, &label.aliases)?;
        conn.execute(
            "INSERT OR REPLACE INTO atom_soft_labels (atom_id, label_id, score) VALUES (?1, ?2, ?3)",
            params![atom_id, id, label.score as f64],
        )
        .map_err(|e| KurultaiError::Store(format!("soft_labels insert: {e}")))?;
    }
    Ok(())
}

fn embedding_f32s_from_blob(bytes: &[u8]) -> Result<Vec<f32>> {
    if !bytes.len().is_multiple_of(4) {
        return Err(KurultaiError::Store(format!(
            "embedding blob length {} not divisible by 4",
            bytes.len()
        )));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4);
    let (chunks, remainder) = bytes.as_chunks();
    debug_assert!(remainder.is_empty());
    for chunk in chunks {
        out.push(f32::from_le_bytes(*chunk));
    }
    Ok(out)
}

fn row_to_atom(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeAtom> {
    let tags_json: String = row.get(8)?;
    let metadata_json: String = row.get(11)?;
    let source_updated_at: String = row.get(9)?;
    let indexed_at: String = row.get(10)?;
    let trust_lane: String = row.get(12)?;
    let quarantine_reason: Option<String> = row.get(13)?;
    let last_accessed_raw: String = row.get(14).unwrap_or_default();
    let visibility_raw: String = row.get(15).unwrap_or_else(|_| "personal".into());
    let corpus_tier_raw: String = row.get(16).unwrap_or_else(|_| "public".into());
    let visibility_labels_json: String = row.get(17).unwrap_or_else(|_| "[]".into());

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let metadata: HashMap<String, String> =
        serde_json::from_str(&metadata_json).unwrap_or_default();
    let indexed = parse_dt(&indexed_at);
    let last_accessed_at = if last_accessed_raw.is_empty() {
        indexed
    } else {
        parse_dt(&last_accessed_raw)
    };

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
        indexed_at: indexed,
        last_accessed_at,
        embedding: None, // not loaded on read path by default (token budget)
        metadata,
        trust_lane: TrustLane::parse(&trust_lane),
        quarantine_reason,
        soft_labels: Vec::new(),
        corpus_tier: CorpusTier::parse(&corpus_tier_raw),
        visibility_labels: serde_json::from_str(&visibility_labels_json).unwrap_or_default(),
        visibility: VisibilityScope::parse(&visibility_raw),
    })
}

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// ── Ingestion staging implementations ──────────────────────────────────────

impl SqliteVecStore {
    /// Record a new ingestion job with status `'pending'`; returns new row `id`.
    fn record_ingestion_start_sync(
        conn: &Connection,
        batch_id: &str,
        source: &str,
        file_path: &str,
    ) -> Result<i64> {
        conn.execute(
            "INSERT INTO ingestion_jobs (batch_id, source, file_path, status) \
             VALUES (?1, ?2, ?3, 'pending')",
            params![batch_id, source, file_path],
        )
        .map_err(|e| KurultaiError::Store(format!("record_ingestion_start insert: {e}")))?;
        let id = conn.last_insert_rowid();
        Ok(id)
    }

    /// Mark job completed or failed; sets `completed_at = datetime('now')`.
    fn record_ingestion_finish_sync(
        conn: &Connection,
        job_id: i64,
        atoms_count: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let (status, err_msg) = match error_message {
            Some(msg) => ("failed", Some(msg)),
            None => ("completed", None),
        };
        conn.execute(
            "UPDATE ingestion_jobs \
             SET status = ?1, atoms_count = ?2, error_message = ?3, \
                 completed_at = datetime('now') \
             WHERE id = ?4",
            params![status, atoms_count, err_msg, job_id],
        )
        .map_err(|e| KurultaiError::Store(format!("record_ingestion_finish update: {e}")))?;
        Ok(())
    }

    /// Query all rows with `status = 'pending'`.
    fn list_pending_ingestion_jobs_sync(conn: &Connection) -> Result<Vec<IngestionJob>> {
        let mut stmt = conn
            .prepare(
                "SELECT id, batch_id, source, file_path, status, atoms_count, \
                        error_message, created_at, completed_at \
                 FROM ingestion_jobs WHERE status = 'pending' ORDER BY id ASC",
            )
            .map_err(|e| {
                KurultaiError::Store(format!("list_pending_ingestion_jobs prepare: {e}"))
            })?;
        let rows = stmt
            .query_map([], |row| {
                Ok(IngestionJob {
                    id: row.get(0)?,
                    batch_id: row.get(1)?,
                    source: row.get(2)?,
                    file_path: row.get(3)?,
                    status: row.get(4)?,
                    atoms_count: row.get(5)?,
                    error_message: row.get(6)?,
                    created_at: row.get(7)?,
                    completed_at: row.get(8)?,
                })
            })
            .map_err(|e| KurultaiError::Store(format!("list_pending_ingestion_jobs query: {e}")))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| KurultaiError::Store(format!("list_pending_ingestion_jobs collect: {e}")))
    }
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
            soft_labels: vec![],
            source_updated_at: Utc::now(),
            indexed_at: Utc::now(),
            embedding: emb,
            metadata: HashMap::new(),
            ..Default::default()
        }
    }

    fn temp_store(dim: usize) -> SqliteVecStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "kurultai-store-test-{}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0),
            N.fetch_add(1, Ordering::Relaxed)
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

        let hits = store
            .fts_search("database migration", 10, SearchFilter::default())
            .await
            .unwrap();
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
            .fts_search_ids("database migration", 10, SearchFilter::default())
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
            .vector_search(&[0.85, 0.85, 0.85, 0.85], 2, SearchFilter::default())
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
        let hits = store
            .vector_search(&[0.5, 0.5, 0.5, 0.5], 1, SearchFilter::default())
            .await
            .unwrap();
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
        let hits = store
            .vector_search(&[0.7, 0.1, 0.1, 0.1], 5, SearchFilter::default())
            .await
            .unwrap();
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
        let hits = store
            .vector_search(&[0.1, 0.1, 0.1, 0.1], 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(
            hits.is_empty(),
            "zero vectors must not appear in vec search"
        );
        // Still in FTS / count
        assert_eq!(store.count().await.unwrap(), 1);
        assert!(!store
            .fts_search("zero embed", 5, SearchFilter::default())
            .await
            .unwrap()
            .is_empty());
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
        assert!(store
            .fts_search("delete", 5, SearchFilter::default())
            .await
            .unwrap()
            .is_empty());
        assert!(store
            .vector_search(&[0.2, 0.2, 0.2, 0.2], 5, SearchFilter::default())
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

    #[tokio::test]
    async fn list_atoms_orders_newest_first_and_respects_limit() {
        let store = temp_store(4);
        let mut oldest = sample_atom("oldest", "Oldest", "oldest content", None);
        let mut middle = sample_atom("middle", "Middle", "middle content", None);
        let mut newest = sample_atom("newest", "Newest", "newest content", None);
        oldest.indexed_at = Utc::now() - chrono::Duration::hours(2);
        middle.indexed_at = Utc::now() - chrono::Duration::hours(1);
        newest.indexed_at = Utc::now();
        store.upsert(&oldest).await.unwrap();
        store.upsert(&middle).await.unwrap();
        store.upsert(&newest).await.unwrap();

        let all = store.list_atoms_sync(10, SearchFilter::default()).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].title, "Newest");
        assert_eq!(all[1].title, "Middle");
        assert_eq!(all[2].title, "Oldest");

        let limited = store.list_atoms_sync(2, SearchFilter::default()).unwrap();
        assert_eq!(limited.len(), 2);
        assert_eq!(limited[0].title, "Newest");
        assert_eq!(limited[1].title, "Middle");
    }

    #[test]
    fn migration_v4_adds_trust_lane_audit_and_merge_tables() {
        let store = temp_store(4);
        let conn = store.lock().unwrap();
        let version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, migrations::CURRENT_SCHEMA_VERSION);

        let has_col = |name: &str| -> bool {
            let mut stmt = conn.prepare("PRAGMA table_info(knowledge_atoms)").unwrap();
            let cols: Vec<String> = stmt
                .query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .map(|c| c.unwrap())
                .collect();
            cols.iter().any(|c| c == name)
        };
        assert!(has_col("trust_lane"));
        assert!(has_col("quarantine_reason"));
        assert!(has_col("visibility"));

        let index_count = |name: &str| -> i32 {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                [name],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(index_count("idx_atoms_indexed_at"), 1);
        assert_eq!(index_count("idx_atoms_trust_lane"), 1);
        assert_eq!(index_count("idx_atoms_hash_trusted"), 1);
        assert_eq!(index_count("idx_atoms_visibility"), 1);

        let table_count = |name: &str| -> i32 {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [name],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(table_count("quality_audit"), 1);
        assert_eq!(table_count("merge_candidates"), 1);
        assert_eq!(table_count("ontology_entities"), 1);
        assert_eq!(table_count("ontology_links"), 1);
        let class_n: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM ontology_entities WHERE kind = 'class'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(class_n, 6);
    }

    #[tokio::test]
    async fn trust_lane_and_quarantine_reason_round_trip() {
        let store = temp_store(4);
        let mut atom = sample_atom("q-rt", "Q", "quarantine round trip body", None);
        atom.trust_lane = TrustLane::Quarantine;
        atom.quarantine_reason = Some("untagged".into());
        store.upsert(&atom).await.unwrap();
        let loaded = store.get("q-rt").await.unwrap().unwrap();
        assert_eq!(loaded.trust_lane, TrustLane::Quarantine);
        assert_eq!(loaded.quarantine_reason.as_deref(), Some("untagged"));
    }

    #[tokio::test]
    async fn fts_vector_list_exclude_quarantine_by_default() {
        let store = temp_store(4);
        let trusted = sample_atom(
            "t-lane",
            "Trusted Lane",
            "LANEFILTERTOKEN trusted body",
            Some(vec![0.9, 0.1, 0.0, 0.0]),
        );
        let mut quarantine = sample_atom(
            "q-lane",
            "Quarantine Lane",
            "LANEFILTERTOKEN quarantine body",
            Some(vec![0.85, 0.15, 0.0, 0.0]),
        );
        quarantine.trust_lane = TrustLane::Quarantine;
        quarantine.quarantine_reason = Some("untagged".into());
        store.upsert(&trusted).await.unwrap();
        store.upsert(&quarantine).await.unwrap();

        let fts = store
            .fts_search("LANEFILTERTOKEN", 10, SearchFilter::default())
            .await
            .unwrap();
        assert_eq!(fts.len(), 1);
        assert_eq!(fts[0].0.id, "t-lane");

        let fts_all = store
            .fts_search("LANEFILTERTOKEN", 10, SearchFilter::trusted(false))
            .await
            .unwrap();
        assert_eq!(fts_all.len(), 2);

        let vec_hits = store
            .vector_search(&[0.9, 0.1, 0.0, 0.0], 5, SearchFilter::default())
            .await
            .unwrap();
        assert!(vec_hits.iter().all(|(a, _)| a.id == "t-lane"));

        let listed = store.list_atoms(10, SearchFilter::default()).await.unwrap();
        assert!(listed.iter().all(|a| a.trust_lane == TrustLane::Trusted));
        assert!(!listed.iter().any(|a| a.id == "q-lane"));
    }

    #[tokio::test]
    async fn upsert_preserves_indexed_at_when_hash_unchanged() {
        let store = temp_store(4);
        let mut atom = sample_atom("a1", "V1", "same content", None);
        let first_indexed = Utc::now() - chrono::Duration::days(10);
        atom.indexed_at = first_indexed;
        atom.last_accessed_at = first_indexed;
        store.upsert(&atom).await.unwrap();

        atom.title = "V2".into();
        atom.indexed_at = Utc::now();
        atom.last_accessed_at = Utc::now();
        store.upsert(&atom).await.unwrap();

        let loaded = store.get("a1").await.unwrap().unwrap();
        assert_eq!(loaded.title, "V2");
        assert_eq!(
            loaded.indexed_at.timestamp(),
            first_indexed.timestamp(),
            "unchanged content must not refresh indexed_at"
        );
        assert_eq!(
            loaded.last_accessed_at.timestamp(),
            first_indexed.timestamp(),
            "re-index must not overwrite last_accessed_at"
        );
    }

    #[tokio::test]
    async fn memory_tiers_touch_and_graph() {
        let store = temp_store(4);
        let now = Utc::now();
        let mut hot = sample_atom("hot1", "Hot", "recent access", None);
        hot.indexed_at = now - chrono::Duration::days(30);
        hot.last_accessed_at = now - chrono::Duration::days(1);
        let mut warm = sample_atom("warm1", "Warm", "mid age", None);
        warm.indexed_at = now - chrono::Duration::days(40);
        warm.last_accessed_at = now - chrono::Duration::days(40);
        let mut cold = sample_atom("cold1", "Cold", "ancient", None);
        cold.indexed_at = now - chrono::Duration::days(200);
        cold.last_accessed_at = now - chrono::Duration::days(200);
        store.upsert(&hot).await.unwrap();
        store.upsert(&warm).await.unwrap();
        store.upsert(&cold).await.unwrap();

        let (h, w, c) = store.count_by_tier(TierPolicy::default()).await.unwrap();
        assert_eq!((h, w, c), (1, 1, 1));

        let hot_nodes = store
            .list_graph_nodes(
                Some(MemoryTier::Hot),
                10,
                SearchFilter::default(),
                TierPolicy::default(),
            )
            .await
            .unwrap();
        assert_eq!(hot_nodes.len(), 1);
        assert_eq!(hot_nodes[0].id, "hot1");
        assert!(hot_nodes[0].summary.is_some());

        let warm_nodes = store
            .list_graph_nodes(
                Some(MemoryTier::Warm),
                10,
                SearchFilter::default(),
                TierPolicy::default(),
            )
            .await
            .unwrap();
        assert_eq!(warm_nodes.len(), 1);
        assert!(warm_nodes[0].summary.is_none());

        store.touch_access("cold1").await.unwrap();
        let loaded = store.get("cold1").await.unwrap().unwrap();
        assert!(loaded.last_accessed_at > cold.last_accessed_at);
        let (h2, _, c2) = store.count_by_tier(TierPolicy::default()).await.unwrap();
        assert_eq!(h2, 2);
        assert_eq!(c2, 0);
    }

    #[tokio::test]
    async fn soft_labels_round_trip_and_preserve_when_empty() {
        use crate::types::SoftLabel;
        let store = temp_store(4);
        let mut atom = sample_atom("sl1", "Soft", "kubernetes deploy runbook", None);
        atom.soft_labels = vec![
            SoftLabel {
                label_id: 0,
                name: "kubernetes".into(),
                score: 0.9,
                aliases: vec!["k8s".into()],
            },
            SoftLabel {
                label_id: 0,
                name: "ops".into(),
                score: 0.4,
                aliases: vec![],
            },
        ];
        store.upsert(&atom).await.unwrap();
        let loaded = store.get("sl1").await.unwrap().unwrap();
        assert_eq!(loaded.soft_labels.len(), 2);
        assert_eq!(loaded.soft_labels[0].name, "kubernetes");
        assert!((loaded.soft_labels[0].score - 0.9).abs() < 1e-5);
        assert_eq!(loaded.soft_labels[0].aliases, vec!["k8s".to_string()]);
        assert!(loaded.soft_labels[0].label_id > 0);

        // Empty soft_labels on upsert must preserve existing scores.
        let mut again = loaded.clone();
        again.soft_labels.clear();
        again.content = "kubernetes deploy runbook".into(); // same hash path
        store.upsert(&again).await.unwrap();
        let kept = store.get("sl1").await.unwrap().unwrap();
        assert_eq!(kept.soft_labels.len(), 2);
    }

    #[tokio::test]
    async fn soft_label_vocab_shared_across_atoms() {
        use crate::types::SoftLabel;
        let store = temp_store(4);
        let mut a = sample_atom("a", "A", "alpha content about rust", None);
        a.soft_labels = vec![SoftLabel {
            label_id: 0,
            name: "rust".into(),
            score: 0.8,
            aliases: vec![],
        }];
        let mut b = sample_atom("b", "B", "beta content about rust", None);
        b.soft_labels = vec![SoftLabel {
            label_id: 0,
            name: "Rust".into(), // case-insensitive vocab
            score: 0.5,
            aliases: vec![],
        }];
        store.upsert(&a).await.unwrap();
        store.upsert(&b).await.unwrap();
        let la = store.get("a").await.unwrap().unwrap();
        let lb = store.get("b").await.unwrap().unwrap();
        assert_eq!(la.soft_labels[0].label_id, lb.soft_labels[0].label_id);
    }

    #[tokio::test]
    async fn visibility_defaults_personal_and_round_trips() {
        let store = temp_store(4);
        let atom = sample_atom("vis-default", "Vis", "default personal body", None);
        store.upsert(&atom).await.unwrap();
        let loaded = store.get("vis-default").await.unwrap().unwrap();
        assert_eq!(loaded.visibility, VisibilityScope::Personal);

        let mut team = sample_atom("vis-team", "Team", "explicit team body", None);
        team.visibility = VisibilityScope::Team;
        store.upsert(&team).await.unwrap();
        let loaded_team = store.get("vis-team").await.unwrap().unwrap();
        assert_eq!(loaded_team.visibility, VisibilityScope::Team);

        let mut company = sample_atom("vis-co", "Co", "company body here", None);
        company.visibility = VisibilityScope::Company;
        store.upsert(&company).await.unwrap();
        assert_eq!(
            store.get("vis-co").await.unwrap().unwrap().visibility,
            VisibilityScope::Company
        );
    }

    #[tokio::test]
    async fn visibility_does_not_filter_solo_fts_search() {
        // AE1 / KTD4: no hub → local search returns all lanes of visibility.
        let store = temp_store(4);
        let personal = sample_atom("p1", "Alpha personal", "visibility search personal", None);
        let mut team = sample_atom("t1", "Alpha team", "visibility search team", None);
        team.visibility = VisibilityScope::Team;
        store.upsert(&personal).await.unwrap();
        store.upsert(&team).await.unwrap();

        let hits = store
            .fts_search("visibility search", 10, SearchFilter::default())
            .await
            .unwrap();
        let ids: Vec<_> = hits.iter().map(|(a, _)| a.id.as_str()).collect();
        assert!(ids.contains(&"p1"), "personal atom missing: {ids:?}");
        assert!(
            ids.contains(&"t1"),
            "team atom filtered out of solo search: {ids:?}"
        );
    }

    #[tokio::test]
    async fn open_hub_store_requires_hub_flag() {
        let prev = std::env::var("KURULTAI_FEATURE_HUB").ok();
        std::env::set_var("KURULTAI_FEATURE_HUB", "0");
        let err = match super::open_hub_store("postgres://localhost/kurultai", 4).await {
            Ok(_) => panic!("expected hub-flag error"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("KURULTAI_FEATURE_HUB"),
            "unexpected error: {msg}"
        );
        match prev {
            Some(v) => std::env::set_var("KURULTAI_FEATURE_HUB", v),
            None => std::env::remove_var("KURULTAI_FEATURE_HUB"),
        }
    }

    #[tokio::test]
    async fn open_hub_store_requires_postgres_feature() {
        if cfg!(feature = "postgres") {
            return;
        }
        let prev = std::env::var("KURULTAI_FEATURE_HUB").ok();
        std::env::set_var("KURULTAI_FEATURE_HUB", "1");
        let err = match super::open_hub_store("postgres://localhost/kurultai", 4).await {
            Ok(_) => panic!("expected --features postgres error"),
            Err(e) => e,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("--features postgres"),
            "unexpected error: {msg}"
        );
        match prev {
            Some(v) => std::env::set_var("KURULTAI_FEATURE_HUB", v),
            None => std::env::remove_var("KURULTAI_FEATURE_HUB"),
        }
    }
}
