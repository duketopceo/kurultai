---
index: kurultai/v1
folder: website/src/brain/layout
parent: website/src/brain/INDEX.md
updated: 2026-08-16
version: 1
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
| [`fdg.ts`](fdg.ts) | Brain-shape FDG | `website/src/brain/layout/octree.ts` · `website/src/brain/layout/sdf.ts` · `website/src/brain/layout/types.ts` | `website/src/brain/layout/fdg.worker.ts` · `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`fdg.worker.ts`](fdg.worker.ts) | Barnes–Hut worker | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/sdf.ts` · `website/src/brain/layout/types.ts` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`layout.test.ts`](layout.test.ts) | Layout unit tests | `website/src/brain/layout/mode.ts` · `website/src/brain/layout/octree.ts` · `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/sdf.ts` · `website/src/brain/layout/types.ts` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mode.ts`](mode.ts) | LayoutMode brain / ontology | `website/src/brain/layout/../../types.ts` | `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`octree.ts`](octree.ts) | Octree for FDG | — | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`sdf.ts`](sdf.ts) | Baked SDF hull | `website/src/brain/layout/types.ts` | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/fdg.worker.ts` · `website/src/brain/layout/layout.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`sugiyama.test.ts`](sugiyama.test.ts) | Sugiyama unit tests | `website/src/brain/layout/sugiyama.ts` | — | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`sugiyama.ts`](sugiyama.ts) | Ontology Sugiyama layers (O2) | — | `website/src/brain/layout/sugiyama.test.ts` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |
| [`types.ts`](types.ts) | Layout types | — | `website/src/brain/layout/fdg.ts` · `website/src/brain/layout/fdg.worker.ts` · `website/src/brain/layout/layout.test.ts` · `website/src/brain/layout/sdf.ts` · `src/brain/mod.rs` | 2026-08-16 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-16 — indexed this folder (v1 seed)

