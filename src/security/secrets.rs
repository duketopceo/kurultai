use crate::error::{KurultaiError, Result};
use std::fmt;
use std::sync::{Arc, OnceLock, RwLock};

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

/// Key file written by `kurultai init`'s optional key prompt — lives beside
/// `config.toml`, mode 0600, never committed or echoed.
pub fn key_file_path() -> Option<std::path::PathBuf> {
    crate::config::config_path()
        .ok()
        .map(|p| p.with_file_name("openrouter.key"))
}

/// Fallback key source after env: `<config-dir>/openrouter.key` (0600).
/// `KURULTAI_API_KEY_FILE` overrides the path (agents/containers).
pub fn api_key_from_keyfile() -> Option<SecretString> {
    let path = std::env::var("KURULTAI_API_KEY_FILE")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(key_file_path)?;
    let raw = std::fs::read_to_string(&path).ok()?;
    let key = raw.trim();
    (!key.is_empty()).then(|| SecretString::new(key.to_string()))
}

#[derive(Default)]
struct HeldKeys {
    env: Option<SecretString>,
    file: Option<SecretString>,
}

/// Shared, swappable OpenRouter key. The embedder, reranker and synthesizer
/// all read it per request, so a key saved through Connect takes effect
/// without a restart. Precedence is fixed: env key over key-file key.
#[derive(Clone, Default)]
pub struct KeyHolder(Arc<RwLock<HeldKeys>>);

impl KeyHolder {
    /// The one resolver: env (`OPENROUTER_API_KEY`, then `KURULTAI_API_KEY`)
    /// and the key file, read now.
    pub fn resolve() -> Self {
        let env = api_key_from_env_optional("OPENROUTER_API_KEY")
            .or_else(|| api_key_from_env_optional("KURULTAI_API_KEY"));
        Self(Arc::new(RwLock::new(HeldKeys {
            env,
            file: api_key_from_keyfile(),
        })))
    }

    /// A holder pinned to one key (tests, explicit constructors).
    pub fn fixed(key: String) -> Self {
        Self(Arc::new(RwLock::new(HeldKeys {
            env: Some(SecretString::new(key)),
            file: None,
        })))
    }

    /// Process-wide holder shared by `App` and the Connect routes.
    pub fn global() -> Self {
        static GLOBAL: OnceLock<KeyHolder> = OnceLock::new();
        GLOBAL.get_or_init(KeyHolder::resolve).clone()
    }

    /// Current key: env wins over the file key.
    pub fn get(&self) -> Option<SecretString> {
        let g = self.0.read().unwrap_or_else(|e| e.into_inner());
        g.env.clone().or_else(|| g.file.clone())
    }

    pub fn is_set(&self) -> bool {
        self.get().is_some()
    }

    /// Swap the key-file slot after Connect stores a new key.
    pub fn set_file_key(&self, key: &str) {
        let mut g = self.0.write().unwrap_or_else(|e| e.into_inner());
        g.file = Some(SecretString::new(key.to_string()));
    }
}

/// Key file for a `kurultai connect` agent credential — lives under
/// `<config-dir>/agent-keys/<name>.key`, mode 0600. `name` is sanitized to a
/// filename-safe slug (e.g. `dev-cursor-agent-token`).
pub fn agent_key_file_path(name: &str) -> Option<std::path::PathBuf> {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    crate::config::config_path()
        .ok()
        .map(|p| p.with_file_name("agent-keys").join(format!("{slug}.key")))
}

/// Persist an agent credential to `agent_key_file_path()` with 0600 perms.
/// Fallback when `omaseal` is not installed. Never logs the value.
pub fn write_agent_key_file(name: &str, key: &str) -> Result<std::path::PathBuf> {
    use std::io::Write;
    let path = agent_key_file_path(name)
        .ok_or_else(|| KurultaiError::config("could not resolve agent key file path"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| KurultaiError::config(format!("create agent key dir: {e}")))?;
    }
    let mut f = std::fs::File::create(&path)
        .map_err(|e| KurultaiError::config(format!("write agent key file: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = f.set_permissions(std::fs::Permissions::from_mode(0o600));
    }
    f.write_all(key.trim().as_bytes())
        .and_then(|_| f.write_all(b"\n"))
        .map_err(|e| KurultaiError::config(format!("write agent key file: {e}")))?;
    Ok(path)
}

/// Read an agent credential back: `omaseal get kurultai <name>` first, then
/// the 0600 `agent-keys/<name>.key` fallback written by `write_agent_key_file`.
/// Never logs the value. Returns None when no store has it.
pub fn read_agent_key(name: &str) -> Option<SecretString> {
    use std::process::{Command, Stdio};
    if Command::new("omaseal")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
    {
        let out = Command::new("omaseal")
            .args(["get", "kurultai", name])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        if let Ok(out) = out {
            if out.status.success() {
                let key = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !key.is_empty() {
                    return Some(SecretString::new(key));
                }
            }
        }
    }
    let path = agent_key_file_path(name)?;
    let raw = std::fs::read_to_string(path).ok()?;
    let key = raw.trim();
    (!key.is_empty()).then(|| SecretString::new(key.to_string()))
}

/// Persist a key to `key_file_path()` with 0600 perms. Never logs the value.
pub fn write_key_file(key: &str) -> Result<std::path::PathBuf> {
    use std::io::Write;
    let path = key_file_path()
        .ok_or_else(|| KurultaiError::config("could not resolve config dir for key file"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| KurultaiError::config(format!("create key dir: {e}")))?;
    }
    let mut f = std::fs::File::create(&path)
        .map_err(|e| KurultaiError::config(format!("write key file: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = f.set_permissions(std::fs::Permissions::from_mode(0o600));
    }
    f.write_all(key.trim().as_bytes())
        .and_then(|_| f.write_all(b"\n"))
        .map_err(|e| KurultaiError::config(format!("write key file: {e}")))?;
    Ok(path)
}
