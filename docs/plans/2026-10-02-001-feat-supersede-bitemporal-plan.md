---
plan: feat-supersede-bitemporal
date: 2026-10-02
status: draft
execution: code
parent: docs/plans/2026-10-01-001-feat-competitive-feature-sweep-plan.md
---

# Plan: Supersede / Bi-Temporal-Lite (sweep U4)

## Problem Frame

Corrections today create duplicates: a revised note indexes as a new atom
while the stale original keeps ranking in search/ask. gbrain and
Graphiti/Zep solve this with temporal edges — an old version is
*invalidated*, not deleted, so present-state queries see only current
truth while history remains queryable. Kurultai needs the same: cheap
supersede chains + an opt-in time-travel flag. Full bi-temporal event
sourcing is out of scope.

## Decisions (user-confirmed 2026-10-02)

- **Superseded atoms are excluded entirely** from default search/ask —
  not ranked down, not annotated. History is reachable only via
  `--as-of` or `--include-superseded`.
- **Full `--as-of`** on both `search` and `ask` in v1: filter hits to
  atoms valid at the given timestamp.
- `supersedes` is declared in **frontmatter** (`supersedes: <atom_id>`
  or list), piggybacking the U3 extraction pass — zero LLM, author-typed.
- No `valid_at` override field in v1: `indexed_at` is the valid-from
  proxy. `invalid_at` is realized as `superseded_at` on the old row.

## Implementation Units

### U1: Store columns + `mark_superseded`

- `src/store/migrations.rs` — schema v17:
  `ALTER TABLE knowledge_atoms ADD COLUMN superseded_at TEXT`,
  `ADD COLUMN superseded_by TEXT`. Nullable; existing rows NULL.
- `src/types.rs` — `KnowledgeAtom.superseded_at: Option<DateTime<Utc>>`,
  `superseded_by: Option<String>` (serde defaults, skip-if-none).
- `src/store/mod.rs` — `Store::mark_superseded(target_ids, by_id, ts)`
  (new trait method; SQLite impl UPDATE … WHERE id IN; Postgres impl).
  Row hydrate/select lists gain the two columns.
- Tests: migration idempotent; mark_superseded round-trip; unknown ids
  no-op.

### U2: Frontmatter `supersedes:` at index time

- `src/ontology/extract.rs` — extend the frontmatter scan (already
  parses `related:`/`depends_on:`): `supersedes:` values are atom ids or
  source_ids, comma/`[list]` separated. New `extract_supersedes(content)
  -> Vec<String>`; resolve slug → atom id via store lookup
  (`get_atom`/`get_atom_by_source_id`).
- `src/pipeline/mod.rs` — after `upsert_batch`, for each atom with
  supersedes declarations: `mark_superseded(resolved, atom.id, now)`.
  Also clears superseded state on the *new* atom if a same-source_id
  reindex previously marked it (stale-supersede repair: when upserting
  an atom whose own id is marked superseded by a *different* atom,
  leave it — newest wins only via explicit re-supersede).
- Failure non-fatal, logged, same as U3 edges.
- Tests: frontmatter parse variants; id vs source_id resolution;
  supersede chain A→B→C marks both.

### U3: Query exclusion + `--as-of` / `--include-superseded`

- `SearchFilter` gains `as_of: Option<DateTime<Utc>>` and
  `include_superseded: bool` (default false).
- `src/store/mod.rs` `fts_search_ids` + `vector_search_ids`: default
  predicate `AND a.superseded_at IS NULL`. With `as_of = t`:
  `AND a.indexed_at <= t AND (a.superseded_at IS NULL OR a.superseded_at > t)`.
  `include_superseded` skips the predicate entirely (mutually exclusive
  with as_of; CLI validates).
- `src/main.rs` — `search --as-of <RFC3339|YYYY-MM-DD>`,
  `search --include-superseded`; same flags on `ask`. Propagate into
  `SearchFilter` for both CLI and the MCP/HTTP paths via filter
  construction sites.
- Tests: superseded atom absent from default fts+vector; present under
  `--include-superseded`; as-of before supersede returns old, after
  returns new; as-of before index excludes not-yet-indexed.

### U4: Indexes + docs

- `src/store/INDEX.md`, `src/types.rs` row, `src/ontology/INDEX.md`,
  `src/pipeline/INDEX.md`, `src/INDEX.md`, root `INDEX.md` Recent.
- `config.example.toml` / docs: `supersedes:` frontmatter usage note.
- `python3 scripts/audit-agent-index.py` green.

## Scope Boundaries

- No retroactive supersede detection (no similarity clustering — that's U5).
- No `valid_from` frontmatter field; `indexed_at` is the proxy.
- No UI surface for superseded atoms in v1.
- No soft-delete/restore semantics beyond supersede (trash stays out).

## Test Scenarios

- Migration v16→v17 on existing demo store; columns NULL on legacy rows.
- Atom B with `supersedes: A` → A excluded from `search`; `search
  --include-superseded` returns A; `search --as-of <before>` returns A,
  `--as-of <after>` returns B only.
- `ask` inherits exclusion through the shared filter path.
- Chain A→B→C: only C in default results.
- `supersedes:` naming a nonexistent atom → warn, no crash, no row marked.
- Reindex of corpus without `supersedes:` leaves prior marks untouched
  (no accidental unsupersede).

## Deferred to Implementation

- Exact `SearchFilter` plumbing sites (MCP search tool, HTTP `/api/search`)
  — all must honor the new default exclusion.
- Whether `superseded_by` also emits an ontology `contradicts`-style edge
  for graph traversal (nice-to-have; decide if cheap inside U2).
