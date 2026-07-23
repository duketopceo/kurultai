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

# Kurultai

**Assemble what you know, from wherever it lives.**

Kurultai is a unified knowledge retrieval layer — like having a single brain that indexes everything across your tools without moving your data.

Inspired by Cerebras's internal knowledge base architecture: one embeddings table, per-source connectors, MCP tools as primitives.

## Why

Your knowledge lives in many places: notes (markdown folders), conversations (agents), code (GitHub), activity logs (Dayflow). Kurultai indexes all of them into one queryable store so you can ask anything and get answers with citations — no matter where the source data lives.

## The brain vs your files

| | **Your files** (`.md` folders, agent logs, etc.) | **Kurultai brain** (`store.db`) |
|---|---------------------------------------------------|----------------------------------|
| Format | `.md`, JSONL, SQLite… per tool | **SQLite + vector index** |
| You edit here | ✅ Notes, code, chats | ❌ Index only |
| Agent access | Slow, high tokens (read whole files) | **Fast, low tokens** (excerpts + citations) |

Markdown vaults (including Obsidian folders) are **ingest sources** — Kurultai reads `.md` from disk. It does not integrate with the Obsidian desktop app.

## Agent interface: read & write (MCP)

Agents interact with the brain through two operations — exposed via **MCP** (stdio) and HTTP daemon (Phase 3):

| Operation | MCP tools | What moves | Token budget |
|-----------|-----------|------------|--------------|
| **Read** | `search`, `cite`, `ask` | Excerpts + citations out | Minimal — never full vaults |
| **Write** | `remember` | Distilled facts in | Minimal — summary/tags, not raw chat |

```
Agent ──read──► search/cite/ask ──► SQLite brain ──► ranked excerpts
Agent ─write──► remember ──► distilled KnowledgeAtom ──► SQLite brain
```

