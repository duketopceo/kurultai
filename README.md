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

# Kurultai

**Assemble what you know, from wherever it lives.**

A local knowledge brain for agents and humans: index notes, agent chats, Dayflow, and code checkouts into one SQLite store — then `search` / `ask` / MCP without dumping whole vaults into context.

## Install

No GitHub Release / `v*` tag yet — install from source with Cargo ([rustup](https://rustup.rs)):

```bash
cargo install --git https://github.com/duketopceo/kurultai --locked
```

Optional wrapper (same cargo path until binaries ship):

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

Windows (when a release exists): `irm …/scripts/install.ps1 | iex` — until then, use `cargo install --git` as above.

Tagged binary releases: workflow is ready (`.github/workflows/release.yml`); **not published yet**.

## Mac / laptop — stay in dev + debug

```bash
export KURULTAI_ENV=dev
export RUST_LOG=kurultai=debug

kurultai init --agent cursor
# edit ~/.config/kurultai/config.toml — keep environment = "dev"

kurultai index --full          # FTS-first without OPENROUTER_API_KEY
kurultai status
kurultai search "database migration" --limit 10
kurultai ask "what deployments are we running?"

kurultai mcp                   # Cursor / agents (stdio)
kurultai daemon --port 8421    # HTTP + poll + filesystem watch
```

Longer Mac notes: [docs/mac-dev.md](docs/mac-dev.md).

## Why

Knowledge lives in many places. Kurultai indexes it into one queryable brain so agents get **excerpts + citations**, not full-file dumps.

| | **Your files** | **Kurultai brain** (`store.db`) |
|---|--------------|----------------------------------|
| You edit | ✅ notes, code, chats | ❌ index only |
| Agent access | Slow / high tokens | Fast / low tokens (capped views) |

## Status

| Layer | What ships |
|-------|------------|
| **Connectors** | Markdown · Dayflow · Pond · GitHub (local checkout). AppFlowy deferred ([#4](https://github.com/duketopceo/kurultai/issues/4)) |
| **Embeddings** | OpenRouter when keyed; **NullEmbedder** FTS-first without key |
| **Store** | SQLite + FTS5 + sqlite-vec |
| **Search** | FTS ∥ vector → RRF → optional rerank |
| **Synthesis** | Extractive / optional LLM `ask` with citations |
| **Interface** | CLI + MCP stdio + HTTP daemon (poll + notify watch) |

## Configuration

`~/.config/kurultai/config.toml` (created by `kurultai init`):

```toml
environment = "dev"   # dev | staging | prod

[sources.notes]
enabled = true
kind = "markdown"
root_path = "/Users/you/Documents/notes"   # any .md folder
poll_interval_secs = 60

[sources.dayflow]
enabled = false
kind = "dayflow"
# db_path optional on macOS

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

[runtime]
poll_interval_secs = 300
# reranker_model = "openai/gpt-4o-mini"   # needs API key
```

Overrides: `KURULTAI_ENV=dev`, `kurultai --env staging status`. API keys via env only (`OPENROUTER_API_KEY` / `KURULTAI_API_KEY`) — never in config files.

## Agents (MCP)

| | Tools | Budget |
|---|-------|--------|
| **Read** | `search`, `cite`, `ask`, `who_knows` | Excerpts + citations |
| **Write** | `remember` | Summary / tags only |

```
Agent ──read──► search/cite/ask ──► SQLite brain ──► ranked excerpts
Agent ─write──► remember ──► KnowledgeAtom ──► SQLite brain
```

## Environments

| | **Dev** | **Staging** | **Prod** |
|---|---------|-------------|----------|
| Storage | `…/kurultai/dev/store.db` | `…/staging/…` | `…/store.db` |
| Logging | `kurultai=debug` | info | warn |
| API keys | Optional (FTS) | Optional for FTS; required for embeddings | Same + audit |

## Roadmap

Ship **developer → solo → team → company** ([#25](https://github.com/duketopceo/kurultai/issues/25)). Master plan: [#27](https://github.com/duketopceo/kurultai/issues/27).

| Phase | Status |
|-------|--------|
| 1–3 Foundation / search / synthesis | ✅ |
| 4 Expansion (Dayflow · Pond · GitHub FS) | ✅ |
| 5 Production (daemon poll + watch) | 🚧 local embeddings / ARC / ops follow |
| 6 Launch (release packaging, yurt art) | 📋 [#10](https://github.com/duketopceo/kurultai/issues/10) |

Upstream notes: [docs/upstream-inspiration.md](docs/upstream-inspiration.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Security: [SECURITY.md](SECURITY.md).

## License

MIT

## Name

Kurultai (курултай) — a council or assembly. Fitting for a system that gathers knowledge from many sources.

## v1 work orders

Agent Zero drafts and issue map: [`docs/agent-zero/`](docs/agent-zero/).  
Deep personal install script (checkout tree): `scripts/install/install.sh` (see also top-level `scripts/install.sh`).
