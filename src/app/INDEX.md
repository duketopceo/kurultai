---
index: kurultai/v1
folder: src/app
parent: src/INDEX.md
updated: 2026-08-29
version: 2
---

# `src/app`

**Does:** CLI application orchestration
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`context.rs`](context.rs) | AppContext: config + store + embedder wiring | `src/config` · `src/connectors` · `src/embed` · `src/environment` · `src/error` · `src/store` | — | 2026-08-29 | 2 | 2026-08-29 hub flag opens PostgresStore · 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | CLI App orchestration (index/search/ask/status) | — | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-29 — `from_config` opens `open_hub_store` when `KURULTAI_FEATURE_HUB=1`
- 2026-08-16 — indexed this folder (v1 seed)

