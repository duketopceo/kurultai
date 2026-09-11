---
title: "feat: dogfood retrieval sequester, agent board slice, non-Brain UI cleanup"
date: 2026-09-04
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "LFG 2026-09-04 — sequester pond from hot retrieval, ship agent message board MCP/REST slice, clean non-Brain chrome; preserve Brain; do not break dogfood"
origin: "Conversation audit of duketopceo/kurultai + kurultai-private; requirements-only docs/plans/2026-09-03-001-feat-agent-message-board-plan.md; live MCP flood evidence"
depth: standard
---

# feat: dogfood retrieval sequester, agent board slice, non-Brain UI cleanup

**Target repo:** `duketopceo/kurultai` (public). Ops deploy under `deploy/` stays private-only and is out of this PR.
**Base:** `origin/main` (v0.5.0), folding already-landed private commits for store/CLI auth where they belong in product code — not the server-001 stack.
**Process:** PR-only. Surgical. Do not rewrite Brain visualization.

## Goal Capsule

**Objective:** Make the hosted/dogfood Brain trustworthy again by keeping session-log (pond) noise out of default hot retrieval; give agents a real `hey` message board via MCP + REST on top of the existing SQLite store; tidy non-Brain UI chrome so secondary surfaces stay clean without competing with the Brain.

**Authority:** This plan > session-settled decisions below > `docs/plans/2026-09-03-001-feat-agent-message-board-plan.md` (board product contract) > `AGENTS.md` Brain doctrine > `docs/solutions/architecture-patterns/trust-lanes-quality-gate.md`.

**Stop when:**
- Unscoped `search` / `ask` / `who_knows` / `recall` default-exclude `source=pond` (and optionally other configured noisy sources); pin `source=pond` still works.
- Search/ask hits from excluded or low-trust sources do not `touch_access` (no reheating).
- Diversify (or hard exclude) applies consistently on ask/who_knows paths, not only `search_scoped`.
- Agents can register (CLI already), post, list threads, read history, react, and poll unread via MCP tools + `/api/hey/*` REST using the existing store tables.
- A thin board panel exists in `website/` without changing BrainStage / RepoBrain / synaptic graph behavior.
- `web/` marketing homepage is clearer and less noisy (typography/spacing/contrast only) — no Clerk rewrite, no Three.js Brain changes.
- Tests cover retrieval exclusion, board post/cap, and MCP tool registration; existing Brain UI tests still pass.

**Do not:**
- Redesign or “optimize” Brain synaptic visualization, camera, or layouts (`AGENTS.md`: ask first; user likes Brain).
- Vendor MengTo/ThreeUI or `@designcodeio/threeui` into the Brain (Brain already uses three.js + 3d-force-graph; ThreeUI is reference for secondary polish only).
- Ship full board v1 from the Sept 3 plan in one PR (webhooks/SSRF, atom mirror job, turn-cap owner override UI, federation).
- Push `deploy/server-001` secrets/stack into the public repo PR.
- Bulk-delete hosted pond atoms from this PR (ops note only; code enables sequester + optional quarantine ingest).

## Assumptions

- Inferred: default noisy-source denylist starts as `pond`; config can extend the list later.
- Inferred: board v1 slice uses poll (`hey_poll` / `GET /api/hey/unread`) without outbound webhooks.
- Inferred: non-Brain UI cleanup targets `website/` chrome (TopBar / CommandStrip / new Hey panel) and `web/` homepage — not `ui/` Brain embed behavior beyond rebuild if board assets need embedding.
- Working tree may have WIP UI asset churn; implementer starts a clean feature branch and only keeps diffs that serve these units.

## Product Contract

### Summary

Kurultai’s product is assemble-what-you-know retrieval plus agent coordination. Dogfood shows pond session transcripts flooding default search and keeping themselves hot via access bumps. Diversify-by-source is insufficient. Agents still coordinate via root `hey.md` markdown because MCP board tools are missing despite store/CLI scaffolding on private. Brain UI quality is already good; secondary surfaces and retrieval correctness need the help.

### Requirements

| ID | Requirement | Origin |
|----|-------------|--------|
| R1 | Default unscoped retrieval excludes pond (and configurable noisy sources); explicit `source=` pin remains. | user-directed |
| R2 | Excluded / quarantine / denylisted hits must not bump `last_accessed_at`. | user-directed (hot/medium/cold) |
| R3 | `ask` and `who_knows` use the same source policy as search (exclude + diversify when multi-source). | audit |
| R4 | Optional connector config: pond (or noisy sources) may ingest as `quarantine` by default so they never enter trusted hot without promote. | user-directed sequester |
| R5 | Agent message board: MCP tools for post, list threads, read, react, poll unread; REST under `/api/hey/...`. | user-directed + Sept 3 plan R6 |
| R6 | Use existing agents/threads/messages tables and turn cap on reply; reactions do not consume turns. | Sept 3 plan R4–R5; private store commits |
| R7 | Thin Hey board UI in `website/` — list/read/post — Brain remains focal; no parallel dashboard. | user + AGENTS.md |
| R8 | Non-Brain polish: reduce chrome clutter on secondary panels / `web/` homepage; match existing dark tool aesthetic; do not restyle Brain graph. | user; ui-ux-pro-max + ThreeUI as *reference only* |
| R9 | Do not break existing trusted search, Brain graph endpoints, or daemon embed path. | user “dont bread it” |
| R10 | Public PR must not require private deploy files. | audit |

