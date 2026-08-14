use clap::{Parser, Subcommand};
use kurultai::app::App;
use kurultai::art::{
    effective_plain, env_no_color_set, print_banner_stdout, ArtVariant, BannerMode,
};
use kurultai::config::{config_path, load_config_from, load_config_with_env};
use kurultai::environment::Environment;
use kurultai::error::Result;
use kurultai::export::{export_pack, import_pack, resolve_config_file, ImportMode};
use kurultai::logging;
use kurultai::mcp::{
    ensure_default_config, init_walkthrough, provision_docs, wire_agent, AgentRead, AgentTarget,
    BrainService,
};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "kurultai",
    version,
    about = "Assemble what you know, from wherever it lives.",
    after_help = "Setup        kurultai init --docs  ·  init --agent <cursor|claude|codex|hermes|all|none>\nKnowledge    index [--full]  ·  search  ·  ask  ·  who-knows  ·  status  ·  promote\nServe        mcp  ·  daemon --port 8421    Brain UI → http://127.0.0.1:8421/ui/\nPacks        export  ·  import\nMaintenance  prune --generated  ·  doctor"
)]
struct Cli {
    /// Log filter (overrides KURULTAI_LOG). Example: kurultai=trace,info
    #[arg(long, global = true)]
    log: Option<String>,

    /// Deployment environment: dev, staging, prod (overrides KURULTAI_ENV)
    #[arg(long, global = true, value_name = "ENV")]
    env: Option<String>,

