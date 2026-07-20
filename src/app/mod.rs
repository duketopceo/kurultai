use crate::config;
use crate::connectors;
use crate::embed::{self, Embedder};
use crate::query::{DefaultQueryEngine, QueryEngine};
use crate::store::{SqliteVecStore, Store};
use crate::types::{Config, KurultaiEnv, SourceKind};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppOpts {
    pub env: KurultaiEnv,
    pub config_override: Option<PathBuf>,
}

pub enum Command {
    Init { vault: Option<PathBuf>, force: bool },
    Index { full: bool },
    Ask { question: String },
    Search { query: String, limit: usize },
    Status,
    Doctor,
    Install { client: InstallClient },
    Mcp,
    Daemon { port: u16 },
}

#[derive(Debug, Clone, Copy)]
pub enum InstallClient {
    Cursor,
    Claude,
}

pub struct Runtime {
    pub config: Config,
    pub store: Arc<dyn Store>,
    pub embedder: Arc<dyn Embedder>,
    pub query: Arc<dyn QueryEngine>,
}

impl Runtime {
    pub fn open(config: Config) -> Result<Self> {
        let storage = config::resolve_storage_path(&config)?;
        let store: Arc<dyn Store> = Arc::new(SqliteVecStore::open(
            &storage,
            &config.embed_model,
            config.embed_dim,
        )?);
        let embedder: Arc<dyn Embedder> = Arc::from(embed::embedder_from_env(
            &config.embed_model,
            config.embed_dim,
            &config.openrouter_api_key_env,
        ));
        let query: Arc<dyn QueryEngine> = Arc::new(DefaultQueryEngine::new(
            Arc::clone(&store),
            Arc::clone(&embedder),
        ));
        Ok(Self {
            config,
            store,
            embedder,
            query,
        })
    }

    pub async fn index(&self, full: bool) -> Result<IndexReport> {
        let mut report = IndexReport::default();
        for src in self.config.sources.iter().filter(|s| s.enabled) {
            if !src.kind.is_implemented() {
                report.skipped.push(format!(
                    "{} ({}) — not implemented",
                    src.name,
                    src.kind.as_str()
                ));
                continue;
            }
            let mut connector = connectors::build_connector(src)?;
            connector.init(src).await?;
            let atoms = if full {
                connector.full_sync().await?
            } else {
                connector.poll().await?
            };
            let mut keep_ids = Vec::new();
            let mut upserted = 0u64;
            let mut skipped_unchanged = 0u64;
            for mut atom in atoms {
                keep_ids.push(atom.source_id.clone());
                if let Some(existing) =
                    self.store.get_by_source_id(&atom.source, &atom.source_id)?
                {
                    if existing.content_hash == atom.content_hash {
                        skipped_unchanged += 1;
                        continue;
                    }
                    // Content changed: drop old content-addressed row.
                    self.store.delete_id(&existing.id)?;
                }
                // Embed when API key available; otherwise FTS-only atoms.
                if self.embedder.name() != "fts-only" {
                    match self.embedder.embed(&atom.content).await {
                        Ok(emb) => atom.embedding = Some(emb),
                        Err(e) => {
                            tracing::warn!("embed failed for {}: {e}", atom.source_id);
                        }
                    }
                }
                self.store.upsert(&atom)?;
                upserted += 1;
            }
            if full {
                let orphans = self.store.delete_orphans(connector.name(), &keep_ids)?;
                report.orphans_removed += orphans;
            }
            report.upserted += upserted;
            report.skipped_unchanged += skipped_unchanged;
            report.sources.push(connector.name().to_string());
        }
        Ok(report)
    }
}

#[derive(Debug, Default)]
pub struct IndexReport {
    pub upserted: u64,
    pub skipped_unchanged: u64,
    pub orphans_removed: u64,
    pub sources: Vec<String>,
    pub skipped: Vec<String>,
}

pub async fn run(opts: AppOpts, cmd: Command) -> Result<()> {
    match cmd {
        Command::Init { vault, force } => cmd_init(opts.env, vault, force),
        Command::Index { full } => {
            let rt = open_runtime(&opts)?;
            let report = rt.index(full).await?;
            println!(
                "Indexed: upserted={} unchanged={} orphans_removed={} sources={:?}",
                report.upserted, report.skipped_unchanged, report.orphans_removed, report.sources
            );
            for s in report.skipped {
                println!("  skipped: {s}");
            }
            Ok(())
        }
        Command::Ask { question } => {
            let rt = open_runtime(&opts)?;
            let answer = rt.query.ask(&question).await?;
            println!("Q: {}", answer.question);
            println!("A: {}", answer.answer);
            for c in answer.citations {
                println!(
                    "  - [{}] {} ({})",
                    c.source,
                    c.title,
                    c.url.unwrap_or_default()
                );
            }
            Ok(())
        }
        Command::Search { query, limit } => {
            let rt = open_runtime(&opts)?;
            let results = rt.query.search(&query, limit).await?;
            if results.is_empty() {
                println!("No results.");
            }
            for r in results {
                println!(
                    "{}. [{:.3}] {} — {} ({})",
                    r.rank,
                    r.score,
                    r.atom.title,
                    r.atom.source_id,
                    r.matched_by.join("+")
                );
            }
            Ok(())
        }
        Command::Status => cmd_status(&opts),
        Command::Doctor => cmd_doctor(&opts),
        Command::Install { client } => cmd_install(client),
        Command::Mcp => {
            let rt = open_runtime(&opts)?;
            crate::mcp::serve_stdio(rt).await
        }
        Command::Daemon { port } => {
            bail!("HTTP daemon not productized in Phase 1 (port={port}); use CLI or MCP")
        }
    }
}

