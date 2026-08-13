---
title: "feat: Tiered access — atom visibility scope (HUB-1)"
date: 2026-08-12
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "Phase 6 next queue HUB-1 · GitHub #178 · brainstorm R1–R3/R8/AE1"
depth: standard
origin: "phase-6-next-work-orders.md Wave G; docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md"
---

# feat: Tiered access — atom visibility scope (HUB-1)

**Target repo:** `duketopceo/kurultai`  
**Audience:** solo (must not regress) → team (foundation)  
**Base:** `main` after next-queue docs land  
**Tracking:** [#178](https://github.com/duketopceo/kurultai/issues/178) · milestone [Tiered Access + Hosted Hub](https://github.com/duketopceo/kurultai/milestone/8) · parent [#10](https://github.com/duketopceo/kurultai/issues/10)  
**Process:** PR-only

## Goal Capsule

**Objective:** Give every `KnowledgeAtom` a first-class **visibility scope** (`personal` | `team` | `company`) with **default `personal`**, so solo search/ask stays identical when no hub is configured, and later hub/connector slices have a stable field to honor.

**Authority:** This plan > [phase-6-next-work-orders.md](phase-6-next-work-orders.md) > [docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md](../brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md) > [#178](https://github.com/duketopceo/kurultai/issues/178).

**Stop when:** Schema + Rust types carry scope; migrations backfill `personal`; upsert/search/ask paths preserve default solo behavior; tests cover default + explicit scope round-trip; README/CONCEPTS (or multi-user doc) mention the field; **no** Postgres hub and **no** public bind in this PR.

**Do not:** Implement [#176](https://github.com/duketopceo/kurultai/issues/176) Postgres Store; [#177](https://github.com/duketopceo/kurultai/issues/177) hub transport; device API keys; Slack allowlist; Brain UI chrome for tiers; infer scope after ingest.

**Assumption (LFG headless):** Scope is a column/enum on atoms (and any sync DTO already used by MCP/Brain). Absent hub config, query paths ignore non-`personal` filters and behave as today (AE1).

## Product Contract

### Summary

Foundational data model for tiered access. Personal atoms never leave the device in later slices; this slice only establishes the field and solo-safe defaults.

### Requirements

| ID | Requirement | Brainstorm |
|----|-------------|------------|
| R1 | Every atom has visibility `personal \| team \| company`. | R1 |
| R2 | Default for new atoms and migrated rows is `personal`. | R2 (local half) |
| R3 | Deployments may later enable zero/one/two shared tiers — model allows `team` without requiring `company`. | R3 |
| R4 | With no hub configured, `search`/`ask` match pre-HUB-1 behavior (AE1). | R8 local · AE1 |
| R5 | MCP / AgentAtomView surfaces may expose scope as metadata without dumping full content. | doctrine #37 |
| R6 | Connectors can *set* scope when writing atoms (API hook); no connector policy UI in this slice. | R9 prep |

### Actors

- A1. Solo operator (must not regress)
- A2. Future team member / hub (out of slice except field readiness)
- A3. Implementer / CI

### Scope boundaries

**In:** types, SQLite migration, store upsert/read, tests, short docs.  
**Out:** remote Store, auth middleware, Tailscale, admin CLI, UI, closing sibling issues.

## Planning Contract

### Key Technical Decisions

- KTD1. **SQLite-first column on atoms; Postgres comes in HUB-2.**  
  Provenance: `user-approved` (next-queue KTD3).  
  Rejected: Blocking scope on Postgres landing.  
  Reason: Unblocks HUB-5 tagging language; AE1 is local.

- KTD2. **Default scope = `personal`; unknown/NULL in old rows → `personal` on migrate.**  
  Provenance: `user-directed` (brainstorm R2).  
  Rejected: Nullable forever.  
  Reason: Fail closed toward private.

- KTD3. **No silent promotion of scope in this slice.**  
  Provenance: `user-approved` (headless).  
  Rejected: Heuristics from source path/kind.  
  Reason: Brainstorm — scope set at ingest, never inferred later.

- KTD4. **Query filter remains optional / unused until hub exists.**  
  Provenance: `user-approved` (AE1).  
  Rejected: Filtering out `team` rows from local DB when present.  
  Reason: Local DB may hold tagged rows for future sync experiments; solo UX stays “all local atoms” unless a later plan changes it — document that choice in README.

### Assumptions

- A1. Existing soft-label / tag columns are orthogonal to visibility scope.
- A2. Export/import packs should carry the new column once present (follow-up ok if export tests need a one-line manifest bump).

## Implementation Units

### U1. Schema + types

- **Files:** `src/types/` (or equivalent atom struct), `src/store/` migrations  
- **Approach:** Add `visibility` / `scope` enum; migration backfill `personal`.  
- **Tests:** migration on fixture DB; serde round-trip.

### U2. Store upsert + read path

- **Files:** `src/store/`  
- **Approach:** Persist and return scope; default when omitted.  
- **Tests:** upsert without scope → personal; explicit `team` round-trips.

### U3. Search / ask / MCP surface

- **Files:** brain/search/MCP view mappers as needed  
- **Approach:** Preserve solo result sets; optionally include scope in metadata views.  
- **Tests:** smoke search unchanged on unlabeled corpus; metadata includes scope when present.

### U4. Docs

- **Files:** `docs/multi-user-kurultai.md` and/or `CONCEPTS.md` + README one-liner  
- **Approach:** Document the three scopes and “solo default / no hub = unchanged”.  
- **Verify:** Links to #178 and next-queue pack.

## Verification Contract

- `cargo test` / nextest for store + CLI smoke touching atoms.  
- Explicit AE1-style test: empty/no-hub config → search behavior matches unlabeled baseline.  
- No new network bind in daemon defaults.

## Definition of Done

- [ ] R1–R6 satisfied for the local/SQLite half
- [ ] HUB-2 (#176) not started in the same PR unless trivially shared types only
- [ ] CI green; docs mention scope
- [ ] Ready for HUB-2 Postgres Store `/lfg`
