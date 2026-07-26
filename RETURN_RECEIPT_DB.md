# Return Receipt — DB + Ingestion Agent (U1.1–U1.3)

**Branch:** `feat/v030-db-ingestion`  
**Commit:** `9a436b2`  
**Date:** 2026-07-26  
**Status:** ✅ All verifications passed

---

## Files Changed

| File | Change Type | Summary |
|------|-------------|----------|
| `src/store/migrations.rs` | Modified | Bump `CURRENT_SCHEMA_VERSION` to 6; add `MIGRATION_006` creating `ingestion_jobs` table with indexes; add `if current < 6` execution block |
| `src/store/mod.rs` | Modified | Add `IngestionJob` struct; add 3 Store trait methods (`record_ingestion_start`, `record_ingestion_finish`, `list_pending_ingestion_jobs`); implement them on `SqliteVecStore` via sync helper methods; add stub impls in `FailCountStore` for http tests |
| `src/types.rs` | Modified | Add `SourceKind::Json` variant with doc comment |
| `src/connectors/json.rs` | New | Full `JsonConnector` implementation: walks `.json`/`.jsonl`/`.ndjson` files from `root_path`; parses JSON arrays and NDJSON lines; maps objects → `KnowledgeAtom` with title/content/tags/source_id; rejects malformed JSON with typed `KurultaiError::Connector`; `id_field` config for stable `source_id` |
| `src/connectors/mod.rs` | Modified | Add `pub mod json;` |
| `src/connectors/registry.rs` | Modified | Import `JsonConnector`; add `SourceKind::Json => Box::new(JsonConnector::new())` arm to `build_connector` |
| `src/http/mod.rs` | Modified | Add 3 ingestion stub method implementations to `FailCountStore` test mock |
| `tests/json_ingestion_test.rs` | New | 11 integration tests: migration v006 column presence, idempotency, ingestion job lifecycle (start/finish success/failure), JSON array full_sync, NDJSON full_sync, malformed JSON rejection, stable `source_id`, and registry creation from config |

---

## Verification Commands and Outputs

### 1. `cargo fmt --all -- --check`
```
FMT_OK  (exit 0, no diffs)
```
✅ PASSED

### 2. `cargo clippy --all-targets -- -D warnings`
```
0 errors, 0 warnings  (exit 0)
```
✅ PASSED

### 3. `cargo test --locked`
```
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured (lib tests)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured (cli_smoke)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured (install_script)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured (json_ingestion_test)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured (phase3)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured (phase4)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured (phase5)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured (retrieval)
```
✅ PASSED — all 185 tests pass (11 new)

### 4. Commit hash
```
9a436b2  feat(db+ingestion): schema v006, ingestion_jobs store, JSON/NDJSON connector (U1.1-U1.3)
```

---

## Unit U1.1 — Schema v006 Acceptance Criteria

- [x] `CURRENT_SCHEMA_VERSION` is `6`
- [x] Table `ingestion_jobs(id, batch_id, source, file_path, status, atoms_count, error_message, created_at, completed_at)` created
- [x] Migration is idempotent (verified by `migration_v006_is_idempotent` test)
- [x] Version 6 recorded in `schema_migrations` (verified by `migration_v006_creates_ingestion_jobs_table` test)

## Unit U1.2 — Store Trait Additions Acceptance Criteria

- [x] `record_ingestion_start(&self, batch_id, source, file_path) -> Result<i64>` added to trait and implemented
- [x] `record_ingestion_finish(&self, job_id, atoms_count, error_message) -> Result<()>` added; error path sets `status='failed'` and records `error_message`
- [x] `list_pending_ingestion_jobs(&self) -> Result<Vec<IngestionJob>>` returns only `status='pending'` rows
- [x] Implementation uses `Mutex<Connection>` pattern (via `self.lock()`)
- [x] Errors mapped to `KurultaiError::Store`

## Unit U1.3 — JSON Connector Acceptance Criteria

- [x] `SourceKind::Json` added to `src/types.rs`
- [x] Connector registered in `src/connectors/registry.rs`
- [x] `src/connectors/json.rs` reads `.json` array-of-objects from `root_path`
- [x] `src/connectors/json.rs` reads `.jsonl`/`.ndjson` line-delimited JSON
- [x] `id_field` extra config provides stable `source_id`
- [x] Atoms produced with at least `title`, `content`, `tags` (empty fallback)
- [x] Malformed JSON rejected with `KurultaiError::Connector` (typed error)

---

## Notes / Gotchas

1. **Quality gate**: The pipeline's quality gate (`src/quality/gate.rs`) quarantines atoms without non-empty tags (`reason: "untagged"`). JSON fixture records in integration tests must include `"tags"` fields for atoms to appear in `list_atoms` with the default `trusted_only: true` filter. This is correct behavior — JSON data lacking tags should be quarantined and reviewed.
2. **TempDir lifetime**: Integration tests creating `SqliteVecStore` must bind the `tempdir()` result to a named variable; dropping it early deletes the directory and causes "attempt to write a readonly database" errors.
3. **FailCountStore**: Any new Store trait methods must also be stubbed in `FailCountStore` (test mock in `src/http/mod.rs`) to satisfy the async_trait impl.

---

## Blockers

None. All units implemented and verified.