### Actors

- A1. Agent (MCP client) — primary board author and retrieval consumer.
- A2. Owner — `kurultai agent add`, config for noisy sources, promote quarantined pond if needed.
- A3. Human Brain visitor — uses Brain unchanged; may open Hey panel.

### Acceptance examples

- AE1. Unscoped search for a generic topic returns notes/notion/agent hits before any pond tool-call atom; `source=pond` still returns pond.
- AE2. After unscoped search that would have included pond, pond atoms’ `last_accessed_at` are unchanged.
- AE3. Two agents with tokens post on `hey.md` thread via MCP; third polls unread and sees the posts; turn cap rejects endless reply loops.
- AE4. BrainStage visual regression smoke unchanged; Hey panel does not overlay graph controls.
- AE5. `web/` homepage remains Clerk-signed-in capable with clearer hierarchy and no new purple-gradient/AI-slop theme.

## Planning Contract

### Key Technical Decisions

- KTD1. session-settled: **Hard default-exclude pond from unscoped retrieval**, not diversify-only. Provenance: user-directed. Rejected: rely on `MAX_HITS_PER_SOURCE=3` overflow fill. Reason: live MCP still majority-pond; overflow backfills pond.
- KTD2. session-settled: **Do not change Brain synaptic UI** this PR. Provenance: user-directed + AGENTS.md. Rejected: ThreeUI/Brain redesign. Reason: Brain is liked; UI is not the product point.
- KTD3. session-settled: **Ship board MCP/REST vertical slice now**; defer webhooks, atom mirror job, owner rename UI. Provenance: user-directed board need + “dont bread it”. Rejected: full Sept 3 stop-when in one PR. Reason: store/CLI already exist; agents need tools.
- KTD4. **Noisy-source policy** lives in runtime config (e.g. `runtime.noisy_sources = ["pond"]`) applied in `Brain::search_scoped_hub` / ask / who_knows / recall before diversify; pin `source` bypasses denylist for that source only. Rejected: hardcode only in MCP layer. Reason: HTTP `/api/search` must match MCP.
- KTD5. **touch_access only for returned trusted, non-denylisted atoms**. Rejected: touch all hybrid candidates. Reason: reheating is how pond stays hot.
- KTD6. **Pond ingest option** `extra.trust_lane = "quarantine"` (or source-level `default_trust_lane`) so new pond rows never trusted without promote — complements retrieval exclude for already-indexed rows. Rejected: delete all pond. Reason: surgical; ops can promote rare useful sessions.
- KTD7. **Board API surface** MCP-first names: `hey_post`, `hey_threads`, `hey_read`, `hey_react`, `hey_poll` (+ REST mirrors). Auth: existing agent API key Bearer. Rejected: markdown-file writes as SoT. Reason: durable store already landed.
- KTD8. **UI guidance:** use ui-ux-pro-max principles (contrast, focus, reduce chrome, hover/focus) and ThreeUI catalog only as taste reference for secondary panels; **do not add ThreeUI dependency**. Match existing `website` CSS variables; avoid Inter / purple-gradient / cream-terracotta AI defaults. Rejected: new design system install. Reason: don’t bread Brain embed + keep diff small.
- KTD9. **Branching:** feature branch from `origin/main`; include product commits from private (store/CLI/auth/UI token) that are not `deploy/`; leave deploy on private. Rejected: force-push deploy into public. Reason: public MIT repo hygiene.

### Conflict call-outs

- Sept 3 board plan stop-when includes webhooks + atom mirror — deferred explicitly (KTD3); residual tracked as follow-up, not failure of this PR.
- Pro Max suggested Inter + green CTA; repo + user frontend rules prefer expressive non-Inter and avoid generic AI themes — follow existing Kurultai tokens instead.

### Risks

| Risk | Mitigation |
|------|------------|
| Hosted brains still full of trusted pond until reindex/ops | Code exclude + document `promote`/`UPDATE trust_lane` ops; optional quarantine ingest going forward |
| Board store incomplete on postgres hub | Solo SQLite first; postgres stubs already return “not implemented” — keep that until hub needs board |
| Dirty local `ui/` asset churn | Clean branch; rebuild UI only when website board panel needs embed |
| Breaking Brain embed | No edits to `BrainStage.tsx` / graph shaders; visual smoke via existing acceptance |

### Sequencing

1. U1 retrieval sequester (unblocks dogfood trust)
2. U2 board HTTP + MCP (depends on existing store on branch)
3. U3 thin Hey UI + light `web/` polish (depends on U2 API)
4. U4 docs/ops notes + index plan row (parallel-safe after U1–U3)

## Implementation Units

### U1. Retrieval noisy-source sequester

**Goal:** Pond cannot occupy default hot retrieval or reheating.

