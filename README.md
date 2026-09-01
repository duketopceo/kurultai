# Kurultai

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Release](https://img.shields.io/github/v/release/duketopceo/kurultai)](https://github.com/duketopceo/kurultai/releases/latest)

**Assemble what you know, from wherever it lives.**

Unified knowledge retrieval for agents and humans. Index notes, chats, JSON dumps, Dayflow, Pond, and local code checkouts into one SQLite store — then `search`, `ask`, and MCP with excerpts and citations, not whole-vault dumps.

| | |
|---|---|
| **Author** | [Luke Kimball](https://github.com/duketopceo) (`duketopceo@gmail.com`) |
| **Repo** | Public — [github.com/duketopceo/kurultai](https://github.com/duketopceo/kurultai) |
| **Release** | [v0.5.0](https://github.com/duketopceo/kurultai/releases/tag/v0.5.0) (crate `0.5.0`) |

## Why

Knowledge lives in markdown folders, agent transcripts, JSON exports, and git trees. Kurultai pulls those sources into a local brain with hybrid search, quality gates, and agent wiring so Cursor, Claude, Codex, and Hermes can recall what you already wrote — with source citations.

FTS search works with **no API key**. Vector recall, reranking, and LLM `ask` use **OpenRouter** with one `OPENROUTER_API_KEY` (local ONNX embeddings are an opt-in alternative).

## Quick start

**Install** (macOS / Linux — release binary or `cargo install`):

```bash
curl -fsSL https://raw.githubusercontent.com/duketopceo/kurultai/main/scripts/install.sh | bash
```

From source ([Rust stable](https://rustup.rs)):

```bash
cargo install --git https://github.com/duketopceo/kurultai --tag v0.5.0 --locked
```

**OpenRouter** (full brain — one key for embeddings, rerank, and LLM `ask`):

```bash
export OPENROUTER_API_KEY=sk-or-...   # or KURULTAI_API_KEY
```

That single key routes embeddings (`openai/text-embedding-3-large`), optional rerank, and LLM `ask` (`openai/gpt-4o-mini`) through OpenRouter. FTS search still works without it; vectors + rerank + LLM `ask` need the key.

**Run** (solo markdown folder + config):

```bash
export KURULTAI_ENV=dev          # optional — store under ~/.local/share/kurultai/dev/

kurultai init --docs             # ~/Documents/kurultai (or ~/kurultai) + config
kurultai index --full
kurultai search "welcome" --limit 10
kurultai ask "what is in my notes?"   # extractive without an API key
kurultai status

kurultai mcp                     # stdio MCP for agents
kurultai daemon --port 8421      # HTTP API + Brain UI → http://127.0.0.1:8421/ui/
```

Windows: `irm …/scripts/install.ps1 | iex`. Longer setup notes: [`docs/mac-dev.md`](docs/mac-dev.md), [`INSTALL_GUIDE.md`](INSTALL_GUIDE.md).

**Tag gate:** markdown atoms need ≥1 tag (YAML `tags:` or a dedicated hashtag line like `#vpn #notes`). Untagged writes land in quarantine; promote after fixing: `kurultai promote <atom_id>`.

**Pack / restore:** `kurultai export -o brain.kurultai` · `kurultai import brain.kurultai [--combine]`

## Architecture

```
Sources (markdown · json · inbox · dayflow · pond · github)
        ↓ connectors + quality gate
SQLite store (FTS5 + sqlite-vec) — hot / warm / cold tiers
        ↓ hybrid search (FTS ∥ vector → RRF → optional rerank)
CLI · axum daemon (/api/*) · MCP stdio · Brain UI (embedded)
```

| Layer | Implementation |
|-------|----------------|
| **Core** | Rust — CLI + library (`src/`) |
| **HTTP daemon** | [axum](https://github.com/tokio-rs/axum) — `/api/*`, optional `POST /ingest`, MCP HTTP/SSE |
| **Store (default)** | SQLite + FTS5 + [sqlite-vec](https://github.com/asg017/sqlite-vec) |
| **Search** | FTS ∥ vector → reciprocal rank fusion → soft-label boost → optional rerank |
| **OpenRouter** | One `OPENROUTER_API_KEY` (or `KURULTAI_API_KEY`) — embed `openai/text-embedding-3-large`, rerank + LLM `ask` `openai/gpt-4o-mini`. FTS-only without a key; optional local ONNX via `--features local-embed` |
| **Agents** | MCP stdio — `search`, `cite`, `remember`, `ask`, `who_knows`, `promote`, `ontology_get`, `ontology_promote`, `recall` |
| **Brain UI** | Vite + React + Three.js (`website/` → built `ui/`, rust-embedded at `GET /ui/`) |

**Optional hub track (off by default):** `--features postgres` + `KURULTAI_FEATURE_HUB=1` adds a Postgres/pgvector store for shared `team` / `company` atoms. Default installs stay SQLite-only.

**Other surfaces:** [`website/`](website/) — Brain UI source and Vite dev preview · [`web/`](web/) — Next.js team app (Clerk auth, separate from the embedded brain).

Config templates: [`config.example.toml`](config.example.toml) · [`.env.example`](.env.example). Domain vocabulary: [`CONCEPTS.md`](CONCEPTS.md).

## What ships (v0.5.0)

| Area | Details |
|------|---------|
| **CLI** | `init`, `index`, `search`, `ask`, `who-knows`, `status`, `promote`, `export`, `import`, `mcp`, `daemon`, `prune`, `doctor`, `admin key` |
| **Connectors** | Markdown · JSON · inbox tray · Dayflow · Pond · GitHub (local checkout). AppFlowy is registered but not implemented ([#4](https://github.com/duketopceo/kurultai/issues/4)) |
| **Daemon API** | `/api/status`, `/api/atoms`, `/api/graph`, `/api/search`, `/api/ask`, `/api/recall`, `/api/ontology`, `/api/metrics`, … |
| **MCP** | Stdio (full write tools). Daemon HTTP/SSE is read-only when `KURULTAI_MCP_HTTP_SECRET` is set |
| **Trust** | Atoms in `trusted` or `quarantine` lanes; visibility scopes `personal` / `team` / `company` (HUB-1) |
| **Ontology** | O1 class tree + `ontology_get` / `ontology_promote` MCP tools |

Acceptance coverage: [`FEATURE_MATRIX.md`](FEATURE_MATRIX.md) · [`ACCEPTANCE_REPORT.md`](ACCEPTANCE_REPORT.md).

## Agents (MCP)

```bash
kurultai init --agent cursor    # also: claude | codex | hermes | all | none
kurultai mcp                    # restart the agent after init
```

Copy-paste setup for any agent: [`AGENT_SETUP_PROMPT.md`](AGENT_SETUP_PROMPT.md). Portable skill: [`skills/kurultai-brain/SKILL.md`](skills/kurultai-brain/SKILL.md).

Loopback ingest (opt-in): set `KURULTAI_INGEST_SECRET`, then `POST /ingest` with the secret header.

## Status

| Milestone | State |
|-----------|-------|
| Foundation · hybrid search · MCP · Brain UI | ✅ Shipped in v0.4.x |
| Connectors (Dayflow, Pond, GitHub FS, inbox) | ✅ |
| MCP HTTP/SSE · export/import · solo `init --docs` | ✅ v0.4.1 |
| Tiered hub · Postgres store · team transport | ✅ Shipped in v0.5.0 |
| Team web app (`web/`) | 🚧 Next.js + Clerk scaffold |

Roadmap issues: [#25](https://github.com/duketopceo/kurultai/issues/25) (developer → solo), [#27](https://github.com/duketopceo/kurultai/issues/27) (team → company). Work queue: [`docs/plans/phase-6-next-work-orders.md`](docs/plans/phase-6-next-work-orders.md).

## Contributing

[`CONTRIBUTING.md`](CONTRIBUTING.md) · [`SECURITY.md`](SECURITY.md) · [`CHANGELOG.md`](CHANGELOG.md)

```bash
cargo build && cargo test --locked
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings
```

Brain UI changes: edit `website/`, run `bash scripts/build-ui.sh`, rebuild the daemon.

## License

MIT — see [`LICENSE`](LICENSE).

**Kurultai** (курултай) — a council or assembly; a system that gathers knowledge from many sources.
