---
title: "feat: Brain v2 - explorable wireframe cortex with tag regions"
type: feat
status: active
date: 2026-11-07
execution: code
---

# Brain v2: explorable, optimized wireframe brain

Lean plan (no deepening). Rollback tag: `brain-v1-2026-11-07`. Supersedes docs PR #498 (closed).

## Settled decisions (user-directed, 2026-11-07)

| # | Decision | Rejected | Reason |
|---|----------|----------|--------|
| KTD1 | Low-poly wireframe cortex shell; regions = parcellation of the hull per top-level tag; a region fills as its node count grows | solid GLB, flat 2D, galaxy | cheaper, cleaner, still a brain |
| KTD2 | One `THREE.Points`/instanced draw for all nodes; deterministic O(n) placement inside the region volume; LOD far=region glow+count, near=atoms; edges only for hover/selected/path | per-node meshes, FDG at scale | 50k+ atoms on integrated GPUs |
| KTD3 | No traverse API exists in the repo (the 2026-11-03 traverse plan is absent), so "why connected" = client-side BFS shortest path over loaded edges | new server endpoint now | minimal; endpoint is a follow-up |
| KTD4 | Brain colors per AGENTS.md (deep black, B/W + slight purple, zap/shimmer); ember `#e8a33d` only for selected path / search target; chrome per DESIGN.md | ember as brain tint | owner prefs |
| KTD5 | Ship behind `?brain=v2`, v1 at `?brain=v1`; v2 default at parity (PR4) | big-bang swap | rollback |
| KTD6 | Repos floorplan is a separate optional view, never mixed into the brain | repos as brain regions | owner decision |

## Units (one PR each, stacked)

- **U1 / PR1 core v2 brain.** `website/src/brain/v2/`: `regions.ts` (tag -> region, SDF-bounded placement, deterministic hash, counts), `wire.ts` (low-poly deformed icosphere wireframe + per-region chunk fill), `cloud.ts` (Points cloud, per-point attributes, LOD tiers), `path.ts` (BFS shortest path), `BrainV2.ts` (scene, orbit/zoom, fit-all camera, hover edges, pick). `?brain=v2` switch in `BrainStage.tsx` via dynamic import (code split). Tests: regions, path, LOD pure functions. Smoke screenshots + synthetic 50k fps.
- **U2 / PR2 minimal chrome + panel.** Main page reduced to search + status dot + one menu; drawer/command palette holds Hey, Pulse, Ontology, Ask, Store; search fly-to + clear/reset; pick two nodes -> path; select -> side panel (cited answer via existing ask API + why-connected chain). Tests for palette filter/pure helpers.
- **U3 / PR3 repos floorplan.** Optional toggle view: folders as rooms, files as tiles sized by size/activity; data from codebase-memory-mcp when reachable, else agent-index manifests. Pure layout (treemap) tested.
- **U4 / PR4 whats-new + default flip.** localStorage last-visit (try/catch), "N new" badge lighting new/changed atoms and listing them; tiered fetch beyond 2500 cap using existing `/api/graph?tier=&limit=`; flip default to v2 with `?brain=v1` escape.

## Verification
`cd website && npm test`, `npm run build` (tsc), `timeout -k 5s 180s node scripts/ui-smoke.mjs`, `python3 scripts/audit-agent-index.py`. Per-folder INDEX.md updates. No live model calls, no deploy.

## Risks
SwiftShader fps in headless is far below real GPUs; report numbers as relative. Hosted path untouched.
