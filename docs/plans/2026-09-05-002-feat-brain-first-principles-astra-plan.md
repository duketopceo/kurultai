---
title: "feat: Brain 0→100 first-principles showcase + A2A chatboard (Astra-executed)"
date: 2026-09-05
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
product_contract_source: session
depth: standard
origin: "session 2026-09-05 (this run) · docs/plans/2026-09-05-001-feat-brain-ux-ontology-dashboard-plan.md (adjacent program) · docs/plans/2026-08-13-004 (foundation) · AGENTS.md Brain doctrine"
---

# feat: Brain 0→100 first-principles showcase + A2A chatboard (Astra-executed)

**Target repo:** this checkout (`kurultai`)
**Authority:** This plan > adjacent 2026-09-05-001 program (no overlap conflicts; see Relationship) > Aug-13 foundation plan > AGENTS.md Brain doctrine
**Executor contract:** implementation authored by **Astra Pro batch codegen** (`openai/gpt-6-astra-pro:batch` via OpenRouter); the orchestrating agent integrates, verifies, and ships PRs. See Astra Execution Contract.

## Goal Capsule

**Objective:** One pass, 0→100 on presentation: (1) **Brain mode** reads as a beautiful neuron lattice — nodes, connections, and **labels** showcased, contained to the hull, loaded tier-by-tier with no jank; (2) **Ontology mode** reads as the professional showcase — typed, grouped, labeled hierarchy; (3) the **A2A chatboard** becomes a first-class surface where all of Luke's agents (codename@instance) talk over MCP; (4) buttons/overlay chrome polish rides along only where it does not compete with the cortex.

**Stop when:**

