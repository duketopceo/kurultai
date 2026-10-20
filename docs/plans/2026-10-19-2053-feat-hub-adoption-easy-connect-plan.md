---
title: "feat: First team/company adoption — hub service on the knowledge lane, RLS, connect-join, team app"
date: 2026-10-19
artifact_contract: ce-unified-plan/v1
product_contract_source: legacy-requirements
execution: code
origin:
  - docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md
  - docs/brainstorms/2026-09-28---device-broker-agent-onboarding-requirements.md
---

## Goal Capsule

**Objective:** A second person can join the company brain by running `kurultai connect <hub-url>`, having an admin approve (with a team pick) in a browser, and querying exactly the scopes their key allows — with Postgres row-level security proving the boundary even if the app filter ever fails.

**Means:** a dedicated hub daemon container on the knowledge deployment (`hub.shippedit.dev` / `api-hub.shippedit.dev`), Postgres+pgvector store with enforced RLS, and the existing RFC 8628 device flow extended to mint team-bound keys. (KTD1, KTD2)

**Stop conditions:** `personal` atoms never reach Postgres (AE4); a member never receives another team's `team`-scoped atoms at API or SQL level (AE5/AE6); revoked keys deny at the edge and scope stays bounded by RLS (AE3).

**Authority hierarchy:** this plan → origin requirements docs → `AGENTS.md` identity doctrine → `CONCEPTS.md`.

## Product Contract

### Summary

Kurultai's hub infrastructure shipped in v0.5.0 and has never been adopted: `KURULTAI_FEATURE_HUB` is `0` on both hosted instances, no team key has ever been issued, `web/` 500s without Clerk keys. Document review established the load-bearing fact: **the hub flag swaps the daemon's entire store — there is no dual-lane mode** (`src/app/context.rs`), and the device-flow/agent-seat trait methods exist only on the SQLite store. So the hub must run as its own container (hub-mode daemon + Postgres sidecar) on the knowledge deployment — the user's chosen lane — while `kurultai-personal` keeps serving the solo brain untouched.

### Problem Frame

The solo brain is a daily driver; the shared brain is code-complete but unadopted. Blockers: (a) no deployment, (b) visibility enforced only in app-level `WHERE` clauses — one query bug crosses the team boundary, (c) onboarding requires server shell + manual key minting where solo already has a one-command device flow, (d) no member-vs-admin authorization model — any valid hub key reaches every non-exempt path, and (e) the web admin console has no backend to call.

### Requirements

Carried from `docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md`:

- **R1** — Every knowledge atom carries `personal | team | company` visibility.
- **R2** — `personal` atoms never leave the originating device; never written to the shared store.
- **R3** — `team` without `company` is a first-class configuration.
- **R5** — Public transport authenticates devices via bearer keys issued by admin tooling.
- **R7** — `Store` trait has a Postgres+pgvector implementation without breaking SQLite.
- **R8** — A member daemon queries local personal + remote shared scopes in one call and merges.
- **R10** — Admin CLI: issue/revoke keys, define `team_id`, list scopes.

Origin **deferred by ID**: **R4** (two selectable transport modes — this deployment is public-only), **R6** (Tailscale-ACL auth — not this deployment), **R9** (connector scope-tagging — connectors are deferred scope), **AE2** (tailnet-reachability example — depends on R4/R6).

New for this plan:

