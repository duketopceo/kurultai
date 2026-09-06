---
index: kurultai/v1
folder: website/src/brain
parent: website/src/INDEX.md
updated: 2026-09-06
version: 2
---

# `website/src/brain`

**Does:** 3D view
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`layout/`](layout/INDEX.md) — brain FDG + ontology Sugiyama

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`BrainView.ts`](BrainView.ts) | 3D synaptic canvas + layout tween + label LOD + ontology group tints | `website/src/brain/../assets/brain.glb?url` · `website/src/brain/../types` · `website/src/brain/../state` · `website/src/brain/layout/createWorker` · `website/src/brain/layout/sdf` · `website/src/brain/labels` · `website/src/brain/layout/grouping` | — | 2026-09-06 | 2 | 2026-09-06 U1 label LOD + U2 group tints + curved-synapse containment re-land · 2026-08-16 indexed (v1 seed) |
| [`labels.ts`](labels.ts) | LOD label plan: top-24 always on, camera tiers, hover override, 200 draw cap | — | — | 2026-09-06 | 1 | 2026-09-06 Astra-generated (U1) |
| [`labels.test.ts`](labels.test.ts) | Node tests for label LOD math | `./labels` | — | 2026-09-06 | 1 | 2026-09-06 orchestrator-authored (Astra test truncated; substituted) |

## Recent

- 2026-09-06 — labels module + BrainView wiring (U1/U2 of plan 2026-09-05-002)
- 2026-08-16 — indexed this folder (v1 seed)

