---
index: kurultai/v1
folder: website
parent: INDEX.md
updated: 2026-09-04
version: 2
---

# `website`

**Does:** Brain UI source (Vite) — do not add a second dashboard
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`src/`](src/INDEX.md) — Dashboard React/TS source

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`README.md`](README.md) | How to run Vite preview + build-ui.sh | — | — | 2026-09-04 | 2 | 2026-09-04 clarify source is website/src and scripts/build-ui.sh prunes stale assets · 2026-08-16 indexed (v1 seed) |
| [`brain.html`](brain.html) | Brain page HTML entry | — | `src/http/mod.rs` · `src/mcp/brain.rs` · `src/query/context.rs` · `src/query/hybrid.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`package-lock.json`](package-lock.json) | npm lockfile | — | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`package.json`](package.json) | Brain UI npm package (Vite) | — | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`tsconfig.json`](tsconfig.json) | TS config | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`vite.config.ts`](vite.config.ts) | Vite config; copies build into ui/ | — | — | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-04 — token gate auth flow; README updated for build-ui.sh
- 2026-08-16 — indexed this folder (v1 seed)

