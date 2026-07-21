//! Wire Kurultai into agent MCP configs (`kurultai init --agent cursor`).

use crate::config::default_config_toml;
use crate::error::{KurultaiError, Result};
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

/// Supported agent targets for MCP auto-wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTarget {
    Cursor,
}

impl AgentTarget {
    pub fn parse(s: &str) -> Result<Self> {
        s.parse().map_err(|e: String| KurultaiError::config(e))
    }
}

impl FromStr for AgentTarget {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cursor" => Ok(Self::Cursor),
            other => Err(format!(
                "unsupported agent '{other}' — Phase 1 supports: cursor"
            )),
        }
    }
}

/// Write/merge MCP server entry for the given agent.
pub fn wire_agent(agent: AgentTarget) -> Result<PathBuf> {
    match agent {
        AgentTarget::Cursor => wire_cursor(),
    }
}

fn wire_cursor() -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| KurultaiError::config("cannot resolve home directory"))?;
    let path = home.join(".cursor/mcp.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let kurultai_bin = resolve_kurultai_bin()?;
    let entry = json!({
        "command": kurultai_bin,
        "args": ["mcp"],
    });

    let mut root: Value = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| json!({ "mcpServers": {} })),
        Err(e) if e.kind() == ErrorKind::NotFound => json!({ "mcpServers": {} }),
        Err(e) => return Err(e.into()),
    };

    if root.get("mcpServers").is_none() {
        root["mcpServers"] = json!({});
    }
    root["mcpServers"]["kurultai"] = entry;

    let pretty = serde_json::to_string_pretty(&root)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("encode mcp.json: {e}")))?;
    fs::write(&path, pretty)?;
    Ok(path)
}

fn resolve_kurultai_bin() -> Result<String> {
    if let Ok(exe) = std::env::current_exe() {
        if exe.exists() {
            return Ok(exe.to_string_lossy().into_owned());
        }
    }
    if let Ok(output) = Command::new("which").arg("kurultai").output() {
        if output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                return Ok(p);
            }
        }
    }
    Ok("kurultai".into())
}

/// Ensure a default config.toml exists matching the Rust `FileConfig` shape.
pub fn ensure_default_config() -> Result<PathBuf> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let path = crate::config::config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(default_config_toml().as_bytes())?;
            Ok(path)
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(path),
        Err(e) => Err(e.into()),
    }
}
