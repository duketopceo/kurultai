---
index: kurultai/v1
folder: src/config
parent: src/INDEX.md
updated: 2026-08-31
version: 2
---

# `src/config`

**Does:** config.toml load/merge
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`file.rs`](file.rs) | config.toml serde structs | `src/art` | `src/app/context.rs` · `src/config/loader.rs` · `src/doctor.rs` · `src/export/mod.rs` · `src/mcp/init.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`loader.rs`](loader.rs) | Load/merge config from file + env | `src/art` · `src/config` · `src/environment` · `src/error` · `src/types` | `src/app/context.rs` · `src/doctor.rs` · `src/export/mod.rs` · `src/mcp/init.rs` | 2026-08-31 | 2 | 2026-08-31 removed deprecated `obsidian` source-kind alias |
| [`mod.rs`](mod.rs) | Config module exports | `src/error` · `src/types` | `src/app/context.rs` · `src/config/loader.rs` · `src/doctor.rs` · `src/export/mod.rs` · `src/mcp/init.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-31 — `loader.rs`: removed deprecated `obsidian` source-kind alias
- 2026-08-16 — indexed this folder (v1 seed)
