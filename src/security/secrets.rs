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

/// Detect 1Password CLI references and other common placeholder strings
/// that should not be sent to an HTTP endpoint.
fn is_placeholder_key(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("op://")
        || trimmed.starts_with("1password://")
        || trimmed.starts_with("$(op ")
        || trimmed == "YOUR_API_KEY"
        || trimmed == "TODO"
}

/// Load an API key from environment. Keys must never live in config files.
pub fn api_key_from_env(var: &str) -> Result<SecretString> {
    match std::env::var(var) {
        Ok(value) if value.trim().is_empty() => {
            Err(KurultaiError::security(format!("{var} is set but empty")))
        }
        Ok(value) if is_placeholder_key(&value) => Err(KurultaiError::security(format!(
            "{var} looks like a placeholder reference — set a real key"
        ))),
        Ok(value) => Ok(SecretString::new(value)),
        Err(_) => Err(KurultaiError::security(format!(
            "{var} not set — export it before running kurultai"
        ))),
    }
}

/// Optional API key — returns None if unset or a placeholder (for offline/dev modes).
pub fn api_key_from_env_optional(var: &str) -> Option<SecretString> {
    std::env::var(var)
        .ok()
        .filter(|v| !v.trim().is_empty() && !is_placeholder_key(v))
        .map(SecretString::new)
}