- **R11** — Hub Postgres enforces `visibility`/`team_id` via `FORCE ROW LEVEL SECURITY` policies keyed on per-transaction session GUCs (`kurultai.team_id`, `kurultai.is_admin`), defense-in-depth under the existing `hub_visibility_sql` filter. Runtime role `kurultai_app` is non-owner; `kurultai_migrate` (BYPASSRLS) runs migrations/CLI admin.
- **R12** — `kurultai connect <hub-url>` on a hub daemon mints a team-bound `hub_api_keys` key: the Access-admin-gated `/connect` page renders a server-populated team picker (existing teams + "new team"); optional `--team` pre-fills it, approver's choice wins.
- **R13** — After minting, `connect` stores the key and writes a **remote-URL MCP entry** pointing the agent at `https://api-hub.shippedit.dev/mcp` — ending at "wired, zero manual credential handling."
- **R14** — `web/` is a working admin console behind Clerk (admin-gated, sign-up disabled): members, hub keys (issue/list/revoke), scope census, pending `/connect` approvals. Backed by new daemon admin endpoints; admin credential never reaches the browser.
- **R15** — Deploy recipe: `kurultai-hub` service + `pgvector/pgvector:pg16` sidecar on the knowledge compose stack, `Dockerfile.hub` (postgres feature), tunnel hostnames, admin-scoped Access app, runbook with migration/keys/rollback.
- **R16** — `kurultai hub key issue|list|revoke` stays the fully-supported admin/power path; connect-join is additive.
- **R17** — Member-vs-admin authorization: `hub_api_keys.is_admin` flag; admin endpoints require it; member keys get 403 on admin paths.

### Key Decisions

- **Hub lives on the knowledge deployment as its own service** — `session-settled: user-directed` (host = main lane). Realized as a second container because hub mode swaps the entire store — co-process hosting would silently kill the solo lane. Governs R15.
- **Full team app for `web/`** — `session-settled: user-directed`. Governs R14.
- **Connect-join over new OAuth; admin minting retained** — `session-settled: user-directed`. Governs R12, R13, R16.
- **RLS as defense-in-depth, not sole enforcement** — `session-settled: user-approved` (over RLS-only: silent mis-scope on missed GUC; over app-only: single point of failure). Governs R11.

### Success Criteria

- Second seat joins via `connect` and receives only `company` + own-`team` atoms — proven live.
- `psql` as `kurultai_app` with `team_id='eng'` returns zero `sales` rows (AE6).
- Member key → `search`/`remember` scoped; admin-only endpoints 403 for member keys.
- `web/` issues+revokes a key end-to-end behind Clerk admin gate.

### Acceptance Examples

Origin AE1–AE5 preserved where applicable (AE2 deferred with R4/R6). Additions:

- **AE6** — `BEGIN; SELECT set_config('kurultai.team_id','eng',true); SELECT … WHERE team_id='sales'` → 0 rows as `kurultai_app`.
- **AE7** — `kurultai connect https://api-hub.shippedit.dev` → `/connect` (Access admin) → pick `team=eng` → key minted at approve → MCP remote entry works with `search`/`remember` scoped eng+company.
- **AE8** — `KURULTAI_FEATURE_HUB` container removed/flag-off → solo lane untouched; hub hostnames gone.

### Scope Boundaries

