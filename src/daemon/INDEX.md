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
| [`mod.rs`](mod.rs) | Background poll + notify watch loop | `src/connectors` · `src/error` · `src/http` · `src/mcp` · `src/pipeline` | `src/http/mod.rs` | 2026-09-15 | 3 | 2026-09-15 `DaemonOptions.bind` → `ServeOptions` (#329) |

## Recent

- 2026-08-29 — `serve_with` gets hub gate and bind_all from env (HUB-3)
- 2026-08-16 — indexed this folder (v1 seed)

