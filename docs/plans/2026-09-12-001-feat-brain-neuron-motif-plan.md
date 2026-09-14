---
title: "feat: Brain neuron-motif visual language — soma, dendrite corona, traveling spikes"
date: 2026-09-12
type: feat
artifact_contract: ce-unified-plan/v1
execution: code
product_contract_source: session
depth: standard
origin: "session 2026-09-12 first-principles pass (user-invoked) · docs/plans/2026-09-05-002-feat-brain-first-principles-astra-plan.md (precedent, landed) · AGENTS.md Brain doctrine · branch fix/brain-cortex-doctrine (pale-node/shimmer baseline, uncommitted)"
---

# feat: Brain neuron-motif visual language — soma, dendrite corona, traveling spikes

**Target repo:** this checkout (`kurultai`), Brain UI only (`website/src` → `ui/` embed)
**Scope confirmed by user (2026-09-12):** visual language only — nodes/edges/particles/palette inside the existing FDG + Three.js pipeline. Layout engine, tiered loading, panels, and ontology-mode structure untouched. Neuron treatment is a **stylized motif**, not literal arbors. Engine choice evaluated below (KTD2).

## Goal Capsule

**Objective:** The Brain reads unmistakably as a living neural lattice — not a generic glowing-orb graph — while holding the doctrine: deep black, white/pale nodes, slight purple, electric firing. A viewer who has never read the code should say "neurons," not "bubbles."

**Means:** Replace orb+halo+twinkle with the minimal neural signature derived below: soma (faceted core), dendrite corona (short radial filaments), axon edges carrying traveling spike packets whose rate encodes atom activity. (KTD1, KTD3, KTD4)

**Stop when:** every Requirement in the Product Contract holds on the dogfood-scale local graph and the hosted-scale (500+) graph; Verification Contract green; dark and light themes screenshot-verified.

**Do not:** touch layout/FDG/SDF/sugiyama; restyle panels or chrome; change `/api/*` shapes; add literal dendrite arbors per node; add WebGPU or a new engine dependency (KTD2 evaluates, default stays Three.js); regress hull containment or the pale-node/anti-blob fix currently on `fix/brain-cortex-doctrine`.

## Settled-Decisions Brief (this run)

| # | Decision | Provenance | Rejected alternative |
|---|----------|------------|----------------------|
| D1 | Neuron treatment = **stylized motif** (soma + corona + directed spikes) | `(session-settled: user-directed)` | literal arbors (noise at 500 nodes), zoom-dependent realism (deferred — see Deferred) |
| D2 | Scope = **visual language only**; layout/pipeline untouched | `(session-settled: user-directed)` | visuals+render-pipeline rework; full rework absorbing #137/#139 |
| D3 | Engine evaluated in-plan, default stay Three.js | `(session-settled: user-directed)` | pre-committing to Three.js or WebGPU without evaluation |
| D4 | Three-color doctrine + electric firing carry forward | `(session-settled: user-directed — AGENTS.md Brain doctrine)` | relaxing palette or ambient-only animation |

Unlabeled (agent inference): carve new modules out of `BrainView.ts` (~2090 lines) per prior-plan KTD precedent rather than growing it; the uncommitted `fix/brain-cortex-doctrine` shimmer/palette work is the baseline this plan builds on.

## First-Principles Derivation

This section is the design's source of truth — the requirements below are derived here, not asserted.

### Surface

Conventional answer everyone ships: "knowledge graph = glowing orbs + straight lines + bloom" (Obsidian graph view, constellation art, stock "neural net" imagery). Loaded words: "node" (implies orb), "synapse" (rendered as a wire), "brain" (applied only as a hull shape, not as behavior).

### Question

- **Nodes must be orbs** — believed because every graph viz does it and spheres are cheap. If false: a sphere carries no neural signature; soma identity needs facets + surrounding filament structure.
- **Halos = soft round glow** — believed because bloom reads as "energy." If false: additive radial glow is exactly what produced the purple-blob regression this branch just fixed.
- **Animation = ambient twinkle** — believed because shimmer looks alive. If false: random flicker encodes nothing; the neural signature is the *firing event* — a discrete, directed pulse.
- **Edges are static paths** — believed because links have no direction (`Link { a, b }`). If false: direction can be *inferred* (fire toward the higher-degree hub = signal converging on salient memories), which is a defensible stylization, not a fabricated semantic.

