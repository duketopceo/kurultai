//! Wire Kurultai into agent MCP configs (`kurultai init --agent …`).
//! Also provisions an on-device markdown folder (`kurultai init --docs`).

use crate::config::default_config_toml;
use crate::error::{KurultaiError, Result};
use clap::ValueEnum;
use serde_json::{json, Value};
use std::fmt::Write as _;
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
    /// Skip MCP wiring (`--agent none`).
    None,
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
            "none" => Ok(Self::None),
            other => Err(format!(
                "unsupported agent '{other}' — supports: cursor, claude, codex, hermes, all, none"
            )),
        }
    }
}

/// Result of `init --docs`: folder path and whether a starter note was created.
#[derive(Debug, Clone)]
pub struct DocsProvision {
    pub dir: PathBuf,
    pub starter_written: bool,
}

const STARTER_NOTE_NAME: &str = "welcome.md";

const STARTER_NOTE: &str = r#"---
title: Welcome to Kurultai
tags: [getting-started, notes]
---

# Welcome

This folder is your on-device docs source. Drop markdown here, then run:

    kurultai index --full

Each note needs at least one `tags:` entry in YAML frontmatter (for example `tags: [notes]`) or it stays in quarantine.

## Next

1. `kurultai daemon --port 8421`
2. Open http://127.0.0.1:8421/ui/

Without an API key, FTS search still works. Set `OPENROUTER_API_KEY` (or `KURULTAI_API_KEY`) for embeddings and LLM ask.
"#;

/// Default on-device docs folder: `Documents/kurultai`, else `~/kurultai`.
pub fn default_docs_dir() -> PathBuf {
    if let Some(dir) = dirs::document_dir() {
        return dir.join("kurultai");
    }
    match dirs::home_dir() {
        Some(home) => {
            let documents = home.join("Documents");
            if documents.is_dir() {
                documents.join("kurultai")
            } else {
                home.join("kurultai")
            }
        }
        None => PathBuf::from("kurultai"),
    }
}

/// Resolve `--docs [PATH]`. Empty / omitted path uses [`default_docs_dir`].
pub fn resolve_docs_dir(arg: Option<&str>) -> Result<PathBuf> {
    let raw = match arg {
        None | Some("") => default_docs_dir(),
        Some(p) => PathBuf::from(p),
    };
    if raw.is_absolute() {
        Ok(raw)
    } else {
        Ok(std::env::current_dir()?.join(raw))
    }
}

/// Create the docs folder, write a tagged starter note if missing, enable `[sources.notes]`.
pub fn provision_docs(docs_arg: Option<&str>, config_path: &Path) -> Result<DocsProvision> {
    let dir = resolve_docs_dir(docs_arg)?;
    fs::create_dir_all(&dir)?;
    let starter = dir.join(STARTER_NOTE_NAME);
    let starter_written = if starter.exists() {
        false
    } else {
        fs::write(&starter, STARTER_NOTE)?;
        true
    };

    let raw = fs::read_to_string(config_path)?;
    let root_path = dir.to_string_lossy();
    let updated = upsert_notes_source(&raw, &root_path)?;
    if updated != raw {
        atomic_write(config_path, updated.as_bytes())?;
    }
    Ok(DocsProvision {
        dir,
        starter_written,
    })
}

/// Printed after `kurultai init` so CLI tests can assert the same walkthrough.
pub fn init_walkthrough(
    config_path: &Path,
    docs: Option<&DocsProvision>,
    mcp_paths: &[PathBuf],
    agent: AgentTarget,
    indexed: bool,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Config: {}", config_path.display());
    if let Some(docs) = docs {
        let _ = writeln!(out, "Docs folder: {}", docs.dir.display());
        if docs.starter_written {
            let _ = writeln!(
                out,
                "  Starter note: welcome.md (tagged so it is searchable)"
            );
        } else {
            let _ = writeln!(out, "  Existing notes kept (welcome.md not overwritten)");
        }
    }
    let _ = writeln!(out);
    if agent == AgentTarget::None {
        let _ = writeln!(out, "MCP: skipped (--agent none)");
    } else {
        for path in mcp_paths {
            let _ = writeln!(out, "MCP wired: {}", path.display());
        }
        let _ = writeln!(out, "Restart the agent(s) to load the kurultai MCP server.");
    }
    let _ = writeln!(out);
    if indexed {
        let _ = writeln!(out, "Indexed sources (full).");
    } else {
        let _ = writeln!(out, "Next: kurultai index --full");
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "Without an API key (FTS-only — works now):");
    if !indexed {
        let _ = writeln!(out, "  kurultai index --full");
    }
    let _ = writeln!(out, "  kurultai search \"welcome\"");
    let _ = writeln!(out, "  kurultai daemon --port 8421");
    let _ = writeln!(out, "  # Brain UI → http://127.0.0.1:8421/ui/");
    let _ = writeln!(out);
    let _ = writeln!(out, "With embeddings / LLM ask:");
    let _ = writeln!(
        out,
        "  export OPENROUTER_API_KEY=...   # or KURULTAI_API_KEY"
    );
    let _ = writeln!(out, "  kurultai index --full");
    let _ = writeln!(out, "  kurultai ask \"what is in my notes?\"");
    let _ = writeln!(out);
    if docs.is_none() {
        let _ = writeln!(out, "Store docs on this device:  kurultai init --docs");
    } else {
        let _ = writeln!(
            out,
            "Drop more markdown in the docs folder (YAML tags: required). Then index again."
        );
    }
    out
}

