---
index: kurultai/v1
folder: website/src/brain
parent: website/src/INDEX.md
updated: 2026-09-14
version: 12
---

# `website/src/brain`

**Does:** 3D view
**Up:** [`website/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`layout/`](layout/INDEX.md) — brain FDG + ontology Sugiyama

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`BrainView.ts`](BrainView.ts) | 3D synaptic canvas + layout tween + metrics() + uPulse morph ripple + label LOD + ontology group tints + neuron motif (corona + spikes) | `website/src/brain/../assets/brain.glb?url` · `website/src/brain/../types` · `website/src/brain/../state` · `website/src/brain/layout/createWorker` · `website/src/brain/layout/sdf` · `website/src/brain/labels` · `website/src/brain/layout/grouping` · `website/src/brain/dendrite` · `website/src/brain/spikes` | — | 2026-09-17 | 5 | 2026-09-17 density attenuation: sprite alpha/size, edge + corona + soma opacity all scale with `sizeScale` — dense tiers no longer fuse to a white mass · 2026-09-12 neuron motif (plan 2026-09-12-001): icosahedron soma, dendrite corona sprites, traveling spike pool w/ heat-encoded emission + hover stimulation, shimmer demoted to membrane noise · 2026-09-12 doctrine rebalance: pale-white nodes, halo 2.0x/0.18, emissive 0.55, per-synapse shimmer+zap, edgeActive white-hot · 2026-09-07 merged wave-1 label LOD (U1) + ontology group tints (U2) onto lit-sphere interior-bow base · 2026-09-07 interior-bow synapses + hover-reset restores strength opacity |
| [`dendrite.ts`](dendrite.ts) | Neuron motif: coronaParams (degree→filaments/scale, pure), procedural starburst CanvasTexture, faceted somaGeometry | `three` | — | 2026-09-12 | 1 | 2026-09-12 plan 2026-09-12-001 U1 |
| [`dendrite.test.ts`](dendrite.test.ts) | node:test coverage for coronaParams monotonicity/clamps, DOM-less fallback, soma facet budget | `./dendrite` | — | 2026-09-12 | 1 | 2026-09-12 plan 2026-09-12-001 U1 |
| [`spikes.ts`](spikes.ts) | Axon spike pool (fixed THREE.Points) + pure emission: heatOf, emissionRate, nextEmission accumulator | `three` | — | 2026-09-12 | 1 | 2026-09-12 plan 2026-09-12-001 U2/U3 |
| [`spikes.test.ts`](spikes.test.ts) | node:test coverage for heat ordering, emission floors, deterministic firing, pool saturation/arrival | `./spikes` | — | 2026-09-12 | 1 | 2026-09-12 plan 2026-09-12-001 U2/U3 |
| [`linkSelect.ts`](linkSelect.ts) | Intentional-synapse sparsifier: union of per-node top-K strength nominations + unconditional low-degree (bridge/leaf) survivors, hard cap | — | `website/src/components/BrainStage` | 2026-09-12 | 1 | 2026-09-12 added — clique-fill edges made wiring unreadable at scale |
| [`linkSelect.test.ts`](linkSelect.test.ts) | node:test coverage for clique collapse, bridge survival under cap, determinism, chain keep | `./linkSelect` | — | 2026-09-12 | 1 | 2026-09-12 added |
| [`labels.ts`](labels.ts) | LOD label plan: top-24 always on, camera tiers, hover override, 200 draw cap | — | — | 2026-09-06 | 1 | 2026-09-06 Astra-generated (U1) |
| [`labels.test.ts`](labels.test.ts) | Node tests for label LOD math | `./labels` | — | 2026-09-06 | 1 | 2026-09-06 orchestrator-authored (Astra test truncated; substituted) |

## Recent

- 2026-09-14 — `layout/ontoBoard.ts`: 2D ontology board layout helpers consumed by `components/OntologyBoard` (#316)
- 2026-09-12 — `linkSelect.ts` intentional-synapse sparsifier wired into BrainStage `buildLinks`: union top-K nominations + unconditional low-degree survivors (replaced arbitrary strength-sort cap)
- 2026-09-12 — neuron motif (plan 2026-09-12-001): `dendrite.ts` (soma icosahedron + procedural corona textures), `spikes.ts` (traveling axon spikes, heat-encoded emission, hover stimulation), shimmer demoted to membrane noise; +13 node tests
- 2026-09-12 — `BrainView` cortex-doctrine rebalance: nodes pale-white not purple, halos tightened (2.0x / 0.18), per-synapse shimmer + rare zap spikes in loop, edgeActive → white-hot
- 2026-09-07 — merged wave-1 label LOD + ontology group tints onto the lit-sphere interior-bow base (plan 2026-09-05-002)
- 2026-09-07 — nodes: lit MeshStandardMaterial spheres (emissive recolor sites updated) + hemisphere/key light rig; radius back to 0.0075 base
- 2026-09-07 — explicit-mode: node radius ~1.7×, additive-blend synapses at 0.3–0.85 opacity (was 0.16–0.55 flat)
- 2026-09-07 — `buildEdges`: dropped hull-surface raycast; arcs now bow inward; edge opacity scales with shared-tag count (link.strength)
- 2026-09-06 — `BrainView`: `uPulse` uniform + inward-traveling shell wave on `setData` (data-change morph cue)
- 2026-09-06 — `BrainView.metrics()` + `window.__kurultaiBrain` console handle (BrainStage) for node/edge/hull/FPS measurement
- 2026-09-06 — `BrainView`: send hull SDF to FDG worker via `setSdf` when GLB bakes after layout start (nodes were forming a ring outside the cortex)
- 2026-08-16 — indexed this folder (v1 seed)

