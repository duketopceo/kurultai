//! Hub MCP auth + scoping (plan 2026-10-19-2053 U5, R13): a minted
//! `hub_api_keys` bearer on `POST /mcp` resolves to a `HubPrincipal`, wraps
//! the whole dispatch in `PgScope::member(team_id)` (admin keys get
//! `PgScope::admin()`), so `search`/`ask`/`cite`/`who_knows` read only
//! `company` + own-team rows — RLS re-checked underneath — `remember`
//! stamps `visibility=team` + the resolved `team_id`, and governance tools
//! (`promote`, `ontology_promote`, `ontology_propose`) deny for members.
//!
//! The `hub_mcp` module is the Postgres end-to-end path (real `serve_with`
//! socket; provisioning mirrors `tests/connect_hub.rs`). The `solo_lane`
//! module is the regression half — shared-secret and SQLite seat tokens must
//! behave exactly as before, hub envs removed.

use kurultai::daemon::DaemonStatus;
use kurultai::embed::NullEmbedder;
use kurultai::http::{serve_with, HubGate, ServeOptions};
use kurultai::mcp::brain::BrainService;
use kurultai::rerank::NullReranker;
use kurultai::store::{SqliteVecStore, Store};
use kurultai::synthesize::ExtractiveSynthesizer;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Minimal HTTP/1.1 client (chaos.rs pattern — no reqwest dev-dep).
/// `bearer` adds an `Authorization: Bearer …` header when `Some`.
fn http(
    port: u16,
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = match TcpStream::connect(("127.0.0.1", port)) {
        Ok(s) => s,
        Err(_) => return (0, String::new()),
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    let auth = bearer
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let req = match body {
        Some(b) => format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n{auth}Content-Length: {}\r\nConnection: close\r\n\r\n{b}",
            b.len()
        ),
        None => {
            format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}Connection: close\r\n\r\n")
        }
    };
    stream.write_all(req.as_bytes()).unwrap();
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body_out = text.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status, body_out)
}

/// `POST /mcp` JSON-RPC. Returns `(http_status, response_json)`.
fn mcp(port: u16, bearer: Option<&str>, method: &str, params: Value) -> (u16, Value) {
    let req = json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}).to_string();
    let (status, body) = http(port, "POST", "/mcp", bearer, Some(&req));
    (status, serde_json::from_str(&body).unwrap_or(Value::Null))
}

/// `tools/call` helper — asserts HTTP 200 and returns the JSON-RPC body.
fn tool_call(port: u16, bearer: &str, name: &str, args: Value) -> Value {
    let (status, body) = mcp(
        port,
        Some(bearer),
        "tools/call",
        json!({"name": name, "arguments": args}),
    );
    assert_eq!(status, 200, "tools/call {name} http status");
    body
}

