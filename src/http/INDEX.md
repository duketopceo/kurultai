---
index: kurultai/v1
folder: src/http
parent: src/INDEX.md
updated: 2026-10-21
version: 11
---

# `src/http`

**Does:** Daemon HTTP + Brain UI + MCP SSE
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`auth.rs`](auth.rs) | Hub API-key / admin token gate + CF Access fallback; `request_pg_scope` classifies member/admin/closed `PgScope` from the resolved `HubPrincipal` (KTD1) — `is_admin` principals map to `PgScope::admin()`; `admin_permitted()` gates `/api/hub/*` (member keys 403, operator creds pass) | `src/hashutil` · `src/hub/keys.rs` · `src/http/cf_access.rs` · `src/store/scope.rs` | `src/daemon/mod.rs` | 2026-10-21 | 10 | 2026-10-21 hub adoption U4 (R17/KTD7): `is_admin()` reads the real `hub_api_keys.is_admin` flag; `admin_permitted()` — member→false, admin→true, principal-less operator creds→true; `request_pg_scope` maps admin principals to `PgScope::admin()` (`kurultai.is_admin` GUC) · 2026-10-20 hub adoption U2: `request_pg_scope` — member principal → `PgScope::member(team_id)`, `/api/hub/*` operator creds → admin, else closed; `MaybeHubPrincipal::is_admin()` accessor (false until U4 flag) · 2026-09-16 exempt `/api/device/*` + `/connect` (self-gated); 401 JSON hint `run kurultai connect` · 2026-09-14 `/api/ontology/entity` + `/link` join WRITE_ROUTES (#316) · 2026-09-11 proposals decide route joins SharedClosed write guard (#118) · 2026-09-06 CF Access JWT fallback in `hub_api_auth` · 2026-09-06 exempt `/auth/*` from hub API-key middleware · 2026-08-31 resolve token first; 500 on DB errors · 2026-08-16 indexed (v1 seed) |
| [`cf_access.rs`](cf_access.rs) | Verify Cloudflare Access JWTs (`CF_Authorization` cookie / `Cf-Access-Jwt-Assertion`) for human browser auth | `jsonwebtoken` · `reqwest` | `src/http/auth.rs` · `src/http/device.rs` (tests) | 2026-10-21 | 2 | 2026-10-21 `seeded_for_test` → pub(crate) for device.rs allowlist unit tests · 2026-09-06 added — Google/OTP Access login satisfies `/api/*` auth |
| [`device.rs`](device.rs) | `kurultai connect` endpoints: `POST /api/device/code` (+`--team` hint), `/api/device/token` (hub key handout), `GET/POST /connect` approval page — hub lane renders a server-side team picker and mints at approve; `approve_for_team` is `pub(crate)`, reused by `POST /api/hub/device/approve`; `KURULTAI_ADMIN_EMAILS` allowlist gates CF approvers | `src/http` · `src/store` · `src/hashutil` · `src/store/scope.rs` · `src/hub/keys.rs` | `src/http/mod.rs` | 2026-10-21 | 3 | 2026-10-21 hub adoption U4 (R17): `approve_for_team` → `pub(crate)` so `POST /api/hub/device/approve` shares the mint-at-approve path · 2026-10-21 hub adoption U3: team picker (datalist + free text, `--team` prefill), `approve_hub` mint-at-approve + orphan revoke, hub token handout via atomic exchange, `KURULTAI_ADMIN_EMAILS` gate under `feature:hub`, roster kept out of `/api/device/code` · 2026-09-16 added — seat-scoped agent keys |
| [`device_auth.rs`](device_auth.rs) | Device-code login endpoints (`/auth/device/*`) for hosted agent tokens | `src/http` · `src/store` · `src/hashutil` | `src/http/mod.rs` | 2026-09-16 | 2 | 2026-09-16 `create_device_flow` gains `instance_id`; `human_base_url` shared with `device.rs` · 2026-09-06 added |
| [`hey.rs`](hey.rs) | Multi-agent message board REST (`/api/hey/...`) | `src/store` · `src/brain` · `src/error` | `src/daemon/mod.rs` · `src/http/mod.rs` · `src/http/proposals.rs` · `src/http/ontology_write.rs` | 2026-09-19 | 5 | 2026-09-19 `resolve_thread_id` prefers `get_thread(id)` over name — path ids no longer hop to threads named with that UUID · 2026-09-17 zero-auth solo path: `require_writer` posts as `luke`/local when no auth surface configured · 2026-09-14 `require_agent` reused by ontology_write agent-refusal (#316) · 2026-09-11 `require_agent` now pub(crate) for proposal submit auth (#118) 2026-09-11 `require_agent` now pub(crate) for proposal submit auth (#118) · 2026-09-04 message board endpoints + clamp cleanup |
| [`proposals.rs`](proposals.rs) | O3 ontology proposal queue REST (`/api/ontology/proposals`, decide) — agent bearer submits, humans decide | `src/http/hey.rs` · `src/http/auth.rs` · `src/ontology` | `src/http/mod.rs` | 2026-09-11 | 1 | 2026-09-11 added (#118) |
| [`ontology_write.rs`](ontology_write.rs) | Human-lane ontology board writes (`POST /api/ontology/entity`, `/link`) — agents refused | `src/http/hey.rs` · `src/http/auth.rs` · `src/ontology` | `src/http/mod.rs` | 2026-09-14 | 1 | 2026-09-14 added (#316) |
| [`hub_listen.rs`](hub_listen.rs) | Pure bind × auth start-fail (HUB-3) | `src/http/auth.rs` · `src/error` | `src/http/mod.rs` · `src/main.rs` | 2026-09-15 | 2 | 2026-09-15 `resolve_listen_socket_flag` + `resolve_tailscale_ip` for `--bind` (#329) |
| [`ingest.rs`](ingest.rs) | Opt-in POST /ingest dump; `remote_ingest` flag admits secret-authenticated non-loopback callers | `src/embed` · `src/hashutil` · `src/ingest` · `src/quality` · `src/store` · `src/features` | `src/daemon/mod.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` · `src/query/hybrid.rs` | 2026-10-03 | 2 | 2026-10-03 remote_ingest flag + peer audit log · 2026-08-16 indexed (v1 seed) |
| [`mcp.rs`](mcp.rs) | MCP HTTP/SSE transport — shared secret → read-only; seat token → full surface + stamped WriteContext | `src/mcp` · `src/write_policy` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-09-30 | 2 | 2026-09-30 `McpAuth` seat-token auth via `resolve_agent_by_key_hash` + `X-Kurultai-*` stamp headers (plan 2026-09-28-001 U4) · 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Axum daemon: /api/*, /ui/, /auth/*, /connect, optional /mcp SSE; `pg_rls_scope` middleware installs the request `PgScope` after `hub_api_auth`; `/api/hub/*` admin surface — keys GET/POST/DELETE, device pending/approve, scopes census — gated by `admin_permitted` with `hub_activity` audit | `src/brain` · `src/daemon` · `src/error` · `src/mcp` · `src/metrics` · `src/http/hub_listen.rs` · `src/http/device_auth.rs` · `src/http/device.rs` | `src/daemon/mod.rs` | 2026-10-21 | 8 | 2026-10-21 hub adoption U4 (R17/KTD7): admin routes `GET/POST /api/hub/keys`, `DELETE /api/hub/keys/{id}`, `GET /api/hub/device/pending`, `POST /api/hub/device/approve`, `GET /api/hub/scopes` behind `require_hub_admin` (member→403, admin/operator→pass, 503 without `key_store`); `log_hub_admin` audit on issue/revoke/approve · 2026-10-20 hub adoption U2: `pg_rls_scope` middleware layer (postgres) + `GET /api/hub/activity` member→403 gate (KTD1) · 2026-10-03 `remote` flag plumbed into IngestState + warn log · 2026-09-23 `/api/graph` prepared payload — `graph_cache` keyed by params, invalidated via `Store::atom_epoch`, nodes spliced byte-for-byte (#324) · 2026-09-20 `POST /api/metrics/client` — browser perf samples → client metric family (#102) · 2026-09-18 `/api/ask` `web` flag → ask_with_web (GET+POST) · 2026-09-16 merge `device::routes` (`/api/device/*`, `/connect`) · 2026-09-15 `ServeOptions.bind` plumbed to flag resolver (#329) |
| [`ui.rs`](ui.rs) | Embedded ui/ static; `/ui/` serves ui-next (new surface), `brain.html` still reachable | — | `src/daemon/mod.rs` | 2026-09-30 | 2 | 2026-09-30 default route → ui-next.html · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-21 — `auth.rs`/`mod.rs`/`device.rs` (hub adoption U4, R17/KTD7): `/api/hub/*` admin surface — key list/issue/revoke (plaintext once at mint, `is_admin` role), device pending list + `approve` reusing `approve_for_team`, and the scope census; `admin_permitted` refuses member keys while operator creds (static/CF) still pass; `request_pg_scope` maps admin principals to `PgScope::admin()`; `log_hub_admin` audits issue/revoke/approve
- 2026-10-21 — `device.rs`/`cf_access.rs` (hub adoption U3, KTD4): `/connect` on the hub lane renders an authenticated team picker (existing teams datalist + free text, `--team` prefill — approver wins), `approve_hub` validates `team_id`, mints `HubKeyStore::issue` once and revokes on a lost race; `/api/device/token` hands the flow's `issued_key` out exactly once; `KURULTAI_ADMIN_EMAILS` allowlist gates CF approvers under `feature:hub`; roster never leaves the authenticated page
- 2026-10-20 — `auth.rs`/`mod.rs` (hub adoption U2, KTD1): `request_pg_scope` resolves `PgScope` from the server-side `HubPrincipal` only; `pg_rls_scope` middleware wraps the request stack inside `hub_api_auth`; `/api/hub/activity` 403s member principals (admin-only write log)
- 2026-09-30 — `mcp.rs`: `/mcp` accepts registered agent seat tokens (full write surface, `X-Kurultai-{agent,chat}` stamps → WriteContext) alongside shared-secret read-only — the broker relay's upstream auth path
- 2026-09-23 — `mod.rs`: `/api/graph` serves prepared per-request-shape snapshots — `graph_cache` keyed by (tier, limit, quarantine, source filters), validated against `Store::atom_epoch`, nodes array spliced byte-for-byte into the response; `graph_epoch` field added (#324)
- 2026-09-20 — `POST /api/metrics/client` (mod.rs): Brain UI perf samples (nav/tier-load/fps/long-tasks/heap) validated into the client metric family; numbers + enum labels only (#102)
- 2026-09-19 — `hey.rs` `resolve_thread_id` prefers `get_thread(id)` — fixes messages routing to the wrong thread when a thread is named with another thread's UUID
- 2026-09-16 — `device.rs`: `kurultai connect` device-authorization endpoints + `/connect` approval page; `auth.rs` exempts those paths + 401 `run kurultai connect` hint; `device_auth.rs` `create_device_flow` gains `instance_id`
- 2026-09-15 — `hub_listen.rs`: `--bind` flag path + tailnet IP resolution (#329)
- 2026-09-15 — `ontology_write.rs` DELETE routes (`entity/{id}`, `link/{id}`); `auth.rs` write guard gains DELETE + prefix predicate for ontology write paths (#320)
- 2026-09-14 — `ontology_write.rs`: human-lane `POST /api/ontology/entity` + `/link` for the board (agents 403 — propose-only lane); both join SharedClosed WRITE_ROUTES (#316)
- 2026-09-11 — `proposals.rs`: `/api/ontology/proposals` list+submit (agent bearer) and `/{id}/decide` (human-only; agent keys 403); decide path joins SharedClosed write guard (#118)
- 2026-09-08 — `GET /api/db/{table}` (atoms|links) read-only browse, whitelisted sort/filter, 500-row cap
- 2026-09-06 — add `cf_access.rs`: verified Cloudflare Access JWT satisfies hub auth (`KURULTAI_CF_ACCESS_TEAM` + `KURULTAI_CF_ACCESS_AUDS`); `HubGate.cf_access`; bearer flow extracted to `authorize_bearer`
- 2026-09-06 — add `device_auth.rs` device-code login endpoints; `auth.rs` exempt `/auth/*`; `mod.rs` merge auth routes
- 2026-09-04 — add `hey.rs` message board REST; `mod.rs` HubGate agent_store fixture
- 2026-08-31 — review fixes: validate promote reason, auth returns 500 on DB errors
- 2026-08-29 — secure unprefixed query aliases (/search, /ask, etc.) under `HubAuth::ApiKey`
- 2026-08-29 — HUB-3 `hub_listen.rs` bind policy; `serve_with` start-fail before listen
- 2026-08-16 — indexed this folder (v1 seed)
