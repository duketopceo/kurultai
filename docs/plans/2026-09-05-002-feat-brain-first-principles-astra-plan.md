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

---

# Wave-2: Review-Fix + Ship (2026-09-06)

**Status:** Wave-1 committed `03bc52a` on `feat/brain-first-principles-wave1` (local gates green at commit: tsc, 28/28 node tests, build-ui, cargo test --locked, agent-index audit 77/416). Independent `code-reviewer` pass returned **REQUEST CHANGES** (2 Critical, 5 Important). Every finding re-verified against primary sources before being planned (api.ts lines 240–290, hey.rs routes 19–26 + PostBody/ReactBody/UnreadQuery structs, store/mod.rs add_reaction, Chatboard.tsx, HeyPanel.tsx, chatboard-mapping.ts, package.json).

## Settled-Decisions Brief (wave-2 additions; D1–D9 stand)

| # | Decision | Provenance | Rejected alternative |
|---|----------|------------|----------------------|
| W1 | Wave-2 authored **natively** (session model), not Astra batch | `(session-settled: harness PRO mode disables subagents; findings are patch-size integration fixes; Astra key is the budget-capped design key at $12.10/$50 — raw single-file contract breaks partial edits anyway)` | re-running Astra batches for patch-size diffs (cost + latency + contract mismatch) |
| W2 | **api.ts is the backend-shape mirror** — TS types match Rust Dto exactly; no optimistic fields | `(verified: C2 root cause is TS fields participants/unread_count/reactions absent from HeyThread/HeyMessage; adding them client-side is what created the phantom contract)` | adding `reactions` to `MessageDto` (Rust schema change for a presentational need) |
| W3 | Reactions read by **aggregating `kind='reaction'` message rows** (`parent_id` → parent id, emoji = `content`) client-side in HeyPanel | `(verified: store/mod.rs add_reaction inserts reaction ROWS, kind='reaction', turns_consumed=0; no per-message reactions endpoint exists in hey.rs)` | new `/api/hey/messages/{id}/reactions` endpoint (backend diff reopens the green Rust gate without blessing) |
| W4 | **One chrome shell per surface**: HeyPanel owns header/caption/refresh; Chatboard renders body-only (drop inner `section.panel.chrome-panel.hey-panel` + duplicate `<h2>` + duplicate caption) | `(verified: HeyPanel.tsx wraps Chatboard — both emit section + h2 "Hey board" + identical hey-caption)` | deleting the HeyPanel wrapper (it owns polling/abort/state; converting it to a presenter is a larger refactor than the bug warrants) |

Unlabeled (agent inference): the `unread()` response TS type is pinned by reading the handler's return shape in `hey.rs` at fix time, not guessed; `presence` prop gets a real `HeyPresence[] → Record<string,String>` coerce built in mapping.

## Findings (all evidence-verified 2026-09-06)

| ID | Sev | Location | Root cause (verified) | Fix contract |
|----|-----|----------|----------------------|--------------|
| F1 | Critical | `website/src/api.ts` + `HeyPanel.tsx` | No `postHeyMessage`/`reactHeyMessage`/`fetchHeyUnread` wrappers exist; HeyPanel's `callEndpoint('postHeyMessage'/'reactHeyMessage'/'fetchHeyUnread')` resolves undefined → throw → swallowed by catch → **send/react/unread silently dead**. Routes exist in `hey.rs`: POST `/threads/{id}/messages` (PostBody: content, parent_id?, request_reply default false, thread_name?, repo?, instance_id?), POST `/messages/{id}/react` (ReactBody: emoji, thread_id), GET `/unread` (limit, since ISO?) | Add the three wrappers with exact TS types per W2; HeyPanel calls them directly (retire `callEndpoint` indirection, keep per-call failure tolerance) |
| F2 | Critical | `chatboard-mapping.ts` / `Chatboard.tsx` / `HeyPanel.tsx` | `mapThreads` reads `participants`/`unread`/`unread_count` — absent from HeyThread (peerLabel always falls back to title, unread always 0); `mapMessages` reads `message.reactions` — absent from HeyMessage and never serialized by `MessageDto`; Chatboard re-reads `createdAt`/`timestamp`/`created_at` off MessageVM (which exposes `createdAtMs` as a number) → `Date.parse(NaN)` → time-less sort degenerates; `isOwn` never set | mapping: stop phantom reads — `peerLabel` fed from presence/last-message sender, `unread` from the real GET /unread result thread through HeyPanel; build `reactionIndex: Map<messageId, {emoji,count}[]>` in HeyPanel per W3 and pass into `mapMessages` (opt-in param); Chatboard consumption aligned to actual VM fields (`createdAtMs`, optional `isOwn` keyed by presence agent_id) |
| F3 | Important | `HeyPanel.tsx` + `Chatboard.tsx` | Duplicate chrome per W4 | Chatboard body-only; single shell; CSS margins deduped for the collapsed nesting |
| F4 | Important | `Chatboard.tsx` `send()` + `HeyPanel.tsx` | Draft cleared **before** awaiting the async onSend (fire-and-forget) → failed send erases user text; the poll's thread-id renormalization (`list.find(...)|| activeThreadId`) also orphans drafts keyed by the old id | onSend returns a success bool; clear draft only on success; migrate the draft key when the active id renormalizes (display id → canonical id) |
| F5 | Important | `BrainView.ts` hover-trace path | Hover expansion unchecked against the 200 label draw cap → overflow at dense tiers | Clamp the traced-adjacent set to the cap (top by degree); no stage overflow; excess signaled only in the side panel |
| F6 | Important | `website/package.json` `test:layout` | Globs only `src/brain/layout/*.test.ts` → `labels.test.ts` + `chatboard-mapping.test.ts` never run under the script | Widen to `src/brain/*.test.ts src/brain/layout/*.test.ts src/components/chatboard/*.test.ts` (explicit paths, no glob-engine dependence) |
| F7 | Important | `BrainView.ts` layout rebuild | Rebuilds (tier change / data refresh) re-run the settle camera animation every time | Guard settle by layout identity (hash of node-id set + tier); skip on no-change rebuild |

