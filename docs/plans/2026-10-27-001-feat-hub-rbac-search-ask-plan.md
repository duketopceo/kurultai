---
title: "feat: Hub RBAC on search/ask — roles + lane scoping (slice 1 of #115)"
date: 2026-10-27
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "GitHub #115 · ROADMAP.md Stage 2 exit criteria · docs/multi-user-kurultai.md"
depth: standard
origin: "docs/plans/2026-08-12-002-feat-tiered-access-atom-scope-plan.md (HUB-1) · HUB-4 issued keys · HUB-6 AE5"
---

# feat: Hub RBAC on search/ask — roles + lane scoping (slice 1 of #115)

**Target repo:** `duketopceo/kurultai`
**Tracking:** [#115](https://github.com/duketopceo/kurultai/issues/115) (stays open — see "Deferred")
**Process:** PR-only, never merge without Luke.

## Problem frame

HUB-1..6 shipped visibility scopes (`personal|team|company`), a Postgres hub store, issued per-device keys and AE5 team isolation on `/api/search`, `/api/ask`, `/api/who_knows`. Issue #115 asks for the *authorization* half to actually be enforced. Auditing the current code against the #115 acceptance list found:

| #115 acceptance | State on `main` |
|---|---|
| Search/ask/list respect lane | Search + ask are scoped in SQL, **but**: ask's multi-hop second pass (`multi_hop_expand`) searches with a default (unscoped) filter, so another team's `team` atoms can enter the synthesized answer; `/api/atoms` (list), `/api/recall`, `/cite`, `/api/graph`, `/api/db/{table}` ignore the caller's team; `/api/hub/activity` shows every team's writes. |
| Role model viewer / contributor / admin | **Missing.** Every issued key can do everything (promote, ontology writes, decide proposals). |
| Promote API into shared index with actor audit | `POST /api/promote` is quarantine→trusted only; there is no personal→shared promote. Writes are audited via `hub_activity`. |
| Solo path unchanged | Holds today; must keep holding (no principal ⇒ no filter, no role check). |

## Scope of this slice

**In:** role model + enforcement on the HTTP surface; close the lane/team leaks above on every read path that reaches the store; docs.
**Out (deferred, tracked on #115):** personal→shared promote API (needs a local→hub copy transport, overlaps #80); company-tier org boundaries (many orgs/RBAC beyond one team id); scoping `/mcp` for hub keys; team-scoping `/api/graph` and `/api/db` (this slice fails them closed for key principals instead); ontology read scoping.

## Requirements

- R1. `HubRole` = `viewer < contributor < admin`, stored per issued key; unknown/NULL resolves to the least privilege (`viewer`).
- R2. Existing keys keep today's behavior (migration backfills `admin`); newly issued keys default to `contributor` and take `--role`.
- R3. Requests carrying a hub principal are authorized by `required_role(method, path)`: viewer = read endpoints (including the read-only POSTs search/ask/recall/cite/who_knows); contributor = other mutating routes (hey board, touch, proposal submit); admin = promote, ontology human-lane writes, proposal decide, hub activity log. Unknown mutating routes default to contributor (fail safe).
- R4. Team scoping is applied in Rust as well as SQL (defense in depth, and testable without Postgres): any hit/atom returned for a scoped caller must be `company`, or `team` with the caller's `team_id`; never `personal`.
- R5. Ask multi-hop, list, recall, cite and who_knows are all scoped; `/api/graph` and `/api/db/*` return 403 for key principals; `/api/hub/activity` is admin-only and filtered to the caller's team.
- R6. Solo / no-principal behavior is byte-for-byte unchanged (AE1); `SearchFilter` without `hub_team_id` admits everything.

## Key technical decisions

- KTD1. **Role is a column on `hub_api_keys`, not a new table.** One role per key; ordering is total so a guard is a single comparison. Rejected: per-route ACL tables (YAGNI for the Team stage).
- KTD2. **Legacy keys → `admin`.** Defaulting them to `contributor` would silently break existing operators' promote/decide flows; least-privilege applies to keys issued from now on.
- KTD3. **Role check is a pure function + one middleware** layered directly inside `hub_api_auth` so it sees the resolved principal. Principals from static `KURULTAI_HUB_API_KEYS`, agent keys and CF Access carry no `HubPrincipal` and are untouched.
- KTD4. **Post-filter in `hybrid_hits_filtered`** with `SearchFilter::admits`, in addition to the Postgres SQL predicate. SQLite ignores `hub_team_id`; the Rust filter makes the invariant hold for every store and lets tests run without a database. Over-fetch is already 4×, so recall impact is negligible.
- KTD5. **Fail closed over half-scoping** for graph/db: 403 for key principals until a team-aware implementation exists.

## Implementation units

### U1. Role type
- **Files:** `src/hub/role.rs` (new, not postgres-gated), `src/hub/mod.rs`
- **Approach:** `HubRole` enum with `Ord`, `as_str`, `parse`; `required_role(&Method, &str) -> HubRole`.
- **Tests (in-file):** ordering; parse round-trip and unknown→viewer; `required_role` table (search POST=viewer, GET list=viewer, `/api/hub/activity`=admin, `/api/promote`=admin, decide=admin, ontology entity DELETE=admin, hey POST=contributor, unknown POST=contributor).

### U2. Persist role; principal carries it
- **Files:** `src/hub/keys.rs`, `src/store/postgres.rs` (shared DDL), `src/main.rs` (`hub key issue --role`, list shows role), `src/http/auth.rs`
- **Approach:** `ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'admin'`; `issue(agent, team, role)`; `HubPrincipal.role`; `HubKeyRecord.role`.
- **Tests:** Postgres-backed issue→resolve round-trip asserts role (runs in the `Postgres store` job).

### U3. Role guard middleware
- **Files:** `src/http/auth.rs`, `src/http/mod.rs` (layer order)
- **Approach:** `hub_role_guard` reads `HubPrincipal` from request extensions; 403 JSON `{error:"forbidden", required_role, role}`.
- **Tests:** `tests/acceptance_hub_rbac.rs` (postgres feature): full `build_app` wrapped in a layer that injects a principal; viewer can search but gets 403 on `/api/promote` and hey POST; contributor can POST hey but not promote; admin passes the guard; no principal ⇒ unchanged.

### U4. Lane scoping on every read path
- **Files:** `src/store/mod.rs` (`SearchFilter::admits`), `src/mcp/brain.rs` (post-filter in `hybrid_hits_filtered`; `multi_hop_expand` carries team scope; `list_atoms_scoped`, `recall_for_agent_scoped`, `cite_scoped`), `src/http/mod.rs` (handlers pass `principal.team_id()`; graph/db 403; activity admin + team filter)
- **Tests:** unit tests for `admits`; `tests/acceptance_hub_rbac.rs` over a SQLite store seeded with eng-team, sales-team, company and personal atoms: eng caller never sees sales/personal atoms via search, ask (including a hop-only match), list, recall, cite, who_knows; company atoms visible to both; no-principal sees everything (AE1).

### U5. Docs
- **Files:** `docs/multi-user-kurultai.md`, `ROADMAP.md`, `CHANGELOG.md`, `src/hub/INDEX.md`, `src/http/INDEX.md`, `tests/INDEX.md` (agent-index protocol)
- **Approach:** document roles, route table, the scoping guarantees, and what is still deferred.

## Verification contract

`cargo fmt --check`; `cargo clippy --all-targets -- -D warnings` (default and `--features postgres`); `cargo test --locked` (default and `--features postgres`); `python3 scripts/audit-agent-index.py`.

## Definition of done

- [ ] R1–R6 satisfied and covered by tests
- [ ] Solo behavior tests unchanged and green
- [ ] #115 remains open with the deferred list in the PR body
- [ ] CI green; ce-code-review findings addressed
