---
index: kurultai/v1
folder: website/src/brain/layout
parent: website/src/brain/INDEX.md
updated: 2026-09-06
version: 2
---

# `website/src/brain/layout`

**Does:** brain FDG + ontology Sugiyama
**Up:** [`website/src/brain/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`createWorker.ts`](createWorker.ts) | Worker factory | — | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`fdg.ts`](fdg.ts) | Brain-shape FDG + hard SDF containment | `website/src/brain/layout/octree.ts` · `website/src/brain/layout/sdf.ts` · `website/src/brain/layout/types.ts` | `website/src/brain/layout/fdg.worker.ts` · `website/src/brain/layout/layout.test.ts` | 2026-09-06 | 2 | 2026-09-07 merged: main's hard-clamp containment (out-of-grid ray bisect, inGrid-gated hullK) supersedes wave-1 hard-project re-land · 2026-09-06 hard-clamp escapers inside hull (out-of-grid ray bisect); gate soft hullK on inGrid · 2026-08-16 indexed (v1 seed) |
| [`grouping.ts`](grouping.ts) | Class-group derivation: instance_of/is_a → ClassGroup[] with purple-family tints | — | `website/src/brain/layout/grouping.test.ts` · `website/src/brain/BrainView.ts` | 2026-09-06 | 1 | 2026-09-06 Astra-generated (U2) |
| [`grouping.test.ts`](grouping.test.ts) | Node tests for group derivation | `website/src/brain/layout/grouping.ts` | — | 2026-09-06 | 1 | 2026-09-06 orchestrator-authored (Astra test truncated; substituted) |
| [`fdg.worker.ts`](fdg.worker.ts) | Barnes–Hut worker + setSdf msg | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/sdf.ts` · `website/src/brain/layout/types.ts` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`layout.test.ts`](layout.test.ts) | Layout unit tests incl. containment invariants | `website/src/brain/layout/mode.ts` · `website/src/brain/layout/octree.ts` · `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/sdf.ts` · `website/src/brain/layout/types.ts` | — | 2026-09-06 | 2 | 2026-09-07 merged union: main's escaper-clamp test + wave-1's no-op interior/null-SDF test · 2026-09-06 containment test cases (re-land) · 2026-08-16 indexed (v1 seed) |
| [`mode.ts`](mode.ts) | LayoutMode brain / ontology | `website/src/brain/layout/../../types.ts` | `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`octree.ts`](octree.ts) | Octree for FDG | — | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`sdf.ts`](sdf.ts) | Baked SDF hull | `website/src/brain/layout/types.ts` | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/fdg.worker.ts` · `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`sugiyama.test.ts`](sugiyama.test.ts) | Sugiyama unit tests | `website/src/brain/layout/sugiyama.ts` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`sugiyama.ts`](sugiyama.ts) | Ontology Sugiyama layers (O2) | — | `website/src/brain/layout/sugiyama.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`types.ts`](types.ts) | Layout types + setSdf msg | — | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/fdg.worker.ts` · `website/src/brain/layout/layout.test.ts` · `website/src/brain/layout/sdf.ts` · `src/brain/mod.rs` | 2026-09-06 | 2 | 2026-09-07 merged: setSdf msg variant stands (main); containment type touch folded in · 2026-09-06 containment fix: `setSdf` worker msg (GLB-vs-setData race) + hard clamp in `tickFdg` (out-of-grid → center-ray bisect); `types.ts` msg variant; regression test · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-07 — merged grouping module rows (U1/U2 of plan 2026-09-05-002) with main's containment fix lineage
- 2026-09-06 — containment fix: `setSdf` worker msg (GLB-vs-setData race) + hard clamp in `tickFdg` (out-of-grid → center-ray bisect); `types.ts` msg variant; regression test
- 2026-09-06 — grouping module + containment re-land rows (U1/U2 of plan 2026-09-05-002)
- 2026-08-16 — indexed this folder (v1 seed)

