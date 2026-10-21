---
index: kurultai/v1
folder: deploy/server-001
parent: deploy/INDEX.md
updated: 2026-10-20
version: 5
---

# `deploy/server-001`

**Does:** Docker Compose stack for two isolated Kurultai instances (personal + work), a Postgres-backed team hub (kurultai-hub + pgvector sidecar), a shared landing page, and Cloudflare Tunnel.

## Children

- [`landing/`](landing/INDEX.md) — Static landing page pointing to `knowledge.shippedit.dev` and `work.shippedit.dev`.

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`.dockerignore`](.dockerignore) | Build-context exclusions for the `kurultai:solo` image build. | Docker build context | — | 2026-09-29 | 1 | 2026-09-29 port: keep secrets/target out of image build |
| [`.env.example`](.env.example) | Required environment variables and secrets (incl. hub Postgres + Access app vars). | real `.env` on host | — | 2026-10-20 | 3 | 2026-10-20 `PERSONAL_ADMIN_TOKEN` (required `:?` guard since 10-06, was undocumented) + `PERPLEXITY_API_KEY` optional passthrough (hub adoption U6 parity pass) · 2026-10-20 hub var placeholders (KURULTAI_HUB_*, KURULTAI_ADMIN_EMAILS, HUB_CF_ACCESS_AUDS) · 2026-09-04 private fork seed |
| [`Dockerfile.solo`](Dockerfile.solo) | Solo SQLite Kurultai image (non-root, persistent /data). | Rust build deps | — | 2026-09-04 | 1 | 2026-09-04 private fork seed |
| [`Dockerfile.hub`](Dockerfile.hub) | Hub Postgres Kurultai image (`--features postgres`, non-root, port 8423). | Rust build deps | `docker-compose.kurultai.yml` · `redeploy.sh` | 2026-10-20 | 1 | 2026-10-20 added (hub adoption U7) |
| [`add-kurultai-tunnel.sh`](add-kurultai-tunnel.sh) | Add `knowledge`/`work` DNS + tunnel ingress. | `CLOUDFLARE_API_TOKEN` | — | 2026-09-04 | 1 | 2026-09-04 private fork seed |
| [`add-agent-api-hostnames.sh`](add-agent-api-hostnames.sh) | Add `api-knowledge`/`api-work` agent-only hostnames. | `CLOUDFLARE_API_TOKEN` | — | 2026-09-05 | 1 | 2026-09-05 agent API hostnames bypass Access |
| [`docker-compose.kurultai.yml`](docker-compose.kurultai.yml) | Compose for personal, work, landing, plus `kurultai-hub` :8423 (`KURULTAI_FEATURE_HUB=1`, RLS app/migrate DSNs) and `kurultai-hub-db` (pgvector pg16, `kurultai-hub-pgdata`). Secrets use `:?` guards — compose fails loudly if `.env` isn't passed/populated. Personal enables `KURULTAI_FEATURE_REMOTE_INGEST` + `SHARED_WRITE` (admin token). | `.env` populated | — | 2026-10-20 | 6 | 2026-10-20 kurultai-hub + kurultai-hub-db services + pgdata volume (U7) · 2026-10-06 shared_write + PERSONAL_ADMIN_TOKEN on personal svc · 2026-10-04 `PERPLEXITY_API_KEY` env passthrough on personal+work (pplx-embed backend) |
| [`regen-secrets.sh`](regen-secrets.sh) | Regenerate API keys and MCP/ingest secrets. | `openssl` | — | 2026-09-04 | 1 | 2026-09-04 private fork seed |
| [`redeploy.sh`](redeploy.sh) | Build `kurultai:solo` + `kurultai:hub` + compose up on server-001 | host Docker, `.env` | — | 2026-10-20 | 2 | 2026-10-20 hub image build + health check · 2026-09-04 redeploy #268 +
| [`REINDEX-FROM-OTHER-REPOS.md`](REINDEX-FROM-OTHER-REPOS.md) | Webhook/reindex dispatch snippet for other repos | `gh` PAT, `kurultai-reindex` dispatch | — | 2026-09-04 | 1 | 2026-09-04 reindex trigger docs |
| [`sync-and-reindex-repos.sh`](sync-and-reindex-repos.sh) | Mirror `duketopceo/*` under `/home/khan/kurultai-repos` + `index --full`; merges managed `[sources.*]` sections instead of overwriting config.toml | git, Docker | — | 2026-10-06 | 2 | 2026-10-06 config merge (was clobbering `[embed]` → embed_dim crash loop) · 2026-09-04 Brain Repos lattice |
| [`github-deploy-workflow.yml.example`](github-deploy-workflow.yml.example) | Actions template: main push → SSH redeploy + reindex; `repository_dispatch` reindex-only | secrets | — | 2026-09-04 | 1 | 2026-09-04 CI/CD hook |
| [`init-hub-db.sql`](init-hub-db.sql) | One-time hub DB bootstrap: `kurultai_migrate` (BYPASSRLS) + `kurultai_app` (NOINHERIT) roles + grants. | `kurultai-hub-db` psql superuser | `docs/runbook-hub.md` | 2026-10-20 | 1 | 2026-10-20 added (hub adoption U7, R11/KTD8) |

## Recent

- 2026-10-20 — hub adoption U6 env parity (plan 2026-10-19-2053): `.env.example` gains `PERSONAL_ADMIN_TOKEN` (required `:?` guard) + `PERPLEXITY_API_KEY` passthrough docs
- 2026-10-20 — hub deploy recipe (plan 2026-10-19-2053 U7): `Dockerfile.hub`, `kurultai-hub` + `kurultai-hub-db` compose services + `kurultai-hub-pgdata`, hub `.env` vars, `redeploy.sh` builds `kurultai:hub`; `init-hub-db.sql` catalogued — operator flow in `docs/runbook-hub.md`
- 2026-09-29 — ported from kurultai-private into the public repo; kurultai-private archived. Host `.env` on server-001 is unchanged and remains the only place real secrets live.
- 2026-09-05 — add `add-agent-api-hostnames.sh` for agent-only `api-knowledge`/`api-work` hostnames
- 2026-09-04 — index `REINDEX-FROM-OTHER-REPOS.md`
- 2026-09-04 — repos mirror + redeploy/reindex scripts for knowledge.shippedit.dev
- 2026-09-04 — initial two-instance Kurultai stack for `knowledge.shippedit.dev` and `work.shippedit.dev`.