/// Upsert `[sources.notes]` without dropping other sources. Prefers splicing the
/// dotted table so comments elsewhere in the file survive.
fn upsert_notes_source(raw: &str, root_path: &str) -> Result<String> {
    let mut value: toml::Value = toml::from_str(raw).map_err(|e: toml::de::Error| {
        KurultaiError::config(format!(
            "existing config is not valid TOML ({e}); fix it before re-running init --docs"
        ))
    })?;
    let table = value
        .as_table_mut()
        .ok_or_else(|| KurultaiError::config("config root must be a TOML table"))?;
    apply_notes_fields(table, root_path)?;

    let notes = table
        .get("sources")
        .and_then(|s| s.get("notes"))
        .and_then(toml::Value::as_table)
        .cloned()
        .ok_or_else(|| KurultaiError::config("failed to build sources.notes"))?;
    let rendered_table = render_notes_table(&notes)?;

    if let Some(range) = find_toml_table_range(raw, "sources.notes") {
        let mut out = String::with_capacity(raw.len() + rendered_table.len());
        out.push_str(&raw[..range.start]);
        out.push_str(&rendered_table);
        if !rendered_table.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&raw[range.end..]);
        Ok(out)
    } else {
        let mut out = raw.trim_end().to_string();
        out.push_str("\n\n");
        out.push_str(&rendered_table);
        if !rendered_table.ends_with('\n') {
            out.push('\n');
        }
        Ok(out)
    }
}

