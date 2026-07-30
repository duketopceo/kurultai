---
title: "feat: Phase 6 — MCP over HTTP/SSE for remote agents"
date: 2026-07-30
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "Phase 6 work orders P6-1 · GitHub #104 · user next /lfg"
depth: standard
origin: "phase-6-work-orders.md Wave B; Devin WO-102; remote agent / Perplexity-class integration"
---

# feat: Phase 6 — MCP over HTTP/SSE for remote agents

**Target repo:** `duketopceo/kurultai`  
**Audience:** solo + remote agents (team later)  
**Base:** `main` after work-order pack lands  
**Tracking:** [#104](https://github.com/duketopceo/kurultai/issues/104) · parent [#10](https://github.com/duketopceo/kurultai/issues/10)  
**Process:** PR-only

## Goal Capsule

Expose the existing MCP read tools (`search`, `ask`, `cite`, `who_knows`, and status-class reads as already defined) over **HTTP + SSE** on the daemon so remote agents can use Kurultai without stdio — while keeping **stdio MCP unchanged** and **localhost-first** security defaults.

**Stop when:** `kurultai daemon` serves an MCP HTTP/SSE endpoint; shared-secret (or equivalent) gate; read tools work end-to-end in a test; stdio path regressions none; README documents enablement; CI green.

**Do not:** Multi-tenant Postgres; Clerk on MCP; write tools (`remember` / promote) on the public HTTP surface in v1 of this slice (defer or require stronger auth); full Perplexity product integration; cloud tunnel UI (#101); plugin runtime.

**Assumption (LFG headless):** One transport slice — bind default `127.0.0.1`; opt-in flag or config to enable SSE MCP; reuse `BrainService` / existing tool handlers; prefer `rmcp` (or current MCP stack) streaming patterns already in-tree over a greenfield protocol.

---

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | HTTP/SSE MCP endpoint on daemon (path + config documented). |
| R2 | Auth: shared secret / bearer required when enabled; reject unauthenticated. |
| R3 | Read tools parity with stdio for the agreed tool set; same `AgentAtomView` token discipline (no full `content` by default). |
| R4 | Stdio MCP unchanged and still default for `init --agent`. |
| R5 | Default bind remains loopback; document danger of `0.0.0.0` without auth. |
| R6 | Tests: auth reject; at least one tool round-trip; health unaffected. |
| R7 | README + `phase-6-work-orders.md` P6-1 checkbox guidance updated when shipping. |

### Scope boundaries

**In:** daemon/HTTP/MCP wiring, config, tests, docs.

**Deferred:** `#101` cloud UI tunnel; write tools on SSE; OAuth; Redis/Postgres.

### Acceptance examples

- AE1. With secret set, SSE/HTTP client can `search` and get structured results.  
- AE2. Missing/wrong secret → 401/403.  
- AE3. Stdio `kurultai mcp` still works in smoke/unit coverage.

---

## Technical notes (for implementer)

- Reuse `BrainService` — do not fork retrieval.  
- Align with existing axum router in `src/http/`.  
- Check current `rmcp` / MCP server entrypoints under `src/mcp/` before adding a parallel stack.  
- Metrics: optional touch-points for later P6-2 (#102); not required to block this PR.

---

## Verification

- [ ] `cargo test --locked` / clippy `-D warnings`  
- [ ] Manual: daemon + small SSE client or documented curl/flow  
- [ ] README enablement section  
- [ ] No SQLite → Postgres work in this PR  
