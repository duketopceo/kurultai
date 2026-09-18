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
    ensure_default_config, ensure_default_config_at, init_walkthrough, provision_docs, wire_agent,
    AgentRead, AgentTarget, BrainService,
};
use kurultai::write_policy::{WriteContext, WriteTransport};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "kurultai",
    version,
    about = "Assemble what you know, from wherever it lives.",
    after_help = "Setup        kurultai init --docs  ·  init --agent <cursor|claude|codex|hermes|all|none>  ·  init --doctor\nAuth         kurultai connect <instance-url> [--codename <name>]  ·  login --base-url … --codename <name>\nKnowledge    index [--full]  ·  search  ·  ask  ·  who-knows  ·  status  ·  promote\nServe        webui  ·  mcp  ·  daemon [--port 8421 --bind tailscale]    Brain UI → http://127.0.0.1:8421/ui/\nPacks        export  ·  import\nMaintenance  prune --generated  ·  doctor"
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
        /// Run `doctor` diagnostics after setup
        #[arg(long)]
        doctor: bool,
        /// OpenRouter API key for vector recall + LLM ask (non-interactive)
        #[arg(long, value_name = "KEY", hide_env = true)]
        key: Option<String>,
        /// Read the OpenRouter key from a file instead of prompting
        #[arg(long, value_name = "PATH")]
        key_file: Option<PathBuf>,
        /// Never prompt for a key (agents/CI; also implied by non-TTY stdin)
        #[arg(long)]
        no_key: bool,
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
        /// Augment thin local context with a Perplexity web call (ephemeral;
        /// needs PERPLEXITY_API_KEY and KURULTAI_FEATURE_WEB_SEARCH=1)
        #[arg(long)]
        web: bool,
    },
    /// Pre-merge commit review: judge each commit in a range for secrets,
    /// security regressions, and message/diff mismatches (needs OPENROUTER_API_KEY)
    Review {
        /// Commit range, e.g. origin/main..HEAD
        range: String,
        /// Repo path (default: current directory)
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        /// Judge model override (default: pinned typesafe/jev-1.13)
        #[arg(long)]
        judge_model: Option<String>,
        /// Write the full JSON report to this path
        #[arg(long)]
        json: Option<std::path::PathBuf>,
    },
    /// Run the retrieval eval golden set against a live daemon
    Eval {
        /// Daemon base URL
        #[arg(long, default_value = "http://127.0.0.1:8421")]
        base_url: String,
        /// Path to the golden query set
        #[arg(long, default_value = "evals/golden.json")]
        golden: PathBuf,
        /// Top-k window for metrics
        #[arg(long, default_value = "10")]
        k: usize,
        /// Grade hits and answers with the Jev judge (needs OPENROUTER_API_KEY)
        #[arg(long)]
        judge: bool,
        /// Judge model override (default: pinned typesafe/jev-1.13)
        #[arg(long)]
        judge_model: Option<String>,
        /// Also write the JSON report to this path
        #[arg(long)]
        json: Option<PathBuf>,
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
    Mcp {
        /// Self-asserted session identity stamped on writes (env: KURULTAI_AGENT_ID).
        ///
        /// NOT an authorization claim: on a shared box any session can assert any
        /// value. Used for write provenance and bulk revocation only.
        #[arg(long)]
        agent_id: Option<String>,
        /// Namespace (`project_id`) stamped on writes (env: KURULTAI_NAMESPACE).
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Manage multi-agent message board codenames (solo)
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// HTTP API + Brain UI (`http://127.0.0.1:8421/ui/`) + poll/watch
    Daemon {
        /// Port for the HTTP server (`PORT` env for Railway/containers)
        #[arg(long, env = "PORT", default_value = "8421")]
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
        /// Bind address: 127.0.0.1 (default), 0.0.0.0, a literal IP, or
        /// `tailscale` (resolves the local 100.x tailnet IPv4)
        #[arg(long, value_name = "ADDR")]
        bind: Option<String>,
    },
    /// Print (or open) the Brain UI URL — spawns a daemon if none is serving
    Webui {
        /// Daemon HTTP port
        #[arg(long, default_value = "8421")]
        port: u16,
        /// Open the URL in a browser (default when stdout is a TTY)
        #[arg(long)]
        open: bool,
        /// Do not open a browser even on a TTY
        #[arg(long)]
        no_open: bool,
        /// Print the URL and exit (headless agents)
        #[arg(long)]
        print_url: bool,
        /// Extra args forwarded to the spawned daemon (e.g. --bind tailscale)
        #[arg(last = true, value_name = "DAEMON_ARGS")]
        daemon_args: Vec<String>,
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
    /// Connect this machine to a Kurultai instance — browser-approved device
    /// authorization (RFC 8628) that mints + stores an agent key and wires MCP
    Connect {
        /// Instance URL, e.g. https://knowledge.shippedit.dev or http://127.0.0.1:8421
        url: String,
        /// Codename for this agent (product family, e.g. cursor, claude, devin)
        #[arg(long, short = 'n')]
        codename: Option<String>,
        /// Seat id distinguishing this machine under the codename
        /// (default: $KURULTAI_INSTANCE_ID → hostname)
        #[arg(long)]
        instance_id: Option<String>,
        /// MCP clients to wire: cursor, claude, codex, hermes, all, or none
        #[arg(long, default_value = "all")]
        agent: AgentTarget,
        /// Print the approval URL instead of opening a browser
        #[arg(long)]
        no_open: bool,
    },
    /// Sign in to a hosted Kurultai instance and store a long-lived agent token locally
    Login {
        /// Kurultai API base URL, e.g. https://api-knowledge.shippedit.dev
        #[arg(long, short = 'u')]
        base_url: String,
        /// Codename this agent will use on the message board
        #[arg(long, short = 'n')]
        codename: String,
        /// Do not try to open the approval page in a browser
        #[arg(long)]
        no_browser: bool,
        /// Optional local account name for the keyring (default: <codename>-agent-token)
        #[arg(long, short = 'a')]
        account: Option<String>,
    },
    /// Mint/revoke/list scoped access tokens for HTTP/MCP API consumers
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
    /// Hub admin: issued device keys and write log (requires DATABASE_URL + postgres feature)
    #[cfg(feature = "postgres")]
    Hub {
        #[command(subcommand)]
        command: HubCommands,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Register a new codename and print the one-time API key.
    Add {
        /// Unique agent codename (e.g. "claude", "cursor", "devin")
        codename: String,
    },
    /// List registered codenames. Never shows keys.
    List,
    /// Revoke an agent's credentials — all seats and the primary key, or one
    /// seat with `--instance-id`.
    Revoke {
        /// Agent codename to revoke
        codename: String,
        /// Revoke only this seat (other seats keep working)
        #[arg(long)]
        instance_id: Option<String>,
    },
}

#[derive(Subcommand)]
enum AdminCommands {
    /// Manage scoped access tokens
    Key {
        #[command(subcommand)]
        command: AdminKeyCommands,
    },
}

#[derive(Subcommand)]
enum AdminKeyCommands {
    /// Mint a new scoped token. Prints the raw token ONCE — it is hashed at rest and cannot be
    /// recovered afterward.
    Issue {
        /// Human label for this key (must be unique)
        #[arg(long)]
        name: String,
        /// Comma-separated mesh ids this key is granted access to
        #[arg(long, default_value = "")]
        mesh: String,
        /// Tier ceiling for this key: public or private
        #[arg(long, value_name = "public|private")]
        max_tier: String,
        /// Comma-separated allowed tool names, e.g. search,cite,ask,who_knows,remember,promote,ontology_promote
        #[arg(long, default_value = "")]
        tools: String,
        /// Path to a text file of free-text policy shown to the agent at connect time.
        /// NOT enforced in code — advisory only.
        #[arg(long, value_name = "PATH")]
        rules_doc: Option<PathBuf>,
    },
    /// Revoke a key by name (soft delete — the row is kept, inactive, for audit)
    Revoke {
        /// Name of the key to revoke
        #[arg(long)]
        name: String,
    },
    /// List all keys (active and revoked). Never prints tokens or hashes.
    List,
}

#[cfg(feature = "postgres")]
#[derive(Subcommand)]
enum HubCommands {
    /// Issue/revoke/list hub device API keys (Postgres)
    Key {
        #[command(subcommand)]
        command: HubKeyCommands,
    },
    /// Read append-only hub write activity log
    Log {
        #[arg(long, default_value = "50")]
        limit: usize,
    },
}

#[cfg(feature = "postgres")]
#[derive(Subcommand)]
enum HubKeyCommands {
    /// Issue a device key. Plaintext shown once; sha256 stored at rest.
    Issue {
        #[arg(long)]
        agent: String,
        #[arg(long)]
        team: String,
    },
    /// Revoke by key prefix (first 12 chars shown at issue)
    Revoke {
        #[arg(long, conflicts_with = "id")]
        prefix: Option<String>,
        #[arg(long, conflicts_with = "prefix")]
        id: Option<i64>,
    },
    /// List issued keys (prefix + team; never full secret)
    List,
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
            doctor,
            ref key,
            ref key_file,
            no_key,
        } => {
            let config_path = match cli.config.as_deref() {
                Some(path) => ensure_default_config_at(path.to_path_buf())?,
                None => ensure_default_config()?,
            };
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
            if let Some(msg) = init_key_setup(key.as_deref(), key_file.as_deref(), no_key)? {
                println!("{msg}");
            }
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
            if doctor {
                kurultai::doctor::run(cli.env.as_deref(), Some(config_path.as_ref())).await?;
            }
        }
        Commands::Mcp {
            ref agent_id,
            ref namespace,
        } => {
            // Never print art on MCP stdio — protocol must stay clean.
            let app = bootstrap_app(&cli).await?;
            let brain = brain_from_app(&app);
            let ctx = WriteContext::resolve(
                WriteTransport::Mcp,
                agent_id.as_deref(),
                namespace.as_deref(),
            );
            // MCP must not spam logs to stdout — stderr only via tracing.
            tracing::info!(
                agent_id = ?ctx.agent_id,
                namespace = ?ctx.namespace,
                mode = ?ctx.mode,
                "mcp stdio server starting"
            );
            kurultai::mcp::run_stdio_with(brain, ctx).await?;
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
        Commands::Ask { ref question, web } => {
            let app = bootstrap_app(&cli).await?;
            tracing::info!(question = %question, web, "ask requested");
            let brain = brain_from_app(&app);
            let answer = if web {
                brain.ask_with_web(question, None, 2).await?
            } else {
                brain.ask(question).await?
            };
            println!("Q: {}", answer.question);
            println!("A: {}", answer.answer);
            println!("confidence: {:.2}", answer.confidence);
            for c in &answer.citations {
                println!("  cite: {} / {} — {}", c.source, c.source_id, c.title);
            }
        }
        Commands::Review {
            ref range,
            ref repo,
            ref judge_model,
            ref json,
        } => {
            let judge = kurultai::eval::judge::judge_from_env(judge_model.clone());
            if !judge.is_live() {
                println!("review: no OpenRouter key — skipping (set OPENROUTER_API_KEY)");
                return Ok(());
            }
            let repo = repo
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let commits = kurultai::eval::review::collect_commits(&repo, range)?;
            if commits.is_empty() {
                println!("review: no non-bot commits in {range}");
                return Ok(());
            }
            let report = kurultai::eval::review::review_commits(range, &commits, &judge).await?;
            println!(
                "review: {} commits in {} — {} flagged (judge: {} {})",
                report.reviewed,
                report.range,
                report.failed,
                report.judge,
                report.judge_model.as_deref().unwrap_or("")
            );
            for c in &report.commits {
                let flags: Vec<String> = c
                    .flags
                    .iter()
                    .map(|f| format!("{}={:.2}[{}]", f.check, f.probability, f.severity))
                    .collect();
                let mark = if c.flags.iter().any(|f| f.severity == "hard") {
                    "FAIL"
                } else if !c.flags.is_empty() {
                    "warn"
                } else {
                    " ok "
                };
                println!(
                    "  {} {} {:.60} {}",
                    mark,
                    &c.sha[..c.sha.len().min(8)],
                    c.subject,
                    flags.join(" ")
                );
            }
            if let Some(cost) = report.judge_cost_usd {
                println!("  judge cost ${:.5}", cost);
            }
            if let Some(path) = json {
                let pretty = serde_json::to_string_pretty(&report)
                    .map_err(|e| anyhow::anyhow!("serialize review report: {e}"))?;
                std::fs::write(path, pretty)?;
                println!("  report → {}", path.display());
            }
            if report.failed > 0 {
                return Err(
                    anyhow::anyhow!("{} commit(s) flagged by review", report.failed).into(),
                );
            }
        }
        Commands::Eval {
            ref base_url,
            ref golden,
            k,
            judge,
            ref judge_model,
            ref json,
        } => {
            let set = kurultai::eval::GoldenSet::load(golden)?;
            let cfg = kurultai::eval::EvalConfig {
                base_url: base_url.clone(),
                k,
                judge,
                judge_model: judge_model.clone(),
                judge_override: None,
            };
            let report = kurultai::eval::run_eval(&cfg, &set).await?;
            let a = &report.aggregate;
            println!(
                "eval: {} queries against {} (k={})",
                a.queries, report.base_url, report.k
            );
            println!(
                "  recall@{} {:.3}  precision@{} {:.3}  mrr {:.3}",
                report.k, a.mean_recall_at_k, report.k, a.mean_precision_at_k, a.mean_mrr
            );
            if let Some(n) = a.mean_ndcg_at_k {
                println!(
                    "  ndcg@{} {:.3}  (judge: {} {})",
                    report.k,
                    n,
                    report.judge,
                    report.judge_model.as_deref().unwrap_or("")
                );
            }
            if let Some(g) = a.mean_groundedness {
                println!("  groundedness {:.3}", g);
            }
            if let Some(reason) = &report.judge_disabled_reason {
                println!("  judge disabled mid-run: {}", reason);
            }
            if a.noise_violations > 0 {
                println!("  noise violations: {}", a.noise_violations);
            }
            if let Some(path) = json {
                let pretty = serde_json::to_string_pretty(&report)
                    .map_err(|e| anyhow::anyhow!("serialize eval report: {e}"))?;
                std::fs::write(path, pretty)?;
                println!("  report → {}", path.display());
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
            let actor = WriteContext::resolve(WriteTransport::Cli, None, None).actor();
            let res = brain.promote(atom_id, &actor, reason.as_deref()).await?;
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
            {
                // Best-effort daemon bind surface (#: `webui`/`daemon --bind`).
                let hub = kurultai::http::resolve_hub_gate_from_env();
                let bind_all = kurultai::http::resolve_bind_all_from_env();
                if let Ok(addr) =
                    kurultai::http::resolve_listen_socket_flag(port, bind_all, None, &hub)
                {
                    println!("  Daemon:  http://{addr}/ui/ (`kurultai webui`)");
                }
            }
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
        Commands::Agent { command } => {
            let config = load_config_with_env(cli.config.as_deref(), cli.env.as_deref())?;
            let store = kurultai::store::open_store(&config).await?;
            match command {
                AgentCommands::Add { codename } => {
                    let (id, key) = store.register_agent(&codename).await?;
                    println!("Agent '{}' registered (id={}).", codename, id);
                    println!();
                    println!("  {key}");
                    println!();
                    println!("STORE THIS NOW — it is hashed at rest and cannot be shown again.");
                }
                AgentCommands::List => {
                    let agents = store.list_agents().await?;
                    if agents.is_empty() {
                        println!("No agents registered.");
                    } else {
                        for a in agents {
                            println!("  {} [{}] {}", a.codename, a.id, a.created_at);
                        }
                    }
                }
                AgentCommands::Revoke {
                    codename,
                    instance_id,
                } => {
                    let n = store
                        .revoke_agent(&codename, instance_id.as_deref())
                        .await?;
                    match (n, &instance_id) {
                        (0, _) => println!("No credentials found for '{codename}'."),
                        (_, Some(seat)) => {
                            println!("Revoked seat '{seat}' for '{codename}' ({n} credential).")
                        }
                        (_, None) => println!(
                            "Revoked '{codename}' — primary key + all seats ({n} credential(s))."
                        ),
                    }
                }
            }
        }
        Commands::Daemon {
            port,
            no_poll,
            poll_interval,
            no_watch,
            ref bind,
        } => {
            let hub = kurultai::http::resolve_hub_gate_from_env();
            let bind_all = kurultai::http::resolve_bind_all_from_env();
            let addr =
                kurultai::http::resolve_listen_socket_flag(port, bind_all, bind.as_deref(), &hub)?;
            let app = bootstrap_app(&cli).await?;
            let brain = brain_from_app(&app);
            let interval = kurultai::daemon::normalize_poll_interval_secs(
                poll_interval.unwrap_or(app.config.poll_interval_secs),
            );
            let watch_roots = kurultai::daemon::watch_roots_from_sources(&app.config.sources);
            let inbox_roots = kurultai::daemon::inbox_roots_from_sources(&app.config.sources);
            tracing::info!(
                %addr,
                poll = !no_poll,
                interval,
                watch = !no_watch,
                watch_roots = watch_roots.len(),
                hub = kurultai::features::enabled("hub"),
                "daemon starting"
            );
            println!("Daemon listening on http://{addr}");
            if !addr.ip().is_loopback() {
                eprintln!(
                    "warning: bound non-loopback {addr} — HTTP is unauthenticated unless \
                     KURULTAI_MCP_HTTP_SECRET / KURULTAI_INGEST_SECRET are set \
                     (see docs/deploy/railway-hub.md)"
                );
            }
            if kurultai::features::enabled("hub") {
                println!("Store: hub Postgres (KURULTAI_FEATURE_HUB=1)");
            } else {
                println!("Store: solo SQLite");
            }
            match hub.auth {
                kurultai::http::HubAuth::ApiKey => {
                    println!("Hub auth: api_key (Bearer required on /api/*; /health open)");
                }
                kurultai::http::HubAuth::None => {
                    println!("Hub auth: none");
                }
            }
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
                    bind: bind.clone(),
                },
            )
            .await?;
        }
        Commands::Webui {
            port,
            open,
            no_open,
            print_url,
            ref daemon_args,
        } => {
            kurultai::webui::run(kurultai::webui::WebuiOptions {
                port,
                open: open || (!no_open && std::io::stdout().is_terminal()),
                print_url,
                daemon_args: daemon_args.clone(),
            })
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
        Commands::Connect {
            ref url,
            ref codename,
            ref instance_id,
            agent,
            no_open,
        } => {
            kurultai::connect::run(kurultai::connect::ConnectOptions {
                url: url.clone(),
                codename: codename.clone(),
                instance_id: instance_id.clone(),
                agent,
                no_open,
                lane: env.as_str().to_string(),
            })
            .await?;
        }
        Commands::Login {
            base_url,
            codename,
            no_browser,
            account,
        } => {
            kurultai::login::run(kurultai::login::LoginOptions {
                base_url,
                codename,
                no_browser,
                account,
            })
            .await?;
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
        Commands::Admin {
            command: AdminCommands::Key { command },
        } => {
            let config = load_config_with_env(cli.config.as_deref(), cli.env.as_deref())?;
            let store_path = kurultai::config::expand_path(&config.storage_path)?;
            let admin_store = kurultai::security::AdminKeyStore::open(
                &kurultai::security::default_admin_keys_path(&store_path),
            )?;
            match command {
                AdminKeyCommands::Issue {
                    name,
                    mesh,
                    max_tier,
                    tools,
                    rules_doc,
                } => {
                    let max_tier = kurultai::security::MaxTier::parse(&max_tier)?;
                    let mesh = split_csv(&mesh);
                    let tools = split_csv(&tools);
                    let rules_doc = match rules_doc {
                        Some(path) => Some(std::fs::read_to_string(&path).map_err(|e| {
                            kurultai::KurultaiError::config(format!(
                                "reading --rules-doc {}: {e}",
                                path.display()
                            ))
                        })?),
                        None => None,
                    };
                    let claims = kurultai::security::KeyClaims {
                        name: name.clone(),
                        mesh,
                        max_tier,
                        tools,
                        rules_doc,
                    };
                    let token = admin_store.issue(&claims)?;
                    println!("Key '{name}' issued.");
                    println!();
                    println!("  {token}");
                    println!();
                    println!(
                        "STORE THIS NOW — it is hashed at rest and cannot be shown again. \
                         Anyone holding it can authenticate as this key until it is revoked."
                    );
                }
                AdminKeyCommands::Revoke { name } => {
                    if admin_store.revoke_by_name(&name)? {
                        println!("Key '{name}' revoked (row kept, inactive, for audit).");
                    } else {
                        println!("No active key named '{name}' found (already revoked, or never existed).");
                    }
                }
                AdminKeyCommands::List => {
                    let records = admin_store.list()?;
                    if records.is_empty() {
                        println!("No keys issued.");
                    } else {
                        for r in records {
                            let status = if r.active { "active" } else { "revoked" };
                            println!(
                                "  {} [{}] tier={} mesh=[{}] tools=[{}]",
                                r.name,
                                status,
                                r.max_tier,
                                r.mesh.join(","),
                                r.tools.join(",")
                            );
                        }
                    }
                }
            }
        }
        #[cfg(feature = "postgres")]
        Commands::Hub { command } => {
            let url = kurultai::store::database_url_from_env().ok_or_else(|| {
                kurultai::KurultaiError::config(
                    "kurultai hub commands require DATABASE_URL or KURULTAI_DATABASE_URL",
                )
            })?;
            match command {
                HubCommands::Key { command } => {
                    let key_store = kurultai::hub::HubKeyStore::connect(&url).await?;
                    match command {
                        HubKeyCommands::Issue { agent, team } => {
                            let (token, id) = key_store.issue(&agent, &team).await?;
                            println!("Hub key issued (id={id}, agent={agent}, team={team}).");
                            println!();
                            println!("  {token}");
                            println!();
                            println!(
                                "STORE THIS NOW — sha256 at rest only. Revoke with: kurultai hub key revoke --prefix {}",
                                token.chars().take(12).collect::<String>()
                            );
                        }
                        HubKeyCommands::Revoke { prefix, id } => match (prefix, id) {
                            (Some(p), None) => {
                                if key_store.revoke_by_prefix(&p).await? {
                                    println!("Revoked key with prefix '{p}'.");
                                } else {
                                    println!("No active key with prefix '{p}'.");
                                }
                            }
                            (None, Some(id)) => {
                                if key_store.revoke_by_id(id).await? {
                                    println!("Revoked key id {id}.");
                                } else {
                                    println!("No active key with id {id}.");
                                }
                            }
                            _ => {
                                return Err(kurultai::KurultaiError::config(
                                    "specify exactly one of --prefix or --id",
                                ));
                            }
                        },
                        HubKeyCommands::List => {
                            let records = key_store.list().await?;
                            if records.is_empty() {
                                println!("No hub keys issued.");
                            } else {
                                for r in records {
                                    let status = if r.revoked_at.is_none() {
                                        "active"
                                    } else {
                                        "revoked"
                                    };
                                    println!(
                                        "  id={} prefix={} agent={} team={} [{status}]",
                                        r.id, r.key_prefix, r.agent_id, r.team_id
                                    );
                                }
                            }
                        }
                    }
                }
                HubCommands::Log { limit } => {
                    let key_store = kurultai::hub::HubKeyStore::connect(&url).await?;
                    let activity = kurultai::hub::HubActivityStore::new(key_store.pool().clone());
                    let entries = activity.list(limit.clamp(1, 500)).await?;
                    if entries.is_empty() {
                        println!("No hub write activity recorded.");
                    } else {
                        for e in entries {
                            println!(
                                "  {} {} team={} ns={} transport={} atom={:?} reason={:?}",
                                e.at,
                                e.agent_id,
                                e.team_id,
                                e.namespace,
                                e.transport,
                                e.atom_id,
                                e.reason
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// `init` optional OpenRouter key step: `--key`/`--key-file` store directly;
/// interactive TTYs get a skippable prompt; non-TTY behaves as `--no-key`.
/// Returns a one-line outcome message (never echoes the key).
fn init_key_setup(
    key: Option<&str>,
    key_file: Option<&std::path::Path>,
    no_key: bool,
) -> Result<Option<String>> {
    use kurultai::security::write_key_file;

    let stored = |k: &str| -> Result<Option<String>> {
        let path = write_key_file(k)?;
        Ok(Some(format!(
            "Key saved to {} (0600). Unlocked: vector recall, rerank, LLM `ask`. \
             Works keyless: FTS search, who-knows, extractive ask.",
            path.display()
        )))
    };

    if let Some(k) = key {
        return stored(k);
    }
    if let Some(path) = key_file {
        let k = std::fs::read_to_string(path)
            .map_err(|e| kurultai::KurultaiError::config(format!("--key-file {path:?}: {e}")))?;
        return stored(&k);
    }
    if no_key || !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Ok(None);
    }
    eprint!("Add an OpenRouter key for vector recall + LLM ask? [paste / skip] ");
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return Ok(None);
    }
    let line = line.trim();
    if line.is_empty() || line.eq_ignore_ascii_case("skip") || line.eq_ignore_ascii_case("n") {
        println!("Skipped — FTS search, who-knows, and extractive ask work keyless.");
        return Ok(None);
    }
    stored(line)
}

/// Split a comma-separated CLI value into trimmed, non-empty parts.
fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn brain_from_app(app: &App) -> BrainService {
    BrainService::new(
        Arc::clone(&app.store),
        Arc::clone(&app.embedder),
        Arc::clone(&app.reranker),
        Arc::clone(&app.synthesizer),
    )
    .with_tier_policy(app.config.tier_policy.clone())
    .with_web_searcher(if kurultai::features::enabled("web_search") {
        kurultai::web::web_searcher_from_env()
    } else {
        std::sync::Arc::new(kurultai::web::NullWebSearcher)
    })
    .with_judge(kurultai::eval::judge::judge_from_env(None))
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
