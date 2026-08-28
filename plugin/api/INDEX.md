---
index: kurultai/v1
folder: plugin/api
parent: plugin/INDEX.md
updated: 2026-08-27
version: 2
---

# `plugin/api`

**Does:** A0 API handlers: brain endpoints + loopback daemon proxy
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`INDEX.md`](INDEX.md) | INDEX.md | — | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`brain.py`](brain.py) | A0 API handler exposing brain endpoints to the WebUI | — | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`kproxy.py`](kproxy.py) | Loopback-only daemon proxy to 127.0.0.1:8421 with endpoint whitelist | — | — | 2026-08-27 | 2 | 2026-08-27 drop /api/open from whitelist (host spawn CSRF risk) |

## Recent

- 2026-08-27 — kproxy: remove /api/open from whitelist (security)
- 2026-08-27 — indexed (v1 seed)
