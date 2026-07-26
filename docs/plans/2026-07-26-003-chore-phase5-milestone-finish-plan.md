---
title: Phase 5 Milestone Finish - Plan
type: chore
date: 2026-07-26
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
origin: "User /lfg phase 5 — product exit shipped; finish Milestone 5 closeout land + remilestone handoff"
---

# Phase 5 Milestone Finish - Plan

## Goal Capsule

**Objective:** Finish **Phase 5** by making Milestone 5 closable: keep the closeout package merge-ready on [#97](https://github.com/duketopceo/kurultai/pull/97), tighten post-#98 README/closeout preflights, document the maintainer remilestone path for deferred ops, and attempt remilestone when write access exists.

**Authority:** This plan > [2026-07-26-002-chore-phase5-lfg-closeout-plan.md](2026-07-26-002-chore-phase5-lfg-closeout-plan.md) > [phase-5-complete.md](phase-5-complete.md) > shipped [#65](https://github.com/duketopceo/kurultai/pull/65)/[#66](https://github.com/duketopceo/kurultai/pull/66)/[#84](https://github.com/duketopceo/kurultai/pull/84)/[#83](https://github.com/duketopceo/kurultai/pull/83).

**Stop when:** Closeout branch current with `main`; README Phase 5 ✅ + `phase-4-complete` / `phase-5-complete` links preserved after #98 rewrite; closeout script preflights the ✅ row; `phase-5-closeout.md` names PR #97 as the landing vehicle; remilestone of #20/#29/#35 attempted (succeeds or durable 403 handoff); green PR.

**Do not:** Implement [#20](https://github.com/duketopceo/kurultai/issues/20) ARC, [#29](https://github.com/duketopceo/kurultai/issues/29) env hardening, or [#35](https://github.com/duketopceo/kurultai/issues/35) GlitchTip; rebuild Phase 5 product features; start Phase 6 launch epic (#10) beyond noting yurt/#22 is separate; open a duplicate closeout PR that fights #97.

**Assumption:** Phase 5 **product** exit is already on `main`. Remaining Phase 5 work is **milestone hygiene** already drafted on `cursor/phase5-lfg-closeout-7a74` (#97). This LFG finishes that closeout, not a product rebuild.

**Product Contract preservation:** new bootstrap (finish slice on top of prior closeout plan).

**Execution profile:** Docs/script/README only on the existing closeout branch; open/update PR via LFG; skip simplify/browser when docs-only.

**Tail ownership:** LFG / `ce-work` on #97; maintainer merges #97, runs `./scripts/phase-5-closeout.sh`, closes Milestone 5 (agent often lacks issue/milestone write).

---

## Product Contract

### Summary

Milestone 5 is still **OPEN** with three deferred ops issues (#20, #29, #35). Product PRs are merged; umbrella [#9](https://github.com/duketopceo/kurultai/issues/9) is closed. Closeout artifacts live on draft/ready PR [#97](https://github.com/duketopceo/kurultai/pull/97) but **not** on `main` yet. After [#98](https://github.com/duketopceo/kurultai/pull/98) rewrote the README, #97 merged `main` and restored the Phase roadmap table with Phase 5 complete/closeout links. Agent tokens still get **403** on issue milestone PATCH.

### Problem Frame

Without landing #97 and remilestoning deferred ops, Milestone 5 stays open indefinitely even though product Phase 5 is done — tracker lag, not missing product code.

### Requirements

- R1. Continue on `cursor/phase5-lfg-closeout-7a74` / [#97](https://github.com/duketopceo/kurultai/pull/97) — do not open a competing closeout PR.
- R2. README on the closeout branch keeps Phase 5 ✅ with Markdown links to `docs/plans/phase-5-complete.md` and `docs/plans/phase-5-closeout.md`, plus `docs/plans/phase-4-complete.md`, without undoing the v0.3.0 README rewrite from #98.
- R3. `scripts/phase-5-closeout.sh` preflights that README on `main` has a Phase 5 ✅ row (or equivalent product-exit signal) in addition to the existing `phase-5-complete.md` / `phase-4-complete.md` link checks.
- R4. `docs/plans/phase-5-closeout.md` names [#97](https://github.com/duketopceo/kurultai/pull/97) as the landing PR and states the post-merge maintainer sequence (merge → script → close M5).
- R5. Attempt remilestone of #20/#29/#35 to Milestone 6 once; on 403, leave durable handoff in closeout docs (no fake success).
- R6. Do not implement ARC / GlitchTip / env hardening in this PR.

### Actors

- A1. Maintainer — merge #97, run closeout script, close Milestone 5
- A2. LFG agent — finish docs/script on #97, attempt remilestone, ship PR updates
- A3. CI — green on closeout branch

### Flows

- F1. Finish closeout package on #97 → green PR
- F2. Maintainer merges #97 → artifacts on `main`
- F3. Maintainer runs `./scripts/phase-5-closeout.sh` → #20/#29/#35 on Milestone 6
- F4. Maintainer closes Milestone 5

### Acceptance examples

- AE1. `rg -n 'phase-5-complete\.md' README.md` matches on the closeout branch
- AE2. `rg -n 'Phase 5.*✅|✅.*[Pp]hase 5' README.md` (or table row with ✅ and phase-5-complete link) matches
- AE3. Script contains a preflight for Phase 5 ✅ / product-exit marker on `main` README
- AE4. `phase-5-closeout.md` mentions pull/97 or `#97`
- AE5. Remilestone either succeeds (issues on M6) or closeout docs say agent 403 / maintainer must run script

### Scope boundaries

**In scope:** closeout branch docs/script/README finish; remilestone attempt; PR update on #97.

**Out of scope:** implementing #20/#29/#35; Phase 6 launch (#10); fighting [#99](https://github.com/duketopceo/kurultai/pull/99)/[#100](https://github.com/duketopceo/kurultai/pull/100).

---

## Planning Contract

### Key Technical Decisions

- KTD1. **Reuse #97, do not fork closeout.**  
  Decision: all finish work stays on `cursor/phase5-lfg-closeout-7a74`.  
  Provenance: `user-directed` (prior LFG already opened #97 for Phase 5 closeout).  
  Rejected: new competing PR with duplicate `phase-5-*.md` — causes merge thrash.  
  Reason: single landing vehicle for Milestone 5 hygiene.

- KTD2. **Deferred ops remain open; remilestone to M6 only.**  
  Decision: #20/#29/#35 are not Phase 5 product blockers; do not implement them here.  
  Provenance: `user-approved` (prior Phase 5 LFG closeout).  
  Rejected: implement ARC/GlitchTip/env hardening under this LFG.  
  Reason: product exit already shipped; ops are Milestone 6 stretch.

- KTD3. **Preserve #98 README structure; only keep Phase roadmap table.**  
  Decision: mechanical merge already did this; finish pass must not restore pre-#98 mega-README.  
  Provenance: `user-approved` (babysit conflict resolution).  
  Rejected: revert README to old Phase 5 draft prose.  
  Reason: v0.3.0 rewrite is canonical on `main`.

- KTD4. **403 remilestone is a durable handoff, not a soft pass.**  
  Decision: attempt once; on failure document in closeout — Definition of Done for agent stops at “handoff ready,” not “M5 closed.”  
  Provenance: `user-approved` (agent token limits known).  
  Rejected: claim Milestone 5 closed without write access.  
  Reason: fail loud.

### Technical design

1. Edit `scripts/phase-5-closeout.sh` README preflight to require a Phase 5 ✅ signal (grep for `phase-5-complete` link already present; add `✅` near Phase 5 / complete link).
2. Update `docs/plans/phase-5-closeout.md` with PR #97 + sequence.
3. Touch `docs/plans/phase-5-complete.md` only if Tracker closeout section needs #97 pointer.
4. `gh api` PATCH milestone once for #20/#29/#35; record outcome in commit message / closeout doc if failed.

### Assumptions

- A1. #97 remains open and pushable.
- A2. Maintainer will merge and run the script.
- A3. Cloud agent still cannot close Milestone 5.

### Sequencing

U1 (docs) → U2 (script preflight) → U3 (remilestone attempt) → verify → PR update.

---

## Implementation Units

### U1. Closeout docs name landing PR #97

**Goal:** Maintainer knows which PR to merge and what to run after.

**Files:** `docs/plans/phase-5-closeout.md`, optionally `docs/plans/phase-5-complete.md`

**Approach:** Add a short “Landing vehicle” section pointing at #97; keep remilestone/close commands.

**Tests / scenarios:** none (docs). Manual: `rg -n '#97|pull/97' docs/plans/phase-5-closeout.md`

**Verify:** AE4

### U2. Script preflight Phase 5 ✅ on main README

**Goal:** Closeout script refuses to remilestone if README product-exit signal is missing after merge.

**Files:** `scripts/phase-5-closeout.sh`

**Approach:** After existing link greps, require README text that ties Phase 5 to ✅ (e.g. line containing `phase-5-complete` and `✅`, or `Phase 5` + `✅`).

**Tests / scenarios:**
- S1. Script `bash -n` clean
- S2. Local dry-read of the new grep against current branch README succeeds
- S3. Failure message names the missing ✅ signal

**Verify:** AE2, AE3; `bash -n scripts/phase-5-closeout.sh`

### U3. Remilestone attempt + durable 403 handoff

**Goal:** Clear M5 if permitted; otherwise leave explicit maintainer step.

**Files:** `docs/plans/phase-5-closeout.md` (only if 403 needs a “last attempted” note — keep lean)

**Approach:** Run the three milestone PATCHes; on 403, do not invent success. Ensure closeout doc already says maintainer runs the script.

**Verify:** AE5

---

## Verification Contract

```bash
rg -n 'phase-5-complete\.md' README.md
rg -n 'phase-4-complete\.md' README.md
rg -n '✅' README.md
rg -n '#97|pull/97' docs/plans/phase-5-closeout.md
bash -n scripts/phase-5-closeout.sh
test -x scripts/phase-5-closeout.sh
# optional: cargo test --locked  (docs-only; CI covers)
```

---

## Definition of Done

**Global**

- [ ] U1–U3 complete on `cursor/phase5-lfg-closeout-7a74`
- [ ] README Phase 5 ✅ + phase-4/5 complete links intact
- [ ] Script preflights ✅ / complete links
- [ ] Remilestone attempted; success or documented maintainer path
- [ ] PR #97 updated and CI green

**Per unit**

- U1: closeout doc references #97
- U2: script greps Phase 5 ✅ signal
- U3: AE5 satisfied

**Maintainer after merge (not agent DoD)**

- [ ] `./scripts/phase-5-closeout.sh`
- [ ] `gh api -X PATCH repos/duketopceo/kurultai/milestones/5 -f state=closed`
