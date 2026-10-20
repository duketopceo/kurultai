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
    use kurultai::hub::HubKeyStore;
    use kurultai::store::postgres::PostgresStore;
    use kurultai::store::Store;
    use kurultai::testutil::EnvGuard;
    use sqlx::{Connection, PgConnection};

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
            // Pool is usable: a no-GUC read sees company scope only.
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
}
