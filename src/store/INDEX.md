---
index: kurultai/v1
folder: src/store
parent: src/INDEX.md
updated: 2026-08-29
version: 2
---

# `src/store`

**Does:** SQLite kernel + optional Postgres hub store
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`migrations.rs`](migrations.rs) | SQLite schema migrations (v9 ontology) | `src/error` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | Store trait, open_store, SqliteVecStore | `src/error` · `src/hashutil` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/app/context.rs` | 2026-08-29 | 2 | 2026-08-29 database_url_from_env · 2026-08-16 indexed (v1 seed) |
| [`postgres.rs`](postgres.rs) | Optional PostgresStore (--features postgres) | `src/error` · `src/hashutil` · `src/memory` · `src/types` | `src/export/mod.rs` · `src/http/ingest.rs` · `src/ontology/mod.rs` · `src/quality/gate.rs` · `src/quality/near_dupe.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-29 — `database_url_from_env` (`KURULTAI_DATABASE_URL` then `DATABASE_URL`)
- 2026-08-16 — indexed this folder (v1 seed)

