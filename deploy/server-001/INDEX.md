---
index: kurultai/v1
folder: deploy/server-001
parent: deploy/INDEX.md
updated: 2026-11-09
version: 5
---

# `deploy/server-001`

**Does:** Docker Compose stack for two isolated Kurultai instances (personal + work) with a shared landing page and Cloudflare Tunnel.

## Children

- [`landing/`](landing/INDEX.md) — Static landing page pointing to `knowledge.shippedit.dev` and `work.shippedit.dev`.

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`Dockerfile.solo.dockerignore`](Dockerfile.solo.dockerignore) | Build-context exclusions used by BuildKit for `Dockerfile.solo`. | Docker build context | — | 2026-11-07 | 2 | 2026-11-07 renamed from unused `.dockerignore` (context is repo root) |
| [`.env.example`](.env.example) | Required environment variables and secrets. | real `.env` on host | — | 2026-09-04 | 1 | 2026-09-04 private fork seed |
| [`Dockerfile.solo`](Dockerfile.solo) | Solo SQLite image: cargo-chef multi-stage, digest-pinned, non-root, HEALTHCHECK, OCI labels. | Rust build deps | — | 2026-11-07 | 2 | 2026-11-07 cargo-chef + digests + healthcheck · 2026-09-04 private fork seed |
| [`add-kurultai-tunnel.sh`](add-kurultai-tunnel.sh) | Add `knowledge`/`work` DNS + tunnel ingress. | `CLOUDFLARE_API_TOKEN` | — | 2026-09-04 | 1 | 2026-09-04 private fork seed |
| [`add-agent-api-hostnames.sh`](add-agent-api-hostnames.sh) | Add `api-knowledge`/`api-work` agent-only hostnames. | `CLOUDFLARE_API_TOKEN` | — | 2026-09-05 | 1 | 2026-09-05 agent API hostnames bypass Access |
| [`docker-compose.kurultai.yml`](docker-compose.kurultai.yml) | Compose for personal, work, landing. Secrets use `:?` guards — compose fails loudly if `.env` isn't passed. Personal enables `KURULTAI_FEATURE_REMOTE_INGEST` + `SHARED_WRITE` (admin token). | `.env` populated | — | 2026-10-06 | 5 | 2026-10-06 shared_write + PERSONAL_ADMIN_TOKEN on personal svc · 2026-10-04 `PERPLEXITY_API_KEY` env passthrough on personal+work (pplx-embed backend) ·2026-10-03 remote ingest flag on personal svc · 2026-09-30 dropped cloudflared svc (bartlett tunnel owns name) + required-secret guards · 2026-09-04 private fork seed |
| [`regen-secrets.sh`](regen-secrets.sh) | Regenerate API keys and MCP/ingest secrets. | `openssl` | — | 2026-09-04 | 1 | 2026-09-04 private fork seed |
| [`redeploy.sh`](redeploy.sh) | Build or `--pull` image, cold-backup volumes, swap, health-gate, auto-rollback; `backup`/`rollback` modes. | host Docker, `.env` | — | 2026-11-07 | 2 | 2026-11-07 backup + health gate + rollback + GHCR pull · 2026-09-04 redeploy #268 +
| [`REINDEX-FROM-OTHER-REPOS.md`](REINDEX-FROM-OTHER-REPOS.md) | Webhook/reindex dispatch snippet for other repos | `gh` PAT, `kurultai-reindex` dispatch | — | 2026-09-04 | 1 | 2026-09-04 reindex trigger docs |
| [`sync-and-reindex-repos.sh`](sync-and-reindex-repos.sh) | Mirror `duketopceo/*` under `/home/khan/kurultai-repos` + `index --full`; merges managed `[sources.*]` sections instead of overwriting config.toml | git, Docker | — | 2026-10-06 | 2 | 2026-10-06 config merge (was clobbering `[embed]` → embed_dim crash loop) · 2026-09-04 Brain Repos lattice |
| [`README-CICD.md`](README-CICD.md) | Deploy-from-git: self-hosted runner on server-001 + one-time owner steps (token, labels, svc.sh, `production` env) | GitHub admin, `khan` shell | — | 2026-11-09 | 1 | 2026-11-09 added |

## Recent

- 2026-11-09 — deploy from git: self-hosted runner workflow replaces the SSH example (removed); see `README-CICD.md`
- 2026-11-07 — Dockerfile.solo cargo-chef/pinned; redeploy.sh backup+gate+rollback; see `docs/deploy.md`
- 2026-09-29 — ported from kurultai-private into the public repo; kurultai-private archived. Host `.env` on server-001 is unchanged and remains the only place real secrets live.
- 2026-09-05 — add `add-agent-api-hostnames.sh` for agent-only `api-knowledge`/`api-work` hostnames
- 2026-09-04 — index `REINDEX-FROM-OTHER-REPOS.md`
- 2026-09-04 — repos mirror + redeploy/reindex scripts for knowledge.shippedit.dev
- 2026-09-04 — initial two-instance Kurultai stack for `knowledge.shippedit.dev` and `work.shippedit.dev`.
