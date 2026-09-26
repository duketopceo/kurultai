# Research bases — near-drop-in candidates

**Date:** 2026-09-26 · **Plan:** `docs/plans/2026-09-25-001` (U2) · **Feeds:** `design-lab/prompts/` and the `ui-next` token sheet.

Each base answers a different question. None is vendored wholesale; the stitched UI keeps `website/src/api.ts`/`perf.ts` and our own component tree.

## 1. ThreeUI — visual language reference (not vendored)

`https://threeui.com/browse?sort=recent` — copy-ready Three.js components; recent catalog is consistently **dark, cinematic, editorial**: point-cloud-on-black heroes, hairline-bordered glass panels, mono micro-labels, restrained accent glow.

Relevant recent pieces: **Cortexa** (neural/cloud hero — closest to our Brain hero framing), **Orrery** (dark radial instrument readout), **Cadence**, **Meridian**, **Aniwall**, **MK·78 Keyboard** (dense mono control surfaces).

What to take: the pacing — big quiet canvas, sparse chrome, labels that read like instrumentation. What NOT to take: literal Three.js scenes in chrome ("UI fun, not literal" rule — CSS/motion only in chrome; literal ontology stays in cortex + inspector).

## 2. VoltAgent / VoltOps Console — nearest product analog

`github.com/VoltAgent/voltagent` (MIT, ~10k stars) — open-source TS agent framework + **VoltOps**, an agent-ops observability console (agent list, traces, memory, evals). Closest existing product shape to "agent coordination console around a memory core."

What to take: **layout patterns** — left nav vs tabbed console for agent surfaces, trace/stream row design (agent · event · time — maps directly to our Pulse/Hey rows), status-dot language. What to check during stitching: VoltOps is partly cloud/self-hosted; treat its open examples and screenshots as reference, do not assume copyable component source — verify before lifting any code (MIT covers the framework; console assets may be their SaaS).

## 3. bolt.new — machine-readable token spec

Two usable artifacts:

- `github.com/stackblitz/bolt.new` (MIT) — the open-source Remix app; `app/styles/` has a real dark design-token set.
- **shadcn.io/design/bolt-new `DESIGN.md`** — a machine-readable spec already extracted: near-black page floor + charcoal surface tier, **twin azure "voltages" for action surfaces**, Inter/Inter Display typography (700 display), binary 12px/pill radius, xterm-heritage terminal palette, ~14 component definitions.

What to take: the DESIGN.md as our `tokens.css` seed, remapped: azure voltages → electric purple (`#a855f7`/`#c084fc`/`#7c3aed`), 12px→6px panel radius (keep pill for chips), Inter Display → Orbitron for hero numerals + JetBrains Mono for data. The "put the live instrument above the fold" principle is already our shape.

## Verdict per surface

| Surface | Primary base |
|---|---|
| Global tokens | bolt.new DESIGN.md, purple-remapped |
| CommandStrip / HUD readouts | ThreeUI pacing + bolt radii |
| Hey / Pulse rows | VoltOps trace-row pattern |
| WorkSurface tabs | VoltOps console tab pattern |
| BrainStage | unchanged (protected) |
| Ontology tab | existing Flowsint board, restyled to tokens |
