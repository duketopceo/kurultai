---
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
product_contract_source: ce-plan-bootstrap
created: 2026-08-02
plan_type: fix
---

# fix: Exclude generated/minified files from code connector indexing

## Goal Capsule

The code connector (`src/connectors/github.rs`) currently indexes compiled output — Next.js chunks, hashed bundles, `static/js/` — producing zero-value atoms that flood the brain (weight 0.00, 392 relations, no semantic content). This plan hardens the connector to exclude generated files at walk time, adds a content-entropy gate to catch minified content that slips through, and adds a `prune` CLI command to remove already-indexed noise from the store.

---

## Requirements

- **R1** Walk-time exclusion: generated directories and hashed-bundle filenames must be skipped before any file is read or indexed
- **R2** Content-entropy gate: single-line or near-zero-whitespace content must be rejected regardless of path (catches minified files not caught by path heuristics)
- **R3** Store prune: a CLI command to delete atoms whose `source_id` matches known generated-path patterns, to clean up already-indexed noise
- **R4** Existing unit tests must pass; new tests cover the new filter predicates

---

## Key Technical Decisions

**KTD1 — Hash heuristic scope (already implemented)**
`is_generated_file()` matches stems with a trailing `-<7+alnum>` segment. This correctly catches `7340adf74ff47ec0.js` (pure hash stem, no dash — caught by the `/chunks/` path signal) and `brain-DSYQzh-y.js` (hashed bundle). Risk: false-positives on human-named files like `my-feature-v2.ts`. Mitigation: only apply to `.js`/`.ts`/`.css`; source files in those languages are unlikely to have 7+ random alphanum tails. *(session-settled: user-directed — noise cost outweighs occasional false positive)*

**KTD2 — Content entropy via line-length check**
Minified JS/CSS is characterised by very few newlines relative to byte count. Gate: if the first 4 KB of content has `\n` count < 3 AND total length > 500 bytes, reject as minified. This is O(1) on the already-read buffer, needs no external crate, and avoids reading the full file.

**KTD3 — Prune as CLI command, not daemon background task**
A one-shot `kurultai prune --generated` command is safer than automatic background pruning: it is auditable, interruptible, and requires explicit user intent. Report count before and after.

---

## Implementation Units

### U1. Harden `is_generated_file` and add entropy gate

**Goal:** Close path-heuristic gaps and add content-level minification rejection.

**Requirements:** R1, R2

**Dependencies:** none

**Files:**
- `src/connectors/github.rs`
- `src/connectors/github_tests.rs` (new, or inline `#[cfg(test)]` block)

**Approach:**
1. Narrow hash-stem heuristic in `is_generated_file` to `.js`, `.ts`, `.css`, `.jsx`, `.tsx` only — avoids false positives on config or data files with version suffixes
2. Add `__generated__/` and `/generated/` path signals to `is_generated_file`
3. Add `is_minified_content(content: &str) -> bool` — checks first 4 KB: if `\n` count < 3 AND `content.len() > 500`, return true
4. In `file_to_atoms`, call `is_minified_content` before chunking; return empty vec if true
5. Keep `is_generated_file` and `is_minified_content` as free functions (not methods) — they are pure predicates, easy to test in isolation

**Test scenarios:**
- `is_generated_file` returns true for `7340adf74ff47ec0.js`, `brain-DSYQzh-y.js`, path containing `/chunks/`, `/static/js/`, `/__generated__/`
- `is_generated_file` returns false for `main.rs`, `App.tsx`, `my-feature-v2.ts`, `README.md`
- Hash heuristic only fires on JS/TS/CSS extensions — `config-abc1234.toml` is not flagged
- `is_minified_content` returns true for a 600-byte single-line JS string
- `is_minified_content` returns false for a normal multi-line Rust source snippet
- `is_minified_content` returns false for a 300-byte single-line string (below length threshold)
- `file_to_atoms` returns empty vec when called with minified content

**Verification:** `cargo test` passes; manually confirm `luke-the-duke-blog/out/_next/static/chunks/7340adf74ff47ec0.js` would be skipped at walk time (path signal) and `is_minified_content` would catch any that leak through.

---

### U2. Add `kurultai prune --generated` CLI command

**Goal:** Delete already-indexed generated-file atoms from the store without re-indexing.

**Requirements:** R3

**Dependencies:** U1 (defines the path-pattern set to prune against)

**Files:**
- `src/main.rs` — add `Prune` variant to `Commands` enum
- `src/store/mod.rs` — add `delete_atoms_by_source_id_patterns` to `Store` trait and `SqliteVecStore` impl

**Approach:**
1. Add `Commands::Prune { generated: bool }` to the clap enum with `--generated` flag
2. In the prune handler, open the store, query atoms where `source_id` matches any of: `%/chunks/%`, `%/static/js/%`, `%/static/css/%`, `%/static/media/%`, `%/__generated__/%`, `%/_next/%`, `%/out/_next/%` using SQL `LIKE` with `OR`
3. Print count found, call `delete_atom` in a loop (reuse existing trait method — no new SQL needed beyond the list query), print count deleted
4. Guard behind `--generated` flag so a bare `prune` fails with "specify --generated or another filter"

**Test scenarios:**
- `kurultai prune` (no flags) exits with a non-zero status and a helpful message
- `kurultai prune --generated` with an empty store prints "0 atoms matched, 0 deleted"
- SQL LIKE patterns match `luke-the-duke-blog/out/_next/static/chunks/7340adf74ff47ec0.js#c0` — verify pattern covers the `#cN` chunk suffix
- Integration: insert a fixture atom with a generated `source_id`, run prune, verify it is gone from `list_atoms`

**Verification:** `kurultai prune --generated` runs to completion, prints a count, daemon is not required to be running.

---

## Scope Boundaries

**In scope:**
- Walk-time dir/file exclusion hardening in `github.rs`
- Content entropy gate in `file_to_atoms`
- `prune --generated` CLI command

**Deferred to Follow-Up Work:**
- Configurable exclude patterns in `kurultai.toml` (user-specified globs)
- Automatic prune on re-index (safe but requires user opt-in; defer to config work)
- Entropy gate for other connectors (markdown, json) — not affected by this problem

**Out of scope:**
- Changing any connector other than `github.rs`
- UI changes

---

## Verification Contract

- `cargo test --lib` passes with new unit tests for `is_generated_file` and `is_minified_content`
- `cargo clippy --all-targets -- -D warnings` passes
- `cargo fmt --check` passes
- `kurultai prune --generated` runs without a live daemon
- Manual spot-check: after re-indexing `luke-the-duke-blog`, no atoms with `source_id` containing `/chunks/` appear in `kurultai search code`

---

## Definition of Done

- U1 merged: connector skips generated dirs and minified content at index time
- U2 merged: `prune --generated` removes existing noise atoms from store
- CI green on both units
