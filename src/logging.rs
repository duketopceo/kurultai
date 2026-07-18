use crate::error::{KurultaiError, Result};
use tracing_subscriber::{fmt, EnvFilter};

/// Default log filter: verbose for kurultai internals, info for everything else.
pub const DEFAULT_LOG_FILTER: &str = "kurultai=debug,info";

/// Initialize tracing for CLI/daemon use.
///
/// Priority: explicit `verbosity` arg → `KURULTAI_LOG` env → default.
/// In development, enables file/line numbers for fast debugging.
pub fn init_logging(verbosity: Option<&str>) -> Result<()> {
    let filter = verbosity
        .map(str::to_string)
        .or_else(|| std::env::var("KURULTAI_LOG").ok())
        .unwrap_or_else(|| DEFAULT_LOG_FILTER.to_string());

    let env_filter =
        EnvFilter::try_new(&filter).map_err(|e| KurultaiError::config(format!("invalid log filter '{filter}': {e}")))?;

    fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .try_init()
        .map_err(|e| KurultaiError::config(format!("failed to init logging: {e}")))?;

    tracing::debug!(filter = %filter, "logging initialized");
    Ok(())
}
