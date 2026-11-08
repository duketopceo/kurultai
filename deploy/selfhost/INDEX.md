---
index: kurultai/v1
folder: deploy/selfhost
parent: deploy/INDEX.md
updated: 2026-11-07
version: 1
---

# `deploy/selfhost`

**Does:** Reverse-proxy and tunnel examples for self-hosters
**Up:** [`deploy/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`Caddyfile.example`](Caddyfile.example) | Caddy reverse proxy with automatic HTTPS | `docker-compose.yml` | — | 2026-11-07 | 1 | 2026-11-07 added |
| [`cloudflared.compose.example.yml`](cloudflared.compose.example.yml) | Cloudflare Tunnel sidecar overlay | `docker-compose.yml` | — | 2026-11-07 | 1 | 2026-11-07 added |

## Recent

- 2026-11-07 — added Caddy and Cloudflare Tunnel examples for `docs/self-host.md`
