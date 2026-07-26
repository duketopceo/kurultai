//! Wire Kurultai into agent MCP configs (`kurultai init --agent …`).

use crate::config::default_config_toml;
use crate::error::{KurultaiError, Result};
use clap::ValueEnum;
use serde_json::{json, Value};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

/// Supported agent targets for MCP auto-wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum AgentTarget {
    Cursor,
    Claude,
    Codex,
    Hermes,
    All,
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
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "hermes" => Ok(Self::Hermes),
            "all" => Ok(Self::All),
            other => Err(format!(
                "unsupported agent '{other}' — supports: cursor, claude, codex, hermes, all"
            )),
        }
    }
}

/// Write/merge MCP server entry for the given agent. Returns every path written.
pub fn wire_agent(agent: AgentTarget) -> Result<Vec<PathBuf>> {
    let home = home_dir()?;
    let bin = resolve_kurultai_bin()?;
    match agent {
        AgentTarget::Cursor => Ok(vec![wire_json_mcp_at(
            &home.join(".cursor/mcp.json"),
            &bin,
            false,
        )?]),
        AgentTarget::Claude => Ok(vec![wire_json_mcp_at(
            &home.join(".claude.json"),
            &bin,
            true,
        )?]),
        AgentTarget::Codex => Ok(vec![wire_codex_at(&home.join(".codex/config.toml"), &bin)?]),
        AgentTarget::Hermes => Ok(vec![wire_hermes_at(
            &home.join(".hermes/config.yaml"),
            &bin,
        )?]),
        AgentTarget::All => Ok(vec![
            wire_json_mcp_at(&home.join(".cursor/mcp.json"), &bin, false)?,
            wire_json_mcp_at(&home.join(".claude.json"), &bin, true)?,
            wire_codex_at(&home.join(".codex/config.toml"), &bin)?,
            wire_hermes_at(&home.join(".hermes/config.yaml"), &bin)?,
        ]),
    }
}

fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| KurultaiError::config("cannot resolve home directory"))
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Merge `mcpServers.kurultai` into a Cursor/Claude-style JSON MCP config.
fn wire_json_mcp_at(path: &Path, kurultai_bin: &str, include_stdio_type: bool) -> Result<PathBuf> {
    ensure_parent_dir(path)?;

    let mut entry = json!({
        "command": kurultai_bin,
        "args": ["mcp"],
    });
    if include_stdio_type {
        let obj = entry.as_object_mut().ok_or_else(|| {
            KurultaiError::Other(anyhow::anyhow!("mcp entry must be a JSON object"))
        })?;
        obj.insert("type".into(), json!("stdio"));
    }

    let mut root: Value = match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).map_err(|e| {
            KurultaiError::config(format!(
                "existing {} is not valid JSON ({e}); fix or move it before re-running init — refusing to overwrite other MCP servers",
                path.display()
            ))
        })?,
        Err(e) if e.kind() == ErrorKind::NotFound => json!({ "mcpServers": {} }),
        Err(e) => return Err(e.into()),
    };

    match root.get_mut("mcpServers") {
        Some(servers) if servers.is_object() => {
            servers["kurultai"] = entry;
        }
        Some(_) => {
            return Err(KurultaiError::config(format!(
                "{}: mcpServers must be a JSON object",
                path.display()
            )));
        }
        None => {
            root["mcpServers"] = json!({ "kurultai": entry });
        }
    }

    let pretty = serde_json::to_string_pretty(&root)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("encode mcp json: {e}")))?;
    atomic_write(path, pretty.as_bytes())?;
    Ok(path.to_path_buf())
}

