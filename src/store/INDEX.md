---
index: kurultai/v1
folder: src/store
parent: src/INDEX.md
updated: 2026-09-06
version: 3
---

# `src/store`

**Does:** SQLite kernel + optional Postgres hub store
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`migrations.rs`](migrations.rs) | SQLite schema migrations | `src/error` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-09-06 | 2 | 2026-09-06 v14 `device_flows` table · 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Store trait, open_store, SqliteVecStore | `src/error` · `src/hashutil` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/app/context.rs` · `src/http/device_auth.rs` | 2026-09-06 | 3 | 2026-09-06 `DeviceFlow` + `issue_agent_token` methods · 2026-08-29 database_url_from_env · 2026-08-16 indexed (v1 seed) |
| [`postgres.rs`](postgres.rs) | Optional PostgresStore (--features postgres) | `src/error` · `src/hashutil` · `src/hub/activity.rs` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-08-31 | 2 | 2026-08-31 reject team atoms missing team_id; shared hub_activity DDL · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-06 — v14 `device_flows` migration; `Store` device-flow + `issue_agent_token` methods
- 2026-08-31 — review fixes: reject team atoms missing team_id, use shared hub_activity DDL
- 2026-08-29 — `database_url_from_env` (`KURULTAI_DATABASE_URL` then `DATABASE_URL`)
- 2026-08-16 — indexed this folder (v1 seed)

