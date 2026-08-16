---
index: kurultai/v1
folder: src/query
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/query`

**Does:** FTS/hybrid search + RRF
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`context.rs`](context.rs) | Ask context assembly | `src/brain` · `src/error` · `src/store` · `src/types` | `src/query/hybrid.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`hybrid.rs`](hybrid.rs) | FTS + vector hybrid retrieval | `src/brain` · `src/embed` · `src/error` · `src/ingest` · `src/query` | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Search query module | `src/embed` · `src/error` · `src/rerank` · `src/store` · `src/synthesize` | `src/query/hybrid.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`rrf.rs`](rrf.rs) | Reciprocal Rank Fusion | `src/types` | `src/query/hybrid.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

