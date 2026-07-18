use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kurultai", version, about = "Unified knowledge retrieval layer. Assemble what you know.")]
struct Cli {
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
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { full } => {
            println!("Indexing sources... (full={})", full);
        }
        Commands::Ask { question } => {
            println!("Q: {}", question);
            println!("A: Not implemented yet.");
        }
        Commands::Search { query, limit } => {
            println!("Searching for: {} (limit: {})", query, limit);
        }
        Commands::Status => {
            println!("Kurultai status:");
            println!("  Sources: appflowy, obsidian, pond, tech_tracker, github");
            println!("  Store: sqlite-vec (not initialized)");
            println!("  Atoms: 0");
        }
        Commands::Daemon { port } => {
            println!("Starting Kurultai daemon on port {}...", port);
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        }
    }

    Ok(())
}
