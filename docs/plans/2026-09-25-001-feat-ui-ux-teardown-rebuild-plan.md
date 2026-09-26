---
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
---

# UI/UX teardown + rebuild — plan

**Date:** 2026-09-25 · **Milestone:** v0.6.5 candidate · **Trigger:** "UI besides the brain went in the wrong direction" + "teardown of ui — brain in general we keep but still teardown; research a nearly-already-made replacement; definitions of what we want; design via batch Kimi or GPT; I stitch it together."

## Goal Capsule

- **Objective:** replace the current dashboard chrome (light-lavender card theme bolted around a deep-black Brain) with a coherent dark-instrument UI, without touching data, APIs, or the Brain concept.
- **Means:** teardown inventory → written **definitions** per surface → research near-drop-in bases (**ThreeUI** recent catalog, **VoltAgent/VoltOps console**, **bolt.new** design tokens) → **batch-model design generation** (Kimi or GPT) into a parallel `ui-next` app → Devin stitches it into one coherent product → local preview → user approval → cutover in a tagged release (~v0.6.5).
- **Authority hierarchy:** user direction in this thread > `AGENTS.md` Learned User Preferences (Brain visuals are ask-first, protected) > this plan.
- **Stop conditions:** any change to Brain visual language (deep black, white/purple, synapse shimmer, hover-trace, whole-graph opening camera); any API/store/schema change; anything that loses telemetry hooks from #364.

## Product Contract

### Invariants (hard constraints from AGENTS.md + this thread)

- Brain remains the primary focal point — deep black, black/white + slight purple, electric shimmer, hover highlights connections, opens whole-graph-in-view, no extra chrome.
- Three surfaces survive conceptually: **Brain** (cortex hero), **Ontology** (Flowsint-style typed 2D board — never another organic 3D cloud), **Hey** (agent/human coordination board + kanban).
- "UI fun, not literal" — ThreeUI-style motion/CSS in chrome only; literal ontology stays in cortex + inspector.
- Hosted chrome must not say "LOCAL BRAIN"; must stay password-manager-friendly under Cloudflare Access.
- Tiered loading (hot/warm/cold) and the client perf reporter (`website/src/perf.ts`) carry forward.
- No data loss — pure frontend; all `/api/*` contracts unchanged.

### Current teardown scope (inventory to audit)

`website/src/` — App.tsx shell (TopBar → top-pill CommandStrip → brain-hero → floating-inspector → CommandRail[hey/repos/logs/settings] → MissionControl[pulse/focus/synthesize/ask] → footer); components: `BrainStage.tsx` (236), `OntologyBoard.tsx` (522), `Chatboard.tsx` (417), `HeyKanban.tsx` (200), `InspectorPanel.tsx` (280), `CommandStrip.tsx` (165), `HeyPanel.tsx` (159), `ProposalsPanel.tsx` (134), `DbView.tsx` (234), `HumanAccess.tsx` (210), `RepoBrain.tsx` (198), plus Activity/Ask/Logs/Settings/Stats/TopBar/MissionControl/CommandRail (~5.6k lines TSX + 2.2k-line `styles.css`).

### Requirements

- R1. One coherent dark design language across chrome + Brain — no pastel card theme fighting the black cortex.
- R2. Every component dispositioned: keep-reconcept / rebuild / delete — decided per-surface in the definitions doc, not mid-implementation.
- R3. Batch-generated design variants (Kimi or GPT) are candidates, not authority — Devin stitches/selects; user approves a local build before anything replaces `ui/`.
- R4. Functional parity minimum: tier switching, search+clear, layout switch (brain/ontology), inspector open/close, Hey thread ops (post/reply/react/edit/delete/thread-switch), kanban, proposals, ask, db view, settings, human-access page, mobile widths, low→max render modes.
- R5. Performance posture preserved or better: sprite-path caps, density shrink, telemetry emission — verified against `/api/metrics` `kurultai_client_*` series.

## Planning Contract

### Key technical decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Build replacement as a **parallel app** (`website/` gets a second entry — e.g. `ui-next.html` + `src/next/` tree), not in-place rewrites | User previews locally first; old UI stays intact until cutover; satisfies "no data loss / easy rollback" and the version-tag rule |
| D2 | Batch models generate **per-surface design variants + tokens**, checked into a scratch dir (`design-lab/` gitignored, or a sibling worktree) | Keeps generated slop out of the shipping tree; stitching is deliberate |
| D3 | Reuse `website/src/api.ts` + `perf.ts` verbatim in ui-next | API contracts frozen; telemetry must survive; zero backend work |
| D4 | Component primitive layer: adopt a tokens-first approach (CSS custom properties in a `tokens.css`), derived from bolt.new DESIGN.md spec (near-black canvas, azure/purple accent voltages, Inter/Inter Display, 12px/pill radii) remapped to Kurultai's white/purple palette | Bolt's spec is machine-readable and close to our aesthetic; avoids bespoke slop |
| D5 | Research before build: ThreeUI (recent: Cortexa, Orrery, Cadence, Meridian — dark editorial point-cloud/chrome), VoltAgent VoltOps console (MIT, agent-ops observability UI — nearest product analog), bolt.new repo styles | User named these; each answers a different question (visual language / layout patterns for agent ops / token system) |

