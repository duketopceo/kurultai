---
index: kurultai/v1
folder: website/src/brain/v2
parent: website/src/brain/INDEX.md
updated: 2026-11-07
version: 1
---

# `website/src/brain/v2`

**Does:** Brain v2: wireframe cortex, tag-region parcellation, single-draw atom cloud (opt in `?brain=v2`)
**Up:** [`website/src/brain/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`BrainV2.ts`](BrainV2.ts) | Three.js scene: wireframe shell, Points cloud, region glow + labels, LOD, orbit, fly-to, hover edges, path highlight | `three` · `website/src/brain/v2/regions.ts` · `website/src/brain/v2/path.ts` | `website/src/components/BrainStage.tsx` | 2026-11-07 | 1 | 2026-11-07 added (plan 2026-11-07-001 U1) |
| [`regions.ts`](regions.ts) | Pure tag-region parcellation, hull test, deterministic placement, LOD tiers | — | `website/src/brain/v2/BrainV2.ts` · `website/src/brain/v2/regions.test.ts` | 2026-11-07 | 1 | 2026-11-07 added |
| [`regions.test.ts`](regions.test.ts) | node:test: hull containment, determinism, region cap/fill, LOD, path, links | `website/src/brain/v2/regions.ts` · `website/src/brain/v2/path.ts` · `website/src/brain/v2/links.ts` | — | 2026-11-07 | 1 | 2026-11-07 added |
| [`path.ts`](path.ts) | Client-side BFS shortest path over loaded edges (stand-in for a traverse API) | `website/src/types` | `website/src/brain/v2/BrainV2.ts` | 2026-11-07 | 1 | 2026-11-07 added |
| [`links.ts`](links.ts) | Shared-tag synapse derivation (moved from BrainStage) | `website/src/brain/linkSelect.ts` | `website/src/components/BrainStage.tsx` | 2026-11-07 | 1 | 2026-11-07 added |
| [`flag.ts`](flag.ts) | `?brain=v2` / `?brain=v1` selector | — | `website/src/components/BrainStage.tsx` | 2026-11-07 | 1 | 2026-11-07 added |
| [`flag.test.ts`](flag.test.ts) | node:test for the selector | `website/src/brain/v2/flag.ts` | — | 2026-11-07 | 1 | 2026-11-07 added |

## Recent

- 2026-11-07 — brain v2 core: wireframe cortex + tag regions + one-draw atom cloud, 50k synthetic atoms (plan 2026-11-07-001 U1)
