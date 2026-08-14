//! `kurultai doctor` — diagnostic spine.
//!
//! Runs a battery of checks against the local install and prints a clean
//! PASS/FAIL/WARN table with actionable remediation hints. Designed for Khan
//! to run after a desktop install to quickly see what works and what needs
//! attention (missing config, no API key, daemon not running, …).

use crate::config::{config_path, expand_path, load_config_with_env};
use crate::embed::Embedder;
use crate::environment::Environment;
use crate::error::Result;
use crate::mcp::server::ToolSurface;
use crate::store::{migrations, SqliteVecStore, Store};
use crate::types::Config;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Result of a single doctor check.
#[derive(Debug, Clone)]
struct CheckResult {
    name: &'static str,
    status: Status,
    detail: String,
    hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Pass,
    Warn,
    Fail,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Warn => "WARN",
            Status::Fail => "FAIL",
        }
    }
}

/// Default daemon port probed by the HTTP check.
const DEFAULT_PORT: u16 = 8421;

/// Run all doctor checks and print the table. Returns `Ok(())` when no check
/// hard-failed (WARN is acceptable); returns the first failure otherwise so the
/// CLI exit code is non-zero when something is broken.
pub async fn run(env_override: Option<&str>, config_override: Option<&Path>) -> Result<()> {
    println!("kurultai doctor — diagnostic checks\n");

    let mut results = Vec::new();

    // 1. Config
    let config = check_config(&mut results, env_override, config_override);

    // 2. Database (only if config loaded)
    let store: Option<Arc<SqliteVecStore>> = match &config {
        Some(cfg) => check_database(&mut results, cfg),
        None => {
            results.push(CheckResult {
                name: "database",
                status: Status::Fail,
                detail: "skipped — config did not load".into(),
                hint: Some("Run `kurultai init` first".into()),
            });
            None
        }
    };

    // 3. Embeddings
    check_embeddings(&mut results, &config, env_override).await;

    // 4. Ontology
    check_ontology(&mut results, store.as_deref()).await;

    // 5. MCP
    check_mcp(&mut results, store.clone()).await;

    // 6. HTTP daemon
    check_http_daemon(&mut results).await;

    // 7. Connectors
    check_connectors(&mut results, &config, env_override).await;

    print_table(&results);

    let any_fail = results.iter().any(|r| r.status == Status::Fail);
    if any_fail {
        eprintln!("\nOne or more checks failed. See the FAIL rows above for remediation.");
        std::process::exit(1);
    }
    Ok(())
}

// ── Individual checks ──────────────────────────────────────────────────────

/// Load + validate config; report key settings.
fn check_config(
    results: &mut Vec<CheckResult>,
    env_override: Option<&str>,
    config_override: Option<&Path>,
) -> Option<Config> {
    let path_display = match config_override {
        Some(p) => p.display().to_string(),
        None => match config_path() {
            Ok(p) => p.display().to_string(),
            Err(_) => "(unset)".into(),
        },
    };

    match load_config_with_env(config_override, env_override) {
        Ok(cfg) => {
            results.push(CheckResult {
                name: "config",
                status: Status::Pass,
                detail: format!(
                    "env={} store={} hub={} sources={}",
                    cfg.environment,
                    cfg.storage_path,
                    if cfg.embed_backend.as_deref() == Some("local") {
                        "off"
                    } else {
                        "auto"
                    },
                    cfg.sources.len()
                ),
                hint: None,
            });
            Some(cfg)
        }
        Err(e) => {
            results.push(CheckResult {
                name: "config",
                status: Status::Fail,
                detail: format!("failed to load from {path_display}: {e}"),
                hint: Some("Run `kurultai init` to write a default config.toml".into()),
            });
            None
        }
    }
}

