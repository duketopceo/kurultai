---
index: kurultai/v1
folder: src/daemon
parent: src/INDEX.md
updated: 2026-08-29
version: 2
---

# `src/daemon`

**Does:** Poll + fs watch
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | Background poll + notify watch loop | `src/connectors` · `src/error` · `src/http` · `src/mcp` · `src/pipeline` | `src/http/mod.rs` | 2026-09-20 | 4 | 2026-09-20 `WATCH_MIN_INTERVAL` 30s floor between watch-triggered cycles — sustained event streams could hot-loop at ~2,000 cycles/46min · 2026-09-15 `DaemonOptions.bind` → `ServeOptions` (#329) |

## Recent

- 2026-09-20 — `WATCH_MIN_INTERVAL` (30s) floors watch-triggered index cycles; sustained inotify streams previously ran ~2,000 cycles/46min at ~824% CPU
- 2026-08-29 — `serve_with` gets hub gate and bind_all from env (HUB-3)
- 2026-08-16 — indexed this folder (v1 seed)

