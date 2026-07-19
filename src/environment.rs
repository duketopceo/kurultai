use crate::error::{KurultaiError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Deployment environment. Drives defaults for storage paths, logging, and safety gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Local developer machine. Verbose logs, isolated DB, relaxed embed requirements.
    #[default]
    Dev,
    /// Team pre-production. Shared instance, scrubbed data, real embeddings required.
    Staging,
    /// Company production. Strict redaction, audit, auth required.
    Prod,
}

impl Environment {
    /// Resolve from `KURULTAI_ENV` or config file, defaulting to dev.
    pub fn resolve(explicit: Option<&str>) -> Result<Self> {
        let raw = explicit
            .map(str::to_string)
            .or_else(|| std::env::var("KURULTAI_ENV").ok());

        match raw {
            Some(value) => value.parse(),
            None => Ok(Self::Dev),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Prod => "prod",
        }
    }

    /// Default storage path relative to home.
    pub fn storage_relative(self) -> &'static str {
        match self {
            Self::Dev => ".local/share/kurultai/dev/store.db",
            Self::Staging => ".local/share/kurultai/staging/store.db",
            Self::Prod => ".local/share/kurultai/store.db",
        }
    }

    /// Default log filter when `KURULTAI_LOG` is unset.
    pub fn default_log_filter(self) -> &'static str {
        match self {
            Self::Dev => "kurultai=debug,info",
            Self::Staging => "kurultai=info,warn",
            Self::Prod => "kurultai=warn,error",
        }
    }

    /// Whether missing API keys should warn (dev) or error (staging/prod).
    pub fn requires_embed_api_key(self) -> bool {
        matches!(self, Self::Staging | Self::Prod)
    }
}

impl FromStr for Environment {
    type Err = KurultaiError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "dev" | "development" | "local" => Ok(Self::Dev),
            "staging" | "stage" | "stg" => Ok(Self::Staging),
            "prod" | "production" => Ok(Self::Prod),
            other => Err(KurultaiError::config(format!(
                "invalid environment '{other}' — expected dev, staging, or prod"
            ))),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_environment_aliases() {
        assert_eq!(
            "development".parse::<Environment>().unwrap(),
            Environment::Dev
        );
        assert_eq!(
            "staging".parse::<Environment>().unwrap(),
            Environment::Staging
        );
        assert_eq!(
            "production".parse::<Environment>().unwrap(),
            Environment::Prod
        );
    }

    #[test]
    fn storage_paths_are_isolated() {
        assert!(Environment::Dev.storage_relative().contains("/dev/"));
        assert!(Environment::Staging
            .storage_relative()
            .contains("/staging/"));
        assert!(!Environment::Prod.storage_relative().contains("/dev/"));
    }
}
