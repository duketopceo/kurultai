---
index: kurultai/v1
folder: website/src/brain
parent: website/src/INDEX.md
updated: 2026-09-06
version: 4
---

# `website/src/brain`

**Does:** 3D view
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`layout/`](layout/INDEX.md) — brain FDG + ontology Sugiyama

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`BrainView.ts`](BrainView.ts) | 3D synaptic canvas + layout tween + metrics() + uPulse morph ripple | `website/src/brain/../assets/brain.glb?url` · `website/src/brain/../types` · `website/src/brain/../state` · `website/src/brain/layout/createWorker` · `website/src/brain/layout/sdf` | — | 2026-09-06 | 2 | 2026-09-06 uPulse shell ripple on setData (morph cue, decays in loop) · 2026-09-06 metrics() scene snapshot (edge-length stats, hull containment %, fps) · 2026-09-06 push SDF to in-flight worker (workerHasSdf + setSdf) · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-06 — `BrainView`: `uPulse` uniform + inward-traveling shell wave on `setData` (data-change morph cue)
- 2026-09-06 — `BrainView.metrics()` + `window.__kurultaiBrain` console handle (BrainStage) for node/edge/hull/FPS measurement
- 2026-09-06 — `BrainView`: send hull SDF to FDG worker via `setSdf` when GLB bakes after layout start (nodes were forming a ring outside the cortex)
- 2026-08-16 — indexed this folder (v1 seed)

