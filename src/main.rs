use clap::{Parser, Subcommand};
use kurultai::app::App;
use kurultai::environment::Environment;
use kurultai::error::Result;
use kurultai::logging;
use kurultai::mcp::{ensure_default_config, wire_agent, AgentRead, AgentTarget, BrainService};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "kurultai",
    version,
    about = "Unified knowledge retrieval layer. Assemble what you know."
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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write default config and wire MCP into an agent
    Init {
        /// Agent to wire: cursor
        #[arg(long, default_value = "cursor")]
        agent: AgentTarget,
    },
    /// Index all configured sources
    Index {
        /// Full re-index instead of incremental
        #[arg(long)]
        full: bool,
    },
    /// Ask a question
    Ask {
        /// The question to answer
        question: String,
    },
    /// Search the knowledge base
    Search {
        /// Search query
        query: String,
        /// Number of results
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// List configured sources and status
    Status,
    /// Run MCP server on stdio (for Cursor / Claude)
    Mcp,
    /// Start the daemon (polls sources, serves queries)
    Daemon {
        /// Port for the HTTP server
        #[arg(long, default_value = "8421")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let env = Environment::resolve(cli.env.as_deref())?;
    logging::init_logging(cli.log.as_deref(), env)?;

    match cli.command {
        Commands::Init { agent } => {
            let config_path = ensure_default_config()?;
            let mcp_path = wire_agent(agent)?;
            println!("Config: {}", config_path.display());
            println!("MCP wired: {}", mcp_path.display());
            println!("Restart Cursor to load the kurultai MCP server.");
        }
        Commands::Mcp => {
            let app = bootstrap_app(&cli).await?;
            let brain = BrainService::new(
                Arc::clone(&app.store),
                Arc::clone(&app.embedder),
                Arc::clone(&app.reranker),
            );
            // MCP must not spam logs to stdout — stderr only via tracing.
            tracing::info!("mcp stdio server starting");
            kurultai::mcp::run_stdio(brain).await?;
        }
        Commands::Index { full } => {
            let app = bootstrap_app(&cli).await?;
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
            let brain = BrainService::new(
                Arc::clone(&app.store),
                Arc::clone(&app.embedder),
                Arc::clone(&app.reranker),
            );
            let answer = brain.ask(question).await?;
            println!("Q: {}", answer.question);
            println!("A: {}", answer.answer);
            for c in &answer.citations {
                println!("  cite: {} / {} — {}", c.source, c.source_id, c.title);
            }
        }
        Commands::Search { ref query, limit } => {
            let app = bootstrap_app(&cli).await?;
            tracing::info!(query = %query, limit, "search requested");
            let brain = BrainService::new(
                Arc::clone(&app.store),
                Arc::clone(&app.embedder),
                Arc::clone(&app.reranker),
            );
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
        Commands::Status => {
            let app = bootstrap_app(&cli).await?;
            let atom_count = app.atom_count().await?;
            println!("Kurultai status");
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
                println!("  Embedder: none (FTS-only — set OPENROUTER_API_KEY for vectors)");
            }
            if app.reranker.is_live() {
                println!("  Reranker: {}", app.reranker.name());
            } else {
                println!("  Reranker: none (set runtime.reranker_model + API key)");
            }
            println!("  Atoms:   {}", atom_count);

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
        Commands::Daemon { port } => {
            let app = bootstrap_app(&cli).await?;
            tracing::info!(port, "daemon starting (stub)");
            println!(
                "Daemon on port {} — HTTP not implemented yet. Use `kurultai mcp` for agents (#11).",
                port
            );
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    app.config.poll_interval_secs,
                ))
                .await;
                tracing::debug!("daemon poll tick");
            }
        }
    }

    Ok(())
}

async fn bootstrap_app(cli: &Cli) -> Result<App> {
    if let Some(ref path) = cli.config {
        App::bootstrap_from(path, cli.env.as_deref()).await
    } else {
        App::bootstrap(cli.env.as_deref()).await
    }
}