/// Open the SQLite store, check schema version + FTS5/vec0 extensions.
fn check_database(results: &mut Vec<CheckResult>, config: &Config) -> Option<Arc<SqliteVecStore>> {
    let storage_path = match expand_path(&config.storage_path) {
        Ok(p) => p,
        Err(e) => {
            results.push(CheckResult {
                name: "database",
                status: Status::Fail,
                detail: format!("storage_path '{}' invalid: {e}", config.storage_path),
                hint: Some("Fix [storage] path in config.toml".into()),
            });
            return None;
        }
    };

    let exists = storage_path.exists();

    // Ensure parent dir exists so SqliteVecStore::open can create the file.
    if !exists {
        if let Some(parent) = storage_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    match SqliteVecStore::open(storage_path.clone(), config.embed_dim) {
        Ok(store) => {
            // Probe FTS5 + vec0 by reading sqlite_master for the virtual tables.
            let (fts_ok, vec_ok, schema_v) = probe_store_internals(&store);
            let status = if !exists {
                Status::Warn
            } else if fts_ok && vec_ok {
                Status::Pass
            } else {
                Status::Warn
            };
            results.push(CheckResult {
                name: "database",
                status,
                detail: format!(
                    "schema=v{} fts5={} vec0={} dim={} ({})",
                    schema_v,
                    yes_no(fts_ok),
                    yes_no(vec_ok),
                    config.embed_dim,
                    storage_path.display()
                ),
                hint: if !exists {
                    Some("Store was just created — run `kurultai index` to populate it".into())
                } else if fts_ok && vec_ok {
                    None
                } else {
                    Some("Missing FTS5/vec0 — rebuild the store or re-run migrations".into())
                },
            });
            Some(Arc::new(store))
        }
        Err(e) => {
            results.push(CheckResult {
                name: "database",
                status: Status::Fail,
                detail: format!("failed to open {}: {e}", storage_path.display()),
                hint: Some("Delete the store file and run `kurultai index` to rebuild".into()),
            });
            None
        }
    }
}

/// Inspect the store's sqlite_master for FTS5 + vec0 virtual tables + schema version.
fn probe_store_internals(store: &SqliteVecStore) -> (bool, bool, i32) {
    use rusqlite::Connection;
    let path = store.path();
    let conn = match Connection::open(path) {
        Ok(c) => c,
        Err(_) => return (false, false, 0),
    };

    // FTS5 + vec0 present?
    let fts_ok: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='atoms_fts'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);
    let vec_ok: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='atoms_vec'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(false);

    let schema_v = migrations::current_applied_version(&conn);
    (fts_ok, vec_ok, schema_v)
}

