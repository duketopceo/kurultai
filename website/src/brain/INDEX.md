---
index: kurultai/v1
folder: website/src/brain
parent: website/src/INDEX.md
updated: 2026-09-06
version: 8
---

# `website/src/brain`

**Does:** 3D view
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`layout/`](layout/INDEX.md) — brain FDG + ontology Sugiyama

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`BrainView.ts`](BrainView.ts) | 3D synaptic canvas + layout tween + metrics() + uPulse morph ripple + label LOD + ontology group tints | `website/src/brain/../assets/brain.glb?url` · `website/src/brain/../types` · `website/src/brain/../state` · `website/src/brain/layout/createWorker` · `website/src/brain/layout/sdf` · `website/src/brain/labels` · `website/src/brain/layout/grouping` | — | 2026-09-06 | 2 | 2026-09-07 merged wave-1 label LOD (U1) + ontology group tints (U2) onto lit-sphere interior-bow base · 2026-09-07 interior-bow synapses (no surface raycast; strength-scaled opacity) + hover-reset restores strength opacity · 2026-09-06 uPulse shell ripple on setData (morph cue, decays in loop) · 2026-09-06 metrics() scene snapshot (edge-length stats, hull containment %, fps) · 2026-09-06 push SDF to in-flight worker (workerHasSdf + setSdf) · 2026-08-16 indexed (v1 seed) |
| [`labels.ts`](labels.ts) | LOD label plan: top-24 always on, camera tiers, hover override, 200 draw cap | — | — | 2026-09-06 | 1 | 2026-09-06 Astra-generated (U1) |
| [`labels.test.ts`](labels.test.ts) | Node tests for label LOD math | `./labels` | — | 2026-09-06 | 1 | 2026-09-06 orchestrator-authored (Astra test truncated; substituted) |

## Recent

- 2026-09-07 — merged wave-1 label LOD + ontology group tints onto the lit-sphere interior-bow base (plan 2026-09-05-002)
- 2026-09-07 — nodes: lit MeshStandardMaterial spheres (emissive recolor sites updated) + hemisphere/key light rig; radius back to 0.0075 base
- 2026-09-07 — explicit-mode: node radius ~1.7×, additive-blend synapses at 0.3–0.85 opacity (was 0.16–0.55 flat)
- 2026-09-07 — `buildEdges`: dropped hull-surface raycast; arcs now bow inward; edge opacity scales with shared-tag count (link.strength)
- 2026-09-06 — `BrainView`: `uPulse` uniform + inward-traveling shell wave on `setData` (data-change morph cue)
- 2026-09-06 — `BrainView.metrics()` + `window.__kurultaiBrain` console handle (BrainStage) for node/edge/hull/FPS measurement
- 2026-09-06 — `BrainView`: send hull SDF to FDG worker via `setSdf` when GLB bakes after layout start (nodes were forming a ring outside the cortex)
- 2026-08-16 — indexed this folder (v1 seed)

