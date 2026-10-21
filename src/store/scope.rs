//! Request-scoped RLS scope for the Postgres hub store (hub-adoption U2, KTD1).
//!
//! One scope travels with a request's task tree via a Tokio task-local:
//! `http` middleware resolves it from the **server-side** `HubPrincipal`
//! (never client input), `PostgresStore` applies it inside a per-call
//! transaction (`SELECT set_config('kurultai.team_id', $1, true)` — `SET`/`SET
//! LOCAL` reject bind params), and RLS re-checks every row underneath the
//! app-level `hub_visibility_sql` filter.
//!
//! Path classification (KTD1):
//!   * member   — resolved `HubPrincipal` ⇒ `team_id` scope; `is_admin` is
//!     never reachable from a member path.
//!   * admin    — `/api/hub/*` reached with operator-class credentials
//!     (static hub key / CF Access — no member principal) ⇒ `is_admin`.
//!   * closed   — no principal on a member path ⇒ no GUCs ⇒ company-only
//!     (fail closed).
//!   * internal — no request scope at all (pipeline, merges, `/ingest`,
//!     `touch_access`, CLI, stdio MCP) ⇒ [`PgScope::INTERNAL`] — `is_admin`
//!     so daemon maintenance isn't RLS-blocked.
//!
//! Defined here (not inside the `postgres` feature) so request plumbing in
//! `http` and `mcp::brain` compiles either way; on SQLite-only builds the
//! scope is set but no store consumes it.

use std::future::Future;

tokio::task_local! {
    /// The current request's RLS scope. Unset outside HTTP member handling —
    /// internal callers read [`PgScope::INTERNAL`] via [`PgScope::current`].
    pub(crate) static PG_REQUEST_SCOPE: PgScope;
}

/// Row-level-security scope applied to each `PostgresStore` transaction.
#[derive(Debug, Clone, Default)]
pub struct PgScope {
    /// `kurultai.team_id` member GUC. `None` ⇒ member rows of every team are
    /// invisible (`team_id = NULL` is never true in the policies).
    pub team_id: Option<String>,
    /// `kurultai.is_admin` GUC. Admin/internal paths only — never derived
    /// from a member `HubPrincipal`.
    pub is_admin: bool,
}

impl PgScope {
    /// Internal/admin scope: `is_admin='on'`, no team restriction.
    ///
    /// Used both for admin paths (`/api/hub/*` + operator credentials) and as
    /// the default outside request handling — pipeline upserts,
    /// `apply_auto_merge`, `mark_superseded`, `touch_access`, `/ingest`, and
    /// the `kurultai` CLI are internal paths (KTD1).
    pub const INTERNAL: Self = Self {
        team_id: None,
        is_admin: true,
    };

    /// Member scope for `team` — sets only the `team_id` GUC.
    pub fn member(team_id: impl Into<String>) -> Self {
        Self {
            team_id: Some(team_id.into()),
            is_admin: false,
        }
    }

    /// Admin scope (no member team) — `is_admin='on'` alone.
    pub fn admin() -> Self {
        Self::INTERNAL
    }

    /// Fail-closed scope: no GUCs at all → `company` rows only.
    pub fn closed() -> Self {
        Self::default()
    }

    /// The scope for this task tree: the request's scope when set, else
    /// [`PgScope::INTERNAL`] — every unset context is a server-internal path
    /// (pipeline/CLI/ingest; KTD1).
    pub fn current() -> Self {
        PG_REQUEST_SCOPE
            .try_with(|s| s.clone())
            .unwrap_or(Self::INTERNAL)
    }

    /// Run `f` with `self` as the task-local scope. Also used to re-enter a
    /// captured scope inside a `tokio::spawn`ed task — task-locals do not
    /// propagate across spawns.
    pub async fn run<Fut, T>(self, f: Fut) -> T
    where
        Fut: Future<Output = T>,
    {
        PG_REQUEST_SCOPE.scope(self, f).await
    }
}
