//! Issued hub API keys stored in Postgres (HUB-4).

use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::hub::activity::HubActivityStore;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Authenticated hub caller resolved from a bearer token.
#[derive(Debug, Clone)]
pub struct HubPrincipal {
    pub key_id: i64,
    pub agent_id: String,
    pub team_id: String,
    pub key_prefix: String,
    /// U4/R17 — minted via `hub key issue --admin` or `POST /api/hub/keys
    /// {is_admin:true}`; required by every `/api/hub/*` admin route and mapped
    /// to `PgScope::admin()` (the `kurultai.is_admin` GUC) by
    /// `http::auth::request_pg_scope`.
    pub is_admin: bool,
}

/// Key metadata returned by list — never includes hash or plaintext.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HubKeyRecord {
    pub id: i64,
    pub key_prefix: String,
    pub agent_id: String,
    pub team_id: String,
    pub is_admin: bool,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

/// One row of the `/api/hub/scopes` census — atom counts grouped by
/// `visibility` + `team_id` (U4). `team_id` is `None` for `company` atoms.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HubScopeCount {
    pub visibility: String,
    pub team_id: Option<String>,
    pub atoms: i64,
}

pub struct HubKeyStore {
    pool: PgPool,
}

impl HubKeyStore {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(database_url)
            .await
            .map_err(|e| KurultaiError::Store(format!("hub keys connect: {e}")))?;
        let store = Self { pool };
        // KTD8 — same gate as `PostgresStore::connect`: under
        // `KURULTAI_PG_SKIP_MIGRATE=1` (hub runtime role `kurultai_app`,
        // no DDL rights) the pool skips `migrate()`; schema comes from the
        // `KURULTAI_MIGRATE_DATABASE_URL` path instead.
        if !crate::store::postgres::skip_migrate_enabled() {
            store.migrate().await?;
            HubActivityStore::migrate(store.pool()).await?;
        }
        Ok(store)
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hub_api_keys (
                id BIGSERIAL PRIMARY KEY,
                key_hash TEXT NOT NULL UNIQUE,
                key_prefix TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                team_id TEXT NOT NULL,
                is_admin BOOLEAN NOT NULL DEFAULT false,
                created_at TEXT NOT NULL,
                revoked_at TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub_api_keys ddl: {e}")))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_hub_api_keys_hash ON hub_api_keys(key_hash)")
            .execute(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("hub_api_keys idx: {e}")))?;

        // U4/R17 — self-heal `is_admin` on DBs created before the flag.
        sqlx::query(crate::store::migrations::PG_HUB_KEYS_ADMIN)
            .execute(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("hub_api_keys is_admin: {e}")))?;

        Ok(())
    }

    pub async fn has_active_keys(&self) -> Result<bool> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM hub_api_keys WHERE revoked_at IS NULL")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| KurultaiError::Store(format!("has_active_keys: {e}")))?;
        Ok(row.0 > 0)
    }

    /// Issue a member key; returns `(plaintext_token, record_id)`. `team_id`
    /// is validated at mint (`store::scope::validate_team_id`, U3/KTD1) — it
    /// later becomes SQL-visible via the `kurultai.team_id` session GUC.
    pub async fn issue(&self, agent_id: &str, team_id: &str) -> Result<(String, i64)> {
        self.issue_admin(agent_id, team_id, false).await
    }

    /// Issue a key with an explicit admin flag (U4/R17). `is_admin=true`
    /// mints the operator credential the `/api/hub/*` routes require —
    /// `hub key issue --admin`, connect-join stays on the member default.
    pub async fn issue_admin(
        &self,
        agent_id: &str,
        team_id: &str,
        is_admin: bool,
    ) -> Result<(String, i64)> {
        crate::store::scope::validate_team_id(team_id).map_err(KurultaiError::Store)?;
        let token = format!("krt_{}", Uuid::new_v4().simple());
        let hash = sha256_hex(&token);
        let prefix = token.chars().take(12).collect::<String>();
        let created_at = Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO hub_api_keys (key_hash, key_prefix, agent_id, team_id, is_admin, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(&hash)
        .bind(&prefix)
        .bind(agent_id)
        .bind(team_id)
        .bind(is_admin)
        .bind(&created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub key issue: {e}")))?;
        Ok((token, row.0))
    }

    pub async fn revoke_by_prefix(&self, prefix: &str) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let updated = sqlx::query(
            "UPDATE hub_api_keys
             SET revoked_at = $1
             WHERE id = (
                 SELECT id FROM hub_api_keys
                 WHERE key_prefix = $2 AND revoked_at IS NULL
                 ORDER BY id ASC
                 LIMIT 1
             )",
        )
        .bind(&now)
        .bind(prefix)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub key revoke: {e}")))?;
        Ok(updated.rows_affected() > 0)
    }

    pub async fn revoke_by_id(&self, id: i64) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let updated = sqlx::query(
            "UPDATE hub_api_keys SET revoked_at = $1 WHERE id = $2 AND revoked_at IS NULL",
        )
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub key revoke id: {e}")))?;
        Ok(updated.rows_affected() > 0)
    }

    pub async fn list(&self) -> Result<Vec<HubKeyRecord>> {
        let rows = sqlx::query(
            "SELECT id, key_prefix, agent_id, team_id, is_admin, created_at, revoked_at
             FROM hub_api_keys ORDER BY created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub key list: {e}")))?;

        rows.iter()
            .map(|row| {
                Ok(HubKeyRecord {
                    id: row
                        .try_get("id")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    key_prefix: row
                        .try_get("key_prefix")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    agent_id: row
                        .try_get("agent_id")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    team_id: row
                        .try_get("team_id")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    is_admin: row
                        .try_get("is_admin")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    revoked_at: row.try_get("revoked_at").ok(),
                })
            })
            .collect()
    }

    /// Distinct `team_id`s ever minted — the `/connect` picker's existing-
    /// team suggestions (U3). Revoked keys' teams stay listed: a team
    /// doesn't disappear when a key does.
    pub async fn list_teams(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT DISTINCT team_id FROM hub_api_keys ORDER BY team_id")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| KurultaiError::Store(format!("hub list_teams: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// `connect_lazy` pool for handler unit tests — lets tests construct a
    /// `HubGate` with `key_store = Some(..)` without a live Postgres (any
    /// query still fails fast if reached).
    #[cfg(test)]
    pub(crate) fn lazy_for_test() -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://kurultai:unused@127.0.0.1:1/kurultai_test")
            .expect("lazy pool does not connect");
        Self { pool }
    }

    pub async fn resolve_token(&self, token: &str) -> Result<Option<HubPrincipal>> {
        let hash = sha256_hex(token);
        let row = sqlx::query(
            "SELECT id, key_prefix, agent_id, team_id, is_admin FROM hub_api_keys
             WHERE key_hash = $1 AND revoked_at IS NULL",
        )
        .bind(&hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("resolve hub key: {e}")))?;

        Ok(row.map(|row| HubPrincipal {
            key_id: row.try_get("id").expect("id column"),
            key_prefix: row.try_get("key_prefix").expect("key_prefix column"),
            agent_id: row.try_get("agent_id").expect("agent_id column"),
            team_id: row.try_get("team_id").expect("team_id column"),
            is_admin: row.try_get("is_admin").expect("is_admin column"),
        }))
    }

    /// `/api/hub/scopes` census (U4, R10-over-HTTP): atom counts grouped by
    /// `visibility` + `team_id`. Runs inside a transaction with the
    /// request's [`crate::store::scope::PgScope`] applied — the route is
    /// admin-gated, but the RLS SELECT policy underneath still decides
    /// which rows the GROUP BY can see (admin → every scope).
    pub async fn scope_census(&self) -> Result<Vec<HubScopeCount>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| KurultaiError::Store(format!("scope census begin: {e}")))?;
        crate::store::postgres::apply_pg_scope(&mut tx, &crate::store::scope::PgScope::current())
            .await?;
        let rows = sqlx::query(
            "SELECT visibility, team_id, COUNT(*) AS atoms
             FROM knowledge_atoms GROUP BY visibility, team_id
             ORDER BY visibility, team_id",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| KurultaiError::Store(format!("scope census: {e}")))?;
        let counts = rows
            .iter()
            .map(|row| {
                Ok(HubScopeCount {
                    visibility: row
                        .try_get("visibility")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    team_id: row.try_get("team_id").ok(),
                    atoms: row
                        .try_get("atoms")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                })
            })
            .collect::<Result<Vec<HubScopeCount>>>()?;
        tx.commit()
            .await
            .map_err(|e| KurultaiError::Store(format!("scope census commit: {e}")))?;
        Ok(counts)
    }
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;

    async fn test_keys() -> Option<HubKeyStore> {
        match std::env::var("KURULTAI_TEST_DATABASE_URL") {
            Ok(url) if !url.is_empty() => {
                Some(HubKeyStore::connect(&url).await.expect("connect keys"))
            }
            _ if std::env::var("CI").is_ok() => {
                panic!("KURULTAI_TEST_DATABASE_URL must be set in CI for hub key tests");
            }
            _ => None,
        }
    }

    #[tokio::test]
    async fn issue_resolve_and_revoke_hub_key() {
        let Some(store) = test_keys().await else {
            return;
        };
        let (token, _id) = store.issue("alice", "eng").await.unwrap();
        let principal = store.resolve_token(&token).await.unwrap().unwrap();
        assert_eq!(principal.agent_id, "alice");
        assert_eq!(principal.team_id, "eng");
        assert!(
            !principal.is_admin,
            "member mint must resolve is_admin=false"
        );
        assert!(store
            .revoke_by_prefix(&token.chars().take(12).collect::<String>())
            .await
            .unwrap());
        assert!(store.resolve_token(&token).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn issue_admin_flag_roundtrips() {
        let Some(store) = test_keys().await else {
            return;
        };
        let agent = format!("admin-{}", &Uuid::new_v4().simple().to_string()[..8]);
        let (token, _id) = store.issue_admin(&agent, "ops", true).await.unwrap();
        let principal = store.resolve_token(&token).await.unwrap().unwrap();
        assert!(principal.is_admin, "admin mint must resolve is_admin=true");
        let record = store
            .list()
            .await
            .unwrap()
            .into_iter()
            .find(|r| r.agent_id == agent)
            .expect("minted admin key lists");
        assert!(record.is_admin);
        assert!(store.revoke_by_id(record.id).await.unwrap());
    }
}
