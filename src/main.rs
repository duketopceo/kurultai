use clap::{Parser, Subcommand};
use kurultai::app::App;
use kurultai::error::Result;
use kurultai::logging;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kurultai", version, about = "Unified knowledge retrieval layer. Assemble what you know.")]
struct Cli {
    /// Log filter (overrides KURULTAI_LOG). Example: kurultai=trace,info
    #[arg(long, global = true)]
    log: Option<String>,

    /// Path to config file (overrides KURULTAI_CONFIG)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    logging::init_logging(cli.log.as_deref())?;

    let app = if let Some(ref path) = cli.config {
        App::bootstrap_from(path).await?
    } else {
        App::bootstrap().await?
    };

    match cli.command {
        Commands::Index { full } => {
            tracing::info!(full, "starting index");
            let stats = app.pipeline.index_all(&app.connectors, full).await?;
            for s in &stats {
                println!(
                    "  {} — fetched {}, indexed {} ({}ms)",
                    s.source, s.atoms_fetched, s.atoms_indexed, s.duration_ms
                );
            }
            if stats.is_empty() {
                println!("No enabled sources configured. Add sources to ~/.config/kurultai/config.toml");
            }
        }
        Commands::Ask { question } => {
            tracing::info!(question = %question, "ask requested");
            println!("Q: {}", question);
            println!("A: Not implemented yet. See issue #7.");
        }
        Commands::Search { query, limit } => {
            tracing::info!(query = %query, limit, "search requested");
            let results = app
                .store
                .fts_search(&query, limit)
                .await
                .map_err(|e| kurultai::KurultaiError::Store(e.to_string()))?;
            if results.is_empty() {
                println!("No results (FTS not implemented yet — see issue #6).");
            } else {
                for (atom, score) in results {
                    println!("  [{:.3}] {} — {}", score, atom.source, atom.title);
                }
            }
        }
        Commands::Status => {
            let atom_count = app.atom_count().await?;
            println!("Kurultai status");
            println!("  Storage: {}", app.config.storage_path);
            println!("  Schema:  v{}", app.schema_version());
            println!("  Embedder: {} ({}-dim)", app.embedder.name(), app.embedder.dim());
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
                    println!("    - {} [{}]", name, if enabled { "enabled" } else { "disabled" });
                }
            }
        }
        Commands::Daemon { port } => {
            tracing::info!(port, "daemon starting (stub)");
            println!("Daemon on port {} — HTTP/MCP not implemented yet. See issues #7, #11.", port);
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(app.config.poll_interval_secs)).await;
                tracing::debug!("daemon poll tick");
            }
        }
    }

    Ok(())
}
