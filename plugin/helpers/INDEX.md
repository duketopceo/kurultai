---
index: kurultai/v1
folder: plugin/helpers
parent: plugin/INDEX.md
updated: 2026-08-27
version: 1
---

# `plugin/helpers`

**Does:** Shared Python helpers (client, config, MCP, security)
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`INDEX.md`](INDEX.md) | INDEX.md | — | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`client.py`](client.py) | Daemon HTTP client: find_binary, call, answer/hits formatters | helpers/config.py | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`config.py`](config.py) | resolve_config: merges defaults + user settings for daemon binding | — | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`mcp_client.py`](mcp_client.py) | MCP stdio client for structured remember writes | helpers/client.py · helpers/config.py | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |
| [`security.py`](security.py) | sanitize_content + loopback URL safety guards | — | — | 2026-08-27 | 1 | 2026-08-27 indexed (v1 seed) |

## Recent

- 2026-08-27 — indexed (v1 seed)
