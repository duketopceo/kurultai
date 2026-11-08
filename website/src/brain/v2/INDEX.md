---
index: kurultai/v1
folder: website/src/brain/v2
parent: website/src/brain/INDEX.md
updated: 2026-11-07
version: 3
---

# `website/src/brain/v2`

**Does:** Brain v2: wireframe cortex, tag-region parcellation, single-draw atom cloud (opt in `?brain=v2`)
**Up:** [`website/src/brain/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`BrainV2.ts`](BrainV2.ts) | Three.js scene: baked-cortex wire shell + fresnel, per-region fill, electric single-draw atom cloud, ambient + highlight synapses, collision-free top-N labels, LOD, orbit, fly-to | `three` · `website/src/brain/v2/regions.ts` · `website/src/brain/v2/cortex.ts` · `website/src/brain/v2/path.ts` | `website/src/components/BrainStage.tsx` | 2026-11-08 | 2 | 2026-11-08 real cortex wire (brain.glb) + lobe regions + shimmer/spike shaders + label collision · 2026-11-07 added (plan 2026-11-07-001 U1) |
| [`regions.ts`](regions.ts) | Pure tag-region parcellation onto cortex lobes, deterministic voxel placement, fill curve, LOD tiers | `website/src/brain/v2/cortex.ts` | `website/src/brain/v2/BrainV2.ts` · `website/src/brain/v2/regions.test.ts` | 2026-11-08 | 2 | 2026-11-08 lobes replace ellipsoid Voronoi; log fill · 2026-11-07 added |
| [`cortex.ts`](cortex.ts) | Decodes baked cortex: low-poly mesh, interior voxel mask, lobe seeds + per-lobe voxels | `website/src/brain/v2/cortexData.ts` | `website/src/brain/v2/regions.ts` · `website/src/brain/v2/BrainV2.ts` | 2026-11-08 | 1 | 2026-11-08 added |
| [`cortexData.ts`](cortexData.ts) | GENERATED (scripts/bake-cortex.mjs): ~2k-tri decimated brain.glb, 40^3 mask, 24 k-means lobe seeds | — | `website/src/brain/v2/cortex.ts` | 2026-11-08 | 1 | 2026-11-08 added |
| [`regions.test.ts`](regions.test.ts) | node:test: hull containment, determinism, region cap/fill, LOD, path, links | `website/src/brain/v2/regions.ts` · `website/src/brain/v2/path.ts` · `website/src/brain/v2/links.ts` | — | 2026-11-08 | 2 | 2026-11-08 lobe containment + fill monotonic tests · 2026-11-07 added |
| [`path.ts`](path.ts) | Client-side BFS shortest path over loaded edges (stand-in for a traverse API) | `website/src/types` | `website/src/brain/v2/BrainV2.ts` | 2026-11-07 | 1 | 2026-11-07 added |
| [`links.ts`](links.ts) | Shared-tag synapse derivation (moved from BrainStage) | `website/src/brain/linkSelect.ts` | `website/src/components/BrainStage.tsx` | 2026-11-07 | 1 | 2026-11-07 added |
| [`flag.ts`](flag.ts) | v2 default; `?brain=v1` selects legacy | — | `website/src/components/BrainStage.tsx` | 2026-11-07 | 1 | 2026-11-07 added |
| [`flag.test.ts`](flag.test.ts) | node:test for the selector | `website/src/brain/v2/flag.ts` | — | 2026-11-07 | 1 | 2026-11-07 added |
| [`mode32.ts`](mode32.ts) | 32-bit mode state: `?mode=32bit`, localStorage (try/catch), `useMode32` hook mirrors to `<html data-mode>` | `react` | `website/src/next/App.tsx` | 2026-11-08 | 1 | 2026-11-08 added |
| [`mode32.test.ts`](mode32.test.ts) | node:test: URL param, throwing storage | `website/src/brain/v2/mode32.ts` | — | 2026-11-08 | 1 | 2026-11-08 added |
| [`palette.ts`](palette.ts) | Command-palette model: surfaces + filter (prefix then substring) | — | `website/src/next/V2Shell.tsx` | 2026-11-07 | 1 | 2026-11-07 added |
| [`why.ts`](why.ts) | Path ids to a why-connected chain (shared tags per hop) | `website/src/types` | `website/src/next/App.tsx` | 2026-11-07 | 1 | 2026-11-07 added |
| [`palette.test.ts`](palette.test.ts) | node:test for palette filter + chain | `website/src/brain/v2/palette.ts` · `website/src/brain/v2/why.ts` | — | 2026-11-07 | 1 | 2026-11-07 added |
| [`whatsNew.ts`](whatsNew.ts) | Last-visit marker (localStorage, try/catch) + new/changed atom diff | — | `website/src/next/App.tsx` | 2026-11-07 | 1 | 2026-11-07 added (plan 2026-11-07-001 U4) |
| [`whatsNew.test.ts`](whatsNew.test.ts) | node:test: first visit, throwing storage, new sources | `website/src/brain/v2/whatsNew.ts` | — | 2026-11-07 | 1 | 2026-11-07 added |

## Recent

- 2026-11-08 — 32-bit mode: `BrainV2.setPixelMode` (low-res dithered 256-colour, vertex snap, stepped time) + `mode32.ts`; palette gains view/32-bit commands
- 2026-11-08 — visual pass: wire from the real cortex (brain.glb decimated), tag regions = k-means lobes, electric atoms/synapses, collision-free labels
- 2026-11-07 — v2 is the default renderer (`?brain=v1` keeps v1); what's-new badge (U4)
- 2026-11-07 — palette + why-connected helpers for the minimal chrome (U2)
- 2026-11-07 — brain v2 core: wireframe cortex + tag regions + one-draw atom cloud, 50k synthetic atoms (plan 2026-11-07-001 U1)
