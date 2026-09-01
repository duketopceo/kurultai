---
index: kurultai/v1
folder: src/hub
parent: src/INDEX.md
updated: 2026-08-31
version: 1
---

# `src/hub`

**Does:** HUB-4 issued keys + write activity (Postgres only)
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`activity.rs`](activity.rs) | Append-only `hub_activity` log | `sqlx` postgres feature | `src/http/mod.rs` · `src/main.rs` · `src/store/postgres.rs` | 2026-08-31 | 2 | 2026-08-31 order by at DESC, id DESC; migrate_conn for shared DDL · 2026-08-31 HUB-4 write log |
| [`keys.rs`](keys.rs) | Issued API keys + principal resolution | `src/hashutil` · `src/hub/activity.rs` | `src/http/auth.rs` | 2026-08-31 | 3 | 2026-08-31 revoke_by_prefix targets one key via id · 2026-08-31 ensure hub_activity table migrates on connect · 2026-08-31 HUB-4 issued keys |
| [`mod.rs`](mod.rs) | Hub admin module gate | — | `src/lib.rs` | 2026-08-31 | 1 | 2026-08-31 HUB-4 module |

## Recent

- 2026-08-31 — review fixes: activity index/order, key revoke one, Postgres calls shared DDL
- 2026-08-31 — HUB-4: ensure hub_activity table migrates alongside hub_api_keys
- 2026-08-31 — HUB-4 issued keys, AE5 team filter, durable write log