## Implementation Units (wave-2)

### U6. Chatboard API + view-model contract (F1, F2)
Files: `website/src/api.ts`, `website/src/components/HeyPanel.tsx`, `website/src/components/chatboard/chatboard-mapping.ts`, `website/src/components/chatboard/Chatboard.tsx`, `chatboard-mapping.test.ts`, `website/src/components/INDEX.md`, `website/src/components/chatboard/INDEX.md`.
Approach: read `unread()`'s return shape in `hey.rs` first; add the 3 wrappers + types; build reactionIndex in HeyPanel from already-fetched reaction rows; align mapping/Chatboard VM consumption to real fields; extend mapping tests (reactionIndex, presence coerce, unread-fed threads, phantom-field regression: feed a raw HeyMessage-shaped object, assert no NaN/undefined leaks).
### U7. Chrome + interaction fixes (F3, F4, F5, F6, F7)
Files: `Chatboard.tsx`, `HeyPanel.tsx`, `website/src/styles.css` (margin dedupe, chrome family only), `website/src/brain/BrainView.ts` (F5 clamp, F7 settle guard), `website/package.json` (F6), adjacent test seam only if a clamped selector is extracted as a pure function (otherwise covered by existing layout/labels tests + smoke).
Approach: smallest diffs; zero changes to cortex doctrine, triad, MAX_EDGES (1200), labels LOD math, grouping derivation.
### U8. Verification pass (R7, R8)
Gates, all green required before ship: `node --experimental-strip-types --test <F6-widened globs>` · `npx tsc --noEmit` (website/) · `scripts/build-ui.sh` · `cargo test --locked` (expected untouched-green; failure = stop + surface) · `python3 scripts/audit-agent-index.py` · daemon smoke: `kurultai daemon --port 8421` → `/ui/` renders brain + ontology + chatboard against the dogfood store (send + react round-trip exercised via curl against the same endpoints the UI calls).
### U9. Ship (per standing user pipeline)
ce-simplify-code pass on the U6+U7 diff (inline; expected near-trivial) → final inline report-only re-review against this plan (**substitution disclosed: PRO mode, same context — not an independent reviewer**) → residual: any unapplied finding → tracker ticket + `docs/residual-review-findings/feat-brain-first-principles-wave1.md` → INDEX ritual (every touched folder) → `ce-commit-push-pr` (checkpoint before push) → `ce-babysit-pr` (budget: 3 fix rounds; never merge red; post-merge main run must be green before closeout).

## Verification
Same gate list as U8 + PR CI green + merged-green confirmation on `main` before closeout (house rule: never merge red; CI verdict from the primary log only).

## Assumptions (recorded per pipeline-mode scoping)
- No Rust changes in wave-2; if U6 reveals the unread shape or an auth nuance requires one, **stop and surface** — a backend diff reopens the green Rust gate without blessing.
- `instance_id` present in dogfood data for two-layer identity (`codename@instance`); if absent, identity falls back to codename-only (already handled by `identityLabel`).
- CI for this repo mirrors the local gates (tsc/node tests/build/cargo/audit); babysit treats divergence as a real failure to repair, not a flake.
- Budget: $0 Astra spend expected for wave-2 (native authoring per W1).
