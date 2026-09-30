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
| [`file.rs`](file.rs) | config.toml serde structs | `src/art` | `src/app/context.rs` · `src/config/loader.rs` · `src/doctor.rs` · `src/export/mod.rs` · `src/mcp/init.rs` | 2026-09-18 | 2 | 2026-09-18 `FileJudgeConfig` `[judge] enabled/model` · 2026-09-15 `FileTiersConfig`/`FileTierRule` — `[tiers]` section (#325) |
| [`loader.rs`](loader.rs) | Load/merge config from file + env | `src/art` · `src/config` · `src/environment` · `src/error` · `src/types` | `src/app/context.rs` · `src/doctor.rs` · `src/export/mod.rs` · `src/mcp/init.rs` | 2026-09-18 | 3 | 2026-09-18 [judge] → Config + tests · 2026-09-15 `tiers_to_policy` — strict cap/trust_lane validation (#325) |
| [`mod.rs`](mod.rs) | Config module exports | `src/error` · `src/types` | `src/app/context.rs` · `src/config/loader.rs` · `src/doctor.rs` · `src/export/mod.rs` · `src/mcp/init.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-15 — `loader.rs`: `tiers_to_policy` maps `[tiers]` rules into `TierPolicy` (#325)
- 2026-09-15 — `file.rs`/`loader.rs`: `[tiers]` + `[[tiers.rule]]` → `Config.tier_policy` (#325)
- 2026-08-31 — `loader.rs`: removed deprecated `obsidian` source-kind alias
- 2026-08-16 — indexed this folder (v1 seed)
