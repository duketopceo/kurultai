---
index: kurultai/v1
folder: website/src
parent: website/INDEX.md
updated: 2026-09-04
version: 2
---

# `website/src`

**Does:** Dashboard React/TS source
**Up:** [`website/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

- [`assets/`](assets/INDEX.md) — Static source assets (GLB etc.)
- [`brain/`](brain/INDEX.md) — 3D view
- [`components/`](components/INDEX.md) — Dashboard panels

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`App.tsx`](App.tsx) | Brain dashboard shell (stats + stage + inspector) | `website/src/state` · `website/src/api` · `website/src/components/TopBar` · `website/src/components/BrainStage` · `website/src/components/CommandStrip` | — | 2026-09-04 | 1 | 2026-09-04 token prompt + logout wiring · 2026-08-16 indexed (v1 seed) |
| [`api.ts`](api.ts) | Fetch wrappers for daemon /api/* | `website/src/types` | — | 2026-09-04 | 2 | 2026-09-04 clear token + emit 401 unauthorized event · 2026-08-16 indexed (v1 seed) |
| [`assets.d.ts`](assets.d.ts) | Asset module declarations | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`main.tsx`](main.tsx) | Vite entry; token gate + 401 listener | `website/src/App` · `website/src/components/RepoBrain` | — | 2026-09-04 | 2 | 2026-09-04 listen for kurultai:token-invalid and show prompt · 2026-08-16 indexed (v1 seed) |
| [`repoLattice.test.ts`](repoLattice.test.ts) | Repo lattice unit tests | `website/src/repoLattice.ts` | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`repoLattice.ts`](repoLattice.ts) | Repo lattice helper | — | `website/src/repoLattice.test.ts` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`state.ts`](state.ts) | App state | `website/src/brain/layout/mode` · `website/src/types` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`styles.css`](styles.css) | Dashboard CSS (black/white/purple) | — | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`types.ts`](types.ts) | TS types for atoms/graph/ontology | — | `src/brain/mod.rs` · `src/config/loader.rs` · `src/config/mod.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`version.ts`](version.ts) | UI version string | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-04 — token prompt + 401 auth clear flow; TopBar logout/changeset
- 2026-08-16 — indexed this folder (v1 seed)