### Definitions doc (the "what we want" artifact)

`docs/design/ui-definitions.md` — one section per surface: purpose, primary user action, visual treatment, motion, density, empty/loading/error states, mobile behavior, what dies. This doc is both the batch-model prompt seed and the acceptance spec for stitching.

### Batch pipeline (U3)

- Input: `ui-definitions.md` + teardown notes + token sheet → per-surface prompts.
- Models: **Kimi or GPT batch** (user's pick at run time; not Fable).
- Output contract per surface: self-contained React+CSS (or design spec + reference code), dark theme, no external fonts/CDN deps beyond what's already bundled.
- Devin scores variants against the definitions doc, picks per-surface winners, stitches.

### Sequencing

1. **U1** teardown + definitions (gate: user signs off on definitions)
2. **U2** research shortlist (parallel with U1)
3. **U3** batch generation (needs U1+U2)
4. **U4** stitch into `ui-next` app (needs U3 outputs)
5. **U5** local verification — functional pass, low/max tiers, mobile, FPS check (needs U4)
6. **U6** cutover PR: `ui/` embed swap + tag for rollback (needs user approval from U5)

## Implementation Units

| U | Unit | Files (repo-relative) | Size |
|---|------|------------------------|------|
| U1 | Teardown inventory + definitions doc | `docs/design/ui-definitions.md` (new), `docs/design/INDEX.md` (new) | M |
| U2 | Replacement research shortlist | `docs/design/research-bases.md` (new) | S |
| U3 | Batch design prompts + runner (Kimi/GPT, generates into `design-lab/`) | `design-lab/prompts/*.md`, `design-lab/README.md`, optional `scripts/ui-batch.mjs` | M |
| U4 | ui-next app shell + stitched components | `website/ui-next.html`, `website/src/next/**`, `website/src/next/tokens.css`, reuse `website/src/api.ts`, `website/src/perf.ts` | L |
| U5 | Local verify + parity checklist | `docs/design/parity-checklist.md`, Playwright screenshots under `design-lab/shots/` | M |
| U6 | Cutover: point `ui/` embed + daemon route at new bundle; tag `ui-v0.6.4-legacy` before swap | `ui/` rebuild, `src/http.rs` static route if path changes, version bump | S |

### Test scenarios (feature-bearing units)

- U4: every R4 flow works against a live local daemon (`timeout 300` per AGENTS.md); no console errors; `POST /api/metrics/client` still fires.
- U5: hot→max tier switch renders full cortex; ontology board expands/collapses classes; hey.md post+reply+react round-trip; inspector open/close; 390px and 1440px widths; FPS EMA on max ≥ current baseline (~11fps settle, watch for regression); CF-Access-served build has no "LOCAL BRAIN" copy.
- U6: embedded daemon serves new UI at `/ui/`; old bundle recoverable via tag.

## Verification Contract

- `cd website && npm test` (62 tests green today — keep green or update intentionally)
- `npm run build` in `website/` then embedded check via `timeout 300 ./target/debug/kurultai daemon --port 8421` + screenshot at `http://127.0.0.1:8421/ui/`
- `python3 scripts/audit-agent-index.py` green; INDEX.md rows for `docs/design/`, `design-lab/`, `website/src/next/`
- Parity checklist (U5) fully checked before cutover

## Definition of Done

- User has seen the local preview and approved the direction (explicit gate before U6)
- All surfaces dispositioned — nothing from the old UI survives by default, only by decision
- No API/store changes; no Brain visual-language change without separate approval
- Cutover shipped behind a tagged release; rollback = re-tag/re-build, not a revert PR

## Open questions

- ~~Which batch model~~ → **Kimi or GPT** (user: "kimi or gpt 6 sol batch only", decides at U3 run time).
- Design-lab output format preference (pure code vs spec+code) — defer to U3; default code-first.
- Whether `web/` (Next.js/Clerk team app) inherits the new tokens later — deferred, out of scope.

## Out of scope

- Backend/API/store changes; ontology O1 primitives (#116); `web/` app; connector work.
- Literal ThreeUI vendoring into BrainStage (AGENTS.md rule).