### Bedrock

- [FUNDAMENTAL] A neuron's readable signature = soma + short dendritic corona + an axon that conducts a spike. Nothing more is needed to read "neural."
- [FUNDAMENTAL] The neural *behavior* signature is firing: sparse, discrete, directed pulses — not continuous glow.
- [FUNDAMENTAL] At 500+ nodes, additive emissive overdraw destroys legibility (proven by the purple-blob fix); subtlety and sparse events are the only scaling path inside the doctrine palette.
- [FUNDAMENTAL] Containment (GLB hull + SDF) and tiered loading are solved infrastructure; the visual layer must not regress them.
- [FUNDAMENTAL] Doctrine palette is three colors; all new elements render inside it.
- [STILL ASSUMPTION] Three.js suffices — evaluated as KTD2.
- [STILL ASSUMPTION] `Atom` exposes enough activity signal (score, tier, timestamps via `atom.time`/score) to drive firing rate — verify during U3; fallback is degree/score only.

### Rebuild

From bedrock, in order:

1. **Soma**: keep the mesh path but read organic — faceted geometry (`IcosahedronGeometry` detail-1, flat shading) with the pale emissive core. Degree/score sizing formula unchanged.
2. **Dendrite corona**: replace the round radial halo texture with a procedurally generated starburst sprite (6–9 tapered filaments, slight angular jitter, white-violet at low opacity). Same sprite slot, same cutoff behavior — pure texture/material swap, zero new geometry per node.
3. **Axon spikes**: a pooled set of bright point sprites that travel each edge's existing CatmullRom curve from one endpoint to the other, briefly flaring the soma on arrival. This replaces "zap" as a *locus*: instead of an edge lighting uniformly, a packet traverses it.
4. **Activity-encoded emission**: firing rate per edge ∝ endpoint heat (`score`, recency, tier, hover). Membrane noise (the existing low-amplitude shimmer) stays as the sub-threshold layer — firing rides on top.
5. **Hover**: connected edges emit sustained spike traffic toward the hovered node — "you are stimulating this neuron." Unrelated edges dim (existing behavior preserved).

### Implications

- vs convention: no bloom pass, no uniform edge glow, no ambient randomness as the main animation. The web is quiet; activity is the story.
- Cost of being wrong: if `[STILL ASSUMPTION]` heat signals are too weak in practice, firing rate falls back to degree — visual still works, encoding claim softens (recorded as a deferred question).
- Zoom-dependent realism (literal arbors on hover) stays available as a follow-up; this plan's corona is its cheap ancestor.

## Product Contract

### Summary

Rebuild the brain-mode node's visual identity from orb+halo+twinkle to soma+corona+traveling-spikes, inside the existing Three.js pipeline, sprite-cloud cutoff, and doctrine palette. No layout, API, or panel changes.

### Problem Frame

Two passes already fixed *what* the brain shows (pale nodes, inward-bowed arcs, shimmer). The remaining gap is *what it reads as*: still generic graph-viz orbs. The user wants "nodes, neurons, shape of a brain" — the neural read must come from the elements themselves.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Node = faceted soma + dendrite corona. Corona is a procedurally generated starburst sprite (no binary assets), swapped into the existing halo sprite path AND the `THREE.Points` cloud shader above `NODE_SPRITE_CUTOFF`. |
| R2 | Spikes: pooled bright sprites travel edge curves endpoint→endpoint; arrival flares the destination soma briefly. Pool is bounded (default cap ~150) and shared across all edges. |
| R3 | Emission is activity-encoded: per-edge firing rate derives from endpoint heat (score/recency/tier/hover) via a pure, testable mapping; baseline shimmer remains as low-amplitude membrane noise. |
| R4 | Hover = stimulation: connected edges emit sustained traffic toward the hovered node; unrelated edges dim (existing dim preserved). |
| R5 | Doctrine palette unchanged: deep black stage, white/pale somas, slight purple wiring, spikes white-hot. No bloom pass; additive overdraw capped (corona opacity and spike pool are the only additive surfaces added). |
| R6 | Parity across render paths: mesh path (≤ cutoff) and Points-cloud path render the same motif at appropriate fidelity; sprite-mode picking and labels unaffected. |
| R7 | Performance: no per-frame allocation in spike/corona updates; 60fps at `max` tier on the dogfood graph; `metrics()` reports spike-pool stats. |
| R8 | All verification green (Verification Contract). Rust untouched — `cargo test --locked` expected-green, not a gate on design choices. |

