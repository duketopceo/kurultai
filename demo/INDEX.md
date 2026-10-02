---
index: kurultai/v1
folder: demo
parent: INDEX.md
updated: 2026-09-30
version: 1
---

# `demo`

**Does:** Isolated fixture-corpus demo mode (`kurultai daemon --demo`) — public-safe hosted-demo skeleton behind Cloudflare Tunnel + Access
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`corpus/`](corpus/INDEX.md) — public-safe fixture atoms (6 curated + `gen/` 202 generated)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`README.md`](README.md) | Demo overview + operator runbook for tunnel route, Access app, env wiring | — | — | 2026-09-30 | 1 | 2026-09-30 added |
| [`config.toml`](config.toml) | Demo config — isolated `~/.local/share/kurultai/demo/store.db`, corpus source, 30s poll | `KURULTAI_DEMO_CONFIG` | `src/main.rs` `demo_config_path` | 2026-09-30 | 1 | 2026-09-30 added |
| [`Dockerfile`](Dockerfile) | Slim solo-store image for the demo daemon — corpus baked at `/app/demo`, runs as `nobody` | `docker-compose.demo.yml` | — | 2026-09-30 | 1 | 2026-09-30 added |
| [`generate-corpus.py`](generate-corpus.py) | Seeded generator for `corpus/gen/` — ~200 cross-linked fixture atoms (6 clusters + sync-notes) | — | `demo/corpus/gen` | 2026-10-01 | 1 | 2026-10-01 added (6 atoms was too thin for graph/testing) |

## Recent

- 2026-10-01 — `generate-corpus.py` + `corpus/gen/` 202 atoms (6 curated was too thin for the brain graph)
- 2026-09-30 — added: `daemon --demo` fixture-corpus skeleton (portfolio-hub phase-1 U4 / issue #43)
