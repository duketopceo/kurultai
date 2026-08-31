# Railway (or docker-compose) hub — HUB-3

One **organization** hub. Personal SQLite stays on each laptop. This process opens **Postgres only** (`open_hub_store`). Never mount `store.db` on the host.

Plan: [`docs/plans/2026-08-15-001-feat-hub3-railway-transport-plan.md`](../plans/2026-08-15-001-feat-hub3-railway-transport-plan.md) · GitHub [#177](https://github.com/duketopceo/kurultai/issues/177)

## Defaults

| Setting | Hub v1 |
|---------|--------|
| Product flag | `KURULTAI_FEATURE_HUB=1` (crate default is **off**) |
| Store | `DATABASE_URL` or `KURULTAI_DATABASE_URL` (Railway injects `DATABASE_URL`) |
| Bind | `KURULTAI_HUB_BIND=all` → `0.0.0.0` |
| Auth | `KURULTAI_HUB_AUTH=api_key` + `KURULTAI_HUB_API_KEYS` (CSV plaintext or sha256 hex) |
| Public Railway hostname | **refused** unless `ALLOW_PUBLIC_HUB=1` |
| Health | `GET /health` open |
| REST | `GET /api/*` requires `Authorization: Bearer <key>` |
| MCP HTTP | existing `KURULTAI_MCP_HTTP_SECRET` (optional) |
| Solo loopback | unchanged: `127.0.0.1` + `auth=none`, SQLite |

Non-loopback + `auth=none` **exits non-zero** before listen (except `KURULTAI_HUB_BIND=tailscale` or a `100.64/10` listen address).

## Railway

1. New service from this repo. Dockerfile at repo root (`cargo build --release --locked --features postgres`). The image runs as `nobody` with `HOME=/tmp` so optional `config.toml` defaults resolve without a real home directory.
2. Add Railway Postgres. The plugin sets `DATABASE_URL`.
3. Set:
   - `KURULTAI_FEATURE_HUB=1`
   - `KURULTAI_HUB_BIND=all`
   - `KURULTAI_HUB_AUTH=api_key`
   - `KURULTAI_HUB_API_KEYS=<at least one secret>`
   - `PORT` (Railway sets this; the daemon reads it)
4. Use the **private** hostname for team clients. `*.up.railway.app` is treated as public: set `ALLOW_PUBLIC_HUB=1` only when you intend that URL, and keep API keys.
5. Do **not** attach a volume for SQLite.

Generate a bootstrap key locally (`openssl rand -hex 32`) and put the raw value (or its sha256 hex) in `KURULTAI_HUB_API_KEYS` for first deploy. After Postgres is up, **issue real device keys**:

```bash
DATABASE_URL=postgres://… kurultai hub key issue --agent alice --team eng
DATABASE_URL=postgres://… kurultai hub key list
DATABASE_URL=postgres://… kurultai hub log --limit 20
```

Plaintext is shown once at issue; only sha256 is stored. When the `hub_api_keys` table has active rows, env CSV is ignored for auth. Revoked keys return 401 immediately.

Write activity: `GET /api/hub/activity?limit=50` (Bearer required). Brain UI live feed stays at `GET /api/activity` (in-memory ring).

## Tailscale

Set `KURULTAI_HUB_BIND=tailscale` (binds `0.0.0.0` unless `KURULTAI_HUB_LISTEN` is a `100.x` address) and leave `KURULTAI_HUB_AUTH` unset. Trust tailnet ACLs. Do not enable Funnel in this recipe.

## Local compose

```bash
docker compose -f docker-compose.hub.yml up --build
curl -sS http://127.0.0.1:8421/health
curl -sS -H 'Authorization: Bearer local-hub-secret' http://127.0.0.1:8421/api/status
curl -sS http://127.0.0.1:8421/api/status   # 401
```

Second machine: same `curl` against the hub host with the bearer key. Personal notes stay on that machine’s SQLite; this hub only serves `team` / `company` atoms (AE4: personal upserts error).

## Isolation

One Postgres database = one org. Not multi-tenant SaaS. `team_id` filtering is HUB-4.
