---
index: kurultai/v1
folder: src/http
parent: src/INDEX.md
updated: 2026-09-15
version: 7
---

# `src/http`

**Does:** Daemon HTTP + Brain UI + MCP SSE
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`auth.rs`](auth.rs) | Hub API-key / admin token gate + CF Access fallback | `src/hashutil` · `src/hub/keys.rs` · `src/http/cf_access.rs` | `src/daemon/mod.rs` | 2026-09-14 | 7 | 2026-09-14 `/api/ontology/entity` + `/link` join WRITE_ROUTES (#316) · 2026-09-11 proposals decide route joins SharedClosed write guard (#118) · 2026-09-06 CF Access JWT fallback in `hub_api_auth` · 2026-09-06 exempt `/auth/*` from hub API-key middleware · 2026-08-31 resolve token first; 500 on DB errors · 2026-08-16 indexed (v1 seed) |
| [`cf_access.rs`](cf_access.rs) | Verify Cloudflare Access JWTs (`CF_Authorization` cookie / `Cf-Access-Jwt-Assertion`) for human browser auth | `jsonwebtoken` · `reqwest` | `src/http/auth.rs` | 2026-09-06 | 1 | 2026-09-06 added — Google/OTP Access login satisfies `/api/*` auth |
| [`device_auth.rs`](device_auth.rs) | Device-code login endpoints (`/auth/device/*`) for hosted agent tokens | `src/http` · `src/store` · `src/hashutil` | `src/http/mod.rs` | 2026-09-06 | 1 | 2026-09-06 added |
| [`hey.rs`](hey.rs) | Multi-agent message board REST (`/api/hey/...`) | `src/store` · `src/brain` · `src/error` | `src/daemon/mod.rs` · `src/http/mod.rs` · `src/http/proposals.rs` · `src/http/ontology_write.rs` | 2026-09-14 | 3 | 2026-09-14 `require_agent` reused by ontology_write agent-refusal (#316) · 2026-09-11 `require_agent` now pub(crate) for proposal submit auth (#118) 2026-09-11 `require_agent` now pub(crate) for proposal submit auth (#118) · 2026-09-04 message board endpoints + clamp cleanup |
| [`proposals.rs`](proposals.rs) | O3 ontology proposal queue REST (`/api/ontology/proposals`, decide) — agent bearer submits, humans decide | `src/http/hey.rs` · `src/http/auth.rs` · `src/ontology` | `src/http/mod.rs` | 2026-09-11 | 1 | 2026-09-11 added (#118) |
| [`ontology_write.rs`](ontology_write.rs) | Human-lane ontology board writes (`POST /api/ontology/entity`, `/link`) — agents refused | `src/http/hey.rs` · `src/http/auth.rs` · `src/ontology` | `src/http/mod.rs` | 2026-09-14 | 1 | 2026-09-14 added (#316) |
| [`hub_listen.rs`](hub_listen.rs) | Pure bind × auth start-fail (HUB-3) | `src/http/auth.rs` · `src/error` | `src/http/mod.rs` · `src/main.rs` | 2026-09-15 | 2 | 2026-09-15 `resolve_listen_socket_flag` + `resolve_tailscale_ip` for `--bind` (#329) |
| [`ingest.rs`](ingest.rs) | Opt-in POST /ingest dump | `src/embed` · `src/hashutil` · `src/ingest` · `src/quality` · `src/store` | `src/daemon/mod.rs` · `src/connectors/inbox.rs` · `src/connectors/json.rs` · `src/connectors/markdown.rs` · `src/query/hybrid.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mcp.rs`](mcp.rs) | MCP HTTP/SSE transport | `src/mcp` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Axum daemon: /api/*, /ui/, /auth/*, optional /mcp SSE | `src/brain` · `src/daemon` · `src/error` · `src/mcp` · `src/metrics` · `src/http/hub_listen.rs` · `src/http/device_auth.rs` | `src/daemon/mod.rs` | 2026-09-15 | 9 | 2026-09-15 `ServeOptions.bind` plumbed to flag resolver (#329) |
| [`ui.rs`](ui.rs) | Embedded ui/ static + /ui/ slash redirect | — | `src/daemon/mod.rs` | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

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
