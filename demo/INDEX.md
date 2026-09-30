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

- [`corpus/`](corpus/INDEX.md) — hand-curated public-safe fixture atoms

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`README.md`](README.md) | Demo overview + operator runbook for tunnel route, Access app, env wiring | — | — | 2026-09-30 | 1 | 2026-09-30 added |
| [`config.toml`](config.toml) | Demo config — isolated `~/.local/share/kurultai/demo/store.db`, corpus source, 30s poll | `KURULTAI_DEMO_CONFIG` | `src/main.rs` `demo_config_path` | 2026-09-30 | 1 | 2026-09-30 added |

## Recent

- 2026-09-30 — added: `daemon --demo` fixture-corpus skeleton (portfolio-hub phase-1 U4 / issue #43)