fn tool_names(resp: &Value) -> Vec<String> {
    resp["result"]["tools"]
        .as_array()
        .map(|t| {
            t.iter()
                .filter_map(|t| t["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn wait_healthy(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if http(port, "GET", "/health", None, None).0 == 200 {
            return;
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    panic!("daemon did not become healthy on :{port}");
}

/// Solo-lane regression (U5 must not change it): shared-secret keeps the
/// read-only surface, SQLite seat tokens keep the full one — with every hub
/// env explicitly removed so a CI postgres lane can't leak a key store in.
mod solo_lane {
    use super::*;
    use kurultai::testutil::EnvGuard;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn seat_and_secret_paths_unchanged_without_hub() {
        let _env = EnvGuard::apply(&[
            ("KURULTAI_FEATURE_HUB", None),
            ("KURULTAI_DATABASE_URL", None),
            ("DATABASE_URL", None),
            ("KURULTAI_HUB_AUTH", None),
            ("KURULTAI_HUB_API_KEYS", None),
            ("KURULTAI_PG_SKIP_MIGRATE", None),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(SqliteVecStore::open(dir.path().join("store.db"), 4).unwrap());
        let (_agent, seat) = store
            .issue_agent_seat_token("devin", "laptop", "test")
            .await
            .unwrap();
        let brain = BrainService::new(
            store.clone() as Arc<dyn Store>,
            Arc::new(NullEmbedder::new(4)),
            Arc::new(NullReranker::new()),
            Arc::new(ExtractiveSynthesizer::new()),
        );
        let port = free_port();
        tokio::spawn(serve_with(
            brain,
            Arc::new(DaemonStatus::default()),
            ServeOptions {
                port,
                mcp_http_secret: Some("solo-secret".into()),
                bind_all: false,
                bind: Some("127.0.0.1".into()),
                hub: HubGate::default(),
            },
        ));
        wait_healthy(port);

        // Shared secret → read-only surface (unchanged solo behavior).
        let (_s, list) = mcp(port, Some("solo-secret"), "tools/list", json!({}));
        let names = tool_names(&list);
        assert!(names.iter().any(|n| n == "search"));
        assert!(
            !names.iter().any(|n| n == "remember"),
            "secret surface stays read-only"
        );

        // Seat token → full surface + remember round-trips (unchanged).
        let (_s, list) = mcp(port, Some(&seat), "tools/list", json!({}));
        let names = tool_names(&list);
        assert!(names.iter().any(|n| n == "remember"));
        let resp = tool_call(
            port,
            &seat,
            "remember",
            json!({
                "title": "solo seat regression",
                "summary": "seat token remember still lands",
                "tags": ["u5-regression"]
            }),
        );
        let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
        assert!(text.contains("remembered"), "seat remember works: {text}");

        // Unknown bearer still denies at the edge.
        let (status, _b) = mcp(port, Some("bogus"), "tools/list", json!({}));
        assert_eq!(status, 401);
    }
}

#[cfg(feature = "postgres")]
mod hub_mcp {
    use super::*;
    use kurultai::http::HubAuth;
    use kurultai::hub::HubKeyStore;
    use kurultai::store::postgres::PostgresStore;
    use kurultai::testutil::EnvGuard;
    use sqlx::{Connection, PgConnection};

    /// Ids inside a `tools/call` search result (`AgentAtomView` JSON array
    /// rendered into the text block).
    fn search_ids(resp: &Value) -> Vec<String> {
        let text = resp["result"]["content"][0]["text"]
            .as_str()
            .expect("search result text");
        let views: Vec<Value> = serde_json::from_str(text).expect("views json");
        views
            .iter()
            .filter_map(|v| v["id"].as_str().map(str::to_string))
            .collect()
    }

    const APP_PW: &str = "hub_mcp_app_pw";
    const MIGRATE_PW: &str = "hub_mcp_migrate_pw";
    const OP_KEY: &str = "hub-mcp-op-static";
    const MCP_SECRET: &str = "hub-mcp-shared-secret";

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

    /// Advisory lock id — distinct from migrate (87231401), hub_rls
    /// (87231402) and connect_hub's provisioning locks.
    const PROVISION_LOCK: i64 = 87231405;

    /// Idempotent role/grant bootstrap — mirror of `init-hub-db.sql`; the
    /// lock serializes `CREATE ROLE`/`GRANT` across parallel test processes.
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

        // Tables must exist before grants cover them — migrate as superuser.
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
    /// team, company. Title carries the id so plain-FTS `q=<run>` matches
    /// all three. Returns `(run, eng_id, sales_id, co_id)`.
    async fn fixtures(admin_dsn: &str) -> (String, String, String, String) {
        let mut admin = connect(admin_dsn).await;
        let run = format!("mx{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
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
                 ) VALUES ($1,'hub-mcp-test',$2,$1,'',$3,\
                    '2026-10-20T00:00:00Z','2026-10-20T00:00:00Z',$4,$5)",
            )
            .bind(&id)
            .bind(format!("/mcp/{id}"))
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
        sqlx::query("DELETE FROM hub_api_keys WHERE agent_id LIKE $1")
            .bind(format!("{prefix}-%"))
            .execute(&mut admin)
            .await
            .expect("cleanup keys");
    }

    /// Spawn the real `serve_with` daemon on loopback against the app-role
    /// Postgres store. `EnvGuard` must already hold `KURULTAI_FEATURE_HUB=1`,
    /// `KURULTAI_DATABASE_URL` (app DSN) and `KURULTAI_PG_SKIP_MIGRATE=1` —
    /// `/mcp`'s hub-key store resolves lazily from those same env vars.
    async fn spawn_hub(admin_dsn: &str) -> (u16, Arc<HubKeyStore>, Arc<PostgresStore>) {
        let app_dsn = dsn_for(admin_dsn, "kurultai_app", APP_PW);
        let store = Arc::new(
            PostgresStore::connect(&app_dsn, 4)
                .await
                .expect("app-role PostgresStore"),
        );
        let keys = Arc::new(
            HubKeyStore::connect(&app_dsn)
                .await
                .expect("app-role key store"),
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
        let port = free_port();
        tokio::spawn(serve_with(
            brain,
            Arc::new(DaemonStatus::default()),
            ServeOptions {
                port,
                mcp_http_secret: Some(MCP_SECRET.into()),
                bind_all: false,
                bind: Some("127.0.0.1".into()),
                hub: gate,
            },
        ));
        wait_healthy(port);
        (port, keys, store)
    }

    /// Held for the duration of a test — hub feature + app-role DSN +
    /// skip-migrate, the same env seam the deployed hub daemon uses.
    fn hub_env(admin_dsn: &str) -> EnvGuard {
        let app_dsn: &'static str =
            Box::leak(dsn_for(admin_dsn, "kurultai_app", APP_PW).into_boxed_str());
        EnvGuard::apply(&[
            ("KURULTAI_FEATURE_HUB", Some("1")),
            ("KURULTAI_DATABASE_URL", Some(app_dsn)),
            ("KURULTAI_PG_SKIP_MIGRATE", Some("1")),
        ])
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn member_search_scopes_to_team_and_company() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (run, eng, sales, co) = fixtures(&dsn).await;
        let _env = hub_env(&dsn);
        let (port, keys, _store) = spawn_hub(&dsn).await;
        let (member_token, _id) = keys.issue(&format!("{run}-cursor"), "eng").await.unwrap();

        let resp = tool_call(port, &member_token, "search", json!({"query": run}));
        let ids = search_ids(&resp);
        assert!(ids.contains(&eng), "member sees own team: {ids:?}");
        assert!(ids.contains(&co), "member sees company: {ids:?}");
        assert!(!ids.contains(&sales), "member must NOT see sales: {ids:?}");

        // cite goes through `get_by_source_id` — RLS alone hides sales rows.
        let resp = tool_call(
            port,
            &member_token,
            "cite",
            json!({"source": "hub-mcp-test", "source_id": format!("/mcp/{sales}")}),
        );
        let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
        assert!(
            text.contains("No atom"),
            "member cite must miss sales: {text}"
        );

        cleanup(&dsn, &run).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn member_remember_stamps_team_visibility() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let run = format!("mx{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let _env = hub_env(&dsn);
        let (port, keys, _store) = spawn_hub(&dsn).await;
        let (member_token, _id) = keys.issue(&format!("{run}-codex"), "eng").await.unwrap();

        let resp = tool_call(
            port,
            &member_token,
            "remember",
            json!({
                "title": format!("{run} member write"),
                "summary": "remembered via hub mcp",
                "tags": ["hub-mcp-test"],
                "project": "hub-mcp"
            }),
        );
        let text = resp["result"]["content"][0]["text"].as_str().unwrap_or("");
        let id = text
            .split_whitespace()
            .find_map(|t| t.strip_prefix("id="))
            .expect("remember id")
            .to_string();

        let mut admin = connect(&dsn).await;
        let (visibility, team_id): (String, Option<String>) =
            sqlx::query_as("SELECT visibility, team_id FROM knowledge_atoms WHERE id = $1")
                .bind(&id)
                .fetch_one(&mut admin)
                .await
                .expect("remembered row");
        assert_eq!(visibility, "team", "member write lands visibility=team");
        assert_eq!(
            team_id.as_deref(),
            Some("eng"),
            "member write lands team_id"
        );

        sqlx::query("DELETE FROM knowledge_atoms WHERE id = $1")
            .bind(&id)
            .execute(&mut admin)
            .await
            .expect("cleanup remembered atom");
        cleanup(&dsn, &run).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn member_governance_tools_denied() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let run = format!("mx{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let _env = hub_env(&dsn);
        let (port, keys, _store) = spawn_hub(&dsn).await;
        let (member_token, _id) = keys.issue(&format!("{run}-hermes"), "eng").await.unwrap();

        // tools/list omits governance writes on the member surface.
        let (_s, list) = mcp(port, Some(&member_token), "tools/list", json!({}));
        let names = tool_names(&list);
        for t in ["promote", "ontology_promote", "ontology_propose"] {
            assert!(!names.iter().any(|n| n == t), "member surface lists {t}");
        }
        for t in ["search", "remember", "ask", "who_knows", "cite"] {
            assert!(names.iter().any(|n| n == t), "member surface needs {t}");
        }

        // Direct calls still deny — the request-scope gate is the real check.
        for (tool, args) in [
            ("promote", json!({"atom_id": "x"})),
            ("ontology_promote", json!({"atom_id": "x", "class_id": "c"})),
            (
                "ontology_propose",
                json!({"kind": "new_entity", "payload": {"entity_kind": "k", "name": "n"}}),
            ),
        ] {
            let resp = tool_call(port, &member_token, tool, args);
            let msg = resp["error"]["message"].as_str().unwrap_or("");
            assert!(
                msg.contains("admin"),
                "{tool} must deny member keys: {resp}"
            );
        }
        cleanup(&dsn, &run).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn admin_key_reads_all_teams() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (run, eng, sales, co) = fixtures(&dsn).await;
        let _env = hub_env(&dsn);
        let (port, keys, _store) = spawn_hub(&dsn).await;
        let (admin_token, _id) = keys
            .issue_admin(&format!("{run}-root"), "ops", true)
            .await
            .unwrap();

        // Full tool surface, all teams visible.
        let (_s, list) = mcp(port, Some(&admin_token), "tools/list", json!({}));
        assert!(
            tool_names(&list).iter().any(|n| n == "promote"),
            "admin surface lists governance tools"
        );
        let resp = tool_call(port, &admin_token, "search", json!({"query": run}));
        let ids = search_ids(&resp);
        for want in [&eng, &sales, &co] {
            assert!(ids.contains(want), "admin sees {want}: {ids:?}");
        }
        cleanup(&dsn, &run).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn revoked_key_rejected_at_edge() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let run = format!("mx{}", &uuid::Uuid::new_v4().simple().to_string()[..8]);
        let _env = hub_env(&dsn);
        let (port, keys, _store) = spawn_hub(&dsn).await;
        let (token, id) = keys.issue(&format!("{run}-gone"), "eng").await.unwrap();
        // A second active key keeps `has_active_keys` true so the revoked one
        // hard-denies rather than falling through to the seat path.
        let (_keep, _kid) = keys.issue(&format!("{run}-stay"), "eng").await.unwrap();
        assert!(keys.revoke_by_id(id).await.unwrap(), "revoke lands");

        let (status, _body) = mcp(port, Some(&token), "tools/list", json!({}));
        assert_eq!(status, 401, "revoked hub key must 401");
        cleanup(&dsn, &run).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn shared_secret_stays_readonly_on_hub() {
        let Some(dsn) = test_dsn() else { return };
        provision(&dsn).await;
        let (run, eng, _sales, co) = fixtures(&dsn).await;
        let _env = hub_env(&dsn);
        let (port, keys, _store) = spawn_hub(&dsn).await;
        // One active key so the hub gate is unmistakably live.
        let (_t, _id) = keys.issue(&format!("{run}-noise"), "eng").await.unwrap();

        // tools/list under the shared secret keeps the read-only surface…
        let (_s, list) = mcp(port, Some(MCP_SECRET), "tools/list", json!({}));
        let names = tool_names(&list);
        assert!(names.iter().any(|n| n == "search"));
        assert!(!names.iter().any(|n| n == "remember"));

        // …and reads fail closed to `company` visibility only (the same
        // "authenticated without a member principal" ruling as REST KTD1).
        let resp = tool_call(port, MCP_SECRET, "search", json!({"query": run}));
        let ids = search_ids(&resp);
        assert!(ids.contains(&co), "secret sees company rows: {ids:?}");
        assert!(
            !ids.contains(&eng),
            "secret must NOT see team rows: {ids:?}"
        );
        cleanup(&dsn, &run).await;
    }
}
