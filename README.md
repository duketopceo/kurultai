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

## Roadmap

- [ ] Working AppFlowy connector
- [ ] Working Obsidian connector (Markdown vault reader)
- [ ] OpenRouter embeddings integration
- [ ] SQLite + sqlite-vec vector store
- [ ] FTS5 full-text search
- [ ] RRF fusion + reranking
- [ ] LLM synthesis with citations
- [ ] MCP tool interface
- [ ] HTTP daemon for external querying
- [ ] GitHub connector
- [ ] Tech Tracker connector
- [ ] Local embedding model support (llama.cpp)
- [ ] Yurt terminal art for CLI (`kurultai status`, banners) — see [#22](https://github.com/duketopceo/kurultai/issues/22)
- [ ] Testing & CI gates expanding per phase — see [#23](https://github.com/duketopceo/kurultai/issues/23)

## Quality

CI runs on every PR: `cargo fmt`, `clippy -D warnings`, `cargo test --locked`, `cargo audit`, and a macOS smoke build. Coverage and stricter gates expand by milestone ([#23](https://github.com/duketopceo/kurultai/issues/23)).

## License

MIT

## Name

Kurultai (курултай) — a council or assembly in Turkic/Mongolian tradition. A gathering of voices to reach consensus. Fitting for a system that assembles knowledge from many sources.
