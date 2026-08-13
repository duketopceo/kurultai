---
title: "chore: Phase 6 next queue — Wave B complete + Tiered Hub plans"
date: 2026-08-12
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: docs
authority: "User /lfg — next phase and new plan docs"
depth: standard
origin: "Wave B shipped (P6-1/2/3); docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md; milestone Tiered Access + Hosted Hub (#176–#181); phase-6-work-orders.md"
---

# chore: Phase 6 next queue — Wave B complete + Tiered Hub plans

**Target repo:** `duketopceo/kurultai`  
**Audience:** maintainer + LFG agents  
**Base:** `main` @ post–Wave B (MCP HTTP/SSE, thin metrics, soft labels, export/import, Brain UI)  
**Process:** PR-only (docs)

## Goal Capsule

**Objective:** Publish an updated Phase 6 work-order queue that marks **Wave B foundation complete**, points the next `/lfg` at **Tiered Access + Hosted Hub** (v0.5.0 track), and ships a thin **implementation-ready** first-slice plan — without inventing a Phase 7 number or closing backlog issues.

**Authority:** This plan > [phase-6-work-orders.md](phase-6-work-orders.md) > [YEAR-1-MILESTONES.md](YEAR-1-MILESTONES.md) > [docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md](../brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md) > issues [#176](https://github.com/duketopceo/kurultai/issues/176)–[#181](https://github.com/duketopceo/kurultai/issues/181).

**Stop when:** `phase-6-work-orders.md` reflects Wave B shipped; `phase-6-next-work-orders.md` names Wave G (Tiered Hub) order + recommended first LFG; first-slice plan for atom visibility scope exists; README roadmap line updated; docs PR green.

**Do not:** Implement Postgres/hub/RBAC code; claim Phase 6 or Milestone 6 complete; close or edit open feature issues; start multi-tenant SaaS framing (explicitly rejected in the brainstorm).

## Product Contract

### Summary

Wave B (remote MCP + thin metrics + soft labels) landed on `main`. The cashflow / team path is no longer blocked by missing SSE. Next product phase inside Launch = **Tiered Access + Hosted Hub**, aligning Year-1 **v0.5.0 Team** with milestone issues #176–#181.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Update Wave B rows in `phase-6-work-orders.md` to shipped status with PR/issue links. |
| R2 | Add a next-queue pack (`phase-6-next-work-orders.md`) for Wave G Tiered Hub + residual Wave B items. |
| R3 | Name **HUB-1 / #178** (atom visibility scope) as the recommended next `/lfg` code slice. |
| R4 | Ship an implementation-ready first-slice plan for HUB-1. |
| R5 | README: one roadmap line pointing at the next-queue pack (keep link to parent work orders). |

### Scope boundaries

**In:** docs under `docs/plans/`, README one/two additive lines.  
**Out:** code; GitHub milestone edits; closing #102/#101/#111 etc.

## Planning Contract

### Key Technical Decisions

- KTD1. **session-settled: Continue Phase 6 numbering — do not invent Phase 7.**  
  Provenance: `user-approved` (prior next-queue plan Assumption; “next phase” = next work queue).  
  Rejected: Phase 7 doc tree.  
  Reason: Milestone 6 + Year-1 v0.5.0 still open; hub is Launch → Team, not a new phase brand.

- KTD2. **session-settled: Docs-only PR for this LFG.**  
  Provenance: `user-directed` (“new plan docs”).  
  Rejected: Implementing hub/Postgres in the same PR.  
  Reason: Queue clarity before another large code LFG.

- KTD3. **Recommended next code LFG = #178 scope on atoms (SQLite-first), not #176 Postgres first.**  
  Provenance: `user-approved` (headless default from brainstorm R1 + AE1).  
  Rejected: Jumping straight to Postgres Store (#176) as first slice.  
  Reason: Visibility field unblocks connectors/ACL language without hub infra; solo path must not regress (AE1); #176 follows immediately as HUB-2.

- KTD4. **Wave C (P6-T*) maps into Wave G issue IDs — do not duplicate conflicting orders.**  
  Provenance: `user-approved` (headless).  
  Rejected: Two parallel Postgres work-order IDs.  
  Reason: #176 is the scoped Store work; #111 remains parent/legacy pointer.

### Assumptions

- A1. Thin metrics PR #126 satisfies the *thin* P6-2 slice; #102 stays open for GlitchTip-depth error tracking.
- A2. P6-4 Cloud Brain UI tunnel (#101) stays deferred relative to hub scopes (brainstorm excludes UI).
- A3. Atlas Wave E′ (#128–#135) stays after O1 / structured contract — not the next LFG.

## Implementation Units

### U1. Refresh parent work-order pack

- **Files:** `docs/plans/phase-6-work-orders.md`
- **Approach:** Mark P6-0…P6-3 (+ export/import) shipped; leave P6-4 open/deferred; point `/lfg` playbook at `phase-6-next-work-orders.md`.
- **Verify:** No stale “P6-1 recommended next”; Wave B table shows shipped links.

### U2. Next-queue pack (Wave G)

- **Files:** `docs/plans/phase-6-next-work-orders.md`
- **Approach:** Document Wave G HUB-1…HUB-6 with #176–#181; residual solo items; deferred list; `/lfg` playbook.
- **Verify:** HUB-1 marked recommended; brainstorm + milestone linked; no SaaS multi-tenant framing.

### U3. First-slice LFG plan (HUB-1)

- **Files:** `docs/plans/2026-08-12-002-feat-tiered-access-atom-scope-plan.md`
- **Approach:** Implementation-ready plan for #178 — `personal|team|company` on atoms; solo default; no hub required in this slice.
- **Verify:** Goal capsule, KTDs, units, verification, DoD — enough for `/lfg` without re-planning.

### U4. README pointer

- **Files:** `README.md`
- **Approach:** Update Phase 6 roadmap line to Wave B ✅ + link next-queue pack / Tiered Hub.
- **Verify:** Links resolve; does not claim Phase 6 complete.

## Verification Contract

- Manual: open each new/updated markdown file; confirm issue links use existing numbers.
- No Rust/CI behavior change expected (docs-only).
- `rg -n "Wave G|HUB-1|phase-6-next-work-orders" docs/plans README.md` finds the new queue.

## Definition of Done

- [ ] U1–U4 landed on a docs PR
- [ ] Recommended next `/lfg` is unambiguous (HUB-1 / #178)
- [ ] No feature issues closed or body-edited as part of this PR
- [ ] Phase 6 not marked complete
