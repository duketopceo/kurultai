# Mac laptop — dev / debug

Stay on **dev** until you deliberately move to staging/prod.

## Install

One line (binary from GitHub Releases when present; otherwise cargo):

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

Or pin a tag with cargo ([rustup](https://rustup.rs)):

```bash
cargo install --git https://github.com/duketopceo/kurultai --tag v0.2.0 --locked
```

Ensure `~/.local/bin` or `~/.cargo/bin` is on `PATH`.

## Shell env

```bash
export KURULTAI_ENV=dev
export RUST_LOG=kurultai=debug
# optional: OPENROUTER_API_KEY=… for embeddings / rerank / LLM ask
```

## Wire + index

```bash
# Wire MCP into Cursor, Claude Code, Codex, and/or Hermes (same tools; different configs)
kurultai init --agent all          # or: cursor | claude | codex | hermes
# edit ~/.config/kurultai/config.toml — keep environment = "dev"
# Restart the agent(s) so MCP tools reload
kurultai index --full
kurultai status
```

| Agent | Config path |
|-------|-------------|
| Cursor | `~/.cursor/mcp.json` |
| Claude Code | `~/.claude.json` |
| Codex | `~/.codex/config.toml` |
| Hermes Agent | `~/.hermes/config.yaml` (tools register as `mcp_kurultai_*`) |

A portable `kurultai-brain` SKILL.md (agentskills.io-compatible) lives at
`skills/kurultai-brain/SKILL.md` so skill-host agents (Hermes, Claude Code,
Cursor, Codex) can discover how to use the kurultai MCP tools.

| Source | `kind` | Notes |
|--------|--------|--------|
| Notes | `markdown` | `root_path` to a `.md` folder |
| Dayflow | `dayflow` | macOS Dayflow `chunks.sqlite` |
| Pond | `pond` | `pond` on `PATH` |
| Code | `github` | local checkout `root_path` |

AppFlowy is deferred ([#4](https://github.com/duketopceo/kurultai/issues/4)).

## Smoke

```bash
kurultai search "test" --limit 5
kurultai daemon --port 8421
```

Storage default: `~/.local/share/kurultai/dev/store.db`.
