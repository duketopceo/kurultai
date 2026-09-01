---
index: kurultai/v1
folder: src
parent: INDEX.md
updated: 2026-08-29
version: 2
---

# `src`

**Does:** Rust CLI + daemon (main product)
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`app/`](app/INDEX.md) — CLI application orchestration
- [`brain/`](brain/INDEX.md) — Agent read-model + graph doctrine
- [`config/`](config/INDEX.md) — config.toml load/merge
- [`connectors/`](connectors/INDEX.md) — Source ingest adapters
- [`daemon/`](daemon/INDEX.md) — Poll + fs watch
- [`distill/`](distill/INDEX.md) — Index-time distill
- [`embed/`](embed/INDEX.md) — Embeddings (cloud / local / null)
- [`export/`](export/INDEX.md) — Export/import packs
- [`hub/`](hub/INDEX.md) — HUB-4 issued keys + write activity (postgres)
- [`http/`](http/INDEX.md) — Daemon HTTP + Brain UI + MCP SSE
- [`ingest/`](ingest/INDEX.md) — Ingest jobs
- [`mcp/`](mcp/INDEX.md) — MCP stdio + agent init
- [`memory/`](memory/INDEX.md) — Memory / tier helpers
- [`ontology/`](ontology/INDEX.md) — Typed property graph helpers
- [`pipeline/`](pipeline/INDEX.md) — Index pipeline
- [`quality/`](quality/INDEX.md) — Trust lanes, promote, near-dupe
- [`query/`](query/INDEX.md) — FTS/hybrid search + RRF
- [`rerank/`](rerank/INDEX.md) — Optional rerank
- [`security/`](security/INDEX.md) — Paths, redaction, hub keys
- [`store/`](store/INDEX.md) — SQLite kernel + optional Postgres hub store
- [`synthesize/`](synthesize/INDEX.md) — ask / who-knows

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`activity.rs`](activity.rs) | In-memory activity events for Brain UI / status | — | `src/mcp/brain.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`art.rs`](art.rs) | Yurt terminal banner art | — | `src/config/file.rs` · `src/config/loader.rs` | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |
| [`doctor.rs`](doctor.rs) | kurultai doctor PASS/FAIL/WARN diagnostics | `src/config` · `src/embed` · `src/environment` · `src/error` · `src/mcp` | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`environment.rs`](environment.rs) | KURULTAI_ENV paths (dev/staging/prod store locations) | `src/error` | `src/app/context.rs` · `src/config/loader.rs` · `src/doctor.rs` · `src/logging.rs` · `src/types.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`error.rs`](error.rs) | KurultaiError and Result | — | `src/app/context.rs` · `src/config/loader.rs` · `src/config/mod.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`features.rs`](features.rs) | Versioned feature flags (fts, brain_ui, mcp_http, hub) | — | — | 2026-08-29 | 2 | 2026-08-29 hub summary names HUB-3 transport · 2026-08-16 indexed (v1 seed) |
| [`hashutil.rs`](hashutil.rs) | Content hashing for incremental index skip | — | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/pond.rs` · `src/http/auth.rs` · `src/http/ingest.rs` | 2026-07-21 | 1 | 2026-08-16 indexed (v1 seed) |
| [`lib.rs`](lib.rs) | Crate root: module graph and public error/env re-exports | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`logging.rs`](logging.rs) | tracing-subscriber setup | `src/environment` · `src/error` | — | 2026-07-21 | 1 | 2026-08-16 indexed (v1 seed) |
| [`main.rs`](main.rs) | CLI entry: init, index, search, ask, daemon, mcp, export | — | — | 2026-08-31 | 3 | 2026-08-31 `kurultai hub key` / `hub log` · 2026-08-29 daemon PORT · 2026-08-16 indexed |
| [`metrics.rs`](metrics.rs) | Prometheus text for GET /api/metrics | — | `src/http/mod.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`project.rs`](project.rs) | project_id namespacing for shared-store sessions (#184) | — | `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`types.rs`](types.rs) | KnowledgeAtom, Config, search/ask types, visibility scope | `src/environment` | `src/brain/mod.rs` · `src/config/loader.rs` · `src/config/mod.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`write_policy.rs`](write_policy.rs) | Write provenance + SharedClosed quarantine containment | — | `src/mcp/server.rs` · `src/quality/promote.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-31 — HUB-4: ensure hub_activity table migrates alongside hub_api_keys
- 2026-08-31 — HUB-4: `src/hub/` issued keys, AE5 filter, write activity log
- 2026-08-29 — HUB-3: listen start-fail, hub Postgres when flag on, `PORT` env
- 2026-08-16 — indexed this folder (v1 seed)

