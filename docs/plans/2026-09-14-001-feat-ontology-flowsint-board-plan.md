---
title: "feat: Flowsint-style 2D board replaces ontology layout"
date: 2026-09-14
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "User — 'replace ontology with that' (reconurge/flowsint sketch board); mode keeps the name 'ontology'"
origin: "session dialogue 2026-09-14 (flowsint UI reference)"
depth: standard
---

# feat: Flowsint-style ontology board

**Target repo:** `duketopceo/kurultai`
**Tracking:** [#316](https://github.com/duketopceo/kurultai/issues/316) · supersedes the Slice-C rendering approach of [`2026-08-13-004`](2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md) (3D Sugiyama → 2D board)
**Reference UI:** [reconurge/flowsint](https://github.com/reconurge/flowsint) — `flowsint-app` sketch board (`@xyflow/react`)
**Process:** PR-only · `scripts/build-ui.sh` after every `website/` change · tag `main` before merge for visual rollback

## Goal Capsule

**Objective:** The `ontology` toggle in the Brain stage swaps the 3D cortex for a **Flowsint-style 2D board**: entity nodes as cards on a pan/zoom canvas, typed directed edges, class→instance expand, drag-persisted positions, context-menu actions, connect-to-create-relation, and an add-entity dialog. Brain mode is unchanged.

**Stop when:**

- `layout === 'ontology'` renders the 2D board inside the brain-hero stage; `brain` still renders the volumetric FDG cortex.
- Classes render as scaffold nodes; clicking expand on a class fans out `instance_of` children (capped, e.g. 80).
- Dragging a node persists its position; reload restores it.
- Right-click context menu offers: Inspect, Expand (classes), Propose/Create link.
- Dragging a connection between two nodes opens a rel picker and writes a link through the human lane.
- Add-entity dialog creates a class or instance entity through the human lane.
- Selecting a node opens the existing floating inspector (backing atom when present).

**Do not:** change brain-mode visuals, palette, or camera; add hues beyond black/white/purple; load anything from unpkg/CDN; let agents write through the human lane; keep the 3D ontology scaffold as a third mode.

## Product Contract

### Requirements

| ID | Requirement | Origin |
|----|-------------|--------|
| R1 | `LayoutMode` stays `'brain' \| 'ontology'`; the `ontology` label/toggle is retained, but it mounts a React component (`@xyflow/react` board) instead of the BrainView ontology scaffold. | user |
| R2 | Node = entity card: kind badge (class/instance/metric), name, backing-atom summary snippet, group tint in the purple family (existing `GROUP_TINTS` derivation may be reused). | flowsint pattern + AGENTS palette |
| R3 | Edges are directed and labeled by `rel` (`is_a`, `instance_of`, `associates_with`, `triggered_by`, `contradicts`); differentiate transversal rels by dash/opacity, not a fourth hue. | plan 004 KTD11 carried over |
| R4 | Class nodes expand on demand to show `instance_of` children; default board shows the class scaffold only. | plan 004 canvas C, retained |
| R5 | Node drag positions persist per entity (localStorage keyed by entity id is acceptable v1; entity `attributes.position` via the human write lane is the upgrade path). | flowsint parity, user |
| R6 | Right-click context menu on nodes and canvas: Inspect, Expand/Collapse, Create relation, Add entity. | flowsint parity, user |
| R7 | Connect-drag between two nodes opens a rel picker; confirming writes an `approved` link via a **human-auth** endpoint. The board user is the human approver — do not route through the agent-bearer proposals API. | flowsint parity + auth reality |
| R8 | Add-entity dialog (kind + name + optional atom backing) writes via the human lane. | flowsint parity, user |
| R9 | New write routes join `WRITE_ROUTES` in `src/http/auth.rs` so `write_route_guard` covers them under SharedClosed. | security posture |
| R10 | Palette, density, and chrome stay Kurultai: deep black canvas, white/purple only, existing inspector and caption. Borrow the board *interaction model*, not Flowsint's theme. | AGENTS.md doctrine |
| R11 | Board is fully bundled: `@xyflow/react` via `website/package.json`, Vite build → `ui/`. No runtime CDN. | egress caveat |
| R12 | Empty ontology still renders the seeded class scaffold; empty state copy matches current caption behavior. | existing behavior |

### Backend work (the real gap)

`POST /api/ontology/proposals` requires an agent bearer; `decide` refuses agents. Board authoring needs a human write lane:

- `POST /api/ontology/entity` — upsert entity (kind, name, atom_id?, attributes) as `actor=http_actor(principal)`; refuse agent bearers.
- `POST /api/ontology/link` — upsert directed link (from_id, to_id, rel, confidence=1, status=approved); refuse agent bearers; validate `rel` against the closed enum and endpoints against existing entities.
- Both return the full entity/link row; UI refetches `/api/ontology` after a write.

## Key decisions

- **Settled (user-directed):** Flowsint board replaces the ontology layout, not the whole dashboard; mode keeps the name "ontology".
- **Settled (user-directed):** v1 = full Flowsint parity (context menus, persisted drag, create-relation, add-entity).
- `@xyflow/react` is the board engine — same library Flowsint uses; vendored via npm, satisfying the no-CDN rule that blocked `3d-force-graph`.
- Human lane = new endpoints, not auto-approved proposals: the proposal queue stays the agent-governed path (#118); UI writes are already the human decision.

## Out of scope

- Flowsint enrichers/investigations — no analog in Kurultai.
- Board position sync across devices (localStorage is per-browser).
- Editing/deleting existing links and entities (follow-up).
- Any change to brain-mode rendering, proposals queue, or MCP tools.

## File map (implementation guide)

| File | Role |
|------|------|
| `website/src/components/OntologyBoard.tsx` | React Flow board: nodes/edges from `OntologyResponse`, expand state, drag persist, context menu, add-entity dialog, rel picker |
| `website/src/components/BrainStage.tsx` | swap stage content on `layout==='ontology'` |
| `website/src/api.ts` | `postOntologyEntity`, `postOntologyLink` |
| `website/src/styles.css` | `.ontology-board-*` card/menu/dialog styles (palette-locked) |
| `src/http/mod.rs` + new `src/http/ontology_write.rs` | `POST /api/ontology/entity`, `POST /api/ontology/link` |
| `src/http/auth.rs` | add both routes to `WRITE_ROUTES` |
| `website/package.json` | add `@xyflow/react` (pin a version ≥7 days old) |

## Verification

- `cd website && npx tsc --noEmit && npm test`
- `cargo test --locked` (new routes + guard)
- `bash scripts/build-ui.sh` → `ui/` embeds
- Manual: daemon `:8421` → `/ui/` → ontology toggle shows board; expand, drag-persist across reload, create relation, add entity, inspector on select.