fn open_runtime(opts: &AppOpts) -> Result<Runtime> {
    let config = if let Some(ref p) = opts.config_override {
        let text =
            std::fs::read_to_string(p).with_context(|| format!("read config {}", p.display()))?;
        let mut cfg: Config = toml::from_str(&text)?;
        cfg.env = opts.env;
        cfg
    } else {
        config::load_config(opts.env)?
    };
    Runtime::open(config)
}

fn cmd_init(env: KurultaiEnv, vault: Option<PathBuf>, force: bool) -> Result<()> {
    let path = config::config_path(env)?;
    if path.exists() && !force {
        bail!(
            "config already exists at {} (use --force to overwrite)",
            path.display()
        );
    }
    let cfg = config::default_init_config(env, vault);
    config::write_config(&cfg, &path)?;
    let store = config::default_storage_path(env)?;
    println!("Wrote config: {}", path.display());
    println!("Default store: {}", store.display());
    println!("Registered connector kinds: {:?}", SourceKind::registered());
    Ok(())
}

fn cmd_status(opts: &AppOpts) -> Result<()> {
    let config_path = config::config_path(opts.env)?;
    println!("Kurultai status");
    println!("  env: {}", opts.env.as_str());
    println!("  config: {}", config_path.display());
    println!(
        "  registered kinds: {}",
        SourceKind::registered()
            .iter()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    match open_runtime(opts) {
        Ok(rt) => {
            let storage = config::resolve_storage_path(&rt.config)?;
            println!("  store: {}", storage.display());
            println!("  atoms: {}", rt.store.count()?);
            println!(
                "  embed: model={} dim={} mode={}",
                rt.config.embed_model,
                rt.config.embed_dim,
                rt.embedder.name()
            );
            println!("  sources:");
            for s in &rt.config.sources {
                let state = if !s.enabled {
                    "disabled"
                } else if s.kind.is_implemented() {
                    "ok"
                } else {
                    "unimplemented"
                };
                println!(
                    "    - {} kind={} enabled={} ({state})",
                    s.name,
                    s.kind.as_str(),
                    s.enabled
                );
            }
        }
        Err(e) => {
            println!("  store: not open ({e})");
        }
    }
    Ok(())
}

fn cmd_doctor(opts: &AppOpts) -> Result<()> {
    let mut ok = true;
    let cfg_path = config::config_path(opts.env)?;
    if cfg_path.exists() {
        println!("OK  config {}", cfg_path.display());
    } else {
        println!("ERR config missing — run kurultai init");
        ok = false;
    }
    match open_runtime(opts) {
        Ok(rt) => {
            println!("OK  store {}", rt.store.path().display());
            println!("OK  atoms={}", rt.store.count()?);
            let key = std::env::var(&rt.config.openrouter_api_key_env).unwrap_or_default();
            if key.trim().is_empty() {
                println!(
                    "WARN {} unset — FTS-only mode",
                    rt.config.openrouter_api_key_env
                );
            } else {
                println!(
                    "OK  {} set — embeddings enabled",
                    rt.config.openrouter_api_key_env
                );
            }
        }
        Err(e) => {
            println!("ERR runtime: {e}");
            ok = false;
        }
    }
    if !ok {
        bail!("doctor found problems");
    }
    println!("doctor: all critical checks passed");
    Ok(())
}

fn cmd_install(client: InstallClient) -> Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;
    let entry = serde_json::json!({
        "command": exe,
        "args": ["mcp"],
    });
    match client {
        InstallClient::Cursor => {
            let path = dirs_home()?.join(".cursor/mcp.json");
            merge_mcp_json(&path, "kurultai", entry)?;
            println!("Installed Cursor MCP entry → {}", path.display());
        }
        InstallClient::Claude => {
            let path = dirs_home()?.join(".claude.json");
            // Claude Code style: mcpServers under config root
            merge_claude_mcp(&path, "kurultai", entry)?;
            println!("Installed Claude MCP entry → {}", path.display());
        }
    }
    Ok(())
}

fn dirs_home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("HOME not set"))
}

fn merge_mcp_json(path: &PathBuf, name: &str, server: serde_json::Value) -> Result<()> {
    let mut root = if path.exists() {
        let text = std::fs::read_to_string(path)?;
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if root.get("mcpServers").is_none() {
        root["mcpServers"] = serde_json::json!({});
    }
    root["mcpServers"][name] = server;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_claude_mcp(path: &PathBuf, name: &str, server: serde_json::Value) -> Result<()> {
    merge_mcp_json(path, name, server)
}