/// Merge `[mcp_servers.kurultai]` into a Codex `config.toml`.
fn wire_codex_at(path: &Path, kurultai_bin: &str) -> Result<PathBuf> {
    ensure_parent_dir(path)?;

    let mut root: toml::Value = match fs::read_to_string(path) {
        Ok(raw) => raw.parse::<toml::Value>().map_err(|e| {
            KurultaiError::config(format!(
                "existing {} is not valid TOML ({e}); fix or move it before re-running init — refusing to overwrite other MCP servers",
                path.display()
            ))
        })?,
        Err(e) if e.kind() == ErrorKind::NotFound => toml::Value::Table(toml::map::Map::new()),
        Err(e) => return Err(e.into()),
    };

    let table = root.as_table_mut().ok_or_else(|| {
        KurultaiError::config(format!("{}: root must be a TOML table", path.display()))
    })?;

    let mcp_servers = table
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let servers = mcp_servers.as_table_mut().ok_or_else(|| {
        KurultaiError::config(format!(
            "{}: mcp_servers must be a TOML table",
            path.display()
        ))
    })?;

    let mut kurultai = toml::map::Map::new();
    kurultai.insert(
        "command".into(),
        toml::Value::String(kurultai_bin.to_string()),
    );
    kurultai.insert(
        "args".into(),
        toml::Value::Array(vec![toml::Value::String("mcp".into())]),
    );
    servers.insert("kurultai".into(), toml::Value::Table(kurultai));

    let rendered = toml::to_string_pretty(&root)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("encode codex config.toml: {e}")))?;
    atomic_write(path, rendered.as_bytes())?;
    Ok(path.to_path_buf())
}

