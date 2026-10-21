//! Hub RLS (plan 2026-10-19-2053 U1, R11/KTD1/KTD8): proves Postgres row-level
//! security enforces team/company scope even if the app-layer
//! `hub_visibility_sql` filter is bypassed — and that the runtime role can't
//! run DDL.
//!
//! CI's `POSTGRES_USER` is a superuser and superusers bypass FORCE RLS
//! unconditionally, so every check here self-provisions the `kurultai_app`
//! (non-owner, NOINHERIT) and `kurultai_migrate` (BYPASSRLS) roles — mirroring
//! `deploy/server-001/init-hub-db.sql` — and runs scoped work on dedicated
//! per-role connections, not `SET ROLE` on the superuser session.

#[cfg(feature = "postgres")]
mod rls {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use kurultai::daemon::DaemonStatus;
    use kurultai::embed::NullEmbedder;
    use kurultai::http::{build_app, HubAuth, HubGate};
    use kurultai::hub::HubKeyStore;
    use kurultai::mcp::brain::BrainService;
    use kurultai::rerank::NullReranker;
    use kurultai::store::postgres::PostgresStore;
    use kurultai::store::scope::PgScope;
    use kurultai::store::{SearchFilter, Store};
    use kurultai::synthesize::ExtractiveSynthesizer;
    use kurultai::testutil::EnvGuard;
    use kurultai::types::VisibilityScope;
    use kurultai::write_policy::{WriteContext, WriteTransport};
    use sqlx::{Connection, PgConnection};
    use std::sync::Arc;
    use tower::ServiceExt;

    const APP_PW: &str = "rls_test_app_pw";
    const MIGRATE_PW: &str = "rls_test_migrate_pw";

    /// Admin (CI superuser) DSN. `None` locally → tests skip; CI always sets it.
    fn test_dsn() -> Option<String> {
        match std::env::var("KURULTAI_TEST_DATABASE_URL") {
            Ok(url) if !url.is_empty() => Some(url),
            _ if std::env::var("CI").is_ok() => {
                panic!("KURULTAI_TEST_DATABASE_URL must be set in CI for --features postgres tests")
            }
            _ => None,
        }
    }

    /// Rewrite `postgres://user[:pw]@host/db` for a different role.
    fn dsn_for(admin: &str, role: &str, password: &str) -> String {
        let (scheme, rest) = admin.split_once("://").expect("dsn scheme");
        let host_db = rest.rsplit_once('@').map(|(_, h)| h).unwrap_or(rest);
        format!("{scheme}://{role}:{password}@{host_db}")
    }

    async fn connect(dsn: &str) -> PgConnection {
        PgConnection::connect(dsn).await.expect("pg connect")
    }

    /// Advisory lock id for role/grant provisioning — distinct from the
    /// migrate lock (87231401). Nextest runs each test in its own process, so
    /// cross-process serialization has to live in the database itself.
    const PROVISION_LOCK: i64 = 87231402;

    /// Idempotent role/grant bootstrap — mirror of
    /// `deploy/server-001/init-hub-db.sql`; keep the grant list in sync with it.
    /// The whole bootstrap sits behind a session-level advisory lock because
    /// `CREATE ROLE` and `GRANT` both race badly across parallel test
    /// processes (`pg_authid` duplicate / "tuple concurrently updated").
    async fn provision(admin_dsn: &str) {
        let mut admin = connect(admin_dsn).await;
        sqlx::query("SELECT pg_advisory_lock($1)")
            .bind(PROVISION_LOCK)
            .execute(&mut admin)
            .await
            .expect("provision lock");
        let result = provision_locked(&mut admin, admin_dsn).await;
        let _ = sqlx::query("SELECT pg_advisory_unlock($1)")
            .bind(PROVISION_LOCK)
            .execute(&mut admin)
            .await;
        result
    }