fn apply_notes_fields(
    root: &mut toml::map::Map<String, toml::Value>,
    root_path: &str,
) -> Result<()> {
    let sources = root
        .entry("sources".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let sources = sources
        .as_table_mut()
        .ok_or_else(|| KurultaiError::config("`sources` must be a TOML table"))?;
    let notes = sources
        .entry("notes".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let notes = notes
        .as_table_mut()
        .ok_or_else(|| KurultaiError::config("`sources.notes` must be a TOML table"))?;
    notes.insert("enabled".into(), toml::Value::Boolean(true));
    notes.insert("kind".into(), toml::Value::String("markdown".into()));
    notes.insert(
        "root_path".into(),
        toml::Value::String(root_path.to_string()),
    );
    notes
        .entry("poll_interval_secs".to_string())
        .or_insert(toml::Value::Integer(60));
    Ok(())
}

fn render_notes_table(notes: &toml::map::Map<String, toml::Value>) -> Result<String> {
    let mut sources = toml::map::Map::new();
    sources.insert("notes".into(), toml::Value::Table(notes.clone()));
    let mut root = toml::map::Map::new();
    root.insert("sources".into(), toml::Value::Table(sources));
    toml::to_string(&toml::Value::Table(root))
        .map_err(|e| KurultaiError::Other(anyhow::anyhow!("encode sources.notes: {e}")))
}

fn find_toml_table_range(src: &str, name: &str) -> Option<std::ops::Range<usize>> {
    let header = format!("[{name}]");
    let mut search_from = 0;
    while let Some(rel) = src[search_from..].find(&header) {
        let abs = search_from + rel;
        let line_start = src[..abs].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let prefix = src[line_start..abs].trim();
        if prefix.is_empty() {
            let after = abs + header.len();
            let next = src.as_bytes().get(after).copied();
            let header_ok = matches!(
                next,
                None | Some(b'\n') | Some(b'\r') | Some(b' ') | Some(b'\t') | Some(b'#')
            );
            if header_ok {
                let rest = &src[after..];
                let body_rel = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
                let body_start = after + body_rel;
                let mut end = src.len();
                let bytes = src.as_bytes();
                let mut i = body_start;
                while i < src.len() {
                    if bytes[i] == b'[' && (i == 0 || bytes[i - 1] == b'\n') {
                        end = i;
                        break;
                    }
                    i += 1;
                }
                return Some(abs..end);
            }
        }
        search_from = abs + header.len();
    }
    None
}

/// Write/merge MCP server entry for the given agent. Returns every path written.
pub fn wire_agent(agent: AgentTarget) -> Result<Vec<PathBuf>> {
    if agent == AgentTarget::None {
        return Ok(Vec::new());
    }
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
        AgentTarget::None => unreachable!("None returns before home lookup"),
    }
}

/// Remote-hub wiring payload for `connect` (U5/R13). The minted
/// `hub_api_keys` bearer is written ONLY into config formats that carry an
/// `Authorization` header inside an owner-only (0600) file — everywhere else
/// the member's agent gets a stdio bridge entry and the token stays in
/// omaseal / the 0600 agent-key file under [`RemoteWire::key_name`].
#[derive(Debug, Clone)]
pub struct RemoteWire {
    /// Hub `POST /mcp` endpoint, e.g. `https://api-hub.shippedit.dev/mcp`.
    pub mcp_url: String,
    /// Hub base URL — surfaced in bridge notes as `kurultai broker --upstream`.
    pub upstream_url: String,
    /// The minted bearer (plaintext). See the type docs — never lands in a
    /// TOML/YAML config or on stdout.
    pub token: String,
    /// Key name `connect` stored the token under — referenced by the bridge
    /// note so `kurultai broker --key-name` can resolve it at spawn.
    pub key_name: String,
}

/// What remote wiring produced for one client — `connect` prints these so the
/// member sees where the credential went (or deliberately did not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireOutcome {
    /// Local `command: kurultai mcp` stdio entry (`wire_agent` / solo lane).
    Local(PathBuf),
    /// `{url, headers:{Authorization: Bearer …}}` remote entry — the bearer
    /// lives in the agent's config file, written 0600 by `atomic_write`.
    Remote(PathBuf),
    /// Stdio bridge entry (`kurultai mcp --broker`) for formats that cannot
    /// safely carry the bearer — no secret in the file; `note` is the
    /// member-facing next step (start `kurultai broker`, mint a session).
    Bridge { path: PathBuf, note: String },
}

impl WireOutcome {
    /// The config path this outcome wrote.
    pub fn path(&self) -> &Path {
        match self {
            Self::Local(p) | Self::Remote(p) | Self::Bridge { path: p, .. } => p,
        }
    }
}

/// The member-facing setup line attached to every [`WireOutcome::Bridge`].
fn bridge_note(remote: &RemoteWire) -> String {
    format!(
        "stdio bridge (no token in config): run `kurultai broker --upstream {} --key-name {}`, \
         mint a session with `POST http://127.0.0.1:8420/board`, then set KURULTAI_SESSION_KEY \
         in this entry's env",
        remote.upstream_url, remote.key_name
    )
}

