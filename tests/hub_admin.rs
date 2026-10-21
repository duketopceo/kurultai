//! Hub admin surface (plan 2026-10-19-2053 U4, R17/KTD7): member-vs-admin
//! authorization on `/api/hub/*` — key list/issue/revoke, device pending +
//! approve, and the scope census — plus the `hub key issue --admin` mint
//! path and the `kurultai.is_admin` GUC wiring behind it all.
//!
//! Provisioning mirrors `tests/connect_hub.rs`: self-provision the
//! `kurultai_app` runtime role and run BOTH the `PostgresStore` and the
//! `HubKeyStore` on the app-role DSN. That last bit matters — the census
//! assertions are only scope-discriminating when the pool is subject to
//! RLS (the CI superuser bypasses FORCE RLS unconditionally).

#[cfg(feature = "postgres")]
mod hub_admin {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use kurultai::daemon::DaemonStatus;
    use kurultai::embed::NullEmbedder;
    use kurultai::http::{build_app, HubAuth, HubGate};
    use kurultai::hub::HubKeyStore;
    use kurultai::mcp::brain::BrainService;
    use kurultai::rerank::NullReranker;
    use kurultai::store::postgres::PostgresStore;
    use kurultai::store::Store;
    use kurultai::synthesize::ExtractiveSynthesizer;
    use kurultai::testutil::EnvGuard;
    use sqlx::{Connection, PgConnection};
    use std::sync::Arc;
    use tower::ServiceExt;

    const APP_PW: &str = "hub_admin_app_pw";
    const MIGRATE_PW: &str = "hub_admin_migrate_pw";
    const OP_KEY: &str = "op-static-key";

    fn test_dsn() -> Option<String> {
        match std::env::var("KURULTAI_TEST_DATABASE_URL") {
            Ok(url) if !url.is_empty() => Some(url),
            _ if std::env::var("CI").is_ok() => {
                panic!("KURULTAI_TEST_DATABASE_URL must be set in CI for --features postgres tests")
            }
            _ => None,
        }
    }

    fn dsn_for(admin: &str, role: &str, password: &str) -> String {
        let (scheme, rest) = admin.split_once("://").expect("dsn scheme");
        let host_db = rest.rsplit_once('@').map(|(_, h)| h).unwrap_or(rest);
        format!("{scheme}://{role}:{password}@{host_db}")
    }

    async fn connect(dsn: &str) -> PgConnection {
        PgConnection::connect(dsn).await.expect("pg connect")
    }

