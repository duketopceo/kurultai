---
index: kurultai/v1
folder: ui
parent: INDEX.md
updated: 2026-09-05
version: 3
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
| [`README.md`](README.md) | Built UI surface notes | — | — | 2026-09-04 | 2 | 2026-09-04 clarify source is website/ and assets are built · 2026-08-16 indexed (v1 seed) |
| [`brain.html`](brain.html) | Embedded brain HTML (built) | `website/` source · `website/brain.html` | `src/http/mod.rs` · `src/mcp/brain.rs` · `src/query/context.rs` · `src/query/hybrid.rs` | 2026-09-05 | 3 | 2026-09-05 rebuild with tiered graph fetch · 2026-09-04 rebuild with token gate · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-05 — rebuild with tiered graph fetch (api.ts GraphQuery + load-tier limits)
- 2026-09-04 — rebuild with token gate; clean legacy index.* and unused images from build
- 2026-08-16 — indexed this folder (v1 seed)
