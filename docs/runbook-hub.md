# Hub runbook — server-001 (knowledge deployment)

One shared brain: `kurultai-hub` (`--features postgres`, `KURULTAI_FEATURE_HUB=1`)
plus `kurultai-hub-db` (`pgvector/pgvector:pg16`) on the existing `kurultai`
compose network. Solo lanes `kurultai-personal` (`:8421`) and `kurultai-work`
(`:8422`) are untouched — hub mode swaps the daemon's whole store, so it runs as
its own container, never co-process.

Humans approve joins at `https://hub.shippedit.dev` (admin-scoped Cloudflare
Access). Agents and members use `https://api-hub.shippedit.dev` (bearer hub keys
only). Postgres enforces `visibility`/`team_id` with `FORCE ROW LEVEL SECURITY`
under the app-level filter — the app role is non-owner and cannot run DDL.

Plan: [`docs/plans/2026-10-19-2053-feat-hub-adoption-easy-connect-plan.md`](plans/2026-10-19-2053-feat-hub-adoption-easy-connect-plan.md)

## Postgres roles

| Role | Used by | Power |
|------|---------|-------|
| `postgres` | db container bootstrap only | superuser |
| `kurultai_migrate` | schema migrations + `kurultai hub …` CLI | BYPASSRLS, owns tables |
| `kurultai_app` | daemon runtime (`DATABASE_URL`) | NOINHERIT, non-owner, DML only — RLS applies |

The daemon container carries both DSNs: `DATABASE_URL` (`kurultai_app`, runtime)
and `KURULTAI_MIGRATE_DATABASE_URL` (`kurultai_migrate`, admin). App connections
never migrate (`KURULTAI_PG_SKIP_MIGRATE=1`).

## 1. Prerequisites

1. Add the hub vars to server-local `.env` (never committed — template in
   `deploy/server-001/.env.example`):

   ```
   KURULTAI_HUB_DB_PASSWORD        # kurultai_app — openssl rand -hex 32
   KURULTAI_HUB_MIGRATE_DB_PASSWORD # kurultai_migrate — openssl rand -hex 32
   KURULTAI_HUB_DB_SUPERPASSWORD   # POSTGRES_PASSWORD for kurultai-hub-db
   KURULTAI_ADMIN_EMAILS           # CSV; /connect approver allowlist
   HUB_CF_ACCESS_AUDS              # AUD tag(s) of the admin-scoped Access app
   ```

   The compose `:?` guards fail **every** command on
   `docker-compose.kurultai.yml` (including solo-only ops) until these exist.

