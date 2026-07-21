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

## Design doctrine: speed + token budget

**North star:** SQL agent-optimized brain with pristine structured atoms — not a markdown dump, not full-file RAG.

| Principle | What it means |
|-----------|----------------|
| **Index-time heavy** | Embed, distill, dedupe when ingesting — not when the agent asks |
| **Read-time light** | `search`/`cite` return `AgentAtomView` excerpts (~400 chars), never full `content` by default |
| **Write-time minimal** | `remember` accepts summary + tags only — no raw chat blobs |
| **Structuring rules** | Fixed schema (`title`, `summary`, `question`, `resolution`, `tags`, provenance) — stable for NN export |
| **Bleeding-edge speed** | FTS + vector in SQLite, content-hash skip re-embed, query cache (Phase 2+) |

If we nail **structured SQL + MCP views + structuring rules**, that is enough — we do not need agents reading vaults or SQL directly.

Tracked in work order [#27](https://github.com/duketopceo/kurultai/issues/27) and [#37](https://github.com/duketopceo/kurultai/issues/37).

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
| **API keys** | Optional (zero-vector fallback) | Required for index | Required + audit |
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
Upstream repos (depend / inspire / integrate): **[#40](https://github.com/duketopceo/kurultai/issues/40)** · [docs/upstream-inspiration.md](docs/upstream-inspiration.md)  
Phase 1 CE plan: [docs/plans/phase-1-work-orders.md](docs/plans/phase-1-work-orders.md)

| Phase | Audience unlocked | Milestone | Work order (in sequence) | Upstream (pull / inspire) |
|-------|-------------------|-----------|--------------------------|---------------------------|
| **1** Foundation | Developer | [Phase 1](https://github.com/duketopceo/kurultai/milestone/1) | ✅ [#18](https://github.com/duketopceo/kurultai/issues/18) framework → [#1](https://github.com/duketopceo/kurultai/issues/1) storage → [#2](https://github.com/duketopceo/kurultai/issues/2) embed → [#31](https://github.com/duketopceo/kurultai/issues/31)/[#4](https://github.com/duketopceo/kurultai/issues/4) connectors → [#5](https://github.com/duketopceo/kurultai/issues/5) CLI → [#11](https://github.com/duketopceo/kurultai/issues/11) MCP/install | [sqlite-vec](https://github.com/asg017/sqlite-vec), [layer0](https://github.com/amajorai/layer0), [kb-mcp](https://github.com/alphabet-h/kb-mcp), [mdvault](https://github.com/sderosiaux/mdvault), [Stratum](https://github.com/DakodaStemen/Stratum), [smithery](https://github.com/smithery-ai/cli) |
| **2** Search | Developer | [Phase 2](https://github.com/duketopceo/kurultai/milestone/2) | [#6](https://github.com/duketopceo/kurultai/issues/6) FTS + vector + RRF + rerank | [kb-mcp](https://github.com/alphabet-h/kb-mcp), [Stratum](https://github.com/DakodaStemen/Stratum), [sqmd](https://github.com/itkoren/sqmd), [Cerebras KB](https://mer.vin/2026/07/how-cerebras-built-a-15k-query-day-internal-knowledge-base/) |
| **3** Synthesis | Developer ✓ | [Phase 3](https://github.com/duketopceo/kurultai/milestone/3) | [#7](https://github.com/duketopceo/kurultai/issues/7) synthesis + MCP + daemon + agent capture | [gbrain](https://github.com/imphillip/gbrain-openclaw), [agent-knowledge](https://github.com/keshrath/agent-knowledge), [recall](https://github.com/pratikgajjar/recall), [atomic](https://github.com/yun-lim/atomic) |
| **4** Expansion | Solo ✓ | [Phase 4](https://github.com/duketopceo/kurultai/milestone/4) | [#8](https://github.com/duketopceo/kurultai/issues/8) GitHub/Pond → [#21](https://github.com/duketopceo/kurultai/issues/21) Dayflow | [cocoindex](https://github.com/cocoindex-io/cocoindex), [codebase-graph](https://github.com/Phoenixrr2113/codebase-graph), [Dayflow](https://github.com/JerryZLiu/Dayflow) |
| **5** Production | Team | [Phase 5](https://github.com/duketopceo/kurultai/milestone/5) | [#9](https://github.com/duketopceo/kurultai/issues/9) perf + shared daemon → [#20](https://github.com/duketopceo/kurultai/issues/20) self-hosted CI | [layer0](https://github.com/amajorai/layer0), [engram-mcp](https://github.com/edg-l/engram-mcp) |
| **6** Launch | Company | [Phase 6](https://github.com/duketopceo/kurultai/milestone/6) | [#10](https://github.com/duketopceo/kurultai/issues/10) release → [#22](https://github.com/duketopceo/kurultai/issues/22) yurt art | — |

**Cross-cutting (every phase):** [#37](https://github.com/duketopceo/kurultai/issues/37) speed + token doctrine · [#40](https://github.com/duketopceo/kurultai/issues/40) upstream matrix · [#23](https://github.com/duketopceo/kurultai/issues/23) testing & CI gates — coverage rises 50% → 60% → 75% → 80%.

## Roadmap checklist

- [x] Framework foundation ([#18](https://github.com/duketopceo/kurultai/issues/18) / PR [#19](https://github.com/duketopceo/kurultai/pull/19))
- [x] Storage ([#1](https://github.com/duketopceo/kurultai/issues/1)) — SqliteVecStore FTS + vec0
- [x] Embeddings ([#2](https://github.com/duketopceo/kurultai/issues/2)) — OpenRouter + NullEmbedder FTS-first
- [x] Markdown / filesystem connector ([#31](https://github.com/duketopceo/kurultai/issues/31), was #3)
- [ ] AppFlowy connector ([#4](https://github.com/duketopceo/kurultai/issues/4))
- [x] CLI wired ([#5](https://github.com/duketopceo/kurultai/issues/5)) — index/status/search via brain views
- [x] MCP + installer ([#11](https://github.com/duketopceo/kurultai/issues/11)) — stdio `search`/`cite`/`remember` + `init --agent cursor`
- [ ] Search & retrieval ([#6](https://github.com/duketopceo/kurultai/issues/6))
- [ ] Synthesis & interface ([#7](https://github.com/duketopceo/kurultai/issues/7))
- [ ] Expansion connectors ([#8](https://github.com/duketopceo/kurultai/issues/8), [#21](https://github.com/duketopceo/kurultai/issues/21))
- [ ] Production readiness ([#9](https://github.com/duketopceo/kurultai/issues/9), [#20](https://github.com/duketopceo/kurultai/issues/20))
- [ ] Open source launch ([#10](https://github.com/duketopceo/kurultai/issues/10), [#22](https://github.com/duketopceo/kurultai/issues/22))

## Upstream inspiration

Per-work-order context for external repos we **depend on**, **port patterns from**, or **integrate as connectors** — full detail in [docs/upstream-inspiration.md](docs/upstream-inspiration.md) ([#40](https://github.com/duketopceo/kurultai/issues/40)).

**Study first:** [kb-mcp](https://github.com/alphabet-h/kb-mcp) · [layer0](https://github.com/amajorai/layer0) · [cocoindex](https://github.com/cocoindex-io/cocoindex) · [Stratum](https://github.com/DakodaStemen/Stratum) · [recall](https://github.com/pratikgajjar/recall)

**Avoid fork:** [basic-memory](https://github.com/basicmachines-co/basic-memory) (AGPL, markdown-as-truth) · [Graphiti](https://github.com/getzep/graphiti) (Neo4j stack)

## Quality

CI runs on every PR: `cargo fmt`, `clippy -D warnings`, `cargo test --locked`, `cargo audit`, and a macOS smoke build. Coverage and stricter gates expand by milestone ([#23](https://github.com/duketopceo/kurultai/issues/23)).

## License

MIT

## Name

Kurultai (курултай) — a council or assembly in Turkic/Mongolian tradition. A gathering of voices to reach consensus. Fitting for a system that assembles knowledge from many sources.
