//! Hub connect-join (plan 2026-10-19-2053 U3, R12/R13, AE7; KTD3/KTD4):
//! `kurultai connect <hub-url>` mints a team-bound `hub_api_keys` key at
//! approve time — the approver's team pick wins over the CLI's `--team`
//! hint — and the token poll hands it out exactly once.
//!
//! Approvals ride the existing `ApproverGate` chain; integration tests use
//! the operator static-key bearer (CF Access + `KURULTAI_ADMIN_EMAILS`
//! allowlist coverage lives in `src/http/device.rs` unit tests — a seeded
//! `CfAccess` can't cross the lib/test-crate boundary).
//!
//! Provisioning mirrors `tests/hub_rls.rs`: self-provision the
//! `kurultai_app` runtime role and run the app-role `PostgresStore` under
//! `KURULTAI_PG_SKIP_MIGRATE=1`.

#[cfg(feature = "postgres")]
mod connect_hub {
    use axum::body::Body;
    use axum::extract::ConnectInfo;
    use axum::http::{Request, StatusCode};
    use kurultai::daemon::DaemonStatus;
    use kurultai::embed::NullEmbedder;
    use kurultai::http::{build_app, HubAuth, HubGate};
    use kurultai::hub::HubKeyStore;
    use kurultai::mcp::brain::BrainService;
    use kurultai::rerank::NullReranker;
    use kurultai::store::postgres::PostgresStore;
    use kurultai::store::{SqliteVecStore, Store};
    use kurultai::synthesize::ExtractiveSynthesizer;
    use kurultai::testutil::EnvGuard;
    use sqlx::{Connection, PgConnection};
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tower::ServiceExt;

    const APP_PW: &str = "connect_hub_app_pw";
    const MIGRATE_PW: &str = "connect_hub_migrate_pw";
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

    /// Advisory lock id for role/grant provisioning — distinct from the
    /// migrate lock (87231401) and hub_rls's (87231402).
    const PROVISION_LOCK: i64 = 87231403;

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