2. Cloudflare Access application (Zero Trust, team `duketopceo`), admin-scoped
   interactive policy on `hub.shippedit.dev`. Copy its AUD tag into
   `HUB_CF_ACCESS_AUDS`.
   - Keep `api-hub.shippedit.dev` out of interactive Access policies — member
     `connect` calls and agent `/mcp` traffic carry hub keys, not Access JWTs;
     an interactive policy would edge-block them (same reason
     `api-knowledge.`/`api-work.` bypass Access). A non-interactive
     service-token policy is acceptable later (#333, deferred).

3. Tunnel ingress — the `shippedit-server` cloudflared config lives in the
   shippedit repo, not here. Add:

   | hostname | service |
   |----------|---------|
   | `hub.shippedit.dev` | `http://kurultai-hub:8423` |
   | `api-hub.shippedit.dev` | `http://kurultai-hub:8423` |

   plus CNAMEs `hub` and `api-hub` → `<tunnel-id>.cfargotunnel.com`
   (same pattern as `deploy/server-001/add-agent-api-hostnames.sh`).

## 2. Deploy

```bash
# on server-001, kurultai repo root
./deploy/server-001/redeploy.sh        # builds kurultai:solo + kurultai:hub, compose up -d
docker ps                              # kurultai-hub, kurultai-hub-db healthy
docker exec kurultai-hub curl -fsS http://127.0.0.1:8423/health
```

## 3. One-time DB bootstrap (superuser)

`deploy/server-001/init-hub-db.sql` creates `kurultai_migrate` + `kurultai_app`
and grants on every runtime table. Run it once as the cluster superuser. Role
passwords must equal the `.env` values — pass them via the psql variables the
script's header declares:

```bash
set -a; . ./.env; set +a
docker exec -i kurultai-hub-db psql -U postgres -d kurultai \
  -v app_password="$KURULTAI_HUB_DB_PASSWORD" \
  -v migrate_password="$KURULTAI_HUB_MIGRATE_DB_PASSWORD" \
  < deploy/server-001/init-hub-db.sql
```

## 4. Migrate (schema + RLS)

App role never runs DDL. Any store-opening CLI run against the migrate DSN
applies migrations — `status` opens `PostgresStore`:

```bash
docker exec kurultai-hub sh -c \
  'KURULTAI_PG_SKIP_MIGRATE=0 DATABASE_URL="$KURULTAI_MIGRATE_DATABASE_URL" kurultai status'
```

Verify `FORCE ROW LEVEL SECURITY` + policies on `knowledge_atoms`:

```bash
docker exec kurultai-hub-db psql -U postgres -d kurultai -c '\d+ knowledge_atoms'
```

## 5. First admin key

`kurultai hub` needs the postgres-feature build — the `kurultai:hub` image has
it. Run admin CLI against the migrate DSN:

```bash
docker exec kurultai-hub sh -c \
  'DATABASE_URL="$KURULTAI_MIGRATE_DATABASE_URL" kurultai hub key issue --agent luke --team ops --admin'
```

Plaintext is shown once; sha256 is stored. Save it — this is the
`KURULTAI_ADMIN_KEY` web/ uses server-side (U8) and the bearer for
`api-hub.shippedit.dev/api/hub/*` admin endpoints.

```bash
# list / revoke
docker exec kurultai-hub sh -c 'DATABASE_URL="$KURULTAI_MIGRATE_DATABASE_URL" kurultai hub key list'
docker exec kurultai-hub sh -c 'DATABASE_URL="$KURULTAI_MIGRATE_DATABASE_URL" kurultai hub key revoke --prefix <12char>'
```

## 6. Member join (connect)

```bash
# member laptop — installs a hub-bound key + writes the remote MCP entry
kurultai connect https://api-hub.shippedit.dev            # optional: --team eng prefills
```

1. CLI POSTs `/api/device/code` and prints the approval URL.
2. An admin (email in `KURULTAI_ADMIN_EMAILS`) opens
   `https://hub.shippedit.dev/connect`, passes Access, picks the team —
   existing list or "new team" free text — and approves. The key is minted at
   approve; the approver's team choice wins over `--team`.
3. The member's token poll returns the key once; `connect` stores it and wires
   the agent's MCP config to `https://api-hub.shippedit.dev/mcp`.

Verify from the member side: `search`/`remember` see `company` + own-`team`
atoms only.

## 7. AE6 — RLS proof

As `kurultai_app`, an `eng` session returns zero `sales` rows even when asked:

```bash
set -a; . ./.env; set +a
docker exec -e PGPASSWORD="$KURULTAI_HUB_DB_PASSWORD" -i kurultai-hub-db \
  psql -U kurultai_app -d kurultai <<'SQL'
BEGIN;
SELECT set_config('kurultai.team_id', 'eng', true);
SELECT count(*) FROM knowledge_atoms WHERE team_id = 'sales';              -- 0
SELECT count(*) FROM knowledge_atoms WHERE visibility = 'team';            -- eng rows only
ROLLBACK;
BEGIN;
SELECT set_config('kurultai.is_admin', 'on', true);
SELECT count(*) FROM knowledge_atoms;                                      -- all shared rows
ROLLBACK;
SQL
```

No GUC set → `team_id` is NULL → company-only (fail closed).

## 8. Rollback (AE8)

Solo lanes never touch Postgres — rollback removes only the hub:

```bash
docker compose -f deploy/server-001/docker-compose.kurultai.yml --env-file .env \
  stop kurultai-hub kurultai-hub-db
docker compose -f deploy/server-001/docker-compose.kurultai.yml --env-file .env \
  rm -f kurultai-hub kurultai-hub-db
docker volume rm kurultai_kurultai-hub-pgdata     # destroys all shared atoms — pg_dump first
```

Then delete the `hub.`/`api-hub.` tunnel ingress entries + CNAMEs and the
Access application. `knowledge.`/`work.` keep serving the SQLite brains
throughout. The hub `:?` vars must stay in `.env` while the services remain in
the compose file — remove them only together with the services.

## 9. Backup

```bash
docker exec kurultai-hub-db pg_dump -U postgres -d kurultai > kurultai-hub-$(date +%F).sql
```

**Access/retention:** the dump contains **every team's atoms** — `visibility`/
`team_id` are row-level labels, not physical separation. Treat dumps like an
admin credential: admin-only storage, short retention. Detailed policy is
deferred (plan U7).