    async fn provision_locked(admin: &mut PgConnection, admin_dsn: &str) {
        for (role, attrs, pw) in [
            ("kurultai_migrate", "LOGIN BYPASSRLS", MIGRATE_PW),
            ("kurultai_app", "LOGIN NOINHERIT", APP_PW),
        ] {
            let create = format!("CREATE ROLE {role} {attrs} PASSWORD '{pw}'");
            match sqlx::query(&create).execute(&mut *admin).await {
                Ok(_) => {}
                Err(e)
                    if e.to_string().contains("already exists")
                        || e.to_string().contains("duplicate key") =>
                {
                    // Another provisioner created it first — attributes are
                    // re-asserted below.
                }
                Err(e) => panic!("create role {role}: {e}"),
            }
            let alter = format!("ALTER ROLE {role} WITH {attrs} PASSWORD '{pw}'");
            sqlx::query(&alter)
                .execute(&mut *admin)
                .await
                .expect("alter role");
        }

        // Tables must exist before grants cover them — run the real migration
        // as the superuser (it takes the separate migrate advisory lock on its
        // own connection). Strip a lingering skip-migrate env so the concurrent
        // env-gate test can't suppress this migrate.
        {
            let _env = EnvGuard::remove("KURULTAI_PG_SKIP_MIGRATE");
            PostgresStore::connect(admin_dsn, 4)
                .await
                .expect("migrate as superuser");
        }

        for grant in [
            "GRANT USAGE ON SCHEMA public TO kurultai_app, kurultai_migrate",
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO kurultai_app",
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO kurultai_app",
            "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO kurultai_migrate",
            "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO kurultai_migrate",
        ] {
            sqlx::query(grant)
                .execute(&mut *admin)
                .await
                .expect("grant");
        }
    }

