# Deploying the hosted instances

How the two hosted Kurultai instances run, and how to redeploy, check, and
roll back. Everything here is derived from `deploy/server-001/`; if the repo and
this page disagree, the repo wins. Secrets are **server-local** and never in git.

## Where and how they are hosted

| Instance | Hostname | Container | Port | Volume |
|----------|----------|-----------|------|--------|
| Personal ("Ulaanbaatar") | `knowledge.shippedit.dev` | `kurultai-personal` | 8421 | `kurultai-personal-data` (`/data`) |
| Work | `work.shippedit.dev` | `kurultai-work` | 8422 | `kurultai-work-data` (`/data`) |

- **Host:** server-001. Repo checkout at `~/kurultai`, secrets in `~/kurultai/.env`
  (server-local, mode 600, never committed).
- **Runtime:** Docker, solo image `kurultai:solo` (SQLite store in `/data`, non-root
  uid 1000), two containers from the same image on the `kurultai` Docker network,
  plus a static `landing` nginx. Compose file:
  `deploy/server-001/docker-compose.kurultai.yml`.
- **Ingress:** Cloudflare Tunnel, run by the bartlett project's `cloudflared`
  (this repo does not run its own tunnel). Ingress rules and DNS are created by
  `deploy/server-001/add-kurultai-tunnel.sh`
  (`knowledge.` to `http://kurultai-personal:8421`, `work.` to the work container),
  and `add-agent-api-hostnames.sh` adds agent-only API hostnames that bypass Access.
  Humans sign in through Cloudflare Access (JWT verified in the daemon via
  `KURULTAI_CF_ACCESS_TEAM` / `KURULTAI_CF_ACCESS_AUDS`); agents use API keys
  or the MCP secret.
- **Secrets** (all in the server `.env`, consumed by compose with `:?` guards so a
  missing value fails loudly): `PERSONAL_ADMIN_TOKEN`, `PERSONAL_API_KEYS`,
  `PERSONAL_MCP_SECRET`, `PERSONAL_INGEST_SECRET`, `WORK_API_KEYS`,
  `WORK_MCP_SECRET`, `WORK_INGEST_SECRET`, optional `PERPLEXITY_API_KEY`.
  Regenerate with `deploy/server-001/regen-secrets.sh`.
- **Do not deploy on cluster2.** Its old Ulaanbaatar store is wiped.
- **Always use `deploy/server-001/redeploy.sh`.** It passes `--env-file`; a bare
  `docker compose up` without it fails the secret guards (or worse, recreates
  with blanks).

## Image

`deploy/server-001/Dockerfile.solo` is multi-stage: cargo-chef planner, a cached
dependency layer, the app build, then a `debian:bookworm-slim` runtime with only
`ca-certificates` and `curl`, a non-root `kurultai` user, a `HEALTHCHECK` on
`/health`, and OCI labels. Base images are pinned by digest. To bump them, resolve
the new multi-arch index digest for `lukemathwalker/cargo-chef:latest-rust-<ver>-bookworm`
and `debian:bookworm-slim` and edit the two `FROM ...@sha256:` lines (and the
`BACKUP_IMAGE` digest in `redeploy.sh`).

Releases: pushing a `vX.Y.Z` tag runs `.github/workflows/publish-image.yml`, which
publishes `ghcr.io/duketopceo/kurultai:{X.Y.Z, X.Y, vX.Y.Z, <sha>}` (linux/amd64).
That workflow **does not deploy** and holds no server credentials.

## Redeploy (manual, on server-001)

```bash
cd ~/kurultai
git fetch origin && git checkout main && git pull --ff-only
deploy/server-001/redeploy.sh                # build from this checkout
# or, using a published release image:
deploy/server-001/redeploy.sh --pull v0.7.1
deploy/server-001/sync-and-reindex-repos.sh  # optional: refresh Brain Repos index
```

What the script does:

