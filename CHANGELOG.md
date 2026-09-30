# Changelog

## [0.6.0] — 2026-09-11

Brain wave-1 + hosted access + agent board. All version strings reconciled (`Cargo.toml`, `website/src/version.ts`, `website/package.json` = `0.6.0`).

**Brain UI (cortex-first, wave-1 of the first-principles plan):**

- Explicit nodes + synapses: lit 3D node spheres with smaller radius, synapses follow shared tags and stay inside the cortex hull (hard SDF containment + cheaper edges), morph pulse + live HUD readout, `metrics()` scene snapshot for tests (#286–#291).
- Label LOD (zoom-aware labels) + ontology group tints (#292).
- Tiered graph fetch by load tier (hot/warm/cold) and `source=repos` kept off the cortex graph (#271, #279).
- Read-only store browser at `/ui/#/db` with lane/tier filter chips and humanized copy (#293–#295).
- A2A chatboard: Hey threads/messages/reactions/presence/unread in the dashboard (#292).

**Access + identity:**

- Device-code login flow for hosted agent tokens; human UI hostname required for device approval.
- Verified Cloudflare Access JWT satisfies hub auth for hosted Brain UI human login — no API-key paste.
- 1Password-friendly human login + key-source copy; favicon + de-local hosted copy (#269, #270, #284, #288).
- Hey agent board slice: agent identity (codename = product family, `instance_id` = concurrent seat), message board storage, repo + `instance_id` WIP claims on the Repos strip (#267, #268, #274).

**Retrieval quality + harness:**

- Pond/session/transcript atoms sequestered out of the hot retrieval tier (#268).
- `recall-harness`: randomized recall/search load + correctness suite; MCP transport + corpus bootstrap via `tools/call` (#296, #297).
- Local dev-container dogfood for Debian 12 and Ubuntu 24.04 (#256).

**Fixes + repo hygiene:**

- `kurultai init --config PATH` now writes/uses the explicit config instead of the default path (#310).
- CI now runs all website unit tests (`labels`, `chatboard-mapping`, `repoLattice`, layout) — previously only `src/brain/layout/*.test.ts` ran.

## [0.5.0] — 2026-09-01

Shared hub track (product flag `hub`, default **off**). v0.5.0 Team release.

- Crew (Track A): `SqliteVecStore::open` now configures the connection for multi-process access — `journal_mode = WAL` (read back and verified), `busy_timeout` (5s, override with `KURULTAI_SQLITE_BUSY_TIMEOUT_MS`), `synchronous = NORMAL`. N `kurultai mcp` processes sharing one `store.db` no longer block each other's reads during an index run. `import --replace` now deletes stale `-wal` / `-shm` sidecars so an old WAL is not replayed onto the imported database. New `docs/crew-instance-node3.md` runbook: no daemon, nothing binds a port.
- HUB-2: optional `--features postgres` `PostgresStore` (pgvector) for `team`/`company` atoms. `open_store` remains SQLite. Personal upserts are rejected (AE4). One Postgres database per organization; copy shared-tier atoms only — never personal, never in-place convert of `store.db`.
- HUB-3: hub daemon bind policy (`hub_listen_decision`) — non-loopback + `auth=none` is a start-fail except Tailscale; public `*.up.railway.app` needs `ALLOW_PUBLIC_HUB=1`. `KURULTAI_FEATURE_HUB=1` opens Postgres only (`DATABASE_URL`). Dockerfile + [`docs/deploy/railway-hub.md`](docs/deploy/railway-hub.md). Keys stay env `KURULTAI_HUB_API_KEYS`.
- HUB-4: issued hub device keys in Postgres (`kurultai hub key issue|revoke|list`), env CSV bootstrap when the keys table is empty, AE5 `team_id` filter on hub search/ask, durable write log at `GET /api/hub/activity` and `kurultai hub log`. Network principal stamps `agent_id` on hub writes (overrides self-asserted `KURULTAI_AGENT_ID`).

## [0.4.1] — 2026-08-13

Production GitHub Release for the solo kernel (tag `v0.4.1`).

- CLI help grouped: Setup · Knowledge · Serve · Packs · Maintenance; `who_knows` alias; status lists versioned feature flags (`KURULTAI_FEATURE_<ID>`).
- Solo on-device docs: `kurultai init --docs` provisions a tagged markdown folder (`Documents/kurultai` or `~/kurultai`).
- Brain UI: `scripts/build-ui.sh` + CI job so `website/` source matches embedded `ui/` (daemon `GET /ui/`).
- Boot: graph load is no longer aborted by status polling; HTML and `/api/*` `Cache-Control: no-store`; `index` / `status` / Brain bar show UI and backend versions.

Already on `main` since v0.4.0 (included in this crate bump): MCP HTTP/SSE, thin metrics, HUB-1 visibility scopes, config-not-code adapters (inbox / ingest / folder parity). HUB-2 Postgres store is in tree behind `--features postgres` + `KURULTAI_FEATURE_HUB=1` (default off).

## [0.4.0] — 2026-07-30

Brain UI: solar · pulse · purple · max. See [GitHub Release](https://github.com/duketopceo/kurultai/releases/tag/v0.4.0).

## [0.3.0] — 2026-07-26

Unification: ingestion jobs, hardening, UI. See [GitHub Release](https://github.com/duketopceo/kurultai/releases/tag/v0.3.0).
