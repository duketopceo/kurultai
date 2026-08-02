---
title: "chore: Phase 6 next work orders (post–v0.4.0)"
date: 2026-07-30
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: docs
authority: "User — git pull; plan next phase work orders then LFG"
depth: standard
origin: "Post Brain UI v0.4.0; scale/ontology discussion; Devin Year-1 PR #123; issues #104/#111–#122"
---

# chore: Phase 6 next work orders (post–v0.4.0)

**Target repo:** `duketopceo/kurultai`  
**Audience:** maintainer + LFG agents  
**Base:** `main` @ v0.4.0  
**Process:** PR-only (docs)

## Goal Capsule

Publish an ordered **Phase 6 Wave B+** work-order pack so the next `/lfg` has a single clear first slice (**MCP HTTP/SSE #104**) and a backlog mapped to existing issues — without rewriting history or colliding with shipped **v0.4.0**.

**Stop when:** `docs/plans/phase-6-work-orders.md` + first-slice plan `2026-07-30-002-feat-phase6-mcp-http-sse-plan.md` on a green docs PR; README one-line pointer to the work-order pack.

**Do not:** Implement SSE/Postgres/ontology in this PR; merge Year-1 cashflow doc without version fix; claim Phase 6 complete; close Milestone 6.

**Assumption (LFG headless):** “Next phase” = continue **Phase 6** (Launch), Wave B foundation, not a new Phase 7 number. Solo SQLite stays; team Postgres is Wave C.

---

## Product Contract

### Summary

Work-order sequencing after Brain UI release: remote MCP reach → metrics/labels → team shared store → enterprise connectors → ontology.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Document Wave B–F work orders with issue links and LFG order. |
| R2 | Name **P6-1 / #104** as the recommended next `/lfg` target. |
| R3 | Call out v0.4.0 collision with Devin Year-1 “v0.4.0 Team” (renumber to v0.5.0). |
| R4 | Ship a thin **implementation-ready** plan for P6-1 SSE MCP (separate file). |
| R5 | README: one link under Docs/Roadmap to `phase-6-work-orders.md`. |

### Scope boundaries

**In:** docs under `docs/plans/`, README one-liner.

**Deferred:** code for #104; assigning GitHub milestones (token may 403); merging #123.

---

## Planning Contract

| Decision | Choice |
|----------|--------|
| First LFG | #104 MCP HTTP/SSE (not Postgres, not full ontology) |
| Versioning | Keep shipped v0.4.0; team shared store = v0.5.0 track |
| Year-1 PR #123 | Reference + renumber guidance; do not land unbroken in this PR |

---

## Units

### U1. Work-order pack

- **Files:** `docs/plans/phase-6-work-orders.md`
- **Verify:** Waves B–F present; P6-1 marked recommended; issue links resolve.

### U2. First-slice LFG plan

- **Files:** `docs/plans/2026-07-30-002-feat-phase6-mcp-http-sse-plan.md`
- **Verify:** Goal capsule, DoD, non-goals, test plan — enough for `/lfg` without re-planning.

### U3. README pointer

- **Files:** `README.md` (one additive line near roadmap)
- **Verify:** Link to `docs/plans/phase-6-work-orders.md`.

---

## Verification

- [ ] Docs-only PR; no Rust/UI churn
- [ ] `phase-6-work-orders.md` names P6-1 / #104 as next LFG
- [ ] SSE plan file exists and is implementation-scoped
- [ ] README links the pack