    /// Path to config file (overrides KURULTAI_CONFIG)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Suppress yurt banner art (also: KURULTAI_PLAIN=1, NO_COLOR)
    #[arg(long, global = true, default_value_t = false)]
    plain: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write default config, optionally provision a docs folder, and wire MCP
    Init {
        /// Agent to wire: cursor, claude, codex, hermes, all, or none
        #[arg(long, default_value = "cursor")]
        agent: AgentTarget,
        /// Provision an on-device markdown folder (default: Documents/kurultai)
        #[arg(long, num_args = 0..=1, default_missing_value = "", value_name = "PATH")]
        docs: Option<String>,
        /// Run a full index after writing config
        #[arg(long)]
        index: bool,
    },
    /// Ingest configured sources into the brain
    Index {
        /// Full re-index instead of incremental
        #[arg(long)]
        full: bool,
    },
    /// Search the knowledge base
    Search {
        /// Search query
        query: String,
        /// Number of results
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Ask a question (extractive without an API key)
    Ask {
        /// The question to answer
        question: String,
    },
    /// Which sources know about a topic
    #[command(name = "who-knows", visible_alias = "who_knows")]
    WhoKnows {
        /// Topic / query
        topic: String,
        /// Max search hits to aggregate
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Environment, sources, atom counts, feature flags
    Status {
        /// Print Prometheus metrics from a running local daemon (`GET /api/metrics`)
        #[arg(long)]
        metrics: bool,
        /// Daemon HTTP port when using `--metrics` (default: 8421)
        #[arg(long, default_value = "8421")]
        port: u16,
    },
    /// Promote a quarantined atom to trusted
    Promote {
        /// Atom id
        atom_id: String,
        /// Optional audit note
        #[arg(long)]
        reason: Option<String>,
    },
    /// MCP server on stdio (Cursor / Claude / Codex / Hermes)
    Mcp,
    /// HTTP API + Brain UI (`http://127.0.0.1:8421/ui/`) + poll/watch
    Daemon {
        /// Port for the HTTP server
        #[arg(long, default_value = "8421")]
        port: u16,
        /// Disable background incremental indexing
        #[arg(long)]
        no_poll: bool,
        /// Override config `runtime.poll_interval_secs` for the poll loop
        #[arg(long, value_name = "SECS")]
        poll_interval: Option<u64>,
        /// Disable notify filesystem watch (markdown/github roots)
        #[arg(long)]
        no_watch: bool,
    },
    /// Export this setup to a `.kurultai` pack
    Export {
        /// Output path (default: kurultai-export-YYYYMMDD-HHMMSS.kurultai)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Import a `.kurultai` pack
    Import {
        /// Path to a `.kurultai` pack
        pack: PathBuf,
        /// Overwrite an existing non-empty store.db
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Merge pack atoms into the current store (instead of replacing the DB file)
        #[arg(long, default_value_t = false)]
        combine: bool,
        /// If destination config.toml is missing, write the pack's config there
        #[arg(long, default_value_t = false)]
        write_config: bool,
    },
    /// Remove generated-file noise already in the index
    Prune {
        /// Remove atoms whose source_id contains Next.js / webpack generated-file path segments
        #[arg(long)]
        generated: bool,
    },
    /// Run diagnostic checks (DB, config, MCP, HTTP, embeddings, ontology, connectors)
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Optional help art before clap emits --help (no App / SQLite bootstrap).
    maybe_print_help_banner();

    let cli = Cli::parse();
    let env = Environment::resolve(cli.env.as_deref())?;
    // Daemon keeps verbose default; all other CLI commands stay silent unless --log is set.
    let is_daemon = matches!(cli.command, Commands::Daemon { .. });
    let default_filter = if is_daemon {
        env.default_log_filter()
    } else {
        env.cli_log_filter()
    };
    logging::init_logging(cli.log.as_deref().or(Some(default_filter)), env)?;

    let plain = effective_plain(cli.plain);
    let no_color = env_no_color_set();

    match cli.command {
        Commands::Init {
            agent,
            ref docs,
            index,
        } => {
            let config_path = ensure_default_config()?;
            let banner_mode = load_config_from(&config_path)
                .map(|c| c.banner)
                .unwrap_or(BannerMode::Auto);
            let _ = print_banner_stdout(ArtVariant::Compact, banner_mode, plain, no_color);
            let provisioned = if docs.is_some() {
                Some(provision_docs(docs.as_deref(), &config_path)?)
            } else {
                None
            };
            let mcp_paths = wire_agent(agent)?;
            print!(
                "{}",
                init_walkthrough(&config_path, provisioned.as_ref(), &mcp_paths, agent, index,)
            );
            if index {
                let app = bootstrap_app(&cli).await?;
                tracing::info!(full = true, "starting index");
                let stats = app.pipeline.index_all(&app.connectors, true).await?;
                for s in &stats {
                    println!(
                        "  {} — fetched {}, indexed {} ({}ms)",
                        s.source, s.atoms_fetched, s.atoms_indexed, s.duration_ms
                    );
                }
            }
        }
        Commands::Mcp => {
            // Never print art on MCP stdio — protocol must stay clean.
            let app = bootstrap_app(&cli).await?;
            let brain = brain_from_app(&app);
            // MCP must not spam logs to stdout — stderr only via tracing.
            tracing::info!("mcp stdio server starting");
            kurultai::mcp::run_stdio(brain).await?;
        }
        Commands::Index { full } => {
            let app = bootstrap_app(&cli).await?;
            println!(
                "Kurultai {}  (embedded UI {})",
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_VERSION")
            );
            tracing::info!(full, "starting index");
            let stats = app.pipeline.index_all(&app.connectors, full).await?;
            for s in &stats {
                println!(
                    "  {} — fetched {}, indexed {} ({}ms)",
                    s.source, s.atoms_fetched, s.atoms_indexed, s.duration_ms
                );
            }
            if stats.is_empty() {
                println!(
                    "No enabled sources configured. Add sources to ~/.config/kurultai/config.toml"
                );
            }
        }
        Commands::Ask { ref question } => {
            let app = bootstrap_app(&cli).await?;
            tracing::info!(question = %question, "ask requested");
            let brain = brain_from_app(&app);
            let answer = brain.ask(question).await?;
            println!("Q: {}", answer.question);
            println!("A: {}", answer.answer);
            println!("confidence: {:.2}", answer.confidence);
            for c in &answer.citations {
                println!("  cite: {} / {} — {}", c.source, c.source_id, c.title);
            }
        }
        Commands::WhoKnows { ref topic, limit } => {
            let app = bootstrap_app(&cli).await?;
            let brain = brain_from_app(&app);
            let entries = brain.who_knows(topic, limit).await?;
            if entries.is_empty() {
                println!("No sources matched.");
            } else {
                for e in entries {
                    println!(
                        "  {} ({} hits) — {}",
                        e.source,
                        e.hit_count,
                        e.sample_titles.join("; ")
                    );
                }
            }
        }
        Commands::Search { ref query, limit } => {
            let app = bootstrap_app(&cli).await?;
            tracing::info!(query = %query, limit, "search requested");
            let brain = brain_from_app(&app);
            let views = brain.search_views(query, limit).await?;
            if views.is_empty() {
                println!("No results.");
            } else {
                for v in views {
                    println!(
                        "  [{:.3}] {} — {}\n    {}",
                        v.score, v.source, v.title, v.excerpt
                    );
                }
            }
        }
        Commands::Promote {
            ref atom_id,
            ref reason,
        } => {
            let app = bootstrap_app(&cli).await?;
            let brain = brain_from_app(&app);
            let res = brain.promote(atom_id, "cli", reason.as_deref()).await?;
            println!("promoted {} (actor={})", res.atom_id, res.actor);
        }
        Commands::Status { metrics, port } => {
            let app = bootstrap_app(&cli).await?;
            let _ = print_banner_stdout(ArtVariant::Compact, app.config.banner, plain, no_color);
            let atom_count = app.atom_count().await?;
            let brain = brain_from_app(&app);
            let (trusted, quarantine, merge_pending) = brain.lane_counts().await?;
            println!("Kurultai status");
            println!("  Version: {}", env!("CARGO_PKG_VERSION"));
            println!("  Environment: {}", app.environment);
            println!("  Storage: {}", app.config.storage_path);
            println!("  Schema:  v{}", app.schema_version());
            if app.embedder.is_live() {
                println!(
                    "  Embedder: {} ({}-dim)",
                    app.embedder.name(),
                    app.embedder.dim()
                );
            } else {
                println!(
                    "  Embedder: none (FTS-only — set OPENROUTER_API_KEY or embed.backend=local)"
                );
            }
            if app.reranker.is_live() {
                println!("  Reranker: {}", app.reranker.name());
            } else {
                println!("  Reranker: none (set runtime.reranker_model + API key)");
            }
            if app.synthesizer.is_live() {
                println!("  Synthesizer: {}", app.synthesizer.name());
            } else {
                println!("  Synthesizer: extractive (set OPENROUTER_API_KEY for LLM ask)");
            }
            println!("  Atoms:   {}", atom_count);
            println!("  Trusted: {}", trusted);
            println!("  Quarantine: {}", quarantine);
            println!("  Merge candidates (pending): {}", merge_pending);
            println!("  Features (KURULTAI_FEATURE_<ID>=0|1):");
            for line in kurultai::features::status_lines() {
                println!("{line}");
            }

            let inbox_roots = kurultai::daemon::inbox_roots_from_sources(&app.config.sources);
            if !inbox_roots.is_empty() {
                let mut pending = 0u64;
                let mut failed = 0u64;
                for root in &inbox_roots {
                    let (p, f) = kurultai::connectors::inbox::inbox_tray_counts(root);
                    pending += p;
                    failed += f;
                }
                println!("  Inbox pending: {}", pending);
                println!("  Inbox failed: {}", failed);
            }

            if metrics {
                let url = format!("http://127.0.0.1:{port}/api/metrics");
                match reqwest::Client::new().get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        let body = resp.text().await.unwrap_or_default();
                        println!();
                        println!("Daemon metrics ({url}):");
                        println!("{body}");
                    }
                    Ok(resp) => {
                        println!();
                        println!(
                            "Daemon metrics: HTTP {} from {url} (is `kurultai daemon` running?)",
                            resp.status()
                        );
                    }
                    Err(e) => {
                        println!();
                        println!(
                            "Daemon metrics: unreachable ({e}). Start `kurultai daemon --port {port}` then retry."
                        );
                    }
                }
            }

            if app.connectors.is_empty() {
                println!("  Sources: (none enabled)");
            } else {
                println!("  Sources:");
                for name in app.connectors.names() {
                    let enabled = app
                        .config
                        .sources
                        .iter()
                        .find(|s| s.name == name)
                        .map(|s| s.enabled)
                        .unwrap_or(false);
                    println!(
                        "    - {} [{}]",
                        name,
                        if enabled { "enabled" } else { "disabled" }
                    );
                }
            }
        }
        Commands::Daemon {
            port,
            no_poll,
            poll_interval,
            no_watch,
        } => {
            let app = bootstrap_app(&cli).await?;
            let brain = brain_from_app(&app);
            let interval = kurultai::daemon::normalize_poll_interval_secs(
                poll_interval.unwrap_or(app.config.poll_interval_secs),
            );
            let watch_roots = kurultai::daemon::watch_roots_from_sources(&app.config.sources);
            let inbox_roots = kurultai::daemon::inbox_roots_from_sources(&app.config.sources);
            tracing::info!(
                port,
                poll = !no_poll,
                interval,
                watch = !no_watch,
                watch_roots = watch_roots.len(),
                "daemon starting"
            );
            println!("Daemon listening on http://127.0.0.1:{port} (localhost only)");
            let mcp_secret =
                kurultai::http::resolve_mcp_http_secret(app.config.mcp_http_secret.as_deref());
            if mcp_secret.is_some() {
                println!("MCP HTTP/SSE: POST /mcp · GET /mcp/sse (Authorization: Bearer <secret>)");
            } else {
                println!("MCP HTTP/SSE: off (set KURULTAI_MCP_HTTP_SECRET to enable)");
            }
            if kurultai::http::resolve_ingest_secret().is_some() {
                println!("Loopback ingest: POST /ingest (X-Kurultai-Ingest-Secret or Bearer)");
            } else {
                println!("Loopback ingest: off (set KURULTAI_INGEST_SECRET to enable)");
            }
            if no_poll {
                println!("Background poll: off");
            } else {
                println!("Background poll: every {interval}s (incremental)");
            }
            if no_watch {
                println!("Filesystem watch: off");
            } else if watch_roots.is_empty() {
                println!("Filesystem watch: no markdown/github/json/inbox roots to watch");
            } else {
                println!(
                    "Filesystem watch: {} root(s) (debounced incremental)",
                    watch_roots.len()
                );
            }
            kurultai::daemon::run(
                brain,
                app.pipeline,
                app.connectors,
                kurultai::daemon::DaemonOptions {
                    port,
                    poll: !no_poll,
                    poll_interval_secs: interval,
                    watch: !no_watch,
                    watch_roots,
                    inbox_roots,
                    nightly_full_sync_hour: app.config.nightly_full_sync_hour,
                    inactivity_threshold_hours: app.config.inactivity_threshold_hours,
                    mcp_http_secret: mcp_secret,
                },
            )
            .await?;
        }
        Commands::Prune { generated } => {
            if !generated {
                return Err(kurultai::KurultaiError::config(
                    "specify a filter: --generated",
                ));
            }
            let config = load_config_with_env(cli.config.as_deref(), cli.env.as_deref())?;
            let store = kurultai::store::open_store(&config).await?;
            let patterns: &[&str] = &[
                "%/chunks/%",
                "%/static/js/%",
                "%/static/css/%",
                "%/static/media/%",
                "%/__generated__/%",
                "%/_next/%",
                "%/out/_next/%",
                "%/generated/%",
            ];
            let matched = store.find_atoms_by_source_id_patterns(patterns).await?;
            let total = matched.len();
            if total == 0 {
                println!("0 atoms matched, 0 deleted");
            } else {
                println!("Found {total} atoms matching generated-file patterns. Deleting…");
                let mut deleted = 0usize;
                for atom in &matched {
                    store.delete_atom(&atom.id).await?;
                    deleted += 1;
                }
                println!("Deleted {deleted} / {total} atoms.");
            }
        }
        Commands::Doctor => {
            kurultai::doctor::run(cli.env.as_deref(), cli.config.as_deref()).await?;
        }
        Commands::Export { output } => {
            let cfg_file = resolve_config_file(cli.config.as_deref())?;
            let config = load_config_with_env(cli.config.as_deref(), cli.env.as_deref())?;
            let report = export_pack(&config, &cfg_file, output.as_deref())?;
            println!("Exported {}", report.path.display());
            println!("  Atoms: {}", report.atom_count);
            println!("  Embed dim: {}", report.embed_dim);
            println!(
                "Move this file to another device, then: kurultai import {}",
                report.path.display()
            );
            println!("On the destination: remap [sources.*.root_path], set API keys in env, run `kurultai init`.");
        }
        Commands::Import {
            pack,
            force,
            combine,
            write_config,
        } => {
            if force && combine {
                return Err(kurultai::KurultaiError::config(
                    "use either --force (replace store) or --combine (merge atoms), not both",
                ));
            }
            let cfg_file = resolve_config_file(cli.config.as_deref())?;
            let config = load_config_with_env(cli.config.as_deref(), cli.env.as_deref())?;
            let mode = if combine {
                ImportMode::Combine
            } else {
                ImportMode::Replace { force }
            };
            let report = import_pack(&config, &pack, mode, write_config, &cfg_file).await?;
            println!("Imported {} ({})", pack.display(), report.mode);
            println!("  Storage: {}", report.storage_path.display());
            println!("  Atoms upserted: {}", report.atoms_upserted);
            if report.vectors_skipped {
                println!("  Vectors: skipped (embed_dim mismatch) — FTS works; re-index or re-embed for vectors");
            } else if report.mode == "combine" {
                println!("  Vectors copied: {}", report.vectors_copied);
            }
            println!("Next: fix source root_path values if needed, then `kurultai init --agent …` and `kurultai status`.");
        }
    }

    Ok(())
}

fn brain_from_app(app: &App) -> BrainService {
    BrainService::new(
        Arc::clone(&app.store),
        Arc::clone(&app.embedder),
        Arc::clone(&app.reranker),
        Arc::clone(&app.synthesizer),
    )
}

async fn bootstrap_app(cli: &Cli) -> Result<App> {
    if let Some(ref path) = cli.config {
        App::bootstrap_from(path, cli.env.as_deref()).await
    } else {
        App::bootstrap(cli.env.as_deref()).await
    }
}

/// Best-effort help banner (KTD6): no store open; config only if cheaply readable.
fn maybe_print_help_banner() {
    let args: Vec<String> = std::env::args().collect();
    if !args.iter().any(|a| a == "-h" || a == "--help") {
        return;
    }
    // MCP stdout must stay art-free (R5/AE5), including `mcp --help`.
    if argv_has_mcp_subcommand(&args) {
        return;
    }

    let plain = effective_plain(args.iter().any(|a| a == "--plain"));
    let no_color = env_no_color_set();
    // plain / NO_COLOR win over Always — skip config read when art cannot show.
    if plain || no_color {
        return;
    }

    let mode = cheap_banner_mode(&args);
    let _ = print_banner_stdout(ArtVariant::Wide, mode, false, false);
}

/// True when the first positional CLI subcommand is `mcp`.
fn argv_has_mcp_subcommand(args: &[String]) -> bool {
    let mut skip_next = false;
    for a in args.iter().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "--config" {
            skip_next = true;
            continue;
        }
        if a.starts_with("--config=") || a.starts_with('-') {
            continue;
        }
        return a == "mcp";
    }
    false
}

fn cheap_banner_mode(args: &[String]) -> BannerMode {
    // Prefer explicit --config path; else default config path (load or Auto).
    let mut config_arg: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--config" {
            if let Some(p) = args.get(i + 1) {
                config_arg = Some(p.as_str());
            }
            break;
        }
        if let Some(rest) = args[i].strip_prefix("--config=") {
            config_arg = Some(rest);
            break;
        }
        i += 1;
    }

    let path = match config_arg {
        Some(p) => std::path::PathBuf::from(p),
        None => match config_path() {
            Ok(p) => p,
            Err(_) => return BannerMode::Auto,
        },
    };
    load_config_from(&path)
        .map(|c| c.banner)
        .unwrap_or(BannerMode::Auto)
}