/// Wire the agent's MCP config at a remote hub (`connect` lane, U5/R13).
///
/// Per-format ruling — mirrors the per-agent decision table in the hub plan:
///
/// | Agent | Config | Shape written |
/// |-------|--------|---------------|
/// | cursor | `~/.cursor/mcp.json` | `{url, headers:{Authorization}}` |
/// | claude | `~/.claude.json` | `{type:"http", url, headers:{Authorization}}` |
/// | codex | `~/.codex/config.toml` | broker stdio bridge (no secret in file) |
/// | hermes | `~/.hermes/config.yaml` | broker stdio bridge (no secret in file) |
pub fn wire_agent_remote(agent: AgentTarget, remote: &RemoteWire) -> Result<Vec<WireOutcome>> {
    if agent == AgentTarget::None {
        return Ok(Vec::new());
    }
    let home = home_dir()?;
    let bin = resolve_kurultai_bin()?;
    let one = |target: AgentTarget| -> Result<WireOutcome> {
        match target {
            AgentTarget::Cursor => Ok(WireOutcome::Remote(wire_json_mcp_remote_at(
                &home.join(".cursor/mcp.json"),
                remote,
                false,
            )?)),
            AgentTarget::Claude => Ok(WireOutcome::Remote(wire_json_mcp_remote_at(
                &home.join(".claude.json"),
                remote,
                true,
            )?)),
            // Codex TOML has no secrets mechanism the member's environment can
            // guarantee (a `bearer_token_env_var` entry still needs the var
            // exported into the agent's process env), so it wires the local
            // `mcp --broker` bridge — same pattern as scripts/mcp/kurultai/.
            AgentTarget::Codex => Ok(WireOutcome::Bridge {
                path: wire_codex_bridge_at(&home.join(".codex/config.toml"), &bin)?,
                note: bridge_note(remote),
            }),
            // Hermes' documented shape is stdio `command`/`args` — same bridge.
            AgentTarget::Hermes => Ok(WireOutcome::Bridge {
                path: wire_hermes_bridge_at(&home.join(".hermes/config.yaml"), &bin)?,
                note: bridge_note(remote),
            }),
            AgentTarget::All | AgentTarget::None => unreachable!("handled above"),
        }
    };
    match agent {
        AgentTarget::All => Ok(vec![
            one(AgentTarget::Cursor)?,
            one(AgentTarget::Claude)?,
            one(AgentTarget::Codex)?,
            one(AgentTarget::Hermes)?,
        ]),
        single => Ok(vec![one(single)?]),
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
    upsert_json_mcp_at(path, entry)
}

/// Remote `{url, headers:{Authorization: Bearer …}}` variant (U5/R13) —
/// Cursor `mcp.json` detects URL-keyed servers; Claude needs `"type": "http"`.
/// The file lands 0600 via `atomic_write`, same posture as the key file.
fn wire_json_mcp_remote_at(
    path: &Path,
    remote: &RemoteWire,
    include_http_type: bool,
) -> Result<PathBuf> {
    let mut entry = json!({
        "url": remote.mcp_url,
        "headers": { "Authorization": format!("Bearer {}", remote.token) },
    });
    if include_http_type {
        let obj = entry.as_object_mut().ok_or_else(|| {
            KurultaiError::Other(anyhow::anyhow!("mcp entry must be a JSON object"))
        })?;
        obj.insert("type".into(), json!("http"));
    }
    upsert_json_mcp_at(path, entry)
}

/// Merge/overwrite `mcpServers.kurultai` with `entry`, preserving siblings.
fn upsert_json_mcp_at(path: &Path, entry: Value) -> Result<PathBuf> {
    ensure_parent_dir(path)?;

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
    let mut kurultai = toml::map::Map::new();
    kurultai.insert(
        "command".into(),
        toml::Value::String(kurultai_bin.to_string()),
    );
    kurultai.insert(
        "args".into(),
        toml::Value::Array(vec![toml::Value::String("mcp".into())]),
    );
    upsert_codex_mcp_at(path, toml::Value::Table(kurultai))
}

/// Codex bridge entry (U5/R13): `kurultai mcp --broker` stdio so the hub
/// bearer never lands in `config.toml` — the local broker resolves it from
/// the key store and attaches it upstream. `KURULTAI_BROKER_URL` pins the
/// broker's default port; the member adds `KURULTAI_SESSION_KEY` after
/// `POST /board` (see [`bridge_note`]).
fn wire_codex_bridge_at(path: &Path, kurultai_bin: &str) -> Result<PathBuf> {
    let mut kurultai = toml::map::Map::new();
    kurultai.insert(
        "command".into(),
        toml::Value::String(kurultai_bin.to_string()),
    );
    kurultai.insert(
        "args".into(),
        toml::Value::Array(vec![
            toml::Value::String("mcp".into()),
            toml::Value::String("--broker".into()),
        ]),
    );
    let mut env = toml::map::Map::new();
    env.insert(
        "KURULTAI_BROKER_URL".into(),
        toml::Value::String("http://127.0.0.1:8420".into()),
    );
    kurultai.insert("env".into(), toml::Value::Table(env));
    upsert_codex_mcp_at(path, toml::Value::Table(kurultai))
}

/// Merge/overwrite `[mcp_servers.kurultai]` with `entry`, preserving siblings.
fn upsert_codex_mcp_at(path: &Path, entry: toml::Value) -> Result<PathBuf> {
    ensure_parent_dir(path)?;

    let mut root: toml::Value = match fs::read_to_string(path) {
        Ok(raw) => toml::from_str::<toml::Value>(&raw).map_err(|e| {
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

    servers.insert("kurultai".into(), entry);

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
    let mut kurultai = serde_yaml::Mapping::new();
    kurultai.insert(
        serde_yaml::Value::String("command".into()),
        serde_yaml::Value::String(kurultai_bin.to_string()),
    );
    kurultai.insert(
        serde_yaml::Value::String("args".into()),
        serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("mcp".into())]),
    );
    upsert_hermes_mcp_at(path, serde_yaml::Value::Mapping(kurultai))
}

/// Hermes bridge entry (U5/R13): `kurultai mcp --broker` — the config never
/// holds the hub bearer (no documented secrets mechanism for remote URLs);
/// see [`wire_codex_bridge_at`] / [`bridge_note`].
fn wire_hermes_bridge_at(path: &Path, kurultai_bin: &str) -> Result<PathBuf> {
    let mut kurultai = serde_yaml::Mapping::new();
    kurultai.insert(
        serde_yaml::Value::String("command".into()),
        serde_yaml::Value::String(kurultai_bin.to_string()),
    );
    kurultai.insert(
        serde_yaml::Value::String("args".into()),
        serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("mcp".into()),
            serde_yaml::Value::String("--broker".into()),
        ]),
    );
    let mut env = serde_yaml::Mapping::new();
    env.insert(
        serde_yaml::Value::String("KURULTAI_BROKER_URL".into()),
        serde_yaml::Value::String("http://127.0.0.1:8420".into()),
    );
    kurultai.insert(
        serde_yaml::Value::String("env".into()),
        serde_yaml::Value::Mapping(env),
    );
    upsert_hermes_mcp_at(path, serde_yaml::Value::Mapping(kurultai))
}