fn yes_no(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

/// Is the embedding model configured + can it generate a vector?
async fn check_embeddings(
    results: &mut Vec<CheckResult>,
    config: &Option<Config>,
    env_override: Option<&str>,
) {
    let Some(cfg) = config else {
        results.push(CheckResult {
            name: "embeddings",
            status: Status::Fail,
            detail: "skipped — config did not load".into(),
            hint: Some("Run `kurultai init` first".into()),
        });
        return;
    };

    let environment = match Environment::resolve(env_override) {
        Ok(e) => e,
        Err(e) => {
            results.push(CheckResult {
                name: "embeddings",
                status: Status::Fail,
                detail: format!("environment resolve failed: {e}"),
                hint: None,
            });
            return;
        }
    };

    // Reuse the same build logic as App::from_config.
    let embedder = match crate::app::context::build_embedder(cfg, environment) {
        Ok(e) => e,
        Err(e) => {
            results.push(CheckResult {
                name: "embeddings",
                status: Status::Fail,
                detail: format!("embedder build failed: {e}"),
                hint: Some("Set OPENROUTER_API_KEY or embed.backend=local".into()),
            });
            return;
        }
    };

    if !embedder.is_live() {
        results.push(CheckResult {
            name: "embeddings",
            status: Status::Warn,
            detail: format!(
                "model={} dim={} (FTS-only — NullEmbedder)",
                embedder.name(),
                embedder.dim()
            ),
            hint: Some(
                "Set OPENROUTER_API_KEY (or KURULTAI_API_KEY) for cloud embeddings, or \
                 embed.backend=\"local\" + `--features local-embed`"
                    .into(),
            ),
        });
        return;
    }

    // Live embedder — try generating a vector.
    match tokio::time::timeout(Duration::from_secs(15), embedder.embed("doctor probe")).await {
        Ok(Ok(vec)) => {
            results.push(CheckResult {
                name: "embeddings",
                status: Status::Pass,
                detail: format!(
                    "model={} dim={} generated {}-dim vector",
                    embedder.name(),
                    embedder.dim(),
                    vec.len()
                ),
                hint: None,
            });
        }
        Ok(Err(e)) => {
            results.push(CheckResult {
                name: "embeddings",
                status: Status::Fail,
                detail: format!("embed call failed: {e}"),
                hint: Some("Check API key / network / model name".into()),
            });
        }
        Err(_) => {
            results.push(CheckResult {
                name: "embeddings",
                status: Status::Fail,
                detail: "embed call timed out after 15s".into(),
                hint: Some("Check network connectivity / API endpoint reachability".into()),
            });
        }
    }
}

/// Ontology tables present? How many entities/links?
async fn check_ontology(results: &mut Vec<CheckResult>, store: Option<&SqliteVecStore>) {
    let Some(store) = store else {
        results.push(CheckResult {
            name: "ontology",
            status: Status::Warn,
            detail: "skipped — database not open".into(),
            hint: None,
        });
        return;
    };

    let entities = store.list_ontology_entities(100_000).await;
    let links = store.list_ontology_links(None).await;

    match (entities, links) {
        (Ok(e), Ok(l)) => {
            results.push(CheckResult {
                name: "ontology",
                status: Status::Pass,
                detail: format!("{} entities, {} links", e.len(), l.len()),
                hint: None,
            });
        }
        (Err(e), _) | (_, Err(e)) => {
            results.push(CheckResult {
                name: "ontology",
                status: Status::Fail,
                detail: format!("ontology read failed: {e}"),
                hint: Some("Re-run migrations: delete store + `kurultai index`".into()),
            });
        }
    }
}

/// Is the MCP server enumerable? (Does not start the stdio loop — uses handle_message.)
async fn check_mcp(results: &mut Vec<CheckResult>, store: Option<Arc<SqliteVecStore>>) {
    let Some(sqlite_store) = store else {
        results.push(CheckResult {
            name: "mcp",
            status: Status::Warn,
            detail: "skipped — database not open".into(),
            hint: None,
        });
        return;
    };

    let store: Arc<dyn Store> = sqlite_store.clone();
    let embed_dim = sqlite_store.embed_dim();
    let embedder: Arc<dyn Embedder> = Arc::new(crate::embed::NullEmbedder::new(embed_dim));
    let reranker = Arc::new(crate::rerank::NullReranker::new());
    let synthesizer = Arc::new(crate::synthesize::ExtractiveSynthesizer::new());
    let brain = crate::mcp::BrainService::new(store, embedder, reranker, synthesizer);

    // Use the public handle_message path to list tools (no stdio loop).
    let list_msg = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"});
    match crate::mcp::server::handle_message(&brain, list_msg, ToolSurface::Full).await {
        Ok(Some(resp)) => {
            let tools_arr = resp
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|t| t.as_array());
            let count = tools_arr.map(|a| a.len()).unwrap_or(0);
            let names: Vec<String> = tools_arr
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            results.push(CheckResult {
                name: "mcp",
                status: Status::Pass,
                detail: format!("{count} tools: {}", names.join(", ")),
                hint: None,
            });
        }
        Ok(None) => {
            results.push(CheckResult {
                name: "mcp",
                status: Status::Warn,
                detail: "tools/list returned no response (notification?)".into(),
                hint: None,
            });
        }
        Err(e) => {
            results.push(CheckResult {
                name: "mcp",
                status: Status::Fail,
                detail: format!("tools/list failed: {e}"),
                hint: Some("Check store + embedder wiring".into()),
            });
        }
    }
}

