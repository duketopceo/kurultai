---
title: "U3 Docs drift correction - Plan"
type: docs
date: 2026-10-23
artifact_contract: ce-unified-plan/v1
product_contract_source: ce-plan-bootstrap
execution: code
depth: standard
origin: docs/plans/2026-10-05-001-docs-program-roadmap-plan.md (unit U3)
---

# U3 Docs drift correction - Plan

## Goal Capsule

- **Objective:** Top-level docs on `main` match live repo evidence (MCP tool count, issue open/closed state, supported crate version, Brain ontology gate wording).
- **Authority:** Program unit U3 in `docs/plans/2026-10-05-001-docs-program-roadmap-plan.md`; overlay order in `docs/plans/2026-10-10-001-docs-program-overlay-plan.md` (U3 before U4).
- **Execution profile:** Documentation-only edits + agent-index ritual. No runtime behavior change.
- **Stop conditions:** Do not reopen closed issues; do not invent substitute roadmap items without evidence; do not change Brain visuals.
- **Baseline:** `origin/main` at 6abbc95, crate v0.7.0, researched 2026-10-23.

## Product Contract

### Summary

Several top-level docs still describe an older surface: "8 MCP tools", closed issues #101/#102 listed as Stage 1 open work, `SECURITY.md` supported version `v0.1.0`, and a Brain gate that still points at a pending 3D ontology layout despite Flowsint board #319.

### Requirements

- R1. MCP tool count and names in `ROADMAP.md`, `AGENTS.md`, and `README.md` match the 16 `TOOL_*` constants in `src/mcp/server.rs`.
- R2. `ROADMAP.md` Stage 1 open table does not list closed issues #101 or #102 as open work; note supersession/deferral briefly if residual work remains.
- R3. `SECURITY.md` Supported Versions reflects crate `v0.7.0` (main + tagged releases).
- R4. `ROADMAP.md` Brain gate / Stage 3 wording acknowledges Flowsint 2D board (#319) as the shipped ontology UI path; Slice C 3D hierarchy remains deferred / superseded.
- R5. Agent-index ritual: update `INDEX.md` rows + Recent for every touched indexed file; `python3 scripts/audit-agent-index.py` exits 0.

### Success Criteria

- Grep for `8 MCP` / `provides 8 tools` / `v0.1.0` in the corrected files returns no stale hits.
- `gh issue view 101,102 --json state` stay CLOSED and are not presented as open Stage 1 work.
- Audit script green.

### Scope Boundaries

**In scope:** `ROADMAP.md`, `AGENTS.md`, `SECURITY.md`, `README.md`, root `INDEX.md` (and any parent index rows the ritual requires).

**Out of scope:** U4 plan frontmatter hygiene; closing/relabeling GitHub issues; Brain UI code; #326/#323 implementation.

## Planning Contract

### Key Technical Decisions

- KTD1. **Canonical tool list = `src/mcp/server.rs` `TOOL_*` constants (16).** session-settled: user-directed via program roadmap U3 evidence. Rejected: hardcoding a shorter "core" list without saying so.
- KTD2. **Remove #101/#102 from Stage 1 open table; do not reopen.** Closed 2026-09-01 as superseded/deferred. Residual GlitchTip depth mentioned on #102 may stay in Cross-cutting or residual Wave B only if already present elsewhere — do not invent a new issue row.
- KTD3. **SECURITY supported version → v0.7.0.** Patches still land on `main` and future releases.
- KTD4. **Brain gate:** keep link to brain-shape plan for history; state Flowsint board shipped (#319) and ontology *layout mode* remains gated on O1 primitives for any future typed hierarchy — no Brain visual redesign.

### Assumptions

- Program roadmap U3 evidence still holds on 2026-10-23 (verified: ROADMAP/AGENTS/SECURITY still drift; #101/#102 CLOSED).
- #418 README label fix already merged; remaining README drift is the Agents MCP tool list.

### Risks

| Risk | Mitigation |
|------|------------|
| Over-claiming #102 work as fully shipped | Wording: closed/superseded, not "metrics complete" |
| INDEX ritual miss | Run `scripts/audit-agent-index.py` before push |

## Implementation Units

### IU1. Correct drift in top-level docs

- **Files:** `ROADMAP.md`, `AGENTS.md`, `SECURITY.md`, `README.md`, `INDEX.md`
- **Approach:** Apply R1–R4 text edits; bump INDEX rows + Recent.
- **Test scenarios:** none (docs only). Verification = R5 audit + manual count of TOOL_* = 16.
- **Verification:** `python3 scripts/audit-agent-index.py`; `rg '8 MCP|provides 8 tools|v0\\.1\\.0' ROADMAP.md AGENTS.md SECURITY.md README.md` empty for those stale phrases.

## Verification Contract

- `python3 scripts/audit-agent-index.py` exits 0.
- Tool names listed match the 16 constants in `src/mcp/server.rs`.

## Definition of Done

- PR opened against `main` with U3 corrections.
- CI decided; no merge by agent.
