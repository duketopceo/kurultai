```
                    ╭──────────────────────────╮
                   ╱    ·    kurultai    ·    ╲
                  │    ╭──────────────────╮    │
                  │   ╱   assemble what   ╲   │
                  │  │    you know  ·  yurt │  │
                  │   ╲   from wherever   ╱   │
                  │    ╰──────────────────╯    │
                   ╲         ⌂ ⌂ ⌂         ╱
                    ╰──────────────────────────╯
```

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Release](https://img.shields.io/github/v/release/duketopceo/kurultai)](https://github.com/duketopceo/kurultai/releases/latest)

# Kurultai

**Assemble what you know, from wherever it lives.**

A local knowledge brain for agents and humans. Index notes, chats, JSON dumps, Dayflow, Pond, and code checkouts into one SQLite store — then `search` / `ask` / MCP with excerpts and citations, not whole-vault dumps.

**Current release:** [v0.3.0](https://github.com/duketopceo/kurultai/releases/tag/v0.3.0)

## Install

One line (macOS / Linux — prefers the latest GitHub Release binary, otherwise cargo):

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

Windows:

```powershell
irm https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.ps1 | iex
```

From source ([Rust](https://rustup.rs)):

```bash
cargo install --git https://github.com/duketopceo/kurultai --tag v0.3.0 --locked
```

Binaries ship for macOS (arm64 / amd64), Linux amd64, and Windows amd64 via [`.github/workflows/release.yml`](.github/workflows/release.yml).

## Quick start

```bash
export KURULTAI_ENV=dev          # optional; stores under …/kurultai/dev/
export RUST_LOG=kurultai=debug   # optional

kurultai init --agent all        # cursor + claude + codex + hermes
# edit ~/.config/kurultai/config.toml — enable at least one source

kurultai index --full            # FTS works with no API key
kurultai status
kurultai search "database migration" --limit 10
kurultai ask "what deployments are we running?"

kurultai mcp                     # stdio MCP for agents
kurultai daemon --port 8421      # HTTP API + Brain UI
# Live query histograms (Prometheus text): curl http://127.0.0.1:8421/api/metrics
# Or: kurultai status --metrics [--port 8421]

# Multi-device: pack this brain and restore elsewhere
kurultai export -o brain.kurultai
# …copy over a trusted channel (pack is unencrypted indexed data; delete after import)…
kurultai import brain.kurultai            # new device (empty store)
kurultai import brain.kurultai --combine  # merge into an existing store
```

**Brain UI:** open [`http://127.0.0.1:8421/ui/`](http://127.0.0.1:8421/ui/) (trailing slash matters). Assets live in `ui/` and are embedded in the daemon binary — that is the only Brain dashboard. Do not add a parallel brain under `website/` or `web/`.

Markdown atoms need **≥1 tag** (YAML frontmatter `tags:`); untagged writes land in quarantine and are skipped by default search. Promote after fixing: `kurultai promote <atom_id>`.

Longer Mac notes: [docs/mac-dev.md](docs/mac-dev.md). Concepts: [CONCEPTS.md](CONCEPTS.md).

## What ships (v0.3.0)

| Layer | Reality |
|-------|---------|
| **CLI** | `init`, `index`, `search`, `ask`, `who-knows`, `status`, `promote`, `export`, `import`, `mcp`, `daemon` |
| **Connectors** | Markdown · JSON/NDJSON · Dayflow · Pond · GitHub (local checkout). AppFlowy deferred ([#4](https://github.com/duketopceo/kurultai/issues/4)) |
| **Store** | SQLite + FTS5 + sqlite-vec · hot / warm / cold memory · ingestion staging |
| **Search** | FTS ∥ vector → RRF → optional rerank |
| **Embeddings** | **FTS-first by default** (`NullEmbedder` when no key). OpenRouter when `OPENROUTER_API_KEY` / `KURULTAI_API_KEY` is set. Opt-in local ONNX: `embed.backend = "local"` + `--features local-embed` |
| **Agents** | MCP stdio — `search`, `cite`, `ask`, `who_knows`, `remember`, `promote` |
| **Daemon** | HTTP `/api/*` + poll/watch · Brain UI at `GET /ui/` · `GET /api/metrics` (Prometheus query histograms) |

Without an API key, FTS search / `who-knows` / extractive `ask` all work. Vector recall, reranking, and LLM `ask` stay off until a key (or local embed) is configured. That is expected, not an error.

## Configuration

`~/.config/kurultai/config.toml` (from `kurultai init`):

```toml
environment = "dev"   # dev | staging | prod

[sources.notes]
enabled = true
kind = "markdown"
root_path = "/Users/you/Documents/notes"
poll_interval_secs = 60

[sources.data]
enabled = false
kind = "json"                    # .json arrays or .jsonl / .ndjson
root_path = "/Users/you/data"
# optional: id_field = "url"     # stable source_id field (default: id)

[sources.dayflow]
enabled = false
kind = "dayflow"

[sources.pond]
enabled = false
kind = "pond"

[sources.code]
enabled = false
kind = "github"
root_path = "/Users/you/src/your-repo"

[embed]
model = "openai/text-embedding-3-large"
dimension = 3072
# Offline vectors (requires `cargo install --path . --features local-embed`):
# backend = "local"
# model = "AllMiniLML6V2"
# dimension = 384

[runtime]
poll_interval_secs = 300
# nightly_full_sync_hour = 3
# inactivity_threshold_hours = 6
# reranker_model = "openai/gpt-4o-mini"

[cli]
# banner = "auto"   # true | false | "auto" (TTY only); suppressed by --plain / NO_COLOR / KURULTAI_PLAIN
```

Overrides: `KURULTAI_ENV=dev`, `kurultai --env staging status`. API keys via env only — never in config files.

## Agents (MCP)

Same stdio server (`kurultai mcp`) for every client. `init` only writes the host config:

| `--agent` | Config written |
|-----------|----------------|
| `cursor` (default) | `~/.cursor/mcp.json` |
| `claude` | `~/.claude.json` |
| `codex` | `~/.codex/config.toml` |
| `hermes` | `~/.hermes/config.yaml` |
| `all` | all four |

**Copy-paste setup prompt for any agent:** [`AGENT_SETUP_PROMPT.md`](AGENT_SETUP_PROMPT.md) · full text in [`docs/AGENT_SETUP_PROMPT.md`](docs/AGENT_SETUP_PROMPT.md).

Portable skill: [`skills/kurultai-brain/SKILL.md`](skills/kurultai-brain/SKILL.md). Restart the agent after `init` so tools reload.

## Other surfaces

| Path | Purpose |
|------|---------|
| [`ui/`](ui/) | Brain UI (embedded in daemon) |
| [`website/`](website/) | Public Vite marketing / live `ui/` preview (`npm run dev` → often `:5174`) |
| [`web/`](web/) | Team app — **Clerk + Sign in with GitHub** |

```bash
cd web && cp .env.example .env.local   # Clerk keys
npm install && npm run dev             # http://localhost:3000
```

Multi-user model: [docs/multi-user-kurultai.md](docs/multi-user-kurultai.md). Coolify/Docker scaffolding for `web/` is on branch `feat/coolify-frontend-beginnings` (not on `main` yet).

## Environments

| | **Dev** | **Staging** | **Prod** |
|---|---------|-------------|----------|
| Storage | `…/kurultai/dev/store.db` | `…/staging/…` | `…/store.db` |
| Logging | `kurultai=debug` | info | warn |
| API keys | Optional (FTS) | Optional for FTS; required for **remote** embeddings (OpenRouter). Local ONNX (`embed.backend = "local"`) needs no cloud key | Same |

## Docs & contributing

- [CONTRIBUTING.md](CONTRIBUTING.md) · [SECURITY.md](SECURITY.md) · [CONCEPTS.md](CONCEPTS.md)
- Mac / laptop: [docs/mac-dev.md](docs/mac-dev.md)
- Upstream notes: [docs/upstream-inspiration.md](docs/upstream-inspiration.md)
- Plans / Agent Zero drafts: [`docs/plans/`](docs/plans/) · [`docs/agent-zero/`](docs/agent-zero/)

Roadmap: developer → solo → team → company ([#25](https://github.com/duketopceo/kurultai/issues/25), [#27](https://github.com/duketopceo/kurultai/issues/27)).

| Phase | Status |
|-------|--------|
| 1–3 Foundation / search / synthesis | ✅ |
| 4 Expansion (Dayflow · Pond · GitHub FS) | ✅ [complete](docs/plans/phase-4-complete.md) |
| 5 Production (poll · watch · local ONNX · MCP agents) | ✅ [complete](docs/plans/phase-5-complete.md) · [closeout](docs/plans/phase-5-closeout.md) |
| 6 Launch (release packaging, yurt art) | 📋 [#10](https://github.com/duketopceo/kurultai/issues/10) · yurt CLI art [#22](https://github.com/duketopceo/kurultai/issues/22) |

Deferred ops (not Phase 5 product exit): [#20](https://github.com/duketopceo/kurultai/issues/20) ARC · [#29](https://github.com/duketopceo/kurultai/issues/29) env hardening · [#35](https://github.com/duketopceo/kurultai/issues/35) GlitchTip — see [phase-5-complete.md](docs/plans/phase-5-complete.md).

## License

MIT

## Name

Kurultai (курултай) — a council or assembly. Fitting for a system that gathers knowledge from many sources.
