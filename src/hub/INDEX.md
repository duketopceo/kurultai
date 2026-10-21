---
index: kurultai/v1
folder: src/hub
parent: src/INDEX.md
updated: 2026-10-20
version: 3
---

# `src/hub`

**Does:** HUB-4 issued keys + write activity (Postgres only)
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`activity.rs`](activity.rs) | Append-only `hub_activity` log; `list` runs inside the request's `PgScope` tx (defense-in-depth under admin-only SELECT policy) | `sqlx` postgres feature · `src/store/scope.rs` | `src/http/mod.rs` · `src/main.rs` · `src/store/postgres.rs` | 2026-10-20 | 3 | 2026-10-20 U2: `list` scoped via `PgScope::current()` + apply_pg_scope tx (KTD1) · 2026-08-31 order by at DESC, id DESC; migrate_conn for shared DDL · 2026-08-31 HUB-4 write log |
| [`keys.rs`](keys.rs) | Issued API keys + principal resolution; `KURULTAI_PG_SKIP_MIGRATE` gate on `connect` (KTD8) | `src/hashutil` · `src/hub/activity.rs` · `src/store/postgres.rs` | `src/http/auth.rs` | 2026-10-20 | 4 | 2026-10-20 skip eager `migrate()`/`HubActivityStore::migrate` under `KURULTAI_PG_SKIP_MIGRATE=1` — runtime role `kurultai_app` has no DDL rights (hub adoption U1, KTD8) · 2026-08-31 revoke_by_prefix targets one key via id · 2026-08-31 ensure hub_activity table migrates on connect · 2026-08-31 HUB-4 issued keys |
| [`mod.rs`](mod.rs) | Hub admin module gate | — | `src/lib.rs` | 2026-08-31 | 1 | 2026-08-31 HUB-4 module |

## Recent

- 2026-10-20 — `activity.rs` (hub adoption U2, KTD1): `HubActivityStore::list` executes inside a transaction with the request's `PgScope` applied — the admin-only RLS SELECT policy is exercised on the same scoped path as atom reads
- 2026-10-20 — `keys.rs` (hub adoption U1, KTD8): `HubKeyStore::connect` honors `KURULTAI_PG_SKIP_MIGRATE=1`, sharing `postgres::skip_migrate_enabled()` with `PostgresStore`
- 2026-08-31 — review fixes: activity index/order, key revoke one, Postgres calls shared DDL
- 2026-08-31 — HUB-4: ensure hub_activity table migrates alongside hub_api_keys
- 2026-08-31 — HUB-4 issued keys, AE5 team filter, durable write log
