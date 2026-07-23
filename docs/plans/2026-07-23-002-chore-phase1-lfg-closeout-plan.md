---
title: Phase 1 LFG Closeout - Plan
type: chore
date: 2026-07-23
deepened: 2026-07-23
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
origin: "User /lfg phase 1 — product already shipped; remaining tracker hygiene"
---

# Phase 1 LFG Closeout - Plan

## Goal Capsule

**Objective:** Finish Phase 1 under LFG by landing tracker closeout docs/script already drafted on [#56](https://github.com/duketopceo/kurultai/pull/56), verifying CI, and documenting maintainer commands to close shipped issues and Milestone 1.

**Authority:** This plan > [phase-1-complete.md](phase-1-complete.md) > [phase-1-closeout.md](phase-1-closeout.md) > [#42](https://github.com/duketopceo/kurultai/issues/42).

**Stop when:** Closeout docs + `scripts/phase-1-closeout.sh` on a green PR; README links Phase 1 complete + closeout; maintainer can close #5/#29/#40/#42/#25 and move #4/#23/#33 off Milestone 1.

**Do not:** Re-implement storage/embed/MCP; implement AppFlowy (#4); start Phase 2/3 product work in this PR.

**Assumption:** Product Phase 1 exit loop already shipped on `main`. LFG “phase 1” = **hygiene closeout**, not rebuild.

**Product Contract preservation:** unchanged.

---

## Product Contract

### Summary

Phase 1 product is done (`phase-1-complete.md`). Open Milestone 1 issues (#5, #4, #23, #33) and stale trackers (#42, #29, #40, #25) make the phase look unfinished. Closeout docs/script already exist on PR #56.

### Requirements

- R1. `docs/plans/phase-1-closeout.md` + `scripts/phase-1-closeout.sh` present and documented.
- R2. `phase-1-complete.md` points at closeout.
- R3. README links complete + closeout; AppFlowy marked deferred Expansion.
- R4. PR CI green (docs-only; no product regression).
- R5. Maintainer one-shot closes shipped issues / moves non-exit items / closes Milestone 1.

### Scope Boundaries

**In:** Docs, script, README.  
**Out:** New connectors, search, synthesis.

### Sources

- [phase-1-complete.md](phase-1-complete.md), [phase-1-work-orders.md](phase-1-work-orders.md), PR #56

---

## Planning Contract

### Key Technical Decisions

- KTD1. Prefer completing/shipping existing #56 branch over parallel closeout.
- KTD2. Agent cannot `closeIssue` — script is the durable handoff.
- KTD3. Docs-only → skip simplify/browser in LFG.

### Assumptions

- A1. No code behavior change required for closeout.
- A2. Headless LFG from “/lfg phase 1”.

---

## Implementation Units

### U1. Unified plan artifact (this file)

**Verify:** Frontmatter `implementation-ready` + `execution: code`.

### U2. Confirm closeout package on branch

**Files:** `docs/plans/phase-1-closeout.md`, `scripts/phase-1-closeout.sh`, `docs/plans/phase-1-complete.md`, `README.md`

**Verify:** Script executable; commands list #5 #29 #40 #42 #25; moves #4 #23 #33.

### U3. PR + CI green

**Verify:** Lint & Test / macOS / audit pass on closeout PR.

---

## Verification Contract

```bash
# Docs PR — still:
cargo test --locked   # no regressions if run
./scripts/phase-1-closeout.sh   # maintainer only (issue write)
```

---

## Definition of Done

- [x] Closeout docs/script on branch (#56)
- [ ] This plan committed; PR updated
- [x] CI green on #56
- [ ] Maintainer ran closeout script (external)
