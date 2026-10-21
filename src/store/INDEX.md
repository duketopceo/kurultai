---
index: kurultai/v1
folder: src/store
parent: src/INDEX.md
updated: 2026-10-21
version: 10
---

# `src/store`

**Does:** SQLite kernel + optional Postgres hub store
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`migrations.rs`](migrations.rs) | SQLite schema migrations + `ensure_multivec_table` (lazy `atoms_multivec`) + `PG_HUB_RLS` Postgres RLS policy DDL + `PG_HUB_DEVICE_FLOWS` connect-join table DDL (consumed by `postgres::migrate_locked`) | `src/error` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` · `src/store/postgres.rs` | 2026-10-21 | 8 | 2026-10-21 `PG_HUB_DEVICE_FLOWS` — `hub_device_flows` (team_hint/team_id/issued_key, no RLS; runs before `PG_HUB_RLS`, hub adoption U3) · 2026-10-20 `PG_HUB_RLS` — FORCE RLS + GUC-keyed scope policies on `knowledge_atoms`/`atoms_vec`/`hub_activity` (R11/KTD1, hub adoption U1) · 2026-10-07 `ensure_multivec_table` (lazy, no schema bump) (#424) · schema v17 `superseded_at`/`superseded_by` + idx ·2026-09-16 v16 `agent_seats` + `device_flows.instance_id` · 2026-09-11 v15 `ontology_proposals` table (#118) · 2026-09-06 v14 `device_flows` table · 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Store trait, open_store, SqliteVecStore, multivec lane; `DeviceFlow` carries hub `team_hint`/`team_id`/`issued_key` (never serialized) | `src/error` · `src/hashutil` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/app/context.rs` · `src/http/device_auth.rs` · `src/http/device.rs` | 2026-10-21 | 13 | 2026-10-21 hub adoption U3: `DeviceFlow.{team_hint,team_id,issued_key}` + trait defaults `create_device_flow_with_hint`/`approve_device_flow_hub`/`list_device_flows`; SQLite rows default new fields to None (solo unchanged) · 2026-10-20 `pub mod scope` — task-local RLS scope registration (hub adoption U2, KTD1) · 2026-10-07 `upsert_multivecs`/`get_multivecs` (trait defaults + SqliteVec, lazy table, FK cascade) (#424) · U5: `duplicate_content_groups` + `prune_stale_ontology` (trait+SQLite) ·2026-10-03 U4: `superseded_at/by` cols in ATOM_COLUMNS+hydrate; `SearchFilter.as_of/include_superseded` + `mark_superseded`; fts+vector supersede predicates ·2026-09-23 `atom_epoch()` — mutation epoch on `SqliteVecStore` (upsert/delete/lane/auto-merge bump; `touch_access` doesn't) for prepared `/api/graph` invalidation; `u64::MAX` default = untrackable (#324) · 2026-09-19 `get_thread(id)` added — id-vs-name collision resolution for Hey threads · 2026-09-16 `issue_agent_seat_token`/`revoke_agent`/`deny_device_flow`; key-hash resolve covers seats · 2026-09-15 `classify_atom` in tier paths — sequester rules apply (#325) |
| [`postgres.rs`](postgres.rs) | Optional PostgresStore (--features postgres); `KURULTAI_PG_SKIP_MIGRATE` env gate on `connect` (KTD8); every method runs inside a `begin_scope` transaction with transaction-local `set_config` GUCs (KTD1 `PathClass`); device-flow ops are `Internal`-scoped conditional UPDATEs | `src/error` · `src/hashutil` · `src/hub/activity.rs` · `src/memory` · `src/types` · `src/store/migrations.rs` · `src/store/scope.rs` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` · `src/hub/keys.rs` | 2026-10-21 | 9 | 2026-10-21 hub adoption U3: `hub_device_flows` migrate step + full device-flow impl (create_with_hint/get-by-code/approve/approve_hub/deny/exchange/list) — pending→approved and approved→exchanged are conditional UPDATEs (exactly-once mint + handout) · 2026-10-20 hub adoption U2: `PathClass` member/internal classification + `begin_scope`/`commit_scope`/`apply_pg_scope` — `set_config('kurultai.team_id'/'is_admin',…,true)` per tx; upsert+member reads `Request`-scoped, deletes/merges/touch_access/audit/ingestion `Internal` · 2026-10-20 hub adoption U1: RLS loop from `migrations::PG_HUB_RLS` in `migrate_locked` + `skip_migrate_enabled()` env gate (KTD8) · U5: `duplicate_content_groups` + `prune_stale_ontology` (Postgres parity) ·2026-10-03 U4: superseded columns ddl + fts/vector predicates + `mark_superseded` ·2026-09-15 `classify_atom` in tier paths — sequester rules apply (#325) |
| [`scope.rs`](scope.rs) | Task-local `PgScope` (team_id/is_admin) — request RLS scope resolved by http middleware, applied per-tx by PostgresStore; `validate_team_id` mint-shape check; compiles without `postgres` feature | `tokio` task_local | `src/store/postgres.rs` · `src/http/auth.rs` · `src/http/mod.rs` · `src/mcp/brain.rs` · `src/http/device.rs` · `src/hub/keys.rs` | 2026-10-21 | 2 | 2026-10-21 `validate_team_id` — `[a-z0-9._-]{1,64}` mint/picker gate (hub adoption U3) · 2026-10-20 added (hub adoption U2, KTD1) |

## Recent

- 2026-10-21 — hub adoption U3 connect-join: `migrations.rs` `PG_HUB_DEVICE_FLOWS` (`hub_device_flows`, no RLS); `postgres.rs` full device-flow impl on `Internal` scope with conditional-UPDATE transitions (exactly-once mint + handout); `mod.rs` `DeviceFlow.{team_hint,team_id,issued_key}` + `create_device_flow_with_hint`/`approve_device_flow_hub`/`list_device_flows` trait defaults; `scope.rs` `validate_team_id`
- 2026-10-20 — `scope.rs` + `postgres.rs` + `mod.rs` (hub adoption U2, KTD1): `PgScope` task-local (`member`/`admin`/`closed`/`INTERNAL`) resolved per request; `begin_scope` wraps every PostgresStore op in a tx issuing `SELECT set_config(…, true)`; `PathClass::Request` reads task-local, `PathClass::Internal` forces `is_admin` for maintenance writes (`touch_access`, deletes, merges, audits, ingestion jobs)
- 2026-10-20 — `migrations.rs`/`postgres.rs` (hub adoption U1, R11/KTD1/KTD8): `PG_HUB_RLS` — ENABLE+FORCE RLS on `knowledge_atoms` (select/insert/update/delete policies keyed on `kurultai.team_id`/`kurultai.is_admin` GUCs), `atoms_vec` (parent-atom EXISTS scope), `hub_activity` (append-only, admin SELECT); `KURULTAI_PG_SKIP_MIGRATE` skips eager `migrate()` on `connect`

- 2026-10-07 — `mod.rs`/`migrations.rs`: `atoms_multivec` sidecar — `Store::upsert_multivecs`/`get_multivecs` (defaults for other stores), lazy table, cascade delete; roundtrip+cascade tests

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

