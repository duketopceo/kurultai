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

Your knowledge lives in many places: notes (AppFlowy/Obsidian), conversations (Pond), code (GitHub), activity logs. Kurultai indexes all of them into one queryable store so you can ask anything and get answers with citations — no matter where the source data lives.

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
| **Connectors** | Trait-based, one per source (AppFlowy, Obsidian, Pond, GitHub, Tech Tracker) | 🚧 Stubs |
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
[sources]
[sources.appflowy]
enabled = true
kind = "appflowy"
poll_interval_secs = 300

[sources.obsidian]
enabled = true
kind = "obsidian"
vault_path = "/Users/you/Documents/Obsidian/Vault"
poll_interval_secs = 60

[storage]
path = "~/.local/share/kurultai/store.db"

[embed]
model = "openai/text-embedding-3-large"
dimension = 3072
```

## Connectors

- **AppFlowy** — Index pages, databases, and AI chats via REST API or MCP
- **Obsidian** — Watch and index local Markdown vault files
- **Pond** — Index agent conversation history (FTS5 + embeddings)
- **Tech Tracker** — Index Dayflow activity and git history
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
- [ ] Obsidian connector ([#3](https://github.com/duketopceo/kurultai/issues/3))
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