    /// Insert one atom per scope under a unique `run` prefix (plus a vector so
    /// `atoms_vec` scoping is testable). Returns `(prefix, eng_id, sales_id,
    /// company_id)`.
    async fn fixtures(admin_dsn: &str) -> (String, String, String, String) {
        let mut admin = connect(admin_dsn).await;
        let run = format!("rls{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        for (suffix, visibility, team) in [
            ("eng", "team", Some("eng")),
            ("sales", "team", Some("sales")),
            ("co", "company", None),
        ] {
            let id = format!("{run}-{suffix}");
            sqlx::query(
                "INSERT INTO knowledge_atoms (
                    id, source, source_id, title, summary, content,
                    source_updated_at, indexed_at, visibility, team_id
                 ) VALUES ($1,'rls-test','/rls',$1,'',$2,\
                    '2026-10-20T00:00:00Z','2026-10-20T00:00:00Z',$3,$4)",
            )
            .bind(&id)
            .bind(format!("fixture {suffix}"))
            .bind(visibility)
            .bind(team)
            .execute(&mut admin)
            .await
            .expect("fixture atom");
            // Per-run distinct vector — identical distance-0 rows from
            // leftover fixtures could otherwise crowd a shared-DB kNN limit.
            let v1: f32 = rand_frac();
            sqlx::query("INSERT INTO atoms_vec (atom_id, embedding) VALUES ($1, $2::vector)")
                .bind(&id)
                .bind(format!("[{v1},{},{},{}]", v1 * 0.5, 0.9 - v1, 0.3))
                .execute(&mut admin)
                .await
                .expect("fixture vec");
        }
        (
            run.clone(),
            format!("{run}-eng"),
            format!("{run}-sales"),
            format!("{run}-co"),
        )
    }

    async fn cleanup(admin_dsn: &str, prefix: &str) {
        let mut admin = connect(admin_dsn).await;
        sqlx::query("DELETE FROM knowledge_atoms WHERE id LIKE $1")
            .bind(format!("{prefix}-%"))
            .execute(&mut admin)
            .await
            .expect("cleanup"); // atoms_vec cascades
    }

    /// Random 0..1 fraction for per-run fixture vectors (no `rand` dep — a
    /// uuid's first bytes are random enough for this).
    fn rand_frac() -> f32 {
        let b = uuid::Uuid::new_v4().as_bytes()[0];
        (b as f32) / 255.0
    }

    /// Atom ids visible under a scoped transaction on `conn`. GUCs are set
    /// transaction-local (`set_config(…, true)`) exactly like the runtime path.
    async fn scoped_ids(
        conn: &mut PgConnection,
        prefix: &str,
        team: Option<&str>,
        admin_guc: bool,
    ) -> Vec<String> {
        let mut tx = conn.begin().await.expect("begin");
        if let Some(t) = team {
            sqlx::query("SELECT set_config('kurultai.team_id', $1, true)")
                .bind(t)
                .execute(&mut *tx)
                .await
                .expect("set team GUC");
        }
        if admin_guc {
            sqlx::query("SELECT set_config('kurultai.is_admin', 'on', true)")
                .execute(&mut *tx)
                .await
                .expect("set admin GUC");
        }
        let ids: Vec<String> =
            sqlx::query_scalar("SELECT id FROM knowledge_atoms WHERE id LIKE $1 ORDER BY id")
                .bind(format!("{prefix}-%"))
                .fetch_all(&mut *tx)
                .await
                .expect("scoped select");
        tx.rollback().await.expect("rollback");
        ids
    }

    /// AE6: an eng-scoped session sees eng + company rows and zero sales rows.
    #[tokio::test]
    async fn member_eng_scope_sees_eng_and_company_never_sales() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let ids = scoped_ids(&mut app, &prefix, Some("eng"), false).await;
        assert!(ids.contains(&eng), "eng team atom must be visible: {ids:?}");
        assert!(ids.contains(&co), "company atom must be visible: {ids:?}");
        assert!(
            !ids.contains(&sales),
            "sales atom must not leak to eng session (AE6): {ids:?}"
        );

        cleanup(&dsn, &prefix).await;
    }

    /// No GUCs → `current_setting(…, true)` is NULL → company-only (fail closed).
    #[tokio::test]
    async fn no_guc_fails_closed_to_company_only() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let ids = scoped_ids(&mut app, &prefix, None, false).await;
        assert_eq!(ids, vec![co], "no GUC must see company only: {ids:?}");
        assert!(!ids.contains(&eng) && !ids.contains(&sales));

        cleanup(&dsn, &prefix).await;
    }

    /// `kurultai.is_admin='on'` → all scopes visible (admin/internal paths).
    #[tokio::test]
    async fn admin_guc_sees_all_scopes() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let ids = scoped_ids(&mut app, &prefix, None, true).await;
        assert_eq!(ids, vec![co, eng, sales], "admin sees all: {ids:?}");

        cleanup(&dsn, &prefix).await;
    }

    /// Member write posture (KTD5): own-team inserts pass; other team,
    /// company, and personal writes are rejected — personal by the column
    /// CHECK (different error), team-hijack and company by RLS WITH CHECK.
    #[tokio::test]
    async fn member_write_scope_enforced_by_with_check() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, _eng, _sales, _co) = fixtures(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.team_id', 'eng', true)")
            .execute(&mut *tx)
            .await
            .unwrap();

        // Own-team insert passes.
        let ok_id = format!("{prefix}-ownwrite");
        sqlx::query(
            "INSERT INTO knowledge_atoms (
                id, source, source_id, title, content,
                source_updated_at, indexed_at, visibility, team_id
             ) VALUES ($1,'rls-test','/rls','w','c',\
                '2026-10-20T00:00:00Z','2026-10-20T00:00:00Z','team','eng')",
        )
        .bind(&ok_id)
        .execute(&mut *tx)
        .await
        .expect("own-team insert must pass WITH CHECK");

        // Each expect-fail sits behind a savepoint — a rejected statement
        // aborts the tx and every later statement would error with 25P02
        // instead of the real violation.
        let insert = |id: String, visibility: &'static str, team: Option<&'static str>| {
            format!(
                "INSERT INTO knowledge_atoms (
                    id, source, source_id, title, content,
                    source_updated_at, indexed_at, visibility, team_id
                 ) VALUES ('{id}','rls-test','/rls','w','c',\
                    '2026-10-20T00:00:00Z','2026-10-20T00:00:00Z','{visibility}',{})",
                team.map(|t| format!("'{t}'")).unwrap_or("NULL".into())
            )
        };

        // Cross-team insert → WITH CHECK rejection.
        sqlx::query("SAVEPOINT sp1")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query(&insert(format!("{prefix}-hijack"), "team", Some("sales")))
            .execute(&mut *tx)
            .await
            .expect_err("team_id='sales' under eng GUC must fail");
        assert!(
            err.to_string().contains("row-level security"),
            "expected RLS violation, got: {err}"
        );
        sqlx::query("ROLLBACK TO SAVEPOINT sp1")
            .execute(&mut *tx)
            .await
            .unwrap();

        // Company write → requires is_admin → WITH CHECK rejection.
        sqlx::query("SAVEPOINT sp2")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query(&insert(format!("{prefix}-cowrite"), "company", None))
            .execute(&mut *tx)
            .await
            .expect_err("visibility='company' under member GUC must fail");
        assert!(
            err.to_string().contains("row-level security"),
            "expected RLS violation, got: {err}"
        );
        sqlx::query("ROLLBACK TO SAVEPOINT sp2")
            .execute(&mut *tx)
            .await
            .unwrap();

        // UPDATE that would re-scope an own-team row to sales → rejected.
        sqlx::query("SAVEPOINT sp4")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query("UPDATE knowledge_atoms SET team_id = 'sales' WHERE id = $1")
            .bind(&ok_id)
            .execute(&mut *tx)
            .await
            .expect_err("re-scoping update must fail");
        assert!(err.to_string().contains("row-level security"), "{err}");
        sqlx::query("ROLLBACK TO SAVEPOINT sp4")
            .execute(&mut *tx)
            .await
            .unwrap();

        // UPDATE targeting a company row: visible to the member's USING scope
        // but the result fails WITH CHECK (company writes need is_admin).
        sqlx::query("SAVEPOINT sp5")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query("UPDATE knowledge_atoms SET title = 'hax' WHERE id = $1")
            .bind(format!("{prefix}-co"))
            .execute(&mut *tx)
            .await
            .expect_err("member update of a company row must fail WITH CHECK");
        assert!(err.to_string().contains("row-level security"), "{err}");
        sqlx::query("ROLLBACK TO SAVEPOINT sp5")
            .execute(&mut *tx)
            .await
            .unwrap();

        // UPDATE targeting an invisible (sales) row → silently zero rows:
        // the row doesn't even exist from the member's perspective.
        let res = sqlx::query("UPDATE knowledge_atoms SET title = 'hax' WHERE id = $1")
            .bind(format!("{prefix}-sales"))
            .execute(&mut *tx)
            .await
            .expect("invisible update runs");
        assert_eq!(res.rows_affected(), 0, "invisible rows are not updatable");

        tx.rollback().await.unwrap();

        // Personal write → column CHECK rejection (AE4 backstop). Under a
        // member GUC the RLS WITH CHECK fires first, so run it as admin —
        // the admin passes the policy and lands on the column CHECK, proving
        // `visibility='personal'` is excluded independently of RLS.
        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.is_admin', 'on', true)")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query(&insert(format!("{prefix}-pers"), "personal", None))
            .execute(&mut *tx)
            .await
            .expect_err("visibility='personal' must fail the column CHECK");
        assert!(
            err.to_string().contains("check constraint"),
            "expected CHECK violation, got: {err}"
        );
        tx.rollback().await.unwrap();

        cleanup(&dsn, &prefix).await;
    }

    /// No DELETE policy for members: deletes match zero rows. Admin GUC can
    /// delete (internal paths — merge/source cleanup — still work as app).
    #[tokio::test]
    async fn member_cannot_delete_admin_can() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, _sales, _co) = fixtures(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;

        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.team_id', 'eng', true)")
            .execute(&mut *tx)
            .await
            .unwrap();
        let res = sqlx::query("DELETE FROM knowledge_atoms WHERE id = $1")
            .bind(&eng)
            .execute(&mut *tx)
            .await
            .expect("member delete runs but affects nothing");
        assert_eq!(res.rows_affected(), 0, "member delete must affect 0 rows");
        tx.rollback().await.unwrap();

        // Still there under admin.
        let ids = scoped_ids(&mut app, &prefix, None, true).await;
        assert!(ids.contains(&eng), "member delete must not remove the row");

        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.is_admin', 'on', true)")
            .execute(&mut *tx)
            .await
            .unwrap();
        let res = sqlx::query("DELETE FROM knowledge_atoms WHERE id = $1")
            .bind(&eng)
            .execute(&mut *tx)
            .await
            .expect("admin delete");
        assert_eq!(res.rows_affected(), 1, "admin delete must work");
        tx.rollback().await.unwrap(); // keep fixtures for cleanup parity

        cleanup(&dsn, &prefix).await;
    }

    /// `kurultai_migrate` (BYPASSRLS) reads all rows with no GUCs — the
    /// migrate/CLI context is not scope-limited (sanity for `kurultai hub …`).
    #[tokio::test]
    async fn migrate_role_bypasses_rls() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;

        let mut mig = connect(&dsn_for(&dsn, "kurultai_migrate", MIGRATE_PW)).await;
        let ids = scoped_ids(&mut mig, &prefix, None, false).await;
        assert_eq!(
            ids,
            vec![co, eng, sales],
            "BYPASSRLS migrate role must see all rows: {ids:?}"
        );

        cleanup(&dsn, &prefix).await;
    }

    /// The runtime role cannot run DDL (KTD8) — grants are DML-only.
    #[tokio::test]
    async fn app_role_cannot_run_ddl() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let err = sqlx::query("CREATE TABLE rls_forbidden (id int)")
            .execute(&mut app)
            .await
            .expect_err("kurultai_app must not CREATE TABLE");
        assert!(
            err.to_string().contains("permission denied"),
            "expected permission denied, got: {err}"
        );
    }

    /// atoms_vec follows the parent atom's scope via an RLS-filtered EXISTS —
    /// a member can't read, insert, or delete vectors for out-of-scope atoms,
    /// including through the app's real join shape.
    #[tokio::test]
    async fn atoms_vec_inherits_parent_scope() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.team_id', 'eng', true)")
            .execute(&mut *tx)
            .await
            .unwrap();

        // Direct + join-shaped reads both see only eng + company vec rows.
        let ids: Vec<String> = sqlx::query_scalar(
            "SELECT v.atom_id FROM atoms_vec v
             JOIN knowledge_atoms a ON a.id = v.atom_id
             WHERE v.atom_id LIKE $1 ORDER BY v.atom_id",
        )
        .bind(format!("{prefix}-%"))
        .fetch_all(&mut *tx)
        .await
        .expect("vec join select");
        assert!(ids.contains(&eng) && ids.contains(&co), "{ids:?}");
        assert!(!ids.contains(&sales), "sales vec must not leak: {ids:?}");

        // Inserting a vector for an out-of-scope atom → WITH CHECK denial.
        sqlx::query("SAVEPOINT vec_sp")
            .execute(&mut *tx)
            .await
            .unwrap();
        let err = sqlx::query(
            "INSERT INTO atoms_vec (atom_id, embedding) VALUES ($1, '[0.9,0.9,0.9,0.9]')",
        )
        .bind(&sales)
        .execute(&mut *tx)
        .await
        .expect_err("vec insert for sales atom must fail under eng GUC");
        assert!(err.to_string().contains("row-level security"), "{err}");

        // Deleting an out-of-scope vector → zero rows (the savepoint revives
        // the tx after the rejected insert).
        sqlx::query("ROLLBACK TO SAVEPOINT vec_sp")
            .execute(&mut *tx)
            .await
            .unwrap();
        let res = sqlx::query("DELETE FROM atoms_vec WHERE atom_id = $1")
            .bind(&sales)
            .execute(&mut *tx)
            .await
            .expect("vec delete runs");
        assert_eq!(
            res.rows_affected(),
            0,
            "member cannot drop other teams' vectors"
        );

        tx.rollback().await.unwrap();
        cleanup(&dsn, &prefix).await;
    }

    /// hub_activity is append-only for members (writes log must record member
    /// traffic) but SELECT is admin-only and UPDATE/DELETE are denied to all
    /// RLS-subject roles — the audit log can't be read or tampered with.
    #[tokio::test]
    async fn hub_activity_append_only_select_admin() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;

        let mut app = connect(&dsn_for(&dsn, "kurultai_app", APP_PW)).await;
        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.team_id', 'eng', true)")
            .execute(&mut *tx)
            .await
            .unwrap();
        let marker = format!("rls-{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        sqlx::query(
            "INSERT INTO hub_activity (at, agent_id, team_id, namespace, transport, atom_id)
             VALUES ('2026-10-20T00:00:00Z', 'rls-test', 'eng', 'test', 'http', $1)",
        )
        .bind(&marker)
        .execute(&mut *tx)
        .await
        .expect("member append to activity log must be allowed");

        // Even the row the member just wrote is invisible to its own SELECT.
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hub_activity")
            .fetch_one(&mut *tx)
            .await
            .expect("member activity select");
        assert_eq!(n, 0, "member must not read the activity log");
        tx.commit().await.unwrap(); // audit write persists unread

        // Admin reads it.
        let mut tx = app.begin().await.expect("begin");
        sqlx::query("SELECT set_config('kurultai.is_admin', 'on', true)")
            .execute(&mut *tx)
            .await
            .unwrap();
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hub_activity WHERE atom_id = $1")
            .bind(&marker)
            .fetch_one(&mut *tx)
            .await
            .expect("admin activity select");
        assert_eq!(n, 1, "admin must read activity rows");
        tx.rollback().await.unwrap();

        // Scrub the marker (superuser bypasses the immutable-log policy).
        let mut admin = connect(&dsn).await;
        sqlx::query("DELETE FROM hub_activity WHERE atom_id = $1")
            .bind(&marker)
            .execute(&mut admin)
            .await
            .expect("activity cleanup");
    }

    /// KTD8: with `KURULTAI_PG_SKIP_MIGRATE=1` both stores connect as
    /// `kurultai_app` (which cannot run DDL — proven above) without attempting
    /// `migrate()`. Without the env, connect-as-app fails on DDL privileges.
    #[tokio::test]
    async fn skip_migrate_env_allows_unprivileged_connect() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let app_dsn = dsn_for(&dsn, "kurultai_app", APP_PW);

        {
            let _env = EnvGuard::set("KURULTAI_PG_SKIP_MIGRATE", "1");
            let store = PostgresStore::connect(&app_dsn, 4)
                .await
                .expect("PostgresStore must connect under KURULTAI_PG_SKIP_MIGRATE=1");
            // Pool is usable: an internal (unset) scope sees every row.
            store.count().await.expect("count under app role");
            HubKeyStore::connect(&app_dsn)
                .await
                .expect("HubKeyStore must connect under KURULTAI_PG_SKIP_MIGRATE=1");
        }

        // Negative control: without the gate both connects still try to
        // migrate and fail — the app role owns nothing and can't ALTER.
        assert!(
            PostgresStore::connect(&app_dsn, 4).await.is_err(),
            "without skip-migrate, app-role connect must fail at migrate"
        );
        assert!(
            HubKeyStore::connect(&app_dsn).await.is_err(),
            "without skip-migrate, app-role keys connect must fail at migrate"
        );
    }

    // ── U2: session-GUC wiring + server-side write stamping (KTD1/KTD5) ────

    /// Connect a `PostgresStore` as `kurultai_app` — the RLS-enforced runtime
    /// role — with `KURULTAI_PG_SKIP_MIGRATE=1` (no DDL rights, KTD8).
    async fn app_store(admin_dsn: &str) -> PostgresStore {
        let _env = EnvGuard::set("KURULTAI_PG_SKIP_MIGRATE", "1");
        PostgresStore::connect(&dsn_for(admin_dsn, "kurultai_app", APP_PW), 4)
            .await
            .expect("app-role PostgresStore")
    }

    /// Atom ids (this run's `prefix`) visible through the real
    /// `PostgresStore` under `scope` — the same `begin_scope` code path every
    /// HTTP member request takes.
    async fn store_visible_ids(store: &PostgresStore, scope: PgScope, prefix: &str) -> Vec<String> {
        let mut ids: Vec<String> = scope
            .run(store.list_atoms(500, SearchFilter::default()))
            .await
            .expect("scoped list_atoms")
            .into_iter()
            .map(|a| a.id)
            .filter(|id| id.starts_with(prefix))
            .collect();
        ids.sort();
        ids
    }

    /// Build the real request stack a member `kurultai connect` hits:
    /// `BrainService` on the app-role store + `hub_api_auth` + `pg_rls_scope`
    /// + `HubKeyStore` member-key resolution.
    async fn hub_http_app(dsn: &str) -> (axum::Router, Arc<HubKeyStore>) {
        let store = app_store(dsn).await;
        let keys = Arc::new(HubKeyStore::connect(dsn).await.expect("key store"));
        let brain = BrainService::new(
            Arc::new(store) as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let gate = HubGate {
            auth: HubAuth::ApiKey,
            api_keys: vec!["op-static-key".into()],
            agent_store: None,
            cf_access: None,
            key_store: Some(Arc::clone(&keys)),
        };
        (
            build_app(brain, Arc::new(DaemonStatus::default()), gate),
            keys,
        )
    }

    /// U2/AE5: the same `PostgresStore` pool serves eng then sales scopes —
    /// `set_config(…, true)` is transaction-local, so a member scope can
    /// never bleed into the next checkout.
    #[tokio::test]
    async fn member_scope_does_not_bleed_across_pool_checkouts() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;
        let store = app_store(&dsn).await;

        // Sequential checkouts over one pool: eng → sales → eng again.
        let ids = store_visible_ids(&store, PgScope::member("eng"), &prefix).await;
        assert!(
            ids.contains(&eng) && ids.contains(&co) && !ids.contains(&sales),
            "eng scope must see eng+co only: {ids:?}"
        );
        let ids = store_visible_ids(&store, PgScope::member("sales"), &prefix).await;
        assert!(
            ids.contains(&sales) && ids.contains(&co) && !ids.contains(&eng),
            "sales scope must see sales+co only: {ids:?}"
        );
        let ids = store_visible_ids(&store, PgScope::member("eng"), &prefix).await;
        assert!(
            ids.contains(&eng) && !ids.contains(&sales),
            "reused checkout must re-scope, not accumulate: {ids:?}"
        );

        // A closed (no-principal) request after member traffic must still
        // fail closed — discriminating proof the GUCs were transaction-local,
        // not session-level.
        let ids = store_visible_ids(&store, PgScope::closed(), &prefix).await;
        assert_eq!(ids, vec![co], "closed scope after member traffic: {ids:?}");

        // Unset ambient scope is internal (KTD1): pipeline/CLI/maintenance
        // callers see every shared row.
        let ids = store_visible_ids(&store, PgScope::INTERNAL, &prefix).await;
        assert_eq!(ids.len(), 3, "internal scope must see all rows: {ids:?}");

        cleanup(&dsn, &prefix).await;
    }

    /// U2: `team_id=None` + no admin flag ⇒ company-only through the real
    /// store path. `HubPrincipal` always carries a team today, so the closed
    /// scope is what a principal-less member path resolves to — KTD1 says it
    /// must fail closed, never silently become admin.
    #[tokio::test]
    async fn closed_scope_is_company_only_and_member_never_admin() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, _eng, _sales, co) = fixtures(&dsn).await;
        let store = app_store(&dsn).await;

        let ids = store_visible_ids(&store, PgScope::closed(), &prefix).await;
        assert_eq!(ids, vec![co], "closed scope must see company only: {ids:?}");

        // Negative: `is_admin` is unreachable from member paths — the member
        // constructors simply have no way to set it.
        let member = PgScope::member("eng");
        assert!(
            !member.is_admin,
            "member scope must never carry is_admin (KTD1 negative)"
        );
        assert_eq!(member.team_id.as_deref(), Some("eng"));
        let closed = PgScope::closed();
        assert!(closed.team_id.is_none() && !closed.is_admin);
    }

    /// U2/AE5 (HTTP): an issued eng member key through `build_app` —
    /// `hub_api_auth` resolves the `HubPrincipal`, `pg_rls_scope` installs the
    /// member scope, `/api/search` returns eng+company and never sales. The
    /// result would contain all three fixture ids if `is_admin` could leak
    /// into a member path, so this is also the member→admin negative proof.
    #[tokio::test]
    async fn member_key_http_search_is_team_scoped() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;
        let (app, keys) = hub_http_app(&dsn).await;
        let (member_key, key_id) = keys.issue("agent-eng", "eng").await.expect("issue key");

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/search?q={prefix}"))
                    .header("authorization", format!("Bearer {member_key}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("member search response");
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "member search must be a 200, not an auth/scope error"
        );
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let mut ids: Vec<String> = json
            .as_array()
            .expect("search returns a result array")
            .iter()
            .filter_map(|r| r["atom"]["id"].as_str().map(str::to_string))
            .filter(|id| id.starts_with(&prefix))
            .collect();
        ids.sort();
        assert!(ids.contains(&eng), "member must see own-team atom: {ids:?}");
        assert!(ids.contains(&co), "member must see company atom: {ids:?}");
        assert!(
            !ids.contains(&sales),
            "sales must not leak over HTTP (AE5): {ids:?}"
        );

        let _ = keys.revoke_by_id(key_id).await;
        cleanup(&dsn, &prefix).await;
    }

    /// U2: `GET /api/hub/activity` is admin-only — a member key gets 403
    /// (previously any authenticated caller could read every team's write
    /// log). An operator static key passes the member gate; it 503s here only
    /// because `build_app` wires no `HubActivityStore`. (The static key is
    /// exercised on a `key_store`-less gate: `authorize_bearer` short-circuits
    /// static keys the moment any issued key exists, and parallel tests share
    /// `hub_api_keys`.)
    #[tokio::test]
    async fn hub_activity_member_key_forbidden() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys) = hub_http_app(&dsn).await;

        // Operator-class credential, no member principal → admin scope: the
        // member gate must not fire. 503 because the test state wires no
        // `HubActivityStore`.
        let static_gate = HubGate {
            auth: HubAuth::ApiKey,
            api_keys: vec!["op-static-key".into()],
            agent_store: None,
            cf_access: None,
            key_store: None,
        };
        let static_app = {
            let store = app_store(&dsn).await;
            let brain = BrainService::new(
                Arc::new(store) as Arc<dyn Store>,
                Arc::new(NullEmbedder::new(4)),
                Arc::new(NullReranker::new()),
                Arc::new(ExtractiveSynthesizer::new()),
            );
            build_app(brain, Arc::new(DaemonStatus::default()), static_gate)
        };
        let resp = static_app
            .oneshot(
                Request::builder()
                    .uri("/api/hub/activity")
                    .header("authorization", "Bearer op-static-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("operator activity response");
        assert_eq!(
            resp.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "operator credential passes the member gate (503 = no activity store wired)"
        );

        let (member_key, key_id) = keys.issue("agent-eng", "eng").await.expect("issue key");
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/hub/activity")
                    .header("authorization", format!("Bearer {member_key}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("member activity response");
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "member key must get 403 on /api/hub/activity"
        );

        let _ = keys.revoke_by_id(key_id).await;
    }

    /// U2/KTD5: `remember` under a member scope is stamped server-side —
    /// `visibility=team` + the resolved `team_id`. A client-supplied
    /// `team_id` is overwritten (provenance stamping precedent), and the RLS
    /// `WITH CHECK` underneath would reject a wrong-team write anyway.
    #[tokio::test]
    async fn member_remember_stamps_team_from_resolved_scope() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let store = Arc::new(app_store(&dsn).await);
        let brain = BrainService::new(
            store.clone() as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );

        let ctx = WriteContext::solo(WriteTransport::Mcp);
        let id = PgScope::member("eng")
            .run(brain.remember_with(
                "U2 member remember",
                "member-scoped write through the resolved principal",
                &["rls-u2".to_string()],
                // Forgery attempt: the client claims team 'sales'.
                &[("team_id", "sales")],
                &ctx,
            ))
            .await
            .expect("member remember must succeed");

        let atom = PgScope::admin()
            .run(store.get(&id))
            .await
            .expect("admin get")
            .expect("remembered atom must exist");
        assert_eq!(
            atom.visibility,
            VisibilityScope::Team,
            "member write must land visibility=team"
        );
        assert_eq!(
            atom.metadata.get("team_id").map(String::as_str),
            Some("eng"),
            "team_id must come from the resolved principal, not client input"
        );

        // The stamped atom is member-readable; a sales member cannot see it.
        let ids = store_visible_ids(&store, PgScope::member("sales"), &id[..6]).await;
        assert!(
            !ids.contains(&id),
            "eng atom must not leak to sales: {ids:?}"
        );

        PgScope::admin()
            .run(store.delete_atom(&id))
            .await
            .expect("cleanup delete");
    }
}