/// Merge `mcp_servers.kurultai` into a Hermes Agent `config.yaml`.
///
/// Hermes reads MCP servers from `~/.hermes/config.yaml` under `mcp_servers`
/// (see https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp).
/// Stdio shape: `command` + `args`. Tools register as `mcp_kurultai_<tool>`.
fn wire_hermes_at(path: &Path, kurultai_bin: &str) -> Result<PathBuf> {
    ensure_parent_dir(path)?;

    let mut root: serde_yaml::Value = match fs::read_to_string(path) {
        Ok(raw) => serde_yaml::from_str(&raw).map_err(|e| {
            KurultaiError::config(format!(
                "existing {} is not valid YAML ({e}); fix or move it before re-running init — refusing to overwrite other MCP servers",
                path.display()
            ))
        })?,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
        }
        Err(e) => return Err(e.into()),
    };

    let mapping = root.as_mapping_mut().ok_or_else(|| {
        KurultaiError::config(format!("{}: root must be a YAML mapping", path.display()))
    })?;

    let mcp_servers = mapping
        .entry(serde_yaml::Value::String("mcp_servers".into()))
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    let servers = mcp_servers.as_mapping_mut().ok_or_else(|| {
        KurultaiError::config(format!(
            "{}: mcp_servers must be a YAML mapping",
            path.display()
        ))
    })?;

    let mut kurultai = serde_yaml::Mapping::new();
    kurultai.insert(
        serde_yaml::Value::String("command".into()),
        serde_yaml::Value::String(kurultai_bin.to_string()),
    );
    kurultai.insert(
        serde_yaml::Value::String("args".into()),
        serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("mcp".into())]),
    );
    servers.insert(
        serde_yaml::Value::String("kurultai".into()),
        serde_yaml::Value::Mapping(kurultai),
    );

    let rendered = serde_yaml::to_string(&root)
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("encode hermes config.yaml: {e}")))?;
    atomic_write(path, rendered.as_bytes())?;
    Ok(path.to_path_buf())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("cfg")
    ));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
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
    ensure_parent_dir(&path)?;
    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(default_config_toml().as_bytes())?;
            Ok(path)
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(path),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as JsonValue;

    #[test]
    fn parse_agent_targets() {
        assert_eq!(AgentTarget::parse("cursor").unwrap(), AgentTarget::Cursor);
        assert_eq!(AgentTarget::parse("Claude").unwrap(), AgentTarget::Claude);
        assert_eq!(AgentTarget::parse("CODEX").unwrap(), AgentTarget::Codex);
        assert_eq!(AgentTarget::parse("hermes").unwrap(), AgentTarget::Hermes);
        assert_eq!(AgentTarget::parse("Hermes").unwrap(), AgentTarget::Hermes);
        assert_eq!(AgentTarget::parse("all").unwrap(), AgentTarget::All);
        let err = AgentTarget::parse("bogus").unwrap_err().to_string();
        assert!(err.contains("cursor"));
        assert!(err.contains("claude"));
        assert!(err.contains("codex"));
        assert!(err.contains("hermes"));
        assert!(err.contains("all"));
    }

    #[test]
    fn json_merge_creates_and_preserves_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(
            &path,
            r#"{ "mcpServers": { "other": { "command": "x" } } }"#,
        )
        .unwrap();

        wire_json_mcp_at(&path, "/bin/kurultai", false).unwrap();
        let root: JsonValue = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(root["mcpServers"]["other"]["command"], "x");
        assert_eq!(root["mcpServers"]["kurultai"]["command"], "/bin/kurultai");
        assert_eq!(root["mcpServers"]["kurultai"]["args"][0], "mcp");
        assert!(root["mcpServers"]["kurultai"].get("type").is_none());

        wire_json_mcp_at(&path, "/opt/kurultai", true).unwrap();
        let root: JsonValue = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(root["mcpServers"]["kurultai"]["command"], "/opt/kurultai");
        assert_eq!(root["mcpServers"]["kurultai"]["type"], "stdio");
        assert_eq!(root["mcpServers"]["other"]["command"], "x");
    }

    #[test]
    fn json_malformed_refuses_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        fs::write(&path, "NOT JSON {{{").unwrap();
        let err = wire_json_mcp_at(&path, "/bin/kurultai", false).unwrap_err();
        assert!(err.to_string().contains("not valid JSON"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "NOT JSON {{{");
    }

    #[test]
    fn codex_toml_creates_and_preserves() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            "model = \"gpt-5\"\n\n[mcp_servers.other]\ncommand = \"x\"\n",
        )
        .unwrap();

        wire_codex_at(&path, "/bin/kurultai").unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        let root: toml::Value = raw.parse().unwrap();
        assert_eq!(root["model"].as_str(), Some("gpt-5"));
        assert_eq!(root["mcp_servers"]["other"]["command"].as_str(), Some("x"));
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/bin/kurultai")
        );
        let args = root["mcp_servers"]["kurultai"]["args"].as_array().unwrap();
        assert_eq!(args[0].as_str(), Some("mcp"));

        wire_codex_at(&path, "/opt/kurultai").unwrap();
        let root: toml::Value = fs::read_to_string(&path).unwrap().parse().unwrap();
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/opt/kurultai")
        );
        assert_eq!(root["mcp_servers"]["other"]["command"].as_str(), Some("x"));
    }

    #[test]
    fn codex_invalid_toml_refuses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "[[[broken").unwrap();
        let err = wire_codex_at(&path, "/bin/kurultai").unwrap_err();
        assert!(err.to_string().contains("not valid TOML"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "[[[broken");
    }

    #[test]
    fn json_creates_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude.json");
        wire_json_mcp_at(&path, "kurultai", true).unwrap();
        let root: JsonValue = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(root["mcpServers"]["kurultai"]["type"], "stdio");
    }

    #[test]
    fn hermes_yaml_creates_and_preserves_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(
            &path,
            "model: \"gpt-5\"\n\nmcp_servers:\n  other:\n    command: \"x\"\n    args: [\"run\"]\n",
        )
        .unwrap();

        wire_hermes_at(&path, "/bin/kurultai").unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        let root: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
        assert_eq!(root["model"].as_str(), Some("gpt-5"));
        assert_eq!(
            root["mcp_servers"]["other"]["command"].as_str(),
            Some("x")
        );
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/bin/kurultai")
        );
        let args = root["mcp_servers"]["kurultai"]["args"].as_sequence().unwrap();
        assert_eq!(args[0].as_str(), Some("mcp"));

        // Idempotent: re-run updates only kurultai, preserves other + model
        wire_hermes_at(&path, "/opt/kurultai").unwrap();
        let root: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(root["model"].as_str(), Some("gpt-5"));
        assert_eq!(
            root["mcp_servers"]["other"]["command"].as_str(),
            Some("x")
        );
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/opt/kurultai")
        );
    }

    #[test]
    fn hermes_yaml_creates_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        wire_hermes_at(&path, "kurultai").unwrap();
        let root: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("kurultai")
        );
        assert_eq!(
            root["mcp_servers"]["kurultai"]["args"][0].as_str(),
            Some("mcp")
        );
    }

    #[test]
    fn hermes_yaml_invalid_refuses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, "model: \"gpt-5\n  : broken yaml\n").unwrap();
        let err = wire_hermes_at(&path, "/bin/kurultai").unwrap_err();
        assert!(err.to_string().contains("not valid YAML"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "model: \"gpt-5\n  : broken yaml\n"
        );
    }
}
