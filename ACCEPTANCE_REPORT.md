# Acceptance Report — KHAN-251

Findings from building the comprehensive acceptance/integration test suite
in `7cdeb36` (#216). Each broken / partial feature below is documented with:
feature name, what was broken, the error (or behavioral gap), and a
suggested fix — **as it stood at #216.**

**Update (`110f371`, #217, HEAD):** Items 1–4 below were fixed in the very
next PR. The `#[ignore]` attributes on the three pinning tests have been
removed and the fix is implemented; only item 5 (deliberate feature-gate)
still reflects current reality unchanged. Verified directly against source
on HEAD, not taken on faith — see the "Fixed" note under each item for the
exact function/column/line. This report is kept for its original
root-cause analysis, not as a live status board — check `FEATURE_MATRIX.md`
for current status first.

## Summary

| # | Feature | Status (#216, historical) | Status (HEAD, `110f371`) | Test |
|---|---------|---------------------------|---------------------------|------|
| 1 | Hashtag-line ingest | ❌ broken (not implemented) | ✅ fixed — `parse_hashtag_line_tags` in `src/ingest/dump.rs` | `acceptance_ingest::hashtag_line_tags_without_frontmatter` (no longer ignored) |
| 2 | Corpus tier persistence | ❌ broken (not persisted) | ✅ fixed — schema v10 column, SQLite + Postgres | `acceptance_visibility::corpus_tier_private_round_trips_through_store` (no longer ignored) |
| 3 | Visibility labels persistence | ❌ broken (not persisted) | ✅ fixed — schema v10 column, SQLite + Postgres | `acceptance_visibility::visibility_labels_round_trip_through_store` (no longer ignored) |
| 4 | SourceConfig tier/label helpers unused at ingest | ⚠️ partial | ✅ fixed — `IndexPipeline::index_connector` in `src/pipeline/mod.rs` now applies the defaults | covered by passing type-level tests |
| 5 | Hub store behind feature flag + env toggle | ⚠️ partial (by design) | ⚠️ unchanged — still deliberate | `acceptance_visibility::open_hub_store_refuses_without_feature_flag` |

---

## 1. Hashtag-line ingest — FIXED in `110f371` (#217)

> Historical analysis below is from #216, before the fix. Current
> implementation: `parse_hashtag_line_tags` (`src/ingest/dump.rs:249`),
> called as a fallback when frontmatter tags are empty
> (`src/ingest/dump.rs:140-142`) — YAML frontmatter still wins, matching the
> suggested fix below. Test no longer `#[ignore]`'d.

**Feature (TA-6):** Markdown corpora without YAML frontmatter (e.g. the
`kb-it-docs` pattern) should have their tags parsed from dedicated
whitespace-separated `#tag` lines so the files are not wholesale-quarantined
by the trust-lane gate. Headings and inline `#mentions` in prose must be
ignored; YAML frontmatter tags still win.

**What's broken:** There is **no hashtag-line parsing code** on `main`. The
feature was declared in commit `b43d43d` ("feat(connectors): accept dedicated
hashtag-line tags") but that commit only added the `CorpusTier` /
`visibility_labels` type fields — the actual connector parsing logic was
never landed. The merge-conflict resolution in `b2c4276` ("accept main's
markdown.rs") dropped the connector change entirely. Today
`MarkdownConnector`/`atomize_markdown` only read tags from YAML frontmatter
(`parse_tags` in `src/ingest/dump.rs`); a markdown file whose only tags are
on a `#ops #deploy` line is indexed with `tags = []` and quarantined as
`untagged`.

**Error / behavior:** A file like
```md
# IT Doc

#ops #deploy

Run the migration scripts before cutover with full detail.
```
yields atoms with `tags: []` → `GateOutcome::Quarantine { reason: "untagged" }`
→ excluded from default FTS/vector retrieval.

**Suggested fix:** Add a `parse_hashtag_line` step to `atomize_markdown`
(`src/ingest/dump.rs`) that scans the body's leading non-heading lines for a
line composed entirely of `#word` tokens (regex `^(\s*#\w+)+\s*$`), splits
them into tags, and merges them under YAML-frontmatter tags (YAML wins).
Guard against markdown ATX headings (`# Heading`) by requiring every token on
the line to be a `#tag` with no trailing prose.

---

## 2. Corpus tier persistence — FIXED in `110f371` (#217)

> Historical analysis below is from #216, before the fix. Current
> implementation: schema v10 migration adds `corpus_tier TEXT NOT NULL
> DEFAULT 'public'` (`src/store/migrations.rs:342-347`); `ATOM_COLUMNS`,
> `upsert_sync`, and `row_to_atom` all read/write it in `src/store/mod.rs`;
> mirrored in `src/store/postgres.rs`. Test no longer `#[ignore]`'d.

**Feature (TA-3):** `KnowledgeAtom.corpus_tier` (`public` / `private`) should
round-trip through the SQLite store so the two-tier corpus isolation
(public vs. private/IT) is enforceable at retrieval.

**What's broken:** The `corpus_tier` field exists on the `KnowledgeAtom`
struct (`src/types.rs`) and `CorpusTier::parse`/`as_str` work, but the
SQLite store **never persists it**:

- `ATOM_COLUMNS` in `src/store/mod.rs` does not include `corpus_tier`.
- `upsert_sync` does not write a `corpus_tier` column (no INSERT/UPDATE
  column, no migration adding the column).
- `row_to_atom` hardcodes `corpus_tier: CorpusTier::Public` on read.
- The Postgres store (`src/store/postgres.rs`) likewise hardcodes
  `CorpusTier::Public` on read.

So any atom upserted with `corpus_tier = Private` is read back as `Public`.

**Error / behavior:**
```
corpus_tier_private_round_trips_through_store ... ignored
```
(the `#[ignore]`d test asserts `loaded.corpus_tier == Private` after upsert;
removing the ignore makes it fail with `left: Public, right: Private`).

**Suggested fix:**
1. Add a migration (schema v10) that `ALTER TABLE knowledge_atoms ADD COLUMN
   corpus_tier TEXT NOT NULL DEFAULT 'public'`.
2. Add `corpus_tier` to `ATOM_COLUMNS` and the `upsert_sync` INSERT/UPDATE.
3. Read it in `row_to_atom` via `CorpusTier::parse(...)`.
4. Mirror the column in the Postgres store.

---

## 3. Visibility labels persistence — FIXED in `110f371` (#217)

> Historical analysis below is from #216, before the fix. Current
> implementation: schema v10 migration adds `visibility_labels_json TEXT
> NOT NULL DEFAULT '[]'` (`src/store/migrations.rs:348-353`), serialized via
> `serde_json` in `upsert_sync` and deserialized in `row_to_atom`
> (`src/store/mod.rs`); mirrored in `src/store/postgres.rs`. Test no longer
> `#[ignore]`'d.

**Feature (TA-4):** KTD15 per-document visibility labels (e.g. `finance`,
`exec`) on `KnowledgeAtom.visibility_labels` should round-trip through the
store for row-level-security-style filtering.

**What's broken:** Same root cause as #2. `visibility_labels` exists on the
struct but is **not persisted**:

- Not in `ATOM_COLUMNS`.
- Not written by `upsert_sync`.
- `row_to_atom` hardcodes `visibility_labels: Vec::new()`.
- Postgres read path also hardcodes empty.

An atom upserted with `visibility_labels = ["finance"]` is read back with
`visibility_labels = []`.

**Error / behavior:**
```
visibility_labels_round_trip_through_store ... ignored
```
(removing the ignore fails: `left: [], right: ["finance", "exec"]`).

**Suggested fix:** Add a `visibility_labels_json TEXT` column in the v10
migration, write `serde_json::to_string(&atom.visibility_labels)` in
`upsert_sync`, deserialize in `row_to_atom` (defaulting to empty). Mirror in
Postgres. (Could share the migration with #2.)

---

## 4. SourceConfig tier/label helpers unused at ingest — FIXED in `110f371` (#217)

> Historical analysis below is from #216, before the fix. Current
> implementation: `IndexPipeline::index_connector` (`src/pipeline/mod.rs`,
> ~line 116) calls `default_corpus_tier()` / `default_visibility_labels()`
> after fetch and fills in atoms that are still at the default `Public`
> tier with no labels — frontmatter/connector-supplied values still win,
> matching the suggested fix below.

**Feature (TA-5):** `SourceConfig::default_corpus_tier()` and
`default_visibility_labels()` parse `extra.default_corpus_tier` /
`extra.default_visibility_labels` so a markdown source can declare
corpus/visibility defaults without code.

**What's broken:** The helpers exist and parse correctly (covered by passing
type-level tests), but **the index pipeline never calls them**. Nothing in
`IndexPipeline::index_connector` (`src/pipeline/mod.rs`) or the connectors
reads `default_corpus_tier()` / `default_visibility_labels()` to stamp
atoms at ingest time. So even after #2/#3 are fixed, the config-driven
defaults would not be applied.

**Suggested fix:** In `IndexPipeline::index_connector`, after collecting
atoms and before the quality gate, look up the `SourceConfig` for the
connector and apply `default_corpus_tier()` / `default_visibility_labels()`
to each atom (only when the atom hasn't already set them). This requires
threading the `SourceConfig` (or just the two resolved values) into
`index_connector`.

---

## 5. Hub store behind feature flag + env toggle — PARTIAL (by design)

**Feature (TA-11):** Postgres + pgvector shared-tier hub store.

**What's broken:** Nothing is broken — this is a deliberate gate.
`open_hub_store` refuses unless **both** `--features postgres` (compile time)
**and** `KURULTAI_FEATURE_HUB=1` (runtime) are set. The default CI build is
SQLite-only, so the acceptance suite asserts the refuse path
(`open_hub_store_refuses_without_feature_flag` passes). The Postgres path
itself is not exercised in CI (no live Postgres).

**Suggested fix (to harden later):** Add a `#[cfg(feature = "postgres")]`
integration test that spins a `testcontainers` Postgres + pgvector and
round-trips an atom, gated behind a `postgres-integration` feature.

---

## What works (verified green)

All A0 original features and the wired Tiered Access features pass acceptance:

- Markdown / JSON / NDJSON / plain-text ingest, frontmatter tags, heading chunks
- FTS5 search (bm25, stopword sanitization, blank/stopword-only → empty)
- Vector search (sqlite-vec kNN, zero-norm short-circuit)
- Hybrid search (FTS + vector RRF, source diversification)
- Ask (extractive RAG with citations + confidence)
- MCP tools: `search`, `cite`, `remember`, `ask`, `who_knows`, `promote`,
  `ontology_get`, `ontology_promote` (8/8)
- Read-only (HTTP/SSE) MCP surface gate (excludes remember/promote/ontology_promote)
- HTTP API: `/health`, `/api/status`, `/api/atoms`, `/api/search` (GET+POST),
  `/ask` (GET+POST), `/api/recall`, `/api/ontology`, `/api/graph`,
  `/api/promote`, `/api/touch`, brain UI
- Hub API-key auth middleware (bearer, sha256-hashed keys, non-API routes
  exempt, wrong token rejected)
- Inbox connector (processed/failed tray finalization)
- Loopback dump atomizer + upsert
- Project-scoped recall (`recall_for_agent` / `POST /api/recall`)
- Visibility scope `personal`/`team`/`company` round-trip (persisted)
- Ontology O1: seeded class tree (6 classes), entity CRUD, typed links,
  `promote_atom_to_entity` (instance_of), idempotency, fail-closed link parse
- Quality gate (untagged/too-short/thin-boilerplate/exact-duplicate quarantine)
