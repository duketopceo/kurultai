---
index: kurultai/v1
folder: website/src/repos
parent: website/src/INDEX.md
updated: 2026-11-07
version: 1
---

# `website/src/repos`

**Does:** Optional Repos floorplan view (folders = rooms, files = tiles); separate from the brain
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`floorplan.ts`](floorplan.ts) | Pure treemap layout, atoms to entries, agent-index INDEX.md parser | — | `website/src/repos/RepoFloorplan.tsx` · `website/src/repos/floorplan.test.ts` | 2026-11-07 | 1 | 2026-11-07 added (plan 2026-11-07-001 U3) |
| [`floorplan.test.ts`](floorplan.test.ts) | node:test: bounds/no-overlap, proportional tiles, entries, parser | `website/src/repos/floorplan.ts` | — | 2026-11-07 | 1 | 2026-11-07 added |
| [`RepoFloorplan.tsx`](RepoFloorplan.tsx) | SVG floorplan view; optional baked manifest at `/ui/repo-manifests/<repo>.json` | `website/src/api.ts` · `website/src/repoLattice.ts` · `website/src/components/RepoBrain.tsx` · `website/src/repos/floorplan.ts` | `website/src/next/App.tsx` | 2026-11-07 | 1 | 2026-11-07 added |

## Recent

- 2026-11-07 — repos floorplan view behind the v2 palette (U3)
