//! Issued hub API keys stored in Postgres (HUB-4).

use crate::error::{KurultaiError, Result};
use crate::hashutil::sha256_hex;
use crate::hub::activity::HubActivityStore;
use crate::hub::role::HubRole;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Idempotent upgrade for `hub_api_keys` tables created before roles (#115).
pub(crate) const HUB_KEYS_ADD_ROLE_SQL: &str =
    "ALTER TABLE hub_api_keys ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'admin'";

/// Concurrent first-time DDL (parallel test processes, multiple daemons booting
/// together) can lose a catalog race that is harmless: the other side created it.
fn tolerate_catalog_race(err: sqlx::Error) -> std::result::Result<(), sqlx::Error> {
    if crate::store::postgres::is_benign_catalog_race(&err) {
        Ok(())
    } else {
        Err(err)
    }
}

/// Authenticated hub caller resolved from a bearer token.
#[derive(Debug, Clone)]
pub struct HubPrincipal {
    pub key_id: i64,
    pub agent_id: String,
    pub team_id: String,
    pub key_prefix: String,
    pub role: HubRole,
}

/// Key metadata returned by list — never includes hash or plaintext.
#[derive(Debug, Clone)]
pub struct HubKeyRecord {
    pub id: i64,
    pub key_prefix: String,
    pub agent_id: String,
    pub team_id: String,
    pub role: HubRole,
    pub created_at: String,
    pub revoked_at: Option<String>,
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
        store.migrate().await?;
        HubActivityStore::migrate(store.pool()).await?;
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
                role TEXT NOT NULL DEFAULT 'admin',
                created_at TEXT NOT NULL,
                revoked_at TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .or_else(tolerate_catalog_race)
        .map_err(|e| KurultaiError::Store(format!("hub_api_keys ddl: {e}")))?;

        // #115: keys issued before roles existed could do everything, so they
        // backfill as `admin`; `issue` always sets the role explicitly.
        sqlx::query(HUB_KEYS_ADD_ROLE_SQL)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .or_else(tolerate_catalog_race)
            .map_err(|e| KurultaiError::Store(format!("hub_api_keys role ddl: {e}")))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_hub_api_keys_hash ON hub_api_keys(key_hash)")
            .execute(&self.pool)
            .await
            .map_err(|e| KurultaiError::Store(format!("hub_api_keys idx: {e}")))?;

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

    /// Issue a key; returns `(plaintext_token, record_id)`.
    pub async fn issue(
        &self,
        agent_id: &str,
        team_id: &str,
        role: HubRole,
    ) -> Result<(String, i64)> {
        let token = format!("krt_{}", Uuid::new_v4().simple());
        let hash = sha256_hex(&token);
        let prefix = token.chars().take(12).collect::<String>();
        let created_at = Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO hub_api_keys (key_hash, key_prefix, agent_id, team_id, role, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(&hash)
        .bind(&prefix)
        .bind(agent_id)
        .bind(team_id)
        .bind(role.as_str())
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
            "SELECT id, key_prefix, agent_id, team_id, role, created_at, revoked_at
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
                    role: HubRole::from_stored(
                        &row.try_get::<String, _>("role")
                            .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    ),
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    revoked_at: row.try_get("revoked_at").ok(),
                })
            })
            .collect()
    }

    pub async fn resolve_token(&self, token: &str) -> Result<Option<HubPrincipal>> {
        let hash = sha256_hex(token);
        let row = sqlx::query(
            "SELECT id, key_prefix, agent_id, team_id, role FROM hub_api_keys
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
            role: HubRole::from_stored(&row.try_get::<String, _>("role").expect("role column")),
        }))
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
        let (token, key_id) = store
            .issue("alice", "eng", HubRole::Contributor)
            .await
            .unwrap();
        let principal = store.resolve_token(&token).await.unwrap().unwrap();
        assert_eq!(principal.agent_id, "alice");
        assert_eq!(principal.team_id, "eng");
        assert_eq!(principal.role, HubRole::Contributor);
        let rec = store
            .list()
            .await
            .unwrap()
            .into_iter()
            .find(|r| r.id == key_id)
            .expect("issued key listed");
        assert_eq!(rec.role, HubRole::Contributor);
        assert!(store
            .revoke_by_prefix(&token.chars().take(12).collect::<String>())
            .await
            .unwrap());
        assert!(store.resolve_token(&token).await.unwrap().is_none());
    }
}
