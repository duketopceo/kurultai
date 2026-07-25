---
title: Phase 4 LFG Closeout - Plan
type: chore
date: 2026-07-25
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
origin: "User /lfg phase 4 — solo connectors shipped (#62/#63); remaining = tracker hygiene"
---

# Phase 4 LFG Closeout - Plan

## Goal Capsule

**Objective:** Finish Phase 4 under LFG by landing wrap + closeout docs/script, updating README to ✅ Phase 4 solo exit, and documenting maintainer commands to close [#8](https://github.com/duketopceo/kurultai/issues/8) and Milestone 4.

**Authority:** This plan > shipped [#62](https://github.com/duketopceo/kurultai/pull/62) / [#63](https://github.com/duketopceo/kurultai/pull/63) > [#8](https://github.com/duketopceo/kurultai/issues/8) / [#27](https://github.com/duketopceo/kurultai/issues/27).

**Stop when:** `phase-4-complete.md` + `phase-4-closeout.md` + `scripts/phase-4-closeout.sh` on a green PR; README marks Phase 4 complete with deferred Composio/plugins; maintainer can close #8 and Milestone 4.

**Do not:** Implement Composio, WASM/Python plugins (#14), CodeGraph, AppFlowy (#4), OpenRouter batch embed; start Phase 5 product work in this PR.

**Assumption:** Solo Phase 4 exit (markdown + Pond + Dayflow + GitHub FS) already shipped on `main`. LFG “phase 4” = **hygiene closeout**, not rebuild.

**Product Contract preservation:** new bootstrap.

---

## Product Contract

### Summary

Phase 4 first-wave connectors are on `main`. Umbrella [#8](https://github.com/duketopceo/kurultai/issues/8) and Milestone 4 still look open because Composio/plugins remain. Closeout documents the solo exit and defers the rest.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | `docs/plans/phase-4-complete.md` lists shipped PRs (#62, #63) and deferred items. |
| R2 | `docs/plans/phase-4-closeout.md` + `scripts/phase-4-closeout.sh` for maintainer tracker hygiene. |
| R3 | README: Phase 4 row ✅ with complete/closeout links; checklist Expansion marked done with deferred note. |
| R4 | PR CI green (docs/script only). |
| R5 | Script closes #8 (with deferred comment) and prints Milestone 4 close command. |

### Actors / flows

- A1 Maintainer · F1 merge closeout PR · F2 run script · F3 close Milestone 4

### Scope boundaries

**In:** Docs, script, README.  
**Out:** New connectors, coverage 60% hard gate, Phase 5.

### Acceptance examples

- AE1. `phase-4-complete.md` exists and links #62/#63.  
- AE2. Script is executable and references issue #8.  
- AE3. README Phase 4 shows complete, not 🚧.

---

## Planning Contract

### Assumptions

| Decision | Class | Rejected | Why |
|----------|-------|----------|-----|
| Closeout not Composio | LFG headless (phase 1/2 pattern) | Build Composio in this LFG | Solo exit already shipped |
| Close #8 on exit | inferred | Leave #8 open forever | Umbrella done; deferrals named in wrap |
| AppFlowy stays deferred | prior | Implement #4 | Non-blocking since Phase 1 |

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | Docs-only PR | Matches phase-1/2 closeout |
| KTD2 | Script is durable handoff | Agent often cannot `closeIssue` |
| KTD3 | Skip simplify/browser | Docs-only |

### Implementation Units

### U1. Unified plan (this file)

**Verify:** `implementation-ready` + `execution: code`.

### U2. Complete + closeout package

**Files:** `docs/plans/phase-4-complete.md`, `docs/plans/phase-4-closeout.md`, `scripts/phase-4-closeout.sh`, `README.md`

**Verify:** AE1–AE3; script executable.

### U3. PR + CI green

**Verify:** Lint & Test / macOS / audit pass.

---

## Verification Contract

```bash
test -x scripts/phase-4-closeout.sh
test -f docs/plans/phase-4-complete.md
rg -n 'phase-4-complete' README.md
cargo test --locked
cargo clippy --all-targets -- -D warnings
# Maintainer only:
# ./scripts/phase-4-closeout.sh
```

---

## Definition of Done

- [ ] Wrap + closeout docs/script on `main` via green PR  
- [ ] README Phase 4 ✅  
- [ ] Maintainer can close #8 + Milestone 4  
