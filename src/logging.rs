use crate::environment::Environment;
use crate::error::{KurultaiError, Result};
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize tracing for CLI/daemon use.
///
/// Priority: explicit `verbosity` arg → `KURULTAI_LOG` env → environment default.
pub fn init_logging(verbosity: Option<&str>, env: Environment) -> Result<()> {
    let filter = verbosity
        .map(str::to_string)
        .or_else(|| std::env::var("KURULTAI_LOG").ok())
        .unwrap_or_else(|| env.default_log_filter().to_string());

    let env_filter = EnvFilter::try_new(&filter)
        .map_err(|e| KurultaiError::config(format!("invalid log filter '{filter}': {e}")))?;

    // Always log to stderr — stdout is reserved for CLI results and MCP JSON-RPC.
    fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .try_init()
        .map_err(|e| KurultaiError::config(format!("failed to init logging: {e}")))?;

    tracing::debug!(filter = %filter, env = %env, "logging initialized");
    Ok(())
}