1. Preflight: Docker present, `.env` present.
2. Keeps the running image as `kurultai:solo-prev`.
3. Builds `kurultai:solo` (also tagged `kurultai:solo-<sha>`), or pulls the GHCR tag.
4. Stops both solo containers and takes a **cold backup** of each `/data` volume to
   `~/kurultai-backups/<UTC timestamp>/<volume>.tar.gz` (newest 7 kept; override with
   `KEEP_BACKUPS`, `KURULTAI_BACKUP_DIR`). Stopped means the SQLite WAL is idle,
   so the tarball is consistent. Downtime is the backup plus recreate, typically seconds.
5. `docker compose --env-file .env up -d --force-recreate`.
6. Health gate: waits up to `HEALTH_TIMEOUT` (120 s) for both containers to report
   `healthy`.
7. If the gate fails it re-tags `kurultai:solo-prev` as `kurultai:solo`, recreates, and
   re-checks. It exits non-zero either way and prints the backup path.

Other modes: `redeploy.sh backup` (backup only), `redeploy.sh rollback`.

## Health checks

```bash
docker ps --filter name=kurultai --format '{{.Names}} {{.Status}}'
docker exec kurultai-personal curl -fsS http://127.0.0.1:8421/health
docker exec kurultai-work     curl -fsS http://127.0.0.1:8422/health
docker exec kurultai-personal kurultai status
curl -fsS https://knowledge.shippedit.dev/health     # through the tunnel
docker logs --tail 100 kurultai-personal
```

## Rollback

- **Bad image, data fine:** `deploy/server-001/redeploy.sh rollback` (restores
  `kurultai:solo-prev`). Older builds stay available as `kurultai:solo-<sha>` or
  `kurultai:solo-<tag>`; to pin one, `docker tag kurultai:solo-<sha> kurultai:solo`
  then `redeploy.sh` is not needed, just
  `docker compose -f deploy/server-001/docker-compose.kurultai.yml --env-file .env up -d --force-recreate kurultai-personal kurultai-work`.
- **Bad data (a migration or corruption):** restore the volume from a backup.

```bash
cd ~/kurultai
COMPOSE="docker compose -f deploy/server-001/docker-compose.kurultai.yml --env-file .env"
B=$(cat ~/kurultai-backups/LATEST)                   # or any timestamp dir
$COMPOSE stop kurultai-personal
VOL=$(docker inspect kurultai-personal --format '{{range .Mounts}}{{if eq .Destination "/data"}}{{.Name}}{{end}}{{end}}')
docker run --rm -v "$VOL":/data -v "$B":/backup:ro \
  debian:bookworm-slim sh -c 'rm -rf /data/* /data/.[!.]* 2>/dev/null; tar -xzf /backup/'"$VOL"'.tar.gz -C /data && chown -R 1000:1000 /data'
$COMPOSE up -d kurultai-personal
```

Combine with the image rollback if the new build changed the schema (the old image
cannot read a newer schema).

## Volumes and backups

| What | Where | Backed up by |
|------|-------|--------------|
| Personal store, config | volume `kurultai-personal-data` | `redeploy.sh` (every deploy) |
| Work store, config | volume `kurultai-work-data` | `redeploy.sh` (every deploy) |
| Org checkouts for Brain Repos | `/home/khan/kurultai-repos` (bind, read-only) | not needed, re-synced by `sync-and-reindex-repos.sh` |
| Secrets | `~/kurultai/.env` | operator's password manager; **not** in backups |

Backups live on the same host. Copy `~/kurultai-backups/` off-box (rsync or restic) if
the host's disk is not enough protection. For a nightly backup, run
`deploy/server-001/redeploy.sh backup` from cron at a quiet hour (it stops the
containers for the duration of the tar).

## Existing CI deploy

`.github/workflows/deploy-server-001.yml` can SSH to the host and run `redeploy.sh`
on push to `main` when `KURULTAI_DEPLOY_HOST` / `KURULTAI_DEPLOY_SSH_KEY` secrets are
set (inside the `production` environment). If you want approval-gated deploys, add
required reviewers to that environment, or remove the `push` trigger and keep
`workflow_dispatch`. The image-publish workflow is independent of it.
