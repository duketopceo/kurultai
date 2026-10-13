---
index: kurultai/v1
folder: src/rerank
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/rerank`

**Does:** Optional rerank
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | Reranker trait + NullReranker + `LateInteractionReranker` (MaxSim over `atoms_multivec`) | `src/error` · `src/security` · `src/embed` · `src/store` | `src/query/mod.rs` · `src/app/context.rs` | 2026-10-07 | 2 | 2026-10-07 late-interaction reranker (#424) · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-07 — `mod.rs`: `LateInteractionReranker` — embed_query once, `get_multivecs` per candidate, MaxSim order; `apply_rerank_order` keeps non-multivec tail (#424)

- 2026-08-16 — indexed this folder (v1 seed)