### Scope Boundaries

**In:** brain-mode nodes, edges, particles, palette, hover; new modules `dendrite.ts`, `spikes.ts`; BrainView wiring.

**Deferred to follow-up work:** zoom-dependent literal arbors; ontology-mode visual pass beyond keeping spikes off (ontology edges stay as-is); drill-down traversal (#139); repos/code-lattice surfaces; anything on `web/`.

**Outside identity:** galaxy/solar layouts, chrome on the 3D stage, literal biology, any parallel dashboard.

### Open Questions

- Does `Atom`/`/api/graph` expose a usable recency field for heat (U3 verifies)? If not, heat = degree + score only — acceptable, recorded here as the known thin spot.

## Planning Contract

### Key Technical Decisions

- KTD1. **The motif is soma + corona + traveling spikes** — the minimal neural signature from the first-principles rebuild (D1). Literal arbors deferred; the corona sprite is their cheap ancestor. Governs R1–R3.
- KTD2. **Engine: stay on Three.js** — evaluated per D3. The binding constraint is draw calls and additive overdraw, not API headroom: the codebase already has both render paths (mesh, instanced Points) and shader seams this plan needs. Alternatives considered: raw WebGL2/regl (rewrite of GLTF hull loading, raycasting, SDF sampling for zero measurable gain at ≤1k nodes — rejected); `three/webgpu`+TSL (immature Points/Sprite surface, untested on this Asahi/Hyprland stack — rejected now, revisit if node counts cross ~5k). Governs R6–R7.
- KTD3. **New modules, not BrainView growth** — `website/src/brain/dendrite.ts` (corona texture + soma geometry/material factories) and `website/src/brain/spikes.ts` (pool, emission scheduler as pure functions, traversal updater). Precedent: prior plan's KTD1 carved `labels.ts`. Governs R1–R4.
- KTD4. **Firing encodes activity; shimmer demoted to membrane noise** — random twinkle is decorative; directed spikes carry the "electric" doctrine literally. Governs R3–R4.
- KTD5. **Corona texture is generated at boot on an offscreen canvas** — keeps `ui/` bundle free of binary assets and lets filament count/length stay tweakable constants. Governs R1.

### High-Level Technical Design

```
per frame (tick):
  ┌─ spikes.ts ────────────────────────────────┐
  │ emission: for each edge,                   │
  │   rate = heat(endpoint a) + heat(b)  ──────┐│
  │   (hovered: sustained burst toward hover)  ││
  │   if fire(t) → claim free spike sprite     ││
  │ traversal: spike.t += dt/edgeDuration;     ││
  │   pos = edge.curve.getPointAt(t)           ││
  │   t>=1 → release + flare soma (emissive)   ││
  └────────────────────────────────────────────┘│
  BrainView: shimmer tick (membrane noise, dimmer) ◄┘
  dendrite.ts (boot): canvas → starburst CanvasTexture
     → halo SpriteMaterial.map & Points ShaderMaterial map
```

Edge curves already exist per line (`CatmullRomCurve3` through 6 points); spikes re-sample them — store the `Curve` object on `line.userData` at build time so traversal needs no re-derivation.

### Assumptions

- The uncommitted `fix/brain-cortex-doctrine` palette/shimmer work is the baseline and merges first (or this branch continues it) — the plan layers on it, not on `main`'s purple.
- `Link` stays `a`/`b` undirected; spike direction is stylized (→ higher-degree endpoint; → hovered node on hover).
- No new npm dependencies; canvas 2D is sufficient for the corona texture.

## Implementation Units

### U1. Dendrite corona texture + soma facelift

**Goal:** Nodes stop reading as glowing orbs.
**Requirements:** R1, R5, R6
**Dependencies:** none
**Files:** `website/src/brain/dendrite.ts` (new), `website/src/brain/dendrite.test.ts` (new), `website/src/brain/BrainView.ts`
**Approach:** `dendrite.ts` exports `coronaTexture(opts): CanvasTexture` — draws N tapered filaments (6–9) radiating with angular jitter, white-violet radial falloff, on an offscreen canvas — plus `somaGeometry()` (IcosahedronGeometry detail-1) and `coronaParams(degree): {filaments, jitter, scale}` as a pure function for tests. BrainView: swap halo sprite material map to `coronaTexture`, tighten sprite scale to ~2.4× soma radius; soma geometry swap keeps MeshStandardMaterial + existing palette; pass the same texture into the Points-cloud `ShaderMaterial` for parity above cutoff.
**Patterns to follow:** `labels.ts` module split (pure logic + thin GL glue); existing `haloMap`/`nodeSpriteCloud` seams.
**Test scenarios:**
- `coronaParams` is monotonic in degree and clamps to filament bounds (happy path)
- `coronaParams(0)` yields minimum corona; `coronaParams(veryLarge)` saturates (edges)
- Texture generator degrades gracefully when called without DOM (returns null → BrainView falls back to current halo texture) (failure path)
- Sprite-cloud and mesh paths both receive the texture — assert the material map reference is set in a construction seam test if extractable (integration)
**Verification:** dark + light screenshots show corona, not round halo; FPS unchanged at `max` tier.

### U2. Traveling spike pool

**Goal:** Edges conduct discrete directed spikes instead of only lighting up.
**Requirements:** R2, R6, R7
**Dependencies:** U1 (soma flare target)
**Files:** `website/src/brain/spikes.ts` (new), `website/src/brain/spikes.test.ts` (new), `website/src/brain/BrainView.ts`
**Approach:** `spikes.ts` exports a `SpikePool` (fixed-size `THREE.Points` with per-spike `edge`, `t`, `speed`, direction) and pure scheduler `nextEmission(edgeState, heat, dt, rng)` returning whether to fire. BrainView `buildEdges` stores the `CatmullRomCurve3` on `userData.curve`; the tick advances spikes via `curve.getPointAt`, releases on `t>=1` and pulses the destination soma's emissive. Direction: toward the higher-degree endpoint by default. Pool overflow = drop oldest (never allocate).
**Test scenarios:**
- Pool never exceeds cap under dense emission requests (edge)
- `nextEmission` fires more under higher heat, never under zero heat (mapping correctness)
- Released spike returns to pool; `t>=1` arrival triggers exactly one flare call (integration)
- Zero edges → zero allocations per tick (perf guard)
**Verification:** screenshot/screencast shows discrete packets traversing edges; `metrics()` reports pool size + active count.

### U3. Activity-encoded firing rate

**Goal:** Motion carries meaning — hot memories fire more.
**Requirements:** R3, R5
**Dependencies:** U2
**Files:** `website/src/brain/spikes.ts`, `website/src/brain/spikes.test.ts`, `website/src/brain/BrainView.ts`
**Approach:** Pure `heatOf(atom, now): number` in `spikes.ts` — weighted mix of `atom.score`, recency (verify available field; else omit), tier (hot/warm/cold), degree. Edge emission rate = `f(heatA + heatB)` with a floor so the cortex never goes fully dark. `tickSynapseShimmer` keeps only the low-amplitude flicker (membrane noise); the `zap` uniform-spike is removed in favor of U2 traversal. Reduced-motion pref: spikes off, noise stays.
**Test scenarios:**
- `heatOf` ordering: hot-tier high-score atom > cold low-score (happy path)
- Missing recency field doesn't throw; weight redistributes (edge)
- Rate floor: coldest edges still fire occasionally (boundary)
- `reducedMotion` flag suppresses spike claims entirely (integration)
**Verification:** recently-touched notes visibly fire more on the seeded local graph; doctrine screenshot still passes (no blob).

### U4. Hover stimulation + contrast pass

**Goal:** Hover reads as stimulating a neuron; overall contrast holds doctrine.
**Requirements:** R4, R5
**Dependencies:** U2, U3
**Files:** `website/src/brain/BrainView.ts`, `website/src/brain/spikes.ts`
**Approach:** On hover, connected edges switch emission direction toward the hovered node at a raised rate for the hover duration; existing white-hot color lift + unrelated-edge dim stay. Contrast pass: corona opacity, edge base opacity, and spike size tuned together so the three additive surfaces (corona, edge, spike) never stack into bloom — document the tuned constants with a comment citing the blob regression.
**Test scenarios:**
- Hover flips emission direction on connected edges only (pure scheduler test)
- Unrelated edges' shimmer continues dimmed, unchanged (regression)
- Unhover restores base state exactly (no residue)
**Verification:** hover screencast: spikes converge on the stimulated soma; rest of graph dims.

### U5. Build, index, and visual verify

**Goal:** Ship-shape artifact: regenerated embed + green audits + evidence.
**Requirements:** R6–R8
**Dependencies:** U1–U4
**Files:** `ui/` (regenerated), `website/src/brain/INDEX.md`, `website/src/INDEX.md`, `website/INDEX.md`, `ui/INDEX.md`, `INDEX.md`, `docs/plans/INDEX.md`, `docs/INDEX.md`
**Approach:** `scripts/build-ui.sh`; update per-file INDEX rows + Recent chains per `docs/agent-index.md`; run the audit; capture dark+light screenshots on the seeded local graph and (if accessible) the hosted instance for the before/after record.
**Test expectation:** none — build/index verification is covered by the commands below.
**Verification:** all Verification Contract commands green; screenshots attached to the PR.

## Verification Contract

- `cd website && npm test` — all suites incl. new `dendrite.test.ts`, `spikes.test.ts`
- `cd website && npx tsc --noEmit` (runs inside `npm run build` too)
- `bash scripts/build-ui.sh` — regenerates embedded `ui/`
- `python3 scripts/audit-agent-index.py` — index audit green
- `cargo test --locked` — expected green (Rust untouched)
- Visual gate: local daemon `:8421` + Vite `:5174`, dark and light mode screenshots; no halo-blob regression at `max` tier.

## Definition of Done

- Brain reads as neurons (soma + corona + directed spikes) on first paint — the Objective is checkable by a viewer who never read this plan.
- All Requirements hold; all Verification Contract commands green.
- Hover stimulates (R4); firing encodes heat (R3) with documented fallback if recency proves unavailable.
- No new dependencies; no binary assets; palette unchanged.
- Dead-end experiment code (abandoned textures/pools) removed before ship.
- INDEX rows + Recent chains updated; `ui/` regenerated.

## Sources / Research

- First-principles pass (this session, `first-principles-thinking` skill) — the Derivation section above is its output.
- `last30days` run (2026-09-12, topic: 3D knowledge-graph/neuron visualization) — **thin evidence** (2 sources, no relevant technique signal); recorded honestly, not load-bearing. One tangential signal: connectome-scale tools (FlyWire/Neuroglancer-class renderers) prove "reads as neural" comes from arbor structure + firing events, not glow — consistent with the Rebuild.
- `docs/plans/2026-09-05-002-feat-brain-first-principles-astra-plan.md` — landed presentation pass; supplies module-split precedent and doctrine constraints.
- `docs/plans/2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md` + `docs/brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md` — hull/SDF/FDG foundation this plan must not regress.
- Code seams: `website/src/brain/BrainView.ts` (`buildEdges` ~805, `tickSynapseShimmer` ~1291, `nodeSpriteCloud` ~748, `nodeRadius` ~699), `website/src/brain/labels.ts`, `website/src/types.ts` (`Link {a,b}` ~59).