/// Merge/overwrite `mcp_servers.kurultai` with `entry`, preserving siblings.
fn upsert_hermes_mcp_at(path: &Path, entry: serde_yaml::Value) -> Result<PathBuf> {
    ensure_parent_dir(path)?;

    let mut root: serde_yaml::Value = match fs::read_to_string(path) {
        Ok(raw) if raw.trim().is_empty() => {
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
        }
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

    servers.insert(serde_yaml::Value::String("kurultai".into()), entry);

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
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))
            .map_err(|e| KurultaiError::Other(anyhow::anyhow!("chmod temp config: {e}")))?;
    }
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
    ensure_default_config_at(crate::config::config_path()?)
}

/// Same as [`ensure_default_config`] but at an explicit path (`--config` flag).
pub fn ensure_default_config_at(path: PathBuf) -> Result<PathBuf> {
    use std::fs::OpenOptions;
    use std::io::Write;

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
        assert_eq!(AgentTarget::parse("none").unwrap(), AgentTarget::None);
        assert_eq!(AgentTarget::parse("None").unwrap(), AgentTarget::None);
        let err = AgentTarget::parse("bogus").unwrap_err().to_string();
        assert!(err.contains("cursor"));
        assert!(err.contains("claude"));
        assert!(err.contains("codex"));
        assert!(err.contains("hermes"));
        assert!(err.contains("all"));
        assert!(err.contains("none"));
    }

    #[test]
    fn wire_agent_none_writes_nothing() {
        assert!(wire_agent(AgentTarget::None).unwrap().is_empty());
    }

    // ---- U5/R13 remote (connect-to-hub) wiring ----

    fn test_remote() -> RemoteWire {
        RemoteWire {
            mcp_url: "https://api-hub.shippedit.dev/mcp".into(),
            upstream_url: "https://api-hub.shippedit.dev".into(),
            token: "krt_testtoken123".into(),
            key_name: "work-cursor-seat-agent-token".into(),
        }
    }

    #[cfg(unix)]
    fn file_mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn remote_wire_cursor_writes_bearer_header_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        wire_json_mcp_remote_at(&path, &test_remote(), false).unwrap();
        let root: JsonValue =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let k = &root["mcpServers"]["kurultai"];
        assert_eq!(k["url"], "https://api-hub.shippedit.dev/mcp");
        assert_eq!(
            k["headers"]["Authorization"],
            "Bearer krt_testtoken123"
        );
        assert!(k.get("command").is_none(), "remote entry has no stdio fields");
        #[cfg(unix)]
        assert_eq!(file_mode(&path), 0o600, "token-bearing config is owner-only");
    }

    #[test]
    fn remote_wire_claude_writes_http_type_and_preserves_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude.json");
        fs::write(
            &path,
            r#"{"mcpServers": {"other": {"command": "other-bin"}}, "theme": "dark"}"#,
        )
        .unwrap();
        wire_json_mcp_remote_at(&path, &test_remote(), true).unwrap();
        let root: JsonValue =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let k = &root["mcpServers"]["kurultai"];
        assert_eq!(k["type"], "http");
        assert_eq!(k["url"], "https://api-hub.shippedit.dev/mcp");
        assert_eq!(
            k["headers"]["Authorization"],
            "Bearer krt_testtoken123"
        );
        assert_eq!(
            root["mcpServers"]["other"]["command"],
            "other-bin",
            "sibling server preserved"
        );
        assert_eq!(root["theme"], "dark", "top-level keys preserved");
    }

    #[test]
    fn remote_wire_codex_uses_bridge_without_token_in_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        wire_codex_bridge_at(&path, "kurultai").unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert!(
            !raw.contains("krt_"),
            "hub bearer must never land in codex config.toml"
        );
        let parsed: toml::Value = toml::from_str(&raw).unwrap();
        let k = &parsed["mcp_servers"]["kurultai"];
        assert_eq!(k["command"].as_str(), Some("kurultai"));
        assert_eq!(
            k["args"].as_array().unwrap(),
            &vec![toml::Value::String("mcp".into()), toml::Value::String("--broker".into())]
        );
        assert_eq!(
            k["env"]["KURULTAI_BROKER_URL"].as_str(),
            Some("http://127.0.0.1:8420")
        );
    }

    #[test]
    fn remote_wire_hermes_uses_bridge_without_token_in_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        wire_hermes_bridge_at(&path, "kurultai").unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert!(
            !raw.contains("krt_"),
            "hub bearer must never land in hermes config.yaml"
        );
        let parsed: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
        let k = &parsed["mcp_servers"]["kurultai"];
        assert_eq!(k["command"].as_str(), Some("kurultai"));
        assert_eq!(
            k["args"].as_sequence().unwrap(),
            &vec![
                serde_yaml::Value::String("mcp".into()),
                serde_yaml::Value::String("--broker".into())
            ]
        );
        assert_eq!(
            k["env"]["KURULTAI_BROKER_URL"].as_str(),
            Some("http://127.0.0.1:8420")
        );
    }

    #[test]
    fn bridge_note_points_at_stored_key_not_secret() {
        let note = bridge_note(&test_remote());
        assert!(note.contains("work-cursor-seat-agent-token"));
        assert!(note.contains("https://api-hub.shippedit.dev"));
        assert!(
            !note.contains("krt_testtoken123"),
            "note references the key name, never the secret"
        );
    }

    #[test]
    fn wire_agent_remote_none_writes_nothing() {
        assert!(wire_agent_remote(AgentTarget::None, &test_remote())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn upsert_notes_appends_and_preserves_sibling_source() {
        let raw = r#"environment = "dev"

# keep this comment
[storage]
path = "/tmp/store.db"

[sources.other]
kind = "json"
enabled = false
root_path = "/tmp/data"
"#;
        let out = upsert_notes_source(raw, "/tmp/docs").unwrap();
        assert!(out.contains("# keep this comment"));
        assert!(out.contains("root_path = \"/tmp/data\""));
        let parsed: toml::Value = toml::from_str(&out).unwrap();
        assert_eq!(
            parsed["sources"]["notes"]["kind"].as_str(),
            Some("markdown")
        );
        assert_eq!(
            parsed["sources"]["notes"]["root_path"].as_str(),
            Some("/tmp/docs")
        );
        assert_eq!(parsed["sources"]["notes"]["enabled"].as_bool(), Some(true));
        assert_eq!(parsed["sources"]["other"]["kind"].as_str(), Some("json"));
    }

    #[test]
    fn upsert_notes_refreshes_root_path_keeps_extra_keys() {
        let raw = r#"
[sources.notes]
enabled = false
kind = "markdown"
root_path = "/old"
poll_interval_secs = 90
custom = "keep-me"
"#;
        let out = upsert_notes_source(raw, "/new").unwrap();
        let parsed: toml::Value = toml::from_str(&out).unwrap();
        assert_eq!(
            parsed["sources"]["notes"]["root_path"].as_str(),
            Some("/new")
        );
        assert_eq!(parsed["sources"]["notes"]["enabled"].as_bool(), Some(true));
        assert_eq!(
            parsed["sources"]["notes"]["poll_interval_secs"].as_integer(),
            Some(90)
        );
        assert_eq!(
            parsed["sources"]["notes"]["custom"].as_str(),
            Some("keep-me")
        );
    }

    #[test]
    fn ensure_default_config_at_honors_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("nested").join("config.toml");
        let written = ensure_default_config_at(cfg.clone()).unwrap();
        assert_eq!(written, cfg);
        assert!(cfg.exists());
        // Second call keeps an existing config untouched.
        fs::write(&cfg, "environment = \"dev\"\n").unwrap();
        ensure_default_config_at(cfg.clone()).unwrap();
        assert_eq!(fs::read_to_string(&cfg).unwrap(), "environment = \"dev\"\n");
    }

    #[test]
    fn provision_docs_writes_tagged_starter_once() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("config.toml");
        fs::write(&cfg, default_config_toml()).unwrap();
        let docs = dir.path().join("notes");
        let first = provision_docs(Some(docs.to_str().unwrap()), &cfg).unwrap();
        assert!(first.starter_written);
        let welcome = docs.join("welcome.md");
        let original = fs::read_to_string(&welcome).unwrap();
        assert!(original.contains("tags:"));
        assert!(original.contains("getting-started"));

        fs::write(&welcome, "KEEP ME\n").unwrap();
        let second = provision_docs(Some(docs.to_str().unwrap()), &cfg).unwrap();
        assert!(!second.starter_written);
        assert_eq!(fs::read_to_string(&welcome).unwrap(), "KEEP ME\n");

        let cfg_raw = fs::read_to_string(&cfg).unwrap();
        assert!(cfg_raw.contains("[sources.notes]"));
        assert!(cfg_raw.contains("kind = \"markdown\""));
        let parsed: toml::Value = toml::from_str(&cfg_raw).unwrap();
        assert_eq!(
            parsed["sources"]["notes"]["root_path"].as_str(),
            Some(docs.to_str().unwrap())
        );
    }

    #[test]
    fn walkthrough_covers_fts_and_daemon_ui() {
        let text = init_walkthrough(
            Path::new("/tmp/config.toml"),
            None,
            &[],
            AgentTarget::None,
            false,
        );
        assert!(text.contains("FTS-only"));
        assert!(text.contains("OPENROUTER_API_KEY"));
        assert!(text.contains("KURULTAI_API_KEY"));
        assert!(text.contains("http://127.0.0.1:8421/ui/"));
        assert!(text.contains("kurultai init --docs"));
        assert!(text.contains("MCP: skipped"));
        assert!(text.contains("kurultai index --full"));
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

    #[cfg(unix)]
    #[test]
    fn atomic_write_sets_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mcp.json");
        atomic_write(&path, br#"{"mcpServers":{}}"#).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
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
        let root: toml::Value = toml::from_str(&raw).unwrap();
        assert_eq!(root["model"].as_str(), Some("gpt-5"));
        assert_eq!(root["mcp_servers"]["other"]["command"].as_str(), Some("x"));
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/bin/kurultai")
        );
        let args = root["mcp_servers"]["kurultai"]["args"].as_array().unwrap();
        assert_eq!(args[0].as_str(), Some("mcp"));

        wire_codex_at(&path, "/opt/kurultai").unwrap();
        let root: toml::Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
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
        assert_eq!(root["mcp_servers"]["other"]["command"].as_str(), Some("x"));
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/bin/kurultai")
        );
        let args = root["mcp_servers"]["kurultai"]["args"]
            .as_sequence()
            .unwrap();
        assert_eq!(args[0].as_str(), Some("mcp"));

        // Idempotent: re-run updates only kurultai, preserves other + model
        wire_hermes_at(&path, "/opt/kurultai").unwrap();
        let root: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(root["model"].as_str(), Some("gpt-5"));
        assert_eq!(root["mcp_servers"]["other"]["command"].as_str(), Some("x"));
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

    #[test]
    fn hermes_yaml_empty_treats_as_empty_mapping() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, "").unwrap();
        wire_hermes_at(&path, "/bin/kurultai").unwrap();
        let root: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            root["mcp_servers"]["kurultai"]["command"].as_str(),
            Some("/bin/kurultai")
        );
    }
}
