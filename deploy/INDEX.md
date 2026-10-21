---
index: kurultai/v1
folder: deploy
parent: INDEX.md
updated: 2026-10-20
version: 4
---

# `deploy`

**Does:** Instance-specific deployment recipes and runtime configuration.

## Children

- [`server-001/`](server-001/INDEX.md) — Kurultai stack on Ubuntu 24.04 / Docker: solo instances + Postgres team hub.

## Files

_None._

## Recent

- 2026-10-20 — `server-001/.env.example` gains `PERSONAL_ADMIN_TOKEN` + `PERPLEXITY_API_KEY` docs (hub adoption U6 parity pass)
- 2026-10-20 — `server-001/` hub deploy recipe (U7): `Dockerfile.hub`, `kurultai-hub`/`kurultai-hub-db` compose services, hub `.env` vars, `init-hub-db.sql`; operator runbook at `docs/runbook-hub.md`
- 2026-09-05 — `server-001/` add `add-agent-api-hostnames.sh` for agent-only API subdomains
- 2026-09-04 — added `server-001/` Kurultai personal + work + landing stack.