**Files:** `src/mcp/brain.rs`, `src/types.rs` and/or `src/config/loader.rs`, `config.example.toml`, `src/connectors/pond.rs` or pipeline trust default, tests under `src/mcp/brain.rs` / `tests/retrieval_hybrid.rs`.

**Requirements:** R1–R4, R9.

**Approach:** Add configurable noisy-source denylist (default `["pond"]`). Apply after hybrid fetch when `source` pin is absent; when pin is present, allow that source. Apply same filter in ask/who_knows. Skip `touch_access_many` for denylisted atoms. Optional pond `default_trust_lane=quarantine` via source extra. Strengthen diversify: do not overflow-fill from denylisted sources (or drop overflow from noisy sources).

**Test scenarios:**
- Unscoped search with pond+notes fixtures → no pond in results; notes present.
- `source=pond` → pond returned.
- Ask path excludes pond without pin.
- touch_access not updated for excluded pond atom after unscoped search.
- Quarantine ingest path for pond config (unit or connector test).

### U2. Agent message board MCP + REST slice

**Goal:** Agents can message each other on the daemon without markdown SoT.

**Files:** `src/http/mod.rs` (or `src/http/hey.rs`), `src/mcp/server.rs`, `src/store/mod.rs` (wire existing methods), `src/main.rs` if needed, `tests/` HTTP + MCP.

**Requirements:** R5–R6, R9–R10.

**Approach:** Mount `/api/hey/threads`, `/api/hey/threads/{id}/messages`, POST message/react, GET unread. MCP tools call the same handlers/brain facade. Auth via agent Bearer already started on private. Default thread name `hey.md` lazy-create. Enforce turn cap on reply; reactions free. No webhooks in this unit.

**Test scenarios:**
- Register agent → post → list → read round-trip.
- Reaction does not increment turns_used.
- Reply beyond turn_cap returns error.
- Unauthorized request 401.
- MCP tools/list includes hey_* tools.

### U3. Non-Brain Hey panel + light marketing cleanup

**Goal:** Humans can see the board; secondary chrome stays clean; Brain untouched.

**Files:** `website/src/components/*` (new HeyPanel or similar), `website/src/App.tsx` / `main.tsx` / `api.ts` / `styles.css` as needed, `web/src/app/page.tsx` + `globals.css` light touch, rebuild `ui/` only if embed requires it via existing `scripts/build-ui.sh`.

**Requirements:** R7–R9.

**Approach:** Add a collapsible/side Hey panel or command-strip entry that lists threads and messages using `/api/hey/*`. Do not modify BrainStage graph code. Polish `web/` hero: clearer hierarchy, contrast, focus rings, less clutter — keep Clerk. Reference ThreeUI/Pro Max for restraint only.

**Test scenarios:**
- Frontend unit/test if present for api client hey methods.
- Manual/acceptance: `/ui/` loads; graph still renders; Hey panel fetch mocked or daemon test.
- Prefer existing `website` tests / `repoLattice` pattern over new heavy E2E.

### U4. Plan index + agent skill pointer

**Goal:** Discoverability without claiming full Sept 3 board done.

**Files:** `docs/plans/INDEX.md`, optionally `skills/hey-board/SKILL.md` note that MCP board supersedes file for daemon-connected agents when tools available.

**Requirements:** R10.

**Test scenarios:** doc-only — link check in review.

## Verification Contract

- `cargo test` focusing retrieval + store message board + HTTP hey routes.
- `cargo test` existing brain/http acceptance that still applies.
- `website` / `web` lint or existing npm test if touched.
- Manual: unscoped MCP search on a fixture DB with pond+notes shows notes-first policy.
- Do not require hosted production reindex for CI green; document ops follow-up for `knowledge.shippedit.dev`.

## Definition of Done

- All U1–U3 behavior covered by automated tests listed above.
- Brain graph files unchanged (or only incidental import if unavoidable — prefer zero).
- Public PR opened without `deploy/server-001` private ops as required content.
- Sept 3 deferred items (webhooks, atom mirror, owner rename) listed in PR as follow-ups, not silent drops.
- No false claim that hosted pond is deleted — only that software sequesters it.

## Appendix

### External references (non-dependencies)

- [MengTo/threeui](https://github.com/MengTo/threeui) — Community Three.js UI catalog; **do not vendor into Brain**.
- [nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) — design intelligence; prefer contrast/focus/clutter rules over suggested Inter/green tokens when they conflict with repo taste.

### Live evidence (2026-09-04)

- MCP personal bridges to `knowledge.shippedit.dev`; local `~/.local/share/kurultai/dev` nearly empty.
- Unscoped search majority pond tool-call atoms, `trust_lane: trusted`.
- `diversify_by_source` caps then overflow-fills; ask/who_knows skip diversify.
- Private ahead of public by store/CLI/auth/deploy commits; installed binary `0.5.0` lacks `agent` subcommand.

### Work relationships

<!-- ce-section: work-relationships -->

This plan owns dogfood retrieval sequester + board MCP slice + non-Brain chrome. Separate future plans: board webhooks/SSRF (Sept 3 remainder), pond bulk quarantine ops on hosted, Brain layout ontology (existing Brain plans), hub postgres board support.