/// Is the daemon running on localhost:8421? Can it answer /api/graph?
async fn check_http_daemon(results: &mut Vec<CheckResult>) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    // /health first (cheap).
    let health_url = format!("http://127.0.0.1:{DEFAULT_PORT}/health");
    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Now probe /api/graph.
            let graph_url = format!("http://127.0.0.1:{DEFAULT_PORT}/api/graph?limit=1");
            match client.get(&graph_url).send().await {
                Ok(g) if g.status().is_success() => {
                    results.push(CheckResult {
                        name: "http_daemon",
                        status: Status::Pass,
                        detail: format!("running on :{DEFAULT_PORT} (/health + /api/graph OK)"),
                        hint: None,
                    });
                }
                Ok(g) => {
                    results.push(CheckResult {
                        name: "http_daemon",
                        status: Status::Warn,
                        detail: format!("running but /api/graph returned HTTP {}", g.status()),
                        hint: Some("Check daemon logs (KURULTAI_LOG=debug)".into()),
                    });
                }
                Err(e) => {
                    results.push(CheckResult {
                        name: "http_daemon",
                        status: Status::Warn,
                        detail: format!("/health OK but /api/graph failed: {e}"),
                        hint: None,
                    });
                }
            }
        }
        Ok(resp) => {
            results.push(CheckResult {
                name: "http_daemon",
                status: Status::Warn,
                detail: format!(
                    "port {DEFAULT_PORT} responded HTTP {} (not /health)",
                    resp.status()
                ),
                hint: Some(format!(
                    "Is `kurultai daemon --port {DEFAULT_PORT}` running?"
                )),
            });
        }
        Err(_) => {
            results.push(CheckResult {
                name: "http_daemon",
                status: Status::Warn,
                detail: format!("not running on :{DEFAULT_PORT}"),
                hint: Some(format!("Start it: `kurultai daemon --port {DEFAULT_PORT}`")),
            });
        }
    }
}

/// Which connectors are registered? Any failing to init?
async fn check_connectors(
    results: &mut Vec<CheckResult>,
    config: &Option<Config>,
    env_override: Option<&str>,
) {
    let Some(cfg) = config else {
        results.push(CheckResult {
            name: "connectors",
            status: Status::Warn,
            detail: "skipped — config did not load".into(),
            hint: None,
        });
        return;
    };

    if cfg.sources.is_empty() {
        results.push(CheckResult {
            name: "connectors",
            status: Status::Warn,
            detail: "no sources configured".into(),
            hint: Some("Add [sources.*] sections to config.toml".into()),
        });
        return;
    }

    let environment = match Environment::resolve(env_override) {
        Ok(e) => e,
        Err(_) => cfg.environment,
    };
    let _ = environment;

    // Rebuild registry (init each connector — surfaces init failures).
    match crate::connectors::ConnectorRegistry::from_config(cfg).await {
        Ok(registry) => {
            let names = registry.names();
            let enabled: Vec<&str> = cfg
                .sources
                .iter()
                .filter(|s| s.enabled)
                .map(|s| s.name.as_str())
                .collect();
            let failing: Vec<&str> = enabled
                .iter()
                .filter(|n| !names.iter().any(|r| *r == **n))
                .copied()
                .collect();
            let status = if failing.is_empty() {
                Status::Pass
            } else {
                Status::Warn
            };
            results.push(CheckResult {
                name: "connectors",
                status,
                detail: format!(
                    "{} registered ({} enabled, {} failed init)",
                    names.len(),
                    enabled.len(),
                    failing.len()
                ),
                hint: if failing.is_empty() {
                    None
                } else {
                    Some(format!(
                        "Failing to init: {}. Check root_path / credentials in config.toml",
                        failing.join(", ")
                    ))
                },
            });
        }
        Err(e) => {
            results.push(CheckResult {
                name: "connectors",
                status: Status::Fail,
                detail: format!("registry build failed: {e}"),
                hint: Some("Check [sources.*] config + credentials".into()),
            });
        }
    }
}

// ── Table rendering ─────────────────────────────────────────────────────────

fn print_table(results: &[CheckResult]) {
    let name_w = results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(8)
        .max(8);

    println!(
        "{:<width$}  {:<6}  {}",
        "CHECK",
        "STATUS",
        "DETAIL",
        width = name_w
    );
    println!("{:-<width$}  {:-<6}  {:-<40}", "", "", "", width = name_w);

    for r in results {
        println!(
            "{:<width$}  {:<6}  {}",
            r.name,
            r.status.label(),
            r.detail,
            width = name_w
        );
        if let Some(hint) = &r.hint {
            println!("{:<width$}  {:<6}  → {}", "", "", hint, width = name_w);
        }
    }
}
