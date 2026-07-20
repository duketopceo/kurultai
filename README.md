# Kurultai

**Assemble what you know, from wherever it lives.**

Kurultai is a unified knowledge retrieval layer — like having a single brain that indexes everything across your tools without moving your data.

Inspired by Cerebras's internal knowledge base architecture: one embeddings table, per-source connectors, MCP tools as primitives.

## Why

Your knowledge lives in many places: notes (AppFlowy/Obsidian), conversations (Pond), code (GitHub), activity logs. Kurultai indexes all of them into one queryable store so you can ask anything and get answers with citations — no matter where the source data lives.

## Architecture (Phase 1)

```
Filesystem / Obsidian → raw atoms → SQLite (FTS5 + vec0)
                                      ↓
CLI / MCP → FTS-first search (+ vectors when OpenRouter keyed) → citations
```

### Components

| Layer | Technology | Status |
|-------|-----------|--------|
| **Connectors** | Filesystem + Obsidian alias; AppFlowy honest-unimplemented | ✅ Phase 1 |
| **Distillation** | LLM extractors (question, summary, resolution) | 📋 Later |
| **Embeddings** | OpenRouter when keyed; FTS-only without key; zero-vector refused | ✅ Phase 1 |
| **Vector Store** | SQLite + FTS5 + sqlite-vec (`store_meta` embed contract) | ✅ Phase 1 |
| **Search** | FTS-first + basic vector fuse | ✅ Phase 1 |
| **Synthesis** | Light answer from top hits (full planner later) | 🚧 Minimal |
| **Interface** | CLI + MCP stdio (`search`, `read_atom`, `status`, `reindex`) | ✅ Phase 1 |

## Quick Start

```bash
cargo build --release

# Write config for this env (dev|staging|prod)
kurultai init --vault ~/Documents/Notes
# or: KURULTAI_ENV=staging kurultai init

kurultai index --full
kurultai search "database migration" --limit 10
kurultai status
kurultai doctor

# MCP for agents
kurultai install --client cursor
kurultai mcp
```

FTS works without an API key. Set `OPENROUTER_API_KEY` to enable embeddings.

## Configuration

`kurultai init` writes `~/.config/kurultai/{env}/config.toml` (Rust `Config` shape):

```toml
env = "dev"
storage_path = ""  # empty → ~/.local/share/kurultai/{env}/store.db
embed_model = "openai/text-embedding-3-small"
embed_dim = 1536
openrouter_api_key_env = "OPENROUTER_API_KEY"
poll_interval_secs = 60

[[sources]]
name = "notes"
kind = "filesystem"   # or "obsidian" with extra.vault_path
enabled = true
poll_interval_secs = 60

[sources.extra]
path = "~/Documents/Notes"
```

## Connectors

- **Filesystem** — Index a directory of `.md` files (content-hash IDs; unchanged skip re-embed)
- **Obsidian** — Alias: `vault_path` → filesystem root
- **AppFlowy** — Not implemented in Phase 1 (status reports honestly)
- **Pond / Tech Tracker / GitHub** — Roadmap

**Orphan policy:** `index --full` removes atoms whose `source_id` is no longer present on disk.

## Post-train export contract

Hot SQLite atoms keep these fields stable for a future cold tier / labeling export (no object storage in Phase 1):

| Field | Purpose |
|-------|---------|
| `id` | Content-addressed stable ID |
| `source` / `source_id` / `source_uri` | Provenance join keys |
| `title` / `content` / `summary` | Text corpus |
| `tags` | Durable labels (JSON array) |
| `provenance` | Free-form / JSON provenance |
| `source_updated_at` / `indexed_at` | Temporal ordering |
| `content_hash` | Change detection |
| `metadata` | Source-specific extras |
| `embedding` (optional) | Vector when keyed |

See `docs/plans/2026-07-19-001-feat-phase-1-foundation-plan.md`.

## Roadmap

- [x] Filesystem + Obsidian connectors
- [x] OpenRouter embeddings + FTS-first / zero-vector guard
- [x] SQLite + sqlite-vec + FTS5
- [x] MCP tool interface + install/doctor
- [x] Golden-path GitHub-hosted CI
- [ ] AppFlowy connector
- [ ] RRF polish + LLM rerank / synthesis
- [ ] Object storage cold tier (#34)
- [ ] GlitchTip projects (#35)
- [ ] Self-hosted runners (#20 — Docker ephemeral first)
- [ ] HTTP daemon productization
- [ ] Local embedding model (llama.cpp)

## License

MIT

## Name

Kurultai (курултай) — a council or assembly in Turkic/Mongolian tradition. A gathering of voices to reach consensus. Fitting for a system that assembles knowledge from many sources.