    /// One atom per scope under a unique `run` prefix — eng team, sales
    /// team, company. Returns `(prefix, eng_id, sales_id, co_id)`.
    async fn fixtures(admin_dsn: &str) -> (String, String, String, String) {
        let mut admin = connect(admin_dsn).await;
        let run = format!("cx{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
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
                 ) VALUES ($1,'connect-test','/cx',$1,'',$2,\
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
            .expect("cleanup");
    }

    /// App-role `PostgresStore` (KTD8 skip-migrate) behind a hub gate:
    /// `HubAuth::ApiKey` + operator static key + real `HubKeyStore`.
    async fn hub_app(dsn: &str) -> (axum::Router, Arc<HubKeyStore>, Arc<PostgresStore>) {
        let _env = EnvGuard::set("KURULTAI_PG_SKIP_MIGRATE", "1");
        let store = Arc::new(
            PostgresStore::connect(&dsn_for(dsn, "kurultai_app", APP_PW), 4)
                .await
                .expect("app-role PostgresStore"),
        );
        let keys = Arc::new(HubKeyStore::connect(dsn).await.expect("key store"));
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

    fn json_post(uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    /// POST `/connect` form approve — `team` is already form-encoded.
    fn approve_req(user_code: &str, action: &str, team_encoded: &str) -> Request<Body> {
        let mut req = Request::builder()
            .method("POST")
            .uri("/connect")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("host", "hub.test")
            .header("authorization", format!("Bearer {OP_KEY}"))
            .body(Body::from(format!(
                "code={user_code}&action={action}&team={team_encoded}"
            )))
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 41000))));
        req
    }

    /// GET `/connect?code=…` with the operator bearer + ConnectInfo.
    fn connect_page_req(user_code: &str, bearer: Option<&str>) -> Request<Body> {
        let mut b = Request::builder()
            .uri(format!("/connect?code={user_code}"))
            .header("host", "hub.test");
        if let Some(t) = bearer {
            b = b.header("authorization", format!("Bearer {t}"));
        }
        let mut req = b.body(Body::empty()).unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 41000))));
        req
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        serde_json::from_slice(&body).expect("json body")
    }

    async fn body_text(resp: axum::response::Response) -> String {
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).expect("utf8 body")
    }

    /// Drive a flow to the minted key. Returns `(device_code, agent_key, team)`.
    async fn drive_connect(
        app: &axum::Router,
        codename: &str,
        seat: &str,
        hint: &str,
        pick: &str,
    ) -> (String, String, serde_json::Value) {
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({
                    "client_id": "kurultai-cli",
                    "codename": codename,
                    "instance_id": seat,
                    "team": hint,
                }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let device_code = body["code"].as_str().unwrap().to_string();
        let user_code = body["user_code"].as_str().unwrap().to_string();

        let resp = app
            .clone()
            .oneshot(approve_req(&user_code, "approve", pick))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

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
        (
            device_code,
            token["agent_key"].as_str().unwrap().to_string(),
            token,
        )
    }

    /// AE7: `kurultai connect` on a hub daemon — code → picker page →
    /// approve (team pick) → token poll returns the minted `krt_` key → the
    /// key authenticates member-scoped on `/api/search` — and the poll is
    /// exactly-once.
    #[tokio::test]
    async fn hub_connect_end_to_end_team_scoped_key() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (prefix, eng, sales, co) = fixtures(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        // Seed an existing team so the picker datalist has a suggestion.
        keys.issue("seed-agent", "eng").await.expect("seed key");

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({
                    "client_id": "kurultai-cli",
                    "codename": "devin",
                    "instance_id": "seat-ae7",
                    "team": "sales",
                }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(
            body.get("teams").is_none() && body.get("team_ids").is_none(),
            "team roster must not appear in the unauthenticated code response: {body}"
        );
        let device_code = body["code"].as_str().unwrap().to_string();
        let user_code = body["user_code"].as_str().unwrap().to_string();
        assert_eq!(body["expires_in"].as_u64(), Some(600));

        // Poll before approval → 428 authorization_pending.
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/token",
                serde_json::json!({ "code": device_code }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_REQUIRED);

        // The authenticated picker renders the team input: the `--team`
        // hint prefills it and the seeded roster appears as suggestions.
        let resp = app
            .clone()
            .oneshot(connect_page_req(&user_code, Some(OP_KEY)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let html = body_text(resp).await;
        assert!(html.contains("name=\"team\""), "picker missing: {html}");
        assert!(
            html.contains("value=\"sales\""),
            "hint must prefill the picker: {html}"
        );
        assert!(
            html.contains("<option value=\"eng\""),
            "existing teams must be suggested: {html}"
        );

        // Approver picks `eng` — overriding the `sales` hint.
        let resp = app
            .clone()
            .oneshot(approve_req(&user_code, "approve", "eng"))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let html = body_text(resp).await;
        assert!(html.contains("Agent approved"), "{html}");

        // Token poll returns the minted key bound to `eng`.
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
        assert!(key.starts_with("krt_"), "hub key shape: {key}");
        assert_eq!(token["team_id"].as_str(), Some("eng"));
        assert_eq!(token["codename"].as_str(), Some("devin"));
        assert_eq!(token["instance_id"].as_str(), Some("seat-ae7"));

        // The key resolves to an eng member principal…
        let principal = keys
            .resolve_token(&key)
            .await
            .unwrap()
            .expect("issued key resolves");
        assert_eq!(principal.team_id, "eng");
        assert_eq!(principal.agent_id, "devin@seat-ae7");

        // …and scopes /api/search to eng+company (never sales).
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/search?q={prefix}"))
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::empty())
                    .unwrap(),
            )
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
        assert!(ids.contains(&eng) && ids.contains(&co), "{ids:?}");
        assert!(!ids.contains(&sales), "sales must not leak: {ids:?}");

        // Exactly-once: a second poll is denied — mint happened at approve.
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/token",
                serde_json::json!({ "code": device_code }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        cleanup(&dsn, &prefix).await;
    }

    /// The approver's pick wins over the CLI `--team` hint — a key minted
    /// with pick `eng` despite hint `sales` carries `eng`.
    #[tokio::test]
    async fn approver_pick_overrides_team_hint() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let seat = format!(
            "seat-hint-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        );

        let (_dc, key, token) = drive_connect(&app, "claude", &seat, "sales", "eng").await;
        assert_eq!(token["team_id"].as_str(), Some("eng"));
        let principal = keys.resolve_token(&key).await.unwrap().unwrap();
        assert_eq!(principal.team_id, "eng");
        assert_eq!(principal.agent_id, format!("claude@{seat}"));
    }

    /// Free-text "new team": an id absent from the roster is accepted when
    /// valid and the minted key carries it.
    #[tokio::test]
    async fn new_team_free_text_is_accepted_and_bound() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let team = format!("qa-{}", &uuid::Uuid::new_v4().simple().to_string()[..6]);

        let (_dc, key, token) = drive_connect(&app, "cursor", "seat-qa", "", &team).await;
        assert_eq!(token["team_id"].as_str(), Some(team.as_str()));
        let principal = keys.resolve_token(&key).await.unwrap().unwrap();
        assert_eq!(principal.team_id, team);
        assert!(keys.list_teams().await.unwrap().contains(&team));
    }

    /// Invalid `team_id` is rejected at approve (and at the `--team` hint),
    /// leaving the flow pending — nothing mints.
    #[tokio::test]
    async fn invalid_team_id_is_rejected() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let agent = format!(
            "devin@seat-bad-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        );

        // `--team` hint with an invalid charset → 400 up front.
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({ "codename": "devin", "team": "BAD TEAM" }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Approver picking an invalid id re-renders with the error; flow
        // stays pending (428 on poll) and no key mints for the agent.
        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({ "codename": "devin", "instance_id": "seat-bad" }),
            ))
            .await
            .unwrap();
        let body = body_json(resp).await;
        let device_code = body["code"].as_str().unwrap().to_string();
        let user_code = body["user_code"].as_str().unwrap().to_string();

        let resp = app
            .clone()
            .oneshot(approve_req(&user_code, "approve", "UPPER"))
            .await
            .unwrap();
        let html = body_text(resp).await;
        assert!(html.contains("invalid team_id"), "{html}");

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/token",
                serde_json::json!({ "code": device_code }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PRECONDITION_REQUIRED);

        // No key for this agent label was minted.
        let rows = keys.list().await.unwrap();
        assert!(
            !rows.iter().any(|r| r.agent_id == agent),
            "invalid pick must not mint: {rows:?}"
        );
        // Mint itself also refuses the shape (KTD1 mint-time validation).
        assert!(keys.issue("x", "Bad Team!").await.is_err());
    }

    /// Concurrent token polls: exactly one key is ever minted (at approve)
    /// and handed out — the exchange is the atomic once-only gate.
    #[tokio::test]
    async fn concurrent_token_polls_hand_out_one_key() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, keys, _store) = hub_app(&dsn).await;
        let seat = format!(
            "seat-conc-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..6]
        );

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({
                    "codename": "devin", "instance_id": seat,
                }),
            ))
            .await
            .unwrap();
        let body = body_json(resp).await;
        let device_code = body["code"].as_str().unwrap().to_string();
        let user_code = body["user_code"].as_str().unwrap().to_string();
        app.clone()
            .oneshot(approve_req(&user_code, "approve", "eng"))
            .await
            .unwrap();

        let mut handles = Vec::new();
        for _ in 0..6 {
            let app = app.clone();
            let dc = device_code.clone();
            handles.push(tokio::spawn(async move {
                app.oneshot(json_post(
                    "/api/device/token",
                    serde_json::json!({ "code": dc }),
                ))
                .await
                .unwrap()
            }));
        }
        let mut handed = Vec::new();
        let mut denied = 0;
        for h in handles {
            let resp = h.await.unwrap();
            match resp.status() {
                StatusCode::OK => handed.push(body_json(resp).await),
                StatusCode::FORBIDDEN => denied += 1,
                s => panic!("unexpected poll status {s}"),
            }
        }
        assert_eq!(handed.len(), 1, "the key is handed out exactly once");
        assert_eq!(denied, 5);
        // And the flow minted exactly one key for this agent label.
        let label = format!("devin@{seat}");
        let rows = keys.list().await.unwrap();
        let minted: Vec<_> = rows.iter().filter(|r| r.agent_id == label).collect();
        assert_eq!(minted.len(), 1, "one minted key per flow: {minted:?}");
        assert_eq!(minted[0].team_id, "eng");
    }

    /// No human auth at all → approve POST is rejected before reaching the
    /// flow.
    #[tokio::test]
    async fn approve_requires_human_auth() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, _keys, _store) = hub_app(&dsn).await;

        let mut req = Request::builder()
            .method("POST")
            .uri("/connect")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("code=ABCD1234&action=approve&team=eng"))
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([10, 0, 0, 1], 41000))));
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    /// `list_device_flows` surfaces pending → approved state for the U4
    /// console — and the serialized row never carries `issued_key`.
    #[tokio::test]
    async fn list_device_flows_surfaces_state_without_secret() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (app, _keys, store) = hub_app(&dsn).await;
        let tag = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let seat = format!("seat-list-{tag}");

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({
                    "codename": "devin", "instance_id": seat, "team": "eng",
                }),
            ))
            .await
            .unwrap();
        let body = body_json(resp).await;
        let user_code = body["user_code"].as_str().unwrap().to_string();

        let flows = store.list_device_flows(100).await.unwrap();
        let flow = flows.iter().find(|f| f.user_code == user_code).unwrap();
        assert_eq!(flow.status, "pending");
        assert_eq!(flow.team_hint.as_deref(), Some("eng"));
        assert!(flow.issued_key.is_none());

        app.clone()
            .oneshot(approve_req(&user_code, "approve", "eng"))
            .await
            .unwrap();
        let flows = store.list_device_flows(100).await.unwrap();
        let flow = flows.iter().find(|f| f.user_code == user_code).unwrap();
        assert_eq!(flow.status, "approved");
        assert_eq!(flow.team_id.as_deref(), Some("eng"));
        // Minted key rides on the row until the poll hands it out…
        assert!(flow.issued_key.is_some());
        // …but never serializes (the admin surface sees metadata only).
        let json = serde_json::to_value(flow).unwrap();
        assert!(json.get("issued_key").is_none(), "{json}");
    }

    /// Solo (`hub:false`) lane: no picker, seat token mints at poll, and
    /// the token response carries no `team_id`.
    #[tokio::test]
    async fn solo_lane_unchanged_no_picker() {
        let dir = std::env::temp_dir().join(format!("cx-solo-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.join("s.db"), 4).unwrap());
        let brain = BrainService::new(
            store as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let gate = HubGate {
            auth: HubAuth::ApiKey,
            api_keys: vec![OP_KEY.into()],
            agent_store: None,
            cf_access: None,
            key_store: None,
        };
        let app = build_app(brain, Arc::new(DaemonStatus::default()), gate);

        let resp = app
            .clone()
            .oneshot(json_post(
                "/api/device/code",
                serde_json::json!({ "codename": "cursor", "instance_id": "seat-solo" }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let device_code = body["code"].as_str().unwrap().to_string();
        let user_code = body["user_code"].as_str().unwrap().to_string();

        // No team picker on the solo page.
        let resp = app
            .clone()
            .oneshot(connect_page_req(&user_code, Some(OP_KEY)))
            .await
            .unwrap();
        let html = body_text(resp).await;
        assert!(
            !html.contains("name=\"team\""),
            "solo shows no picker: {html}"
        );

        // Approve with no team field at all — solo path ignores it.
        let resp = app
            .clone()
            .oneshot(approve_req(&user_code, "approve", ""))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Poll mints the seat token (not a hub `krt_` key) and reports no team.
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
        assert!(!key.starts_with("krt_"), "solo mints seat tokens: {key}");
        assert!(token.get("team_id").is_none() || token["team_id"].is_null());

        // The seat token authenticates through the agent-key lane.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