MCP is an agent-ready API: structured tools instead of dumping files into context. See `src/mcp/` for the contract (#7).

## Who we build for (in order)

We ship **developer → solo → team → company**. Each layer builds on the last without rework ([#25](https://github.com/duketopceo/kurultai/issues/25)).

| Audience | Phases | What they get |
|----------|--------|---------------|
| **Developer** | 1–3 | CLI + MCP, local config, agent transcript indexing |
| **Solo** | 1–4 | One-command install, on-prem data, Dayflow + notes unified |
| **Team** | 4–5 | Shared daemon, per-user capture policies, internal network |
| **Company** | 5–6+ | Multi-tenant, RBAC, audit, enterprise connectors, VPC deploy |

> Rule: never build company-wide features before developer + solo paths work.

## Architecture

```
Source Connectors → LLM Distillation → Embeddings → Vector Store
                                                          ↓
Question → Embed → Vector Search + FTS → RRF Fusion → Rerank → Synthesize → Answer + Citations
```

### Components

| Layer | Technology | Status |
|-------|-----------|--------|
| **Connectors** | Trait-based, one per source (markdown, AppFlowy, agents, GitHub, Dayflow) | 🚧 Stubs |
| **Distillation** | LLM extractors (question, summary, resolution, tags) per source | 📋 Planned |
| **Embeddings** | OpenRouter API (initial), local model (future) | 🚧 Stub |
| **Vector Store** | SQLite + sqlite-vec | 🚧 Stub |
| **Search** | Vector similarity + FTS5 + RRF fusion + LLM rerank | 📋 Planned |
| **Synthesis** | Planner → Executor → Answer with citations | 📋 Planned |
| **Interface** | CLI + MCP tools + HTTP daemon | 🚧 CLI stub |

## Quick Start

```bash
# Build
cargo build --release

# Index your sources
kurultai index

# Ask a question
kurultai ask "what deployments are we running?"

# Search
kurultai search "database migration" --limit 10

# Check status
kurultai status

# Run daemon
kurultai daemon --port 8421
```

## Configuration

Create `~/.config/kurultai/config.toml`:

```toml
environment = "dev"   # dev | staging | prod

[sources]
[sources.appflowy]
enabled = true
kind = "appflowy"
poll_interval_secs = 300

[sources.notes]
enabled = true
kind = "markdown"
root_path = "/Users/you/Documents/Obsidian/Vault"  # any .md folder — Obsidian app not required
poll_interval_secs = 60

[storage]
# Optional — defaults per environment (see below)
# path = "~/.local/share/kurultai/dev/store.db"

[embed]
model = "openai/text-embedding-3-large"
dimension = 3072
```

Override via CLI or env: `kurultai --env staging status` or `KURULTAI_ENV=prod kurultai daemon`.

## Environments (dev · staging · prod)

| | **Dev** | **Staging** | **Prod** |
|---|---------|-------------|----------|
| **Who** | Developer laptop | Team pre-prod | Company deployment |
| **Audience** | Developer | Team | Enterprise |
| **Storage** | `~/.local/share/kurultai/dev/store.db` | `.../staging/store.db` | `.../store.db` |
| **Logging** | `kurultai=debug` | `info,warn` | `warn,error` |
| **API keys** | Optional (FTS-only; no zero-vector fake embeds) | Required for index | Required + audit |
| **CI branch** | PR / feature branches | `staging` branch | `main` branch |
| **Phase** | 1–3 | 4–5 | 5–6 |

**GitHub Actions:** PRs run `ci.yml` (dev). Push to `staging` → deploy workflow (staging environment). Push to `main` → production environment. Configure approval gates in GitHub → Settings → Environments.

Track full deployment plan in [#27](https://github.com/duketopceo/kurultai/issues/27).

## Connectors

- **Markdown** — Index any directory of `.md` files (`root_path`). Works with Obsidian vaults, git wikis, plain folders — no desktop app integration
- **AppFlowy** — Index pages, databases, and AI chats via REST API or MCP
- **Agents** — Index Cursor, Codex, Claude Code conversation history (Phase 3)
- **Dayflow** — Mac activity journal (Phase 4)
- **GitHub** — Index code repositories via file system + CodeGraph

Each connector implements the `Connector` trait:

```rust
#[async_trait]
pub trait Connector: Send + Sync {
    fn name(&self) -> &str;
    async fn init(&mut self, config: &SourceConfig) -> Result<()>;
    async fn poll(&self) -> Result<Vec<KnowledgeAtom>>;
    async fn full_sync(&self) -> Result<Vec<KnowledgeAtom>>;
}
```

## Phases & work orders

Master plan: **[#27 — Work Order: Master phase plan](https://github.com/duketopceo/kurultai/issues/27)**  
Audience strategy: **[#25 — Developer → Solo → Team → Company](https://github.com/duketopceo/kurultai/issues/25)**

| Phase | Audience unlocked | Milestone | Work order (in sequence) |
|-------|-------------------|-----------|--------------------------|
| **1** Foundation | Developer | [Phase 1](https://github.com/duketopceo/kurultai/milestone/1) | ✅ [#18](https://github.com/duketopceo/kurultai/issues/18) framework → [#1](https://github.com/duketopceo/kurultai/issues/1) storage → [#2](https://github.com/duketopceo/kurultai/issues/2) embed → [#3](https://github.com/duketopceo/kurultai/issues/3)/[#4](https://github.com/duketopceo/kurultai/issues/4) connectors → [#5](https://github.com/duketopceo/kurultai/issues/5) CLI → [#11](https://github.com/duketopceo/kurultai/issues/11) MCP/install |
| **2** Search | Developer | [Phase 2](https://github.com/duketopceo/kurultai/milestone/2) | [#6](https://github.com/duketopceo/kurultai/issues/6) FTS + vector + RRF + rerank |
| **3** Synthesis | Developer ✓ | [Phase 3](https://github.com/duketopceo/kurultai/milestone/3) | [#7](https://github.com/duketopceo/kurultai/issues/7) synthesis + MCP + daemon + agent capture |
| **4** Expansion | Solo ✓ | [Phase 4](https://github.com/duketopceo/kurultai/milestone/4) | [#8](https://github.com/duketopceo/kurultai/issues/8) GitHub/Pond → [#21](https://github.com/duketopceo/kurultai/issues/21) Dayflow |
| **5** Production | Team | [Phase 5](https://github.com/duketopceo/kurultai/milestone/5) | [#9](https://github.com/duketopceo/kurultai/issues/9) perf + shared daemon → [#20](https://github.com/duketopceo/kurultai/issues/20) self-hosted CI |
| **6** Launch | Company | [Phase 6](https://github.com/duketopceo/kurultai/milestone/6) | [#10](https://github.com/duketopceo/kurultai/issues/10) release → [#22](https://github.com/duketopceo/kurultai/issues/22) yurt art |

**Cross-cutting (every phase):** [#23](https://github.com/duketopceo/kurultai/issues/23) testing & CI gates — coverage rises 50% → 60% → 75% → 80%.

## Roadmap checklist

- [x] Framework foundation ([#18](https://github.com/duketopceo/kurultai/issues/18) / PR [#19](https://github.com/duketopceo/kurultai/pull/19))
- [ ] Storage ([#1](https://github.com/duketopceo/kurultai/issues/1))
- [ ] Embeddings ([#2](https://github.com/duketopceo/kurultai/issues/2))
- [ ] Markdown / filesystem connector ([#31](https://github.com/duketopceo/kurultai/issues/31), was #3)
- [ ] AppFlowy connector ([#4](https://github.com/duketopceo/kurultai/issues/4))
- [ ] CLI wired ([#5](https://github.com/duketopceo/kurultai/issues/5))
- [ ] MCP + installer ([#11](https://github.com/duketopceo/kurultai/issues/11))
- [ ] Search & retrieval ([#6](https://github.com/duketopceo/kurultai/issues/6))
- [ ] Synthesis & interface ([#7](https://github.com/duketopceo/kurultai/issues/7))
- [ ] Expansion connectors ([#8](https://github.com/duketopceo/kurultai/issues/8), [#21](https://github.com/duketopceo/kurultai/issues/21))
- [ ] Production readiness ([#9](https://github.com/duketopceo/kurultai/issues/9), [#20](https://github.com/duketopceo/kurultai/issues/20))
- [ ] Open source launch ([#10](https://github.com/duketopceo/kurultai/issues/10), [#22](https://github.com/duketopceo/kurultai/issues/22))

## Quality

CI runs on every PR: `cargo fmt`, `clippy -D warnings`, `cargo test --locked`, `cargo audit`, and a macOS smoke build. Coverage and stricter gates expand by milestone ([#23](https://github.com/duketopceo/kurultai/issues/23)).

## License

MIT

## Name

Kurultai (курултай) — a council or assembly in Turkic/Mongolian tradition. A gathering of voices to reach consensus. Fitting for a system that assembles knowledge from many sources.
