---
index: kurultai/v1
folder: src/memory
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/memory`

**Does:** Memory / tier helpers
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | Memory helpers | — | `src/store/mod.rs` · `src/store/postgres.rs` | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |
| [`tier.rs`](tier.rs) | Corpus / memory tier helpers | `src/types` | `src/store/mod.rs` · `src/store/postgres.rs` | 2026-09-23 | 3 | 2026-09-23 `MemoryTier` derives `Hash` — graph-payload cache keys (#324) · 2026-09-15 `TierRule` + `classify_atom` — declarative sequester caps (#325) |

## Recent

- 2026-09-23 — `tier.rs`: `MemoryTier` derives `Hash` for `GraphKey` cache keys (#324)
- 2026-09-15 — `tier.rs`: `TierPolicy.rules` + `TierRule` + `classify_atom`; `classify` takes `&TierPolicy` (#325)
- 2026-08-16 — indexed this folder (v1 seed)

