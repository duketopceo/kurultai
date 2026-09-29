---
index: kurultai/v1
folder: ui
parent: INDEX.md
updated: 2026-08-16
version: 1
---

# `ui`

**Does:** Built assets rust-embed serves at GET /ui/
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

_None._

## Skip interiors

- `assets/` — hashed Vite bundles; rebuild with `scripts/build-ui.sh`; do not edit by hand

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`README.md`](README.md) | ui/ — Brain UI source (daemon `GET /ui`) | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`brain.html`](brain.html) | Embedded brain HTML (built) | — | `src/http/mod.rs` · `src/mcp/brain.rs` · `src/query/context.rs` · `src/query/hybrid.rs` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-01 — dropped marketing landing files
- 2026-08-16 — indexed this folder (v1 seed)

