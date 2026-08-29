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
| [`mod.rs`](mod.rs) | Background poll + notify watch loop | `src/connectors` · `src/error` · `src/http` · `src/mcp` · `src/pipeline` | `src/http/mod.rs` | 2026-08-29 | 2 | 2026-08-29 pass hub gate + bind_all from env · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-29 — `serve_with` gets hub gate and bind_all from env (HUB-3)
- 2026-08-16 — indexed this folder (v1 seed)

