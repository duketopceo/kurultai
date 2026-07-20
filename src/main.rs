use clap::{Parser, Subcommand, ValueEnum};
use kurultai::app::{self, AppOpts, Command, InstallClient};
use kurultai::config;
#[derive(Parser)]
#[command(
    name = "kurultai",
    version,
    about = "Unified knowledge retrieval layer. Assemble what you know."
)]
struct Cli {
    /// Environment for config/store roots (dev|staging|prod)
    #[arg(long, env = "KURULTAI_ENV", global = true)]
    env: Option<String>,

    /// Override config.toml path
    #[arg(long, global = true)]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write default config for this environment
    Init {
        /// Optional vault/notes path to put in the filesystem source
        #[arg(long)]
        vault: Option<std::path::PathBuf>,
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Index all configured sources
    Index {
        /// Full re-index (also removes orphans)
        #[arg(long)]
        full: bool,
    },
    /// Ask a question (search + light answer)
    Ask { question: String },
    /// Search the knowledge base
    Search {
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// List configured sources and status
    Status,
    /// Health checks for config/store/keys
    Doctor,
    /// Write MCP server entry for an agent client
    Install {
        #[arg(long, value_enum, default_value = "cursor")]
        client: InstallClientCli,
    },
    /// Run MCP server on stdio
    Mcp,
    /// HTTP daemon (not in Phase 1)
    Daemon {
        #[arg(long, default_value = "8421")]
        port: u16,
    },
}

#[derive(Clone, ValueEnum)]
enum InstallClientCli {
    Cursor,
    Claude,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let env = config::resolve_env(cli.env.as_deref())?;
    let opts = AppOpts {
        env,
        config_override: cli.config,
    };

    let cmd = match cli.command {
        Commands::Init { vault, force } => Command::Init { vault, force },
        Commands::Index { full } => Command::Index { full },
        Commands::Ask { question } => Command::Ask { question },
        Commands::Search { query, limit } => Command::Search { query, limit },
        Commands::Status => Command::Status,
        Commands::Doctor => Command::Doctor,
        Commands::Install { client } => Command::Install {
            client: match client {
                InstallClientCli::Cursor => InstallClient::Cursor,
                InstallClientCli::Claude => InstallClient::Claude,
            },
        },
        Commands::Mcp => Command::Mcp,
        Commands::Daemon { port } => Command::Daemon { port },
    };

    app::run(opts, cmd).await
}
