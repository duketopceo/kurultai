//! Append-only hub write activity log (HUB-4).

use crate::error::{KurultaiError, Result};
use chrono::Utc;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone, serde::Serialize)]
pub struct HubActivityEntry {
    pub id: i64,
    pub at: String,
    pub agent_id: String,
    pub team_id: String,
    pub namespace: String,
    pub transport: String,
    pub reason: Option<String>,
    pub atom_id: Option<String>,
}

pub struct HubActivityStore {
    pool: PgPool,
}

impl HubActivityStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(pool: &PgPool) -> Result<()> {
        let mut conn = pool
            .acquire()
            .await
            .map_err(|e| KurultaiError::Store(format!("hub activity acquire: {e}")))?;
        Self::migrate_conn(&mut conn).await
    }

    pub async fn migrate_conn(conn: &mut sqlx::postgres::PgConnection) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS hub_activity (
                id BIGSERIAL PRIMARY KEY,
                at TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                team_id TEXT NOT NULL,
                namespace TEXT NOT NULL,
                transport TEXT NOT NULL,
                reason TEXT,
                atom_id TEXT
            )
            "#,
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub_activity ddl: {e}")))?;

        sqlx::query("DROP INDEX IF EXISTS idx_hub_activity_at")
            .execute(&mut *conn)
            .await
            .map_err(|e| KurultaiError::Store(format!("hub_activity drop old idx: {e}")))?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_hub_activity_at ON hub_activity(at DESC, id DESC)",
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub_activity idx: {e}")))?;

        Ok(())
    }

    pub async fn append(
        &self,
        agent_id: &str,
        team_id: &str,
        namespace: &str,
        transport: &str,
        reason: Option<&str>,
        atom_id: Option<&str>,
    ) -> Result<()> {
        if let Some(r) = reason {
            if r.chars().count() > 200 {
                return Err(KurultaiError::config(
                    "activity reason must be at most 200 characters",
                ));
            }
        }
        let at = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO hub_activity (at, agent_id, team_id, namespace, transport, reason, atom_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&at)
        .bind(agent_id)
        .bind(team_id)
        .bind(namespace)
        .bind(transport)
        .bind(reason)
        .bind(atom_id)
        .execute(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub activity append: {e}")))?;
        Ok(())
    }

    pub async fn list(&self, limit: usize) -> Result<Vec<HubActivityEntry>> {
        let rows = sqlx::query(
            "SELECT id, at, agent_id, team_id, namespace, transport, reason, atom_id
             FROM hub_activity ORDER BY at DESC, id DESC LIMIT $1",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KurultaiError::Store(format!("hub activity list: {e}")))?;

        rows.iter()
            .map(|row| {
                Ok(HubActivityEntry {
                    id: row
                        .try_get("id")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    at: row
                        .try_get("at")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    agent_id: row
                        .try_get("agent_id")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    team_id: row
                        .try_get("team_id")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    namespace: row
                        .try_get("namespace")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    transport: row
                        .try_get("transport")
                        .map_err(|e| KurultaiError::Store(e.to_string()))?,
                    reason: row.try_get("reason").ok(),
                    atom_id: row.try_get("atom_id").ok(),
                })
            })
            .collect()
    }
}
