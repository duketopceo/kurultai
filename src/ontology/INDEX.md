---
index: kurultai/v1
folder: src/ontology
parent: src/INDEX.md
updated: 2026-09-11
version: 4
---

# `src/ontology`

**Does:** Typed property graph helpers
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | Entity/Link helpers + O3 proposal queue (submit/decide) | `src/error` · `src/store` · `src/types` | `src/mcp/server.rs` · `src/http/proposals.rs` | 2026-09-11 | 4 | 2026-09-11 O3 `submit_proposal`/`decide_proposal` (#118) · 2026-09-06 sync schema version assertion to v14 · 2026-09-04 sync schema version assertion to v12 · 2026-08-14 indexed (v1 seed) |

## Recent

- 2026-09-11 — O3 proposal queue: `submit_proposal` (validate + dedupe, no mutation) / `decide_proposal` (approve applies entity/link, reject no-ops); kinds promote_atom/new_link/new_entity (#118)
- 2026-09-06 — sync schema version assertion to v14
- 2026-09-04 — sync schema version assertion to v12
- 2026-08-16 — indexed this folder (v1 seed)
