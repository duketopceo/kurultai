---
index: kurultai/v1
folder: src/store
parent: src/INDEX.md
updated: 2026-10-03
version: 7
---

# `src/store`

**Does:** SQLite kernel + optional Postgres hub store
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`migrations.rs`](migrations.rs) | SQLite schema migrations | `src/error` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-10-03 | 5 | schema v17 `superseded_at`/`superseded_by` + idx ·2026-09-16 v16 `agent_seats` + `device_flows.instance_id` · 2026-09-11 v15 `ontology_proposals` table (#118) · 2026-09-06 v14 `device_flows` table · 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Store trait, open_store, SqliteVecStore | `src/error` · `src/hashutil` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/app/context.rs` · `src/http/device_auth.rs` · `src/http/device.rs` | 2026-10-04 | 10 | U5: `duplicate_content_groups` + `prune_stale_ontology` (trait+SQLite) ·2026-10-03 U4: `superseded_at/by` cols in ATOM_COLUMNS+hydrate; `SearchFilter.as_of/include_superseded` + `mark_superseded`; fts+vector supersede predicates ·2026-09-23 `atom_epoch()` — mutation epoch on `SqliteVecStore` (upsert/delete/lane/auto-merge bump; `touch_access` doesn't) for prepared `/api/graph` invalidation; `u64::MAX` default = untrackable (#324) · 2026-09-19 `get_thread(id)` added — id-vs-name collision resolution for Hey threads · 2026-09-16 `issue_agent_seat_token`/`revoke_agent`/`deny_device_flow`; key-hash resolve covers seats · 2026-09-15 `classify_atom` in tier paths — sequester rules apply (#325) |
| [`postgres.rs`](postgres.rs) | Optional PostgresStore (--features postgres) | `src/error` · `src/hashutil` · `src/hub/activity.rs` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-10-04 | 6 | U5: `duplicate_content_groups` + `prune_stale_ontology` (Postgres parity) ·2026-10-03 U4: superseded columns ddl + fts/vector predicates + `mark_superseded` ·2026-09-15 `classify_atom` in tier paths — sequester rules apply (#325) |

## Recent

- 2026-10-03 — `mod.rs`/`postgres.rs`/`migrations.rs`: bi-temporal-lite (U4) — schema v17 superseded_at/by, SearchFilter as_of+include_superseded, mark_superseded, supersede predicates in fts+vector search

- 2026-09-23 — `mod.rs`: `Store::atom_epoch()` + `SqliteVecStore.atom_epoch` counter bumped on upsert/batch/delete/lane/auto-merge (not `touch_access`); default `u64::MAX` = untrackable store → always-live assembly (#324)
- 2026-09-19 — `mod.rs`: `get_thread(id)` added (trait + SqliteVecStore) for id-vs-name collision resolution; regression test `get_thread_prefers_id_over_colliding_name`
- 2026-09-16 — schema v16 `agent_seats` + `device_flows.instance_id`; `issue_agent_seat_token` (codename+seat, no `codename-2`), `revoke_agent`, `deny_device_flow`; `resolve_agent_by_key_hash` resolves active seat keys
- 2026-09-15 — `mod.rs`/`postgres.rs`: tier paths use `classify_atom` so `TierPolicy.rules` apply (#325)
- 2026-09-15 — `delete_ontology_entity` (cascades links) + `delete_ontology_link` on Store trait; SQLite impl, Postgres stubs (#320)
- 2026-09-11 — schema v15 `ontology_proposals`; Store: `insert/get/list/decide_ontology_proposal` (decide guarded to pending-only transitions); postgres stubs (#118)
- 2026-09-08 — `Store::db_rows` + `SqliteVecStore::db_rows_sync`: read-only whitelisted browse (atoms table / derived shared-tag links) for the /ui/db view
- 2026-09-06 — v14 `device_flows` migration; `Store` device-flow + `issue_agent_token` methods
- 2026-08-31 — review fixes: reject team atoms missing team_id, use shared hub_activity DDL
- 2026-08-29 — `database_url_from_env` (`KURULTAI_DATABASE_URL` then `DATABASE_URL`)
- 2026-08-16 — indexed this folder (v1 seed)