    /// Distinct from migrate (87231401), hub_rls (87231402), connect_hub
    /// (87231403) — nextest runs each test in its own process, so
    /// provisioning serializes inside the database.
    const PROVISION_LOCK: i64 = 87231404;

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
                        || e.to_string().contains("duplicate key") => {}
                Err(e) => panic!("create role {role}: {e}"),
            }
            let alter = format!("ALTER ROLE {role} WITH {attrs} PASSWORD '{pw}'");
            sqlx::query(&alter)
                .execute(&mut *admin)
                .await
                .expect("alter role");
        }

        {
            let _env = EnvGuard::remove("KURULTAI_PG_SKIP_MIGRATE");
            PostgresStore::connect(admin_dsn, 4)
                .await
                .expect("migrate as superuser");
        }

        for grant in [
            "GRANT USAGE ON SCHEMA public TO kurultai_app, kurultai_migrate",
            // init-hub-db.sql line 53 — the migrate role DDLs (the `hub` CLI
            // runs `HubKeyStore::connect` → `migrate` under it).
            "GRANT CREATE ON SCHEMA public TO kurultai_migrate",
            "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO kurultai_app",
            "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO kurultai_app",
            "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO kurultai_migrate",
            "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO kurultai_migrate",
            // init-hub-db.sql lines 68-78 — tables created later auto-grant
            // DML to the app role whether the migrate role or this
            // provisioning superuser created them.
            "ALTER DEFAULT PRIVILEGES FOR ROLE kurultai_migrate IN SCHEMA public \
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO kurultai_app",
            "ALTER DEFAULT PRIVILEGES FOR ROLE kurultai_migrate IN SCHEMA public \
             GRANT USAGE, SELECT ON SEQUENCES TO kurultai_app",
            "ALTER DEFAULT PRIVILEGES IN SCHEMA public \
             GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO kurultai_app",
            "ALTER DEFAULT PRIVILEGES IN SCHEMA public \
             GRANT USAGE, SELECT ON SEQUENCES TO kurultai_app",
        ] {
            sqlx::query(grant)
                .execute(&mut *admin)
                .await
                .expect("grant");
        }

        // init-hub-db.sql ordering means kurultai_migrate OWNS every hub
        // object in production (it runs the first migrate). Here the
        // provisioning superuser created them, so transfer ownership —
        // `ALTER TABLE` migrations (e.g. PG_HUB_KEYS_ADMIN) require owner.
        sqlx::query(
            r#"DO $$
            DECLARE r RECORD;
            BEGIN
                FOR r IN
                    SELECT c.relname FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE n.nspname = 'public' AND c.relkind IN ('r','S')
                LOOP
                    EXECUTE format('ALTER TABLE public.%I OWNER TO kurultai_migrate', r.relname);
                END LOOP;
            END $$"#,
        )
        .execute(&mut *admin)
        .await
        .expect("transfer object ownership to kurultai_migrate");
    }

    /// One atom per scope under a unique `run` prefix — eng team, sales
    /// team, company.
    async fn fixtures(admin_dsn: &str) -> String {
        let mut admin = connect(admin_dsn).await;
        let run = format!("ha{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
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
                 ) VALUES ($1,'hubadmin-test','/ha',$1,'',$2,\
                    '2026-10-20T00:00:00Z','2026-10-20T00:00:00Z',$3,$4)",
            )
            .bind(&id)
            .bind(format!("fixture {suffix}"))
            .bind(visibility)
            .bind(team)
            .execute(&mut admin)
            .await
            .expect("fixture atom");
        }
        run
    }

    async fn cleanup(admin_dsn: &str, prefix: &str) {
        let mut admin = connect(admin_dsn).await;
        sqlx::query("DELETE FROM knowledge_atoms WHERE id LIKE $1")
            .bind(format!("{prefix}-%"))
            .execute(&mut admin)
            .await
            .expect("cleanup");
    }

    /// The full hub stack on the **app-role** DSN (KTD8 skip-migrate):
    /// `PostgresStore` and `HubKeyStore` both run as `kurultai_app`, so
    /// `scope_census`/activity reads are decided by RLS + the request GUCs,
    /// not by a superuser bypass. Returns `(router, key_store, pg_store)`.
    async fn hub_app(dsn: &str) -> (axum::Router, Arc<HubKeyStore>, Arc<PostgresStore>) {
        let _env = EnvGuard::set("KURULTAI_PG_SKIP_MIGRATE", "1");
        let app_dsn = dsn_for(dsn, "kurultai_app", APP_PW);
        let store = Arc::new(
            PostgresStore::connect(&app_dsn, 4)
                .await
                .expect("app-role PostgresStore"),
        );
        let keys = Arc::new(
            HubKeyStore::connect(&app_dsn)
                .await
                .expect("app-role HubKeyStore"),
        );
        let brain = BrainService::new(
            store.clone() as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let gate = HubGate {
            auth: HubAuth::ApiKey,
            api_keys: vec![OP_KEY.into()],
            agent_store: None,
            cf_access: None,
            key_store: Some(Arc::clone(&keys)),
        };
        (
            build_app(brain, Arc::new(DaemonStatus::default()), gate),
            keys,
            store,
        )
    }

    fn bearer_get(uri: &str, token: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn bearer_post(uri: &str, token: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn bearer_delete(uri: &str, token: &str) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(uri)
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap()
    }

    fn json_post(uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        serde_json::from_slice(&body).expect("json body")
    }

    /// Every `/api/hub/*` admin route with a representative request — the
    /// member-403 and admin-200 sweeps share this list so coverage can't
    /// drift.
    fn admin_requests(token: &str) -> Vec<(&'static str, Request<Body>)> {
        vec![
            ("GET keys", bearer_get("/api/hub/keys", token)),
            (
                "POST keys",
                bearer_post(
                    "/api/hub/keys",
                    token,
                    serde_json::json!({"agent_id": "probe", "team_id": "eng"}),
                ),
            ),
            ("DELETE keys", bearer_delete("/api/hub/keys/999999", token)),
            ("GET pending", bearer_get("/api/hub/device/pending", token)),
            (
                "POST approve",
                bearer_post(
                    "/api/hub/device/approve",
                    token,
                    serde_json::json!({"code": "ZZZZZZZZ", "team_id": "eng"}),
                ),
            ),
            ("GET scopes", bearer_get("/api/hub/scopes", token)),
            ("GET activity", bearer_get("/api/hub/activity", token)),
        ]
    }

    /// R17 core gate: a member key is refused 403 on EVERY admin route —
    /// keys, device admin, census, and the U2 activity log.
    #[tokio::test]
    async fn member_key_forbidden_on_all_admin_routes() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (member_key, member_id) = keys
            .issue(&format!("member-{tag}"), "eng")
            .await
            .expect("issue member key");

        for (name, req) in admin_requests(&member_key) {
            let resp = app.clone().oneshot(req).await.expect(name);
            assert_eq!(
                resp.status(),
                StatusCode::FORBIDDEN,
                "member key must 403 on {name}"
            );
        }

        let _ = keys.revoke_by_id(member_id).await;
    }

    /// An `is_admin` key passes the gate on every admin route. `/api/hub/keys`
    /// POST + `/api/hub/device/approve` get their own success-path tests; here
    /// they only need to clear the admin gate (404/409 on unknown ids/codes,
    /// never 403). `GET /api/hub/activity` 503s because `build_app` wires no
    /// `HubActivityStore` — same posture as the U2 static-key test.
    #[tokio::test]
    async fn admin_key_allowed_on_all_admin_routes() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (admin_key, admin_id) = keys
            .issue_admin(&format!("admin-{tag}"), "ops", true)
            .await
            .expect("issue admin key");

        for (name, req) in admin_requests(&admin_key) {
            let resp = app.clone().oneshot(req).await.expect(name);
            let status = resp.status();
            assert_ne!(
                status,
                StatusCode::FORBIDDEN,
                "admin key must not 403 on {name}"
            );
            assert_ne!(
                status,
                StatusCode::UNAUTHORIZED,
                "admin key must not 401 on {name}"
            );
        }

        // The census and key list are the two that must be real 200s here.
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/keys", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/scopes", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/device/pending", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let _ = keys.revoke_by_id(admin_id).await;
    }

    /// Operator-class credentials (static hub key) carry no `HubPrincipal`
    /// but still pass `admin_permitted` — same design as `/connect`'s Hub
    /// gate. Exercised on a `key_store: None` gate: `authorize_bearer`
    /// refuses static keys the moment any issued `krt_` row exists, so the
    /// principal-less path can only be seen before keys mint.
    #[tokio::test]
    async fn operator_static_key_passes_admin_gate() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let _env = EnvGuard::set("KURULTAI_PG_SKIP_MIGRATE", "1");
        let store = Arc::new(
            PostgresStore::connect(&dsn_for(&dsn, "kurultai_app", APP_PW), 4)
                .await
                .expect("app store"),
        );
        let brain = BrainService::new(
            store as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let app = build_app(
            brain,
            Arc::new(DaemonStatus::default()),
            HubGate {
                auth: HubAuth::ApiKey,
                api_keys: vec![OP_KEY.into()],
                agent_store: None,
                cf_access: None,
                key_store: None,
            },
        );

        // 503 (no HubKeyStore wired) — not 401, not 403: the operator
        // credential cleared both auth and the admin gate.
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/keys", OP_KEY))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/scopes", OP_KEY))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    /// POST issue → GET list shows metadata + role, never the secret; the
    /// minted member token authenticates as a member principal.
    #[tokio::test]
    async fn http_issue_then_list_shows_metadata_never_secret() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (admin_key, admin_id) = keys
            .issue_admin(&format!("admin-{tag}"), "ops", true)
            .await
            .expect("issue admin key");
        let agent = format!("http-issued-{tag}");

        let resp = app
            .clone()
            .oneshot(bearer_post(
                "/api/hub/keys",
                &admin_key,
                serde_json::json!({"agent_id": agent, "team_id": "qa"}),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let issued = body_json(resp).await;
        let minted = issued["key"].as_str().expect("plaintext at issue");
        assert!(minted.starts_with("krt_"), "hub key shape: {minted}");
        assert_eq!(issued["is_admin"].as_bool(), Some(false));
        let new_id = issued["id"].as_i64().expect("key id");

        // The list carries prefix/team/agent/role/lifecycle — and no secret
        // or hash field under any name.
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/keys", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list = body_json(resp).await;
        let row = list["keys"]
            .as_array()
            .expect("keys array")
            .iter()
            .find(|r| r["id"].as_i64() == Some(new_id))
            .expect("issued key in list");
        assert_eq!(row["agent_id"].as_str(), Some(agent.as_str()));
        assert_eq!(row["team_id"].as_str(), Some("qa"));
        assert_eq!(row["is_admin"].as_bool(), Some(false));
        assert_eq!(
            row["key_prefix"].as_str(),
            Some(&minted[..12]),
            "list shows the 12-char prefix only"
        );
        assert!(row["revoked_at"].is_null());
        for forbidden in ["key", "token", "key_hash", "hash", "secret"] {
            assert!(
                row.get(forbidden).is_none(),
                "list must not expose `{forbidden}`: {row}"
            );
        }

        // The minted token resolves to a member principal.
        let principal = keys
            .resolve_token(minted)
            .await
            .unwrap()
            .expect("issued key resolves");
        assert_eq!(principal.agent_id, agent);
        assert_eq!(principal.team_id, "qa");
        assert!(!principal.is_admin);

        let _ = keys.revoke_by_id(new_id).await;
        let _ = keys.revoke_by_id(admin_id).await;
    }

    /// DELETE revoke: the key row flips `revoked_at`, and subsequent
    /// requests with that credential are denied at `hub_api_auth` (401).
    #[tokio::test]
    async fn revoke_denies_subsequent_requests() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (admin_key, admin_id) = keys
            .issue_admin(&format!("admin-{tag}"), "ops", true)
            .await
            .expect("issue admin key");
        let (member_key, member_id) = keys
            .issue(&format!("revokeme-{tag}"), "eng")
            .await
            .expect("issue member key");

        // Member key works before revoke (a member route, not the admin gate).
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/status", &member_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(bearer_delete(
                &format!("/api/hub/keys/{member_id}"),
                &admin_key,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["revoked"].as_bool(), Some(true));

        // Auth layer now refuses the credential outright.
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/status", &member_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/keys", &member_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(
            keys.resolve_token(&member_key).await.unwrap().is_none(),
            "revoked key must not resolve"
        );

        // Deleting a nonexistent id is a 404, not a silent ok.
        let resp = app
            .clone()
            .oneshot(bearer_delete("/api/hub/keys/999999", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let _ = keys.revoke_by_id(admin_id).await;
    }

    /// `GET /api/hub/device/pending` lists the pending connect flow with
    /// safe metadata — `issued_key` is skipped by serialization.
    #[tokio::test]
    async fn pending_device_flow_listed_with_safe_metadata() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (admin_key, admin_id) = keys
            .issue_admin(&format!("admin-{tag}"), "ops", true)
            .await
            .expect("issue admin key");
        let seat = format!("seat-pend-{tag}");

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({
                    "client_id": "kurultai-cli",
                    "codename": "devin",
                    "instance_id": seat,
                    "team": "eng",
                }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let user_code = body["user_code"].as_str().unwrap().to_string();

        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/device/pending", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list = body_json(resp).await;
        let flow = list["flows"]
            .as_array()
            .expect("flows array")
            .iter()
            .find(|f| f["user_code"].as_str() == Some(user_code.as_str()))
            .expect("pending flow listed");
        assert_eq!(flow["status"].as_str(), Some("pending"));
        assert_eq!(flow["instance_id"].as_str(), Some(seat.as_str()));
        assert!(
            flow.get("issued_key").is_none(),
            "flow list must never serialize the minted key: {flow}"
        );

        let _ = keys.revoke_by_id(admin_id).await;
    }

    /// `POST /api/hub/device/approve` mints the team-bound key through the
    /// same `approve_for_team` path as `/connect` — the token poll hands
    /// out a member `krt_` key bound to the approver's pick, exactly once.
    #[tokio::test]
    async fn admin_api_approve_mints_team_bound_member_key() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (admin_key, admin_id) = keys
            .issue_admin(&format!("admin-{tag}"), "ops", true)
            .await
            .expect("issue admin key");
        let (member_key, member_id) = keys
            .issue(&format!("member-{tag}"), "eng")
            .await
            .expect("issue member key");
        let seat = format!("seat-api-{tag}");

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({
                    "client_id": "kurultai-cli",
                    "codename": "devin",
                    "instance_id": seat,
                    "team": "sales",
                }),
            ))
            .await
            .unwrap();
        let body = body_json(resp).await;
        let device_code = body["code"].as_str().unwrap().to_string();
        let user_code = body["user_code"].as_str().unwrap().to_string();

        // Member key cannot approve — the flow stays pending (428 on poll).
        let resp = app
            .clone()
            .oneshot(bearer_post(
                "/api/hub/device/approve",
                &member_key,
                serde_json::json!({"code": user_code, "team_id": "eng"}),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/token",
                serde_json::json!({ "code": device_code }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_REQUIRED);

        // Admin approves with team `eng` — overriding the `sales` hint,
        // exactly like the `/connect` picker.
        let resp = app
            .clone()
            .oneshot(bearer_post(
                "/api/hub/device/approve",
                &admin_key,
                serde_json::json!({"code": user_code, "team_id": "eng"}),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let approved = body_json(resp).await;
        assert_eq!(approved["team_id"].as_str(), Some("eng"));
        assert!(
            approved.get("key").is_none() && approved.get("agent_key").is_none(),
            "the admin response must not carry the minted secret: {approved}"
        );

        // The poll hands out the key minted at approve — member-scoped `eng`.
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/token",
                serde_json::json!({ "code": device_code }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let token = body_json(resp).await;
        let key = token["agent_key"].as_str().unwrap().to_string();
        assert!(key.starts_with("krt_"));
        assert_eq!(token["team_id"].as_str(), Some("eng"));
        let principal = keys
            .resolve_token(&key)
            .await
            .unwrap()
            .expect("connect key resolves");
        assert_eq!(principal.team_id, "eng");
        assert!(!principal.is_admin, "connect keys are always members");
        assert_eq!(principal.agent_id, format!("devin@{seat}"));

        // A second approve on the decided flow conflicts rather than reminting.
        let resp = app
            .clone()
            .oneshot(bearer_post(
                "/api/hub/device/approve",
                &admin_key,
                serde_json::json!({"code": user_code, "team_id": "eng"}),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);

        let _ = keys.revoke_by_id(member_id).await;
        let _ = keys.revoke_by_id(admin_id).await;
    }

    /// `GET /api/hub/scopes` census: the admin `PgScope` sets
    /// `kurultai.is_admin='on'` on the app-role pool, so the GROUP BY sees
    /// every team's rows — eng, sales, AND company. If the GUC wiring
    /// regressed to member/closed scope the team rows would disappear.
    /// Running a member `/api/search` immediately after proves the scope
    /// was transaction-local — no admin bleed across pool checkouts.
    #[tokio::test]
    async fn scope_census_counts_all_scopes_under_admin_guc() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let prefix = fixtures(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let (admin_key, admin_id) = keys
            .issue_admin(&format!("admin-{tag}"), "ops", true)
            .await
            .expect("issue admin key");
        let (member_key, member_id) = keys
            .issue(&format!("member-{tag}"), "eng")
            .await
            .expect("issue member key");

        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/scopes", &admin_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let census = body_json(resp).await;
        let scopes = census["scopes"].as_array().expect("scopes array");
        let has = |visibility: &str, team: Option<&str>| {
            scopes.iter().any(|s| {
                s["visibility"].as_str() == Some(visibility)
                    && s["team_id"].as_str() == team
                    && s["atoms"].as_i64().unwrap_or(0) >= 1
            })
        };
        assert!(has("team", Some("eng")), "eng team count missing: {census}");
        assert!(
            has("team", Some("sales")),
            "sales team count missing: {census}"
        );
        assert!(has("company", None), "company count missing: {census}");

        // Member key on the same pool right after: eng+co only — the admin
        // GUC did not bleed, and `is_admin` is never reachable from a
        // member principal (if it were, sales would appear here).
        let resp = app
            .clone()
            .oneshot(bearer_get(&format!("/api/search?q={prefix}"), &member_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        let ids: Vec<String> = json
            .as_array()
            .expect("search array")
            .iter()
            .filter_map(|r| r["atom"]["id"].as_str().map(str::to_string))
            .filter(|id| id.starts_with(&prefix))
            .collect();
        assert!(ids.contains(&format!("{prefix}-eng")), "{ids:?}");
        assert!(ids.contains(&format!("{prefix}-co")), "{ids:?}");
        assert!(
            !ids.contains(&format!("{prefix}-sales")),
            "member scope must never see sales — is_admin leak: {ids:?}"
        );

        // Member cannot reach the census at all (defense-in-depth: the 403
        // app gate plus the member-scoped RLS underneath).
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/scopes", &member_key))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let _ = keys.revoke_by_id(member_id).await;
        let _ = keys.revoke_by_id(admin_id).await;
        cleanup(&dsn, &prefix).await;
    }

    /// `kurultai hub key issue --admin` mints a key whose bearer
    /// authenticates to the admin surface — the CLI flag is real plumbing,
    /// not a label. The child runs against the migrate-role DSN (DDL rights
    /// for `HubKeyStore::connect`'s self-heal) with the DSN passed via env.
    #[tokio::test]
    async fn cli_issue_admin_mints_working_admin_key() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let agent = format!("cli-admin-{tag}");

        let out = std::process::Command::new(env!("CARGO_BIN_EXE_kurultai"))
            .args([
                "hub", "key", "issue", "--agent", &agent, "--team", "ops", "--admin",
            ])
            .env(
                "KURULTAI_DATABASE_URL",
                dsn_for(&dsn, "kurultai_migrate", MIGRATE_PW),
            )
            .env_remove("DATABASE_URL")
            .env_remove("KURULTAI_PG_SKIP_MIGRATE")
            .output()
            .expect("spawn kurultai hub key issue");
        assert!(
            out.status.success(),
            "hub key issue --admin failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
        assert!(
            stdout.contains("admin"),
            "issue output should name the role: {stdout}"
        );
        let token = stdout
            .split_whitespace()
            .find(|w| w.starts_with("krt_"))
            .expect("stdout carries the one-shot plaintext")
            .to_string();

        // The CLI-minted admin key resolves is_admin and reaches the surface.
        let principal = keys
            .resolve_token(&token)
            .await
            .unwrap()
            .expect("cli admin key resolves");
        assert!(principal.is_admin);
        assert_eq!(principal.agent_id, agent);
        assert_eq!(principal.team_id, "ops");

        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/keys", &token))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "CLI --admin key must pass the admin gate"
        );

        // Negative control: without --admin the same CLI mint is a member.
        let member_agent = format!("cli-member-{tag}");
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_kurultai"))
            .args([
                "hub",
                "key",
                "issue",
                "--agent",
                &member_agent,
                "--team",
                "eng",
            ])
            .env(
                "KURULTAI_DATABASE_URL",
                dsn_for(&dsn, "kurultai_migrate", MIGRATE_PW),
            )
            .env_remove("DATABASE_URL")
            .env_remove("KURULTAI_PG_SKIP_MIGRATE")
            .output()
            .expect("spawn member issue");
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
        let member_token = stdout
            .split_whitespace()
            .find(|w| w.starts_with("krt_"))
            .unwrap()
            .to_string();
        let resp = app
            .clone()
            .oneshot(bearer_get("/api/hub/keys", &member_token))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "CLI default mint must stay member"
        );

        let _ = keys.revoke_by_id(principal.key_id).await;
        let member_principal = keys.resolve_token(&member_token).await.unwrap().unwrap();
        let _ = keys.revoke_by_id(member_principal.key_id).await;
    }
}