- Brain mode: labels render with LOD (top-connected always, others by zoom/hover), hull containment holds (the reverted #273 regression is re-landed correctly), tiered loading keeps the first paint fast at high tiers.
- Ontology mode: class groups are visually grouped and labeled; instance expansion is clean; it reads as the professional view, not a second hairball.
- Chatboard: thread list per agent, message stream, send, presence chips, unread, react — all against the existing `/api/hey/*` endpoints; MCP `hey_*` semantics unchanged.
- `scripts/build-ui.sh` green, node layout tests green, `cargo test --locked` green (Rust untouched expected), `scripts/audit-agent-index.py` green, embedded `ui/` rebuilt.

**Do not:** change the cortex three-color doctrine (deep black; black/white + slight purple; electric); put chrome/widgets on the 3D stage; bring back galaxy; dump 7k instances into the ontology camera; touch MCP `hey_*` tool semantics; open a parallel dashboard; silent ontology writes (O3 stays deferred).

## Settled-Decisions Brief (this run's conversation)

| # | Decision | Provenance | Rejected alternative |
|---|----------|------------|----------------------|
| D1 | Target repo is **kurultai**, not pixel-tycoon | `(session-settled: user-directed — chosen over pixel-tycoon: user corrected the ship target mid-pipeline)` | shipping the wave to pixel-tycoon |
| D2 | Astra Pro batch codegen authors implementation; orchestrator integrates + PRs | `(session-settled: user-directed — "let astra go crazy on it")` | native session-model implementation |
| D3 | Cortex doctrine holds: three colors, no chrome on the stage, brain focal (not full-viewport dashboard) | `(session-settled: user-directed — parked brief §4 + AGENTS.md are the user's own doctrine; "beautiful neuron mess" refines quality within it)` | relaxing the triad or stage-chrome |
| D4 | **Labels on cortex nodes/connections are in scope** — showcase, not hover-only | `(session-settled: user-directed — "showcasing the nodes and connections between them and the labels")` | hover-only metadata (prior implicit behavior) |
| D5 | **A2A chatboard is a first-class deliverable**, built on existing `/api/hey/*` + MCP `hey_*` | `(session-settled: user-directed — "especially the agent to agent A2A chatboard for all my agents over MCP")` | treating Hey as restyle-only (2026-09-05-001's framing) |
| D6 | Tiered hot/warm/cold loading stays mandatory; no ship-all-nodes | `(session-settled: user-directed — AGENTS.md learned preference; tiered fetch landed 2026-09-05)` | shipping the full graph to the browser |
| D7 | Repos/code lattices stay off the cortex | `(session-settled: user-directed — AGENTS.md + brief §4)` | code on cortex |
| D8 | Single UI surface: `website/` source → `ui/` embed via `scripts/build-ui.sh` | `(session-settled: user-directed — AGENTS.md)` | parallel dashboard under website/web |
| D9 | O3 proposal queue stays out | `(session-settled: user-approved — #118 deferred in both prior plans; this run does not reopen)` | in-cortex approve queue |

Unlabeled (agent inference, ordinary planning input): re-land the reverted SDF hull-containment fix (#273→#275) as part of U1; carve new modules out of `BrainView.ts` (1881 lines) instead of growing it; chrome polish is explicitly conditional ("if it can easily") and sequenced last.

## Product Contract

### Summary

Foundation is landed: volumetric FDG in the GLB hull (#200), O1 schema + layered ontology scaffold (#202), tiered graph fetch (2026-09-05), Hey/Access/Repos restyle (#268–#270). This program adds the **presentation layer**: labels, containment, grouping polish, and the chatboard.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Brain-mode labels: LOD system — top-N connected nodes always labeled, others fade in by camera distance/hover; label sprites must not tank FPS at `max` tier (instanced/atlased, hard cap ~200 drawn). |
| R2 | Hull containment: nodes/edges stay inside the GLB cortex (re-land #273's intent without whatever forced the revert — bisect the revert reason first). |
| R3 | Edge language: default low-opacity, hover traces connections (electric), labels never on every edge. |
| R4 | Ontology mode: class **groupings** (visual cluster frames/group tints within the purple family), always-on labels (it is the professional view), Sugiyama layering preserved, instances on expand ≤80. |
| R5 | Chatboard (Hey surface): threads per `codename@instance`, stream + send + react + unread + presence, renders two-layer identity correctly, keyboard friendly, no MCP semantic changes. |
| R6 | Chrome polish (conditional, last): buttons/overlay consistency pass under the secondary playful language; cortex untouched. |
| R7 | Performance: first paint on `low` < 2s on dogfood-scale data; `max` tier progressive; no main-thread n² sim. |
| R8 | All verification green (see Verification). |

### Actors

- A1. Luke / dogfood operator on knowledge.shippedit.dev.
- A2. Agents via MCP — hey_* unchanged; read-only consumers of the same data the chatboard renders.
- A3. Astra (batch author) + orchestrator (integrator/shipper).

## Key Technical Decisions

- KTD1. **Labels are a new module (`website/src/brain/labels.ts`), not BrainView growth.** Sprite/point-label renderer with LOD; BrainView consumes. `(unlabeled; god-file carve-out per prior plan KTD3 precedent)`
- KTD2. **Ontology grouping is presentation-only** — grouping derived from the existing class tree + link types; no schema change, no new tables. `(R4; O1/O2 landed)`
- KTD3. **Chatboard = rework `HeyPanel.tsx` into a chatboard component set** (`website/src/components/chatboard/`), consuming existing endpoints; extend `api.ts` types only. `(D5)`
- KTD4. **Containment re-land starts from the revert's diff** (`git show 0b869c0`) to identify what broke; fix forward in `sdf.ts`, keep `layout.test.ts` cases. `(unlabeled)`
- KTD5. **Astra slices = one file per batch request**, raw single-file output, `max_tokens=16000`, integration sources packed into each prompt (types.ts, api.ts, adjacent module, one spec excerpt). Orchestrator applies via whitelist + guards (bracket balance, func floor, class_name/extends preservation). `(executor contract; verified in pixel-tycoon run 2026-09-05)`
- KTD6. **Sequencing:** U1 (brain+labels) → U2 (ontology grouping) → U3 (chatboard) → U4 (conditional chrome) → U5 (integrate+verify). One PR per unit-pair max; `ui/` rebuild only at integration.

## File Map

| File | Role |
|------|------|
| `website/src/brain/labels.ts` (+`labels.test.ts`) | NEW: LOD label renderer |
| `website/src/brain/BrainView.ts` | consume labels module; containment fix points |
| `website/src/brain/layout/sdf.ts` | containment re-land |
| `website/src/brain/layout/sugiyama.ts` | grouping-aware layer polish |
| `website/src/brain/layout/grouping.ts` (+test) | NEW: class-group derivation |
| `website/src/components/chatboard/Chatboard.tsx` (+`ThreadList.tsx`, `MessageStream.tsx`, tests) | NEW: chatboard set |
| `website/src/components/HeyPanel.tsx` | thin wrapper → Chatboard |
| `website/src/api.ts`, `website/src/types.ts` | chatboard API types/endpoints |
| `website/src/styles.css` | tokens for labels/grouping/chatboard (chrome family) |
| `ui/` | rebuilt artifact (build-ui.sh), not hand-edited |

## Implementation Units

### U1. Brain showcase: labels + containment (R1–R3, R7)
Files: labels.ts(+test), sdf.ts, BrainView.ts, layout tests.
Approach: bisect #275 revert → fix containment forward; labels module with top-N always + zoom/hover LOD + draw cap; wire hover-trace (existing) to also surface the label; node tests for LOD math.
### U2. Ontology professional grouping (R4)
Files: grouping.ts(+test), sugiyama.ts, BrainView.ts, styles.css.
Approach: derive visual groups from class tree; group tint frames (purple family only); always-on labels in ontology mode; instance expand polish; tests for grouping derivation.
### U3. A2A chatboard (R5)
Files: chatboard/*, HeyPanel.tsx, api.ts, types.ts.
Approach: thread list (codename@instance chips, presence, unread), stream, compose/send, react; reuse restyled panel shell; keyboard nav; no MCP changes; component tests for message mapping incl. two-layer identity rendering.
### U4. Conditional chrome polish (R6)
Only if U1–U3 land within budget: button/overlay consistency pass, secondary language, zero BrainView changes.
### U5. Integration + verify (R7–R8)
build-ui.sh, node tests, cargo test, audit-agent-index.py, daemon smoke `/ui/`, INDEX ritual updates (every touched folder), PR ship + babysit.

## Astra Execution Contract (for ce-work)

- Endpoint/keys per `pixel-tycoon` skill `astra-batch-coding` (verified 2026-09-05): batch endpoint without `/v1`, HTTP 202 = accepted, results inline, ~16k completion ceiling → `max_tokens=16000`, raw single-file output, one file per request.
- Each request packs: binding rules (tabs/snake_case-equivalent TS style, no stubs, `prefers-reduced-motion` respected) + the target file's current source + its direct integration imports + the unit's spec excerpt from this plan.
- Orchestrator applies via whitelist (only the file map), guards (bracket balance, export floor, no TODO/FIXME), then `npx tsc --noEmit` + node tests before commit. Est. spend $3–6 of the $50 cap ($12.10 used).

## Risks

| Risk | Mitigation |
|------|------------|
| #273 revert reason unknown | KTD4: bisect first; if the fix is structurally unsound, ship labels without hard containment and file residual finding |
| Labels tank FPS at max tier | R1 hard draw cap + atlas; degrade to top-N only |
| Chatboard fights 001's restyle | restyle already landed (#268–#270); chatboard builds on it |
| Astra truncation | raw contract + compactness directives + guards (proven) |
| Scope creep into O3/WebGPU | D9 + prior Do-not list stand |

## Verification

`node --experimental-strip-types --test website/src/brain/layout/*.test.ts website/src/brain/labels.test.ts website/src/components/chatboard/*.test.ts` · `npx tsc --noEmit` (website/) · `cargo test --locked` · `scripts/build-ui.sh` · `python3 scripts/audit-agent-index.py` · daemon smoke: `kurultai daemon --port 8421` → `/ui/` renders brain+ontology+chatboard against dogfood store.

## Relationship to 2026-09-05-001

Adjacent, not superseding: 001 owns promote UX + secondary token system (its slices 1–2); this plan owns labels/containment/grouping + chatboard. 001's slice 3 (panels restyle) already landed; U3 builds on it. Neither plan reopens the other's settled scope.

## Residual + Handoff

Unapplied review findings → tracker tickets + `docs/residual-review-findings/<branch>.md` (repo convention). Post-merge handoff offers: dogfood seed of `instance_of` links (explicit operator action), O3 queue (#118), Postgres ontology (T1b).