**Deferred for later:** connectors (#114/#135/#134), CF Access service tokens (#333), embedding versioning/outbox (#323/#326), Redis L2 (#112), R2 snapshots, edge gateway, origin R4/R6/R9 + AE2 (Tailscale mode), `duketopceo/repos` reindex.

**Outside this product's identity:** multi-tenant SaaS across unrelated orgs; password/session accounts for agents (R5); dual-store multiplexing inside one daemon (rejected — separate container is the architecture).

**Deferred to follow-up work:** web/ self-serve member dashboards & atom browsing; Hey board in web/; broker-with-team-key smoke (note in U9); `pg_dump` backup access/retention policy detail.

---

## Planning Contract

### Key Technical Decisions

- **KTD1 — RLS via session GUCs; three-tier path classification.** Policies on `knowledge_atoms` (+ decide coverage for `atoms_vec`, `hub_activity`): `USING (current_setting('kurultai.is_admin',true)='on' OR visibility='company' OR (visibility='team' AND team_id=current_setting('kurultai.team_id',true)))` + matching WITH CHECK. Every Postgres path classified **member** (team GUC) / **admin** (`is_admin` GUC; `/api/hub/*`, admin principals) / **internal** (no GUC needed — pipeline upserts, merges, `/ingest`, `touch_access`; set `is_admin` so writes aren't blocked). GUCs via `SELECT set_config(name, $1, true)` — `SET` rejects bind params; `team_id` is approver free text, so validate charset/length at mint and never interpolate. `session-settled: user-approved`.
- **KTD2 — Hub = dedicated container on the knowledge deployment.** `kurultai-hub` service (postgres feature build via `Dockerfile.hub`) + `pgvector/pgvector:pg16` volume, `hub.shippedit.dev` (human/Access) + `api-hub.shippedit.dev` (bearer agent lane) on the existing `shippedit-server` tunnel + admin-scoped Access app. `kurultai-personal` untouched. Chosen over dual-store (unbuilt, larger) and over hosting on work lane (user picked the primary lane).
- **KTD3 — PostgresStore gets device-flow parity, not agent-seat parity.** Hub-mode daemon needs `create/get/approve/deny/exchange_device_flow` + `list_device_flows` impls on Postgres (new `hub_device_flows` table) so `/api/device/*` + `/connect` work; solo seat-token methods stay SQLite-only — hub members authenticate via `hub_api_keys`, not seats.
- **KTD4 — Mint at approve; team picker is server-rendered.** Approve handler calls `HubKeyStore::issue(agent_id, team_id)` once; `/api/device/code` response stays opaque (no team roster to unauthenticated callers — it's Access-exempt). Approver gate: admin-scoped Access app **and** `KURULTAI_ADMIN_EMAILS` allowlist check inside `approver_identity`.
- **KTD5 — Member write posture: team=trusted, company=admin-only.** Hub-authed writes stamp `visibility='team'` + `metadata.team_id = principal.team_id` server-side (never client-trusted); a member `remember` lands eng-scoped. `visibility='company'` inserts rejected for member keys (WITH CHECK + principal guard). `SharedClosed` quarantine applies to the solo lanes only — hub members are already team-trusted principals.
- **KTD6 — `/mcp` accepts hub keys with team threading.** `src/http/mcp.rs` authorize() gains `hub_api_keys` resolution → `McpAuth::Hub{team_id}`; `call_tool` threads `hub_team_id` into `search_scoped_hub` and `remember` stamping; RLS remains the backstop.
- **KTD7 — Admin surface = `is_admin` flag + `/api/hub/*` endpoints.** `hub_api_keys` gains `is_admin` (mint via `hub key issue --admin`); new routes `GET/POST/DELETE /api/hub/keys`, `GET /api/hub/device/pending`, `POST /api/hub/device/approve`, `GET /api/hub/scopes` gated on `principal.is_admin`. web/ calls these server-side with an env-held admin key (`KURULTAI_ADMIN_KEY` — an `is_admin` hub key, not a new credential class).
- **KTD8 — Migrate path is explicit, app role never migrates.** `PostgresStore::connect`/`HubKeyStore::connect` skip auto-migrate when `KURULTAI_PG_SKIP_MIGRATE=1` (set in the hub container); `kurultai hub …` CLI and `init-hub-db.sql` run under `KURULTAI_MIGRATE_DATABASE_URL` (`kurultai_migrate`, BYPASSRLS).
- **KTD9 — `kurultai-personal`/`work` containers unchanged.** Solo lanes keep their existing tokens, CF Access, and SQLite brains. Broker (`kurultai broker`) works against the hub if its upstream token is a team-bound hub key — verified in U9, no broker changes planned.

### High-Level Technical Design

```text
member laptop                    server-001 (knowledge deployment)
┌──────────────────┐             ┌────────────────────────────────────────────┐
│ local SQLite     │             │ kurultai-personal :8421 (UNTOUCHED)        │
│ (personal scope) │             │   solo lane → knowledge./api-knowledge.    │
│                  │             │                                            │
│ kurultai connect─┼─device code▶│ kurultai-hub :8423 (KURULTAI_FEATURE_HUB)   │
│ api-hub.…        │◀─hub key────│   PostgresStore → pgvector sidecar         │
│                  │             │     └─ set_config(team_id) per tx          │
│ MCP cfg →        │  approve    │     └─ FORCE RLS re-checks every row       │
│ api-hub./mcp     │             │   HubGate: hub key → (team_id, is_admin)   │
└──────────────────┘             │   /connect page (Access admin + team pick) │
                                 │ Postgres :5432 (compose-internal)          │
                                 │ web/ container (Clerk) → /api/hub/*        │
                                 └───────── shippedit-server tunnel ─────────┘
```

Query path: hub key → `HubGate` → `HubPrincipal{team_id,is_admin}` → `PostgresStore` → tx `set_config` + `WHERE` filter → RLS policy re-checks rows. Three layers agree or the request fails closed.

### Assumptions

- `hub_api_keys`/`HubKeyStore` schema is sufficient + `is_admin` column migration; only minting paths change.
- Postgres credentials live in server-local `.env` only; `DATABASE_URL` is compose-internal (never public).
- CF Access team `duketopceo` supports an admin-scoped policy for `hub.`/`web.`/`api-hub.` hostnames.

### Deferred Implementation Notes

- Exact policy DDL + which tables beyond `knowledge_atoms` get policies (`atoms_vec` verdict lands in U1).
- `/connect` team picker widget details; `hub_device_flows` column list (mirror SQLite `device_flows`).
- web/ route/component layout; whether `/connect` branches hub-vs-solo on one page or two.
- Broker smoke against hub upstream (verify-only, U9).

---

## Implementation Units

| U-ID | Unit | Files | Depends on |
|---|---|---|---|
| U1 | RLS migration + roles + migrate path | `src/store/migrations.rs`, `src/store/postgres.rs`, `src/hub/keys.rs`, `deploy/server-001/init-hub-db.sql`, `tests/hub_rls.rs` | — |
| U2 | GUC wiring + write stamping + path classification | `src/store/postgres.rs`, `src/http/auth.rs`, `src/http/mod.rs`, `src/mcp/*` (write ctx), `tests/hub_rls.rs` | U1 |
| U3 | Postgres device flows + connect→hub join | `src/store/postgres.rs`, `src/store/migrations.rs`, `src/http/device.rs`, `src/connect.rs`, `tests/connect_hub.rs` | U1 |
| U4 | Admin authz + hub admin endpoints | `src/hub/keys.rs`, `src/http/auth.rs`, `src/http/mod.rs`, `tests/hub_admin.rs` | U1 |
| U5 | `/mcp` hub-key auth + remote `wire_agent` | `src/http/mcp.rs`, `src/mcp/server.rs`, `src/mcp/init.rs`, `tests/hub_mcp.rs` | U2, U3 |
| U6 | Docs + CLI parity (`hub key`) | `src/main.rs`, `docs/AGENT_CONNECTION_PROMPT.md`, `README.md`, `INDEX.md` | U3 |
| U7 | Deploy recipe + runbook | `deploy/server-001/{docker-compose.kurultai.yml,Dockerfile.hub,init-hub-db.sql,redeploy.sh}`, `docs/runbook-hub.md` | U1–U4 |
| U8 | web/ admin console (4 surfaces) | `web/**` | U4, U7 |
| U9 | Live dogfood + AE verification | `tests/acceptance_visibility.rs`, `ROADMAP.md`, issues #111/#115, `INDEX.md` | all |

### U1. Postgres RLS migration, roles, and explicit migrate path

**Goal:** `knowledge_atoms` enforced by RLS even if app filtering is bypassed; runtime role can't run DDL.

**Requirements:** R11, KTD1, KTD8; supports AE6/AE3.

**Files:** `src/store/migrations.rs`, `src/store/postgres.rs`, `src/hub/keys.rs` (skip-migrate flag), `deploy/server-001/init-hub-db.sql`, `tests/hub_rls.rs` (new).

**Approach:**
1. Hub migration `v2_rls`: `ENABLE` + `FORCE ROW LEVEL SECURITY` on `knowledge_atoms`; `CREATE POLICY kurultai_scope_select … FOR SELECT USING (is_admin GUC='on' OR visibility='company' OR (visibility='team' AND team_id=current_setting('kurultai.team_id',true)))`; `INSERT`/`UPDATE` WITH CHECK (member GUC writes own team only; admin writes any shared scope; `personal` already excluded by CHECK).
2. `init-hub-db.sql` (idempotent: `CREATE ROLE … IF NOT EXISTS` via DO block): `kurultai_migrate BYPASSRLS`, `kurultai_app NOINHERIT`; grants on **every** runtime table — `knowledge_atoms` (S/I/U/D), `atoms_vec`, `hub_api_keys`, `hub_activity`, `ingestion_jobs`, `quality_audit`, `merge_candidates`, `hub_device_flows` — + `USAGE` on all sequences. Decide `atoms_vec`/`hub_activity` policy coverage here.
3. `KURULTAI_PG_SKIP_MIGRATE=1` skips `connect()`'s eager `migrate()` in both `PostgresStore` and `HubKeyStore` (KTD8).
4. Tests self-provision roles/policies (`SET ROLE` or create roles) — CI's `POSTGRES_USER` is a superuser which bypasses FORCE RLS; relying on it proves nothing.

**Patterns:** existing migration runner; `hub_api_keys` SQL in `src/hub/keys.rs`.

**Test scenarios:**
- Covers AE6: eng-scoped session sees eng+company, zero sales.
- No GUC set → `NULL` current_setting → company-only (fail-closed).
- `is_admin` GUC → all scopes.
- Member session INSERT `team_id='sales'` → WITH CHECK rejects; INSERT `visibility='personal'` → CHECK rejects.
- `kurultai_migrate` reads all rows (BYPASSRLS sanity); `kurultai_app` DDL attempt errors.

**Verification:** `tests/hub_rls.rs` green under the CI postgres job; `\d+ knowledge_atoms` shows policy.

### U2. Session-GUC wiring + server-side write stamping

**Goal:** every member-scoped query runs under `set_config('kurultai.team_id',…)`; every hub write is stamped from the resolved principal, never client input.

**Requirements:** R11, R8; KTD1, KTD5.

**Files:** `src/store/postgres.rs`, `src/http/auth.rs`, `src/http/mod.rs` (GUC plumbing per route class), `src/mcp/brain.rs` (write ctx), `tests/hub_rls.rs`.

**Approach:**
1. Wrap scoped reads/writes in a tx issuing `SELECT set_config('kurultai.team_id', $1, true)` (+ `kurultai.is_admin` for admin principals — required for admin list/activity/graph paths, since a `team_id=None` principal would otherwise see only `company` under RLS).
2. Classify every Postgres path member/admin/internal (KTD1); internal writers (`/ingest`, pipeline upserts, merges, `touch_access`) run with `is_admin`.
3. Write stamping (KTD5): hub-authed `remember`/ingest sets `visibility='team'`, `metadata.team_id=principal.team_id` server-side; member `company` writes rejected.
4. Scope `GET /api/hub/activity` to admin (currently any authenticated caller sees all teams' write log).
5. Keep the app `WHERE` filter — RLS and filter both apply; divergence = bug signal.

**Test scenarios:**
- Two sequential pool checkouts eng→sales: no scope bleed (`SET LOCAL` semantics via `set_config(…,true)`).
- `team_id=NULL` principal + no is_admin → company-only.
- Covers AE5 end-to-end over HTTP with RLS active.
- `remember` as eng member lands `visibility=team, team_id=eng` regardless of client body; client-supplied `team_id` ignored.
- `is_admin` cannot be set on a member path (negative test).
- `/api/hub/activity` as member key → 403.

**Verification:** `tests/hub_rls.rs` + `tests/acceptance_visibility.rs` green.

### U3. Postgres device flows + connect→hub join

**Goal:** `kurultai connect https://api-hub.shippedit.dev` produces a team-bound key with zero credential handling.

**Requirements:** R12, R13, AE7; KTD3, KTD4.

**Files:** `src/store/migrations.rs` (`hub_device_flows` table), `src/store/postgres.rs` (`create/get/approve/deny/exchange/list_device_flows` impls), `src/http/device.rs` (team picker, mint-at-approve, admin allowlist), `src/http/device_auth.rs`, `src/connect.rs` (`--team`), `src/hub/keys.rs`, `tests/connect_hub.rs` (new).

**Approach:**
1. `hub_device_flows` mirrors the SQLite `device_flows` shape; implement the six trait methods (+ `list_device_flows` for U4's console) on `PostgresStore`.
2. `/connect` (hub mode): approver identity requires Access JWT **and** `KURULTAI_ADMIN_EMAILS` match (KTD4); page server-renders the team picker from `HubKeyStore` distinct `team_id`s + free-text "new team".
3. Approve POST → `HubKeyStore::issue(codename+seat, team_id)` → store key id on the flow → token poll returns it once (KTD4 mint-at-approve; concurrent polls can't double-mint).
4. `connect.rs`: `--team` prefill hint (approver can override); key stored under `<lane>-<codename>-<seat>-agent-token` convention; then remote MCP wiring (U5's `wire_agent` mode) to `api-hub.shippedit.dev/mcp`.
5. `team_id` validated at mint (charset/length) — it later becomes SQL-visible.
6. Solo `/connect` path unchanged.

**Test scenarios:**
- Covers AE7 end-to-end on a hub-mode daemon.
- Approver overrides `--team` → approver wins; "new team" free text → key carries it; invalid charset → rejected.
- Concurrent token polls → exactly one minted key.
- `hub:false` daemon → solo seat path unchanged.
- Non-admin Access identity → approve rejected.
- Team roster absent from `/api/device/code` response (unauthenticated).

**Verification:** `tests/connect_hub.rs` green + manual connect against a local hub-mode daemon.

### U4. Member/admin authorization + hub admin endpoints

**Goal:** real role boundary; the daemon endpoints `web/` needs actually exist.

**Requirements:** R17, R14 backend, R10/R16 parity over HTTP.

**Files:** `src/hub/keys.rs` (`is_admin` column + `issue(…, is_admin)`), `src/http/auth.rs` (admin gate helper), `src/http/mod.rs` (`/api/hub/keys` GET/POST/DELETE, `/api/hub/device/pending`, `/api/hub/device/approve`, `/api/hub/scopes`), `src/store/postgres.rs` (`list_device_flows` use), `tests/hub_admin.rs` (new).

**Approach:**
1. Migration adds `hub_api_keys.is_admin BOOLEAN DEFAULT false`; `hub key issue --admin` mints admin keys.
2. Admin gate helper: `HubPrincipal.is_admin` required on the new routes (+ `/api/hub/activity` per U2).
3. Endpoints: keys issue/list/revoke (mirror CLI), device pending list + approve (powers web/ approvals surface = same handler as `/connect` POST), scopes census (counts by visibility+team).
4. Audit: admin issue/revoke/approve events logged (`hub_activity` or dedicated audit row).

**Test scenarios:**
- Member key → every admin endpoint 403; admin key → 200.
- Issue via HTTP → row + prefix listed, secret never returned after mint.
- Revoke → subsequent request denied (AE3 shape).
- Approvals list shows pending device flow; approve mints with chosen team (same code path as `/connect`).

**Verification:** `tests/hub_admin.rs` green; `hub key issue --admin` → HTTP admin paths work.

### U5. `/mcp` hub-key auth + remote `wire_agent`

**Goal:** the minted hub key works on `/mcp` with team scope; `connect` can wire a remote MCP entry.

**Requirements:** R13, AE7; KTD6.

**Files:** `src/http/mcp.rs` (accept `hub_api_keys` → `McpAuth::Hub{team_id,is_admin}`), `src/mcp/server.rs` (`call_tool` threads `hub_team_id` → `search_scoped_hub` + `remember` stamping), `src/mcp/init.rs` (remote-URL entry shape), `tests/hub_mcp.rs` (new).

**Approach:**
1. `/mcp` authorize() tries `HubKeyStore` resolution after shared-secret check; stamps principal into request extensions.
2. Thread `hub_team_id` through tool calls: `search`/`ask`/`cite`/`who_knows` scoped; `remember` uses U2 stamping; `promote`/ontology writes remain admin-only.
3. `wire_agent` gains a remote mode: `{url: https://api-hub.shippedit.dev/mcp, headers: Authorization: Bearer <key>}` — key resolved from keyring name, never written into the config file where the agent format supports header refs; where it can't (codex TOML), document the bridge pattern (`scripts/mcp/kurultai/` precedent).

**Test scenarios:**
- Hub key → `search` returns eng+company only; member key → `promote` denied.
- Revoked key → `/mcp` 401/403.
- `wire_agent` remote mode produces a working entry for `cursor`/`claude` formats (snapshot tests of written config).
- Solo-lane secret + seat paths unchanged (regression).

**Verification:** `tests/hub_mcp.rs` green; live MCP handshake against hub in U9.

### U6. Docs + CLI parity

**Goal:** `kurultai hub key` documented as the power path; connect-join as the easy path.

**Requirements:** R10, R16.

**Files:** `src/main.rs` (help strings), `docs/AGENT_CONNECTION_PROMPT.md`, `README.md`, `INDEX.md` + `docs/INDEX.md`.

**Approach:** `hub key issue --team/--admin` help accuracy; docs show both onboarding paths; connect doc gains hub-mode section (`api-hub` host, team picker). No behavior change.

**Test expectation:** none — verify `hub key --help` renders and docs match clap strings.

### U7. Deploy recipe + runbook on the knowledge deployment

**Goal:** `kurultai-hub` runs alongside `kurultai-personal` on server-001 behind `hub.`/`api-hub.` hostnames.

**Requirements:** R15, AE8; KTD2, KTD8.

**Files:** `deploy/server-001/docker-compose.kurultai.yml` (add `kurultai-hub` :8423 + `kurultai-hub-db` pgvector service + `kurultai-hub-pgdata` volume), `deploy/server-001/Dockerfile.hub` (`--features postgres`), `deploy/server-001/init-hub-db.sql`, `deploy/server-001/redeploy.sh`, `docs/runbook-hub.md`, `deploy/server-001/INDEX.md`, `docs/INDEX.md`.

**Approach:**
1. `Dockerfile.hub`: `cargo build --release --locked --features postgres --bin kurultai` — the solo image compiles hub code out today; don't change `Dockerfile.solo`.
2. Compose: `kurultai-hub` env `KURULTAI_FEATURE_HUB=1`, `DATABASE_URL` (kurultai_app → compose-internal `kurultai-hub-db`), `KURULTAI_PG_SKIP_MIGRATE=1`, `KURULTAI_ADMIN_EMAILS`, `KURULTAI_ADMIN_KEY` not needed (admin = is_admin hub key minted by runbook).
3. cloudflared config + Access app: `hub.shippedit.dev` (admin-scoped humans), `api-hub.shippedit.dev` (bearer-only).
4. Runbook: apply `init-hub-db.sql` once (superuser), migrate via `KURULTAI_MIGRATE_DATABASE_URL`, mint first admin key via `kurultai hub key issue --admin` (run against migrate DSN), verify AE6, rollback = remove service + hostnames. Note `pg_dump` output access/retention.

**Test expectation:** smoke-first — `docker compose config` validates, containers boot, migration applies, AE6 verified live.

**Verification:** compose config clean; hub reachable on `api-hub.`; `knowledge.` solo lane regression-checked (still serves SQLite brain).

### U8. web/ admin console

**Goal:** humans administer the hub from a browser — members, keys, scopes, approvals.

**Requirements:** R14, R17; KTD7.

**Files:** `web/**` — `.env.example`, `src/proxy.ts` (existing Clerk gate: admin-only emails, sign-up disabled in Clerk config), `src/lib/hub-admin.ts` (server-side wrapper, `KURULTAI_ADMIN_KEY`), `src/app/dashboard/{page,keys,members,approvals}/page.tsx`.

**Approach:**
1. Clerk: disable public sign-up; `proxy.ts` restricts `/dashboard/*` to admin email list.
2. `hub-admin.ts` server-only fetch → `https://api-hub.shippedit.dev/api/hub/*` with `Authorization: Bearer $KURULTAI_ADMIN_KEY` — key stays server-side.
3. Four surfaces: dashboard (scope census + recent activity), keys (issue/list/revoke), members (grouped by team/agent), approvals (pending device flows → approve with team pick, mirroring `/connect`).
4. `web.` hostname added to tunnel + admin-scoped Access app.

**Test scenarios:** smoke — `/dashboard` redirects unauthenticated to Clerk sign-in; non-admin signed-in user → 403; issue → appears in `hub key list`; revoke → denied.

**Verification:** deployed `web.` host or local dev against live hub api; AE3 revoke proves denial.

### U9. Live dogfood + AE verification

**Goal:** "a second person joins a hub with a per-device key and sees only team+company atoms" — met, on the real deployment.

**Requirements:** all; the adoption gate.

**Files:** `tests/acceptance_visibility.rs` (extend if needed), `ROADMAP.md` (stage-2 status + evidence), `docs/plans/phase-6-next-work-orders.md`, issue comments on #111, #115, `INDEX.md`.

**Approach:**
1. Deploy U7; runbook-mint first admin key; run `connect` from a second seat (codename `devin`, `team=qa` or `eng`) end-to-end on `api-hub.shippedit.dev`.
2. Exercise AE3/AE5/AE6/AE7/AE8 live; capture outputs in PR/runbook.
3. Optional smoke: `kurultai broker` upstream = hub key (KTD9 verify-only).
4. Update #111 (hub store adopted), #115 (RLS defense-in-depth landed; richer RBAC still open); ROADMAP stage-2 exit criterion with evidence.

**Test scenarios:** AE3/AE5/AE6/AE7/AE8 demonstrated live; solo `knowledge.`/`work.` lanes regression-checked after deploy.

**Verification:** roadmap exit criterion flips to met with recorded evidence.

---

## Verification Contract

- `cargo test --locked` — green (nextest `journal_mode=WAL` flake = known rerun case).
- CI `postgres-store` job green including new `tests/hub_rls.rs`, `tests/hub_admin.rs`, `tests/hub_mcp.rs`, `tests/connect_hub.rs`.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`.
- `python3 scripts/audit-agent-index.py` green after INDEX updates.
- `web/`: `npm run build` clean; `/dashboard` redirects unauthenticated to sign-in.
- Live (post-deploy): AE6 `psql` policy proof; AE7 connect-join on `api-hub.`; solo-lane regression on `knowledge.`.

## Definition of Done

- All units landed; RLS proven at DB layer (not only API); a second real seat joined via `connect` and observed scope isolation; member key cannot reach admin endpoints or other teams' atoms.
- No abandoned-attempt code in the diff.
- `ROADMAP.md` stage-2 criterion updated with evidence; INDEX rows updated; CI green.
- Secrets (`DATABASE_URL`, `KURULTAI_ADMIN_KEY`, Clerk keys, migrate DSN) exist only in server-local env — never committed.
