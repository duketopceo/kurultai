---
index: kurultai/v1
folder: src/quality
parent: src/INDEX.md
updated: 2026-08-16
version: 1
---

# `src/quality`

**Does:** Trust lanes, promote, near-dupe
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`gate.rs`](gate.rs) | Trust-lane quality gate (tags → quarantine) | `src/error` · `src/hashutil` · `src/store` · `src/types` | `src/http/ingest.rs` · `src/pipeline/mod.rs` · `src/quality/near_dupe.rs` · `src/quality/promote.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`merge.rs`](merge.rs) | Near-dupe merge | `src/hashutil` · `src/types` | `src/http/ingest.rs` · `src/pipeline/mod.rs` · `src/quality/near_dupe.rs` · `src/quality/promote.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Quality gate module | — | `src/http/ingest.rs` · `src/pipeline/mod.rs` · `src/quality/near_dupe.rs` · `src/quality/promote.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`near_dupe.rs`](near_dupe.rs) | Near-duplicate detection | `src/embed` · `src/error` · `src/hashutil` · `src/quality` · `src/store` | `src/http/ingest.rs` · `src/pipeline/mod.rs` · `src/quality/promote.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`promote.rs`](promote.rs) | CLI/MCP promote quarantine → trusted | `src/error` · `src/quality` · `src/store` · `src/types` · `src/write_policy` | `src/http/ingest.rs` · `src/pipeline/mod.rs` · `src/quality/near_dupe.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

