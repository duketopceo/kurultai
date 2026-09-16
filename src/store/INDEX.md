---
index: kurultai/v1
folder: src/store
parent: src/INDEX.md
updated: 2026-09-15
version: 5
---

# `src/store`

**Does:** SQLite kernel + optional Postgres hub store
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`migrations.rs`](migrations.rs) | SQLite schema migrations | `src/error` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-09-11 | 3 | 2026-09-11 v15 `ontology_proposals` table (#118) · 2026-09-06 v14 `device_flows` table · 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Store trait, open_store, SqliteVecStore | `src/error` · `src/hashutil` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/app/context.rs` · `src/http/device_auth.rs` | 2026-09-15 | 5 | 2026-09-15 `classify_atom` in tier paths — sequester rules apply (#325) |
| [`postgres.rs`](postgres.rs) | Optional PostgresStore (--features postgres) | `src/error` · `src/hashutil` · `src/hub/activity.rs` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-09-15 | 4 | 2026-09-15 `classify_atom` in tier paths — sequester rules apply (#325) |

## Recent

- 2026-09-15 — `mod.rs`/`postgres.rs`: tier paths use `classify_atom` so `TierPolicy.rules` apply (#325)
- 2026-09-15 — `delete_ontology_entity` (cascades links) + `delete_ontology_link` on Store trait; SQLite impl, Postgres stubs (#320)
- 2026-09-11 — schema v15 `ontology_proposals`; Store: `insert/get/list/decide_ontology_proposal` (decide guarded to pending-only transitions); postgres stubs (#118)
- 2026-09-08 — `Store::db_rows` + `SqliteVecStore::db_rows_sync`: read-only whitelisted browse (atoms table / derived shared-tag links) for the /ui/db view
- 2026-09-06 — v14 `device_flows` migration; `Store` device-flow + `issue_agent_token` methods
- 2026-08-31 — review fixes: reject team atoms missing team_id, use shared hub_activity DDL
- 2026-08-29 — `database_url_from_env` (`KURULTAI_DATABASE_URL` then `DATABASE_URL`)
- 2026-08-16 — indexed this folder (v1 seed)

