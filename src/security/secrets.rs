use crate::error::{KurultaiError, Result};
use std::fmt;

/// Wrapper around API keys and tokens. Never implement `Debug` that leaks values.
#[derive(Clone)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(***)")
    }
}

/// Load an API key from environment. Keys must never live in config files.
pub fn api_key_from_env(var: &str) -> Result<SecretString> {
    match std::env::var(var) {
        Ok(value) if !value.trim().is_empty() => Ok(SecretString::new(value)),
        Ok(_) => Err(KurultaiError::security(format!("{var} is set but empty"))),
        Err(_) => Err(KurultaiError::security(format!(
            "{var} not set — export it before running kurultai"
        ))),
    }
}

/// Optional API key — returns None if unset (for offline/dev modes).
pub fn api_key_from_env_optional(var: &str) -> Option<SecretString> {
    std::env::var(var)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(SecretString::new)
}
