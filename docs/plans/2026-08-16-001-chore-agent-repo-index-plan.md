---
title: "chore: hierarchical agent INDEX.md tree (token-cheap repo map)"
date: 2026-08-16
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
product_contract_source: ce-plan-bootstrap
authority: "User /lfg — every folder indexed sequentially; per-file does/needs/touches/stamp/version/3-line changelog; rollup to main index; audited; lower tokens for new agents"
origin: "User /lfg 2026-08-16 — huge repo work, full repo in-dev"
depth: standard
---

# chore: hierarchical agent INDEX.md tree

**Target repo:** `duketopceo/kurultai`
**Base:** `main` @ v0.4.1 + O2 (#202)
**Process:** PR-only
**Doc review:** skipped — `ce-doc-review` skill not in this host's available-skills list (`skipped_reason: skill_unreachable`)

## Goal Capsule

**Objective:** Give a brand-new agent a repo-wide map they can search with folder-and-file accuracy: one `INDEX.md` per indexed folder, each file row carrying does / needs / touches / timestamp / version / a 3-line changelog, with changes rolling up to the root index. Keep it clean and audited so later agents spend tokens on the map, not on grepping the whole tree.

**Authority:** This plan > user `/lfg` this turn > `AGENTS.md` (start-here facts) > `CONCEPTS.md` (domain words, not a file map).

**Stop when:**

- Root `INDEX.md` is the agent entry map.
- Every in-scope folder has an `INDEX.md` that lists its files and child folders.
- File rows include does, needs, touches, stamp, version, and up to 3 changelog lines (seeded today; live from now on).
- Folder "Recent" sections roll up to the parent and to root.
- `scripts/audit-agent-index.py` proves coverage; CI runs it.
- `AGENTS.md` points at the root index in the first workspace-facts block.

**Do not:** Per-file sidecar `*.md` next to every source file. Index `target/`, `node_modules/`, `.git/`, hashed `ui/assets/*`, or drop `INDEX.md` into ingestible test fixtures. Rewrite Brain UI. Enable `hub`. Replace SQLite. Touch product code paths except the audit script and CI job.

## Product Contract

### Summary

Agents currently start from `AGENTS.md` + `CONCEPTS.md` + ad-hoc grep. That does not tell them which file to open, what it depends on, or whether a sibling folder even exists. This ships a hierarchical, audited markdown map — folder indexes, not a second knowledge brain and not a generated API dump.

### Problem Frame

A new agent exploring `src/`, `website/`, `docs/plans/`, and `web/` burns tokens rereading trees. The user asked for sequential folder indexes with per-file context and a main index that stays current when children change.

### Requirements

| ID | Requirement | Origin |
|----|-------------|--------|
| R1 | Every in-scope folder has `INDEX.md` (filesystem order: top-to-bottom, then side-to-side / alpha). | user |
| R2 | Each tracked file in that folder appears as a row: **does**, **needs**, **touches**, **stamp**, **version**, **changelog** (max 3 lines, newest first). | user |
| R3 | Child folder indexes are linked from the parent. A change to a file index prepends that folder's Recent line and the parent's Recent, up to root `INDEX.md`. | user |
| R4 | Root `INDEX.md` is the main index: surfaces, skip list, how to search, recent rollup. | user |
| R5 | Schema is stable and greppable (`index: kurultai/v1` frontmatter). | inferred for audit |
| R6 | Audit script fails if a folder/file is missing from the map (except skip list). CI runs it. | user (audited) |
| R7 | `AGENTS.md` tells agents to open `INDEX.md` first. | token goal |
| R8 | Changelog starts **from now on** — seed with one "indexed (v1 seed)" line, do not invent history. | user |
| R9 | Map stays token-cheap: short does lines, 2–5 needs/touches paths, no essay per file. | user (lower tokens) |

### Actors

- A1. Brand-new coding agent — must find the right file from root `INDEX.md` without a full-tree glob.
- A2. Implementing agent on a later PR — must update the owning folder index + rollup when they change a file.
- A3. CI — fail closed on map drift.

### Acceptance Examples

- AE1. Agent wants store schema: root `INDEX.md` → `src/INDEX.md` → `src/store/INDEX.md` → `migrations.rs`.
- AE2. `python3 scripts/audit-agent-index.py` exits 0 on a complete tree; adding `src/foo.rs` without listing it exits non-zero.
- AE3. Historical `docs/plans/*.md` are listed; changelog cells are the seed line only until those files change again.

### Scope Boundaries

**In:** `INDEX.md` tree, protocol doc, audit script, CI job, `AGENTS.md` pointer, `CONTRIBUTING.md` one-paragraph convention.

**Out:** Per-file sidecars; indexing generated/vendor trees; putting `INDEX.md` inside `tests/fixtures/vault/` or `tests/fixtures/code_repo/` (ingest corpora); Brain visual changes; Kurultai FTS reindex of this map; Grafana/Postgres work.

### Outstanding Questions

- None blocking. Dense historical plan rows use a one-line **does** plus seed changelog (deferred richness).

## Planning Contract

### Key Technical Decisions

- KTD1. **session-settled: user-directed — folder `INDEX.md` rows, not per-file sidecar markdown.** Chosen over one `.md` beside every `.rs`/`.ts` file: sidecars would roughly double the tree and raise tokens, which contradicts the goal. File context lives in the parent folder index. `(session-settled: user-directed — chosen over per-file sidecar .md: token cost and grep noise)`

- KTD2. **Skip generated, vendor, and ingest-fixture interiors.** Do not write `INDEX.md` into `target/`, `node_modules/`, `.git/`, `.next/`, `dist/`, `.harness/`, hashed `ui/assets/`, or `tests/fixtures/{vault,code_repo}/` (those markdown files are corpus for connector tests). Catalog those trees from the parent index as skipped/generated. `(not session-settled — required so ingest tests and UI rebuilds do not grow the map)`

- KTD3. **Version is a per-file integer in the index, starting at 1.** Not the crate `0.4.1`. Stamp is last git commit date (`%Y-%m-%d`) for the file at seed time. Changelog max 3 lines; bump version when the file's row is updated for a real change.

- KTD4. **Root Recent is a short rollup (max ~15 lines), not a copy of every file row.** Folder Recent max ~8 lines. Bubble one line per meaningful file change.

- KTD5. **Audit in CI as a cheap independent job** (`python3`, no Rust). Do not block `cargo` lint on markdown typos in the same job, but the PR is red if the map drifts.

- KTD6. **`docs/agent-zero/INDEX.md` stays.** It is a v1 issue pack, not this schema. The new `docs/agent-zero/` folder index is a separate `INDEX.md` only if we would overwrite it — **do not overwrite**. Name the agent-map file in that folder `MAP.md`? No: keep `docs/agent-zero/INDEX.md` as-is and list it as the existing pack index; put the agent-map fields for that folder's *other* files into `docs/INDEX.md` children table plus a short `docs/agent-zero/MAP.md`. Simpler: leave `docs/agent-zero/INDEX.md` untouched and treat it as that folder's index (extend it with the v1 schema block at the top, preserving the issue table). **Preserve the issue table; add schema frontmatter + file rows + up-link.**

- KTD7. **Needs / touches are declared paths, not a compiler graph.** 2–5 repo-relative paths. Empty `—` is allowed for standalone docs.

### Assumptions

- A1. ~70 in-scope directories and ~250 files is the whole map; no need to index ignored build artifacts.
- A2. Agents will follow `AGENTS.md` → `INDEX.md` if the pointer is in Learned Workspace Facts.
- A3. Historical plans do not need reconstructed 3-line histories.

### Sequencing

U1 protocol + root → U2 folder tree (src, website/ui, web, docs, rest) → U3 audit + CI → U4 AGENTS/CONTRIBUTING pointers.

## Implementation Units

### U1. Protocol + root index

- **Goal:** Stable schema and the main index a new agent opens first.
- **Requirements:** R4, R5, R8, R9
- **Files:** `docs/agent-index.md`, `INDEX.md`
- **Approach:** Document skip list, row fields, rollup rules, update ritual (edit file → update folder row → prepend Recent → parent Recent → root). Root `INDEX.md` lists every top-level file and every top-level folder with a one-line does and a link to that folder's index.
- **Test scenarios:**
  - T1. Frontmatter contains `index: kurultai/v1`.
  - T2. Skip list names fixtures and `ui/assets`.
- **Verification:** `rg -n "index: kurultai/v1" INDEX.md docs/agent-index.md`

### U2. Sequential folder indexes

- **Goal:** Every in-scope folder mapped, filesystem alpha order within each parent.
- **Requirements:** R1, R2, R3, R8, R9
- **Files:** `INDEX.md` under `.github/`, `docs/` (and children except overwrite-safe agent-zero), `plans/`, `scripts/`, `skills/`, `src/` (and every module dir), `tests/` (fixtures catalogued from parent), `ui/` (assets summarized, not per-hash), `web/`, `website/`
- **Approach:** Walk top-to-bottom. For code files, does = one sentence from module docs / first heading; needs = primary imports; touches = primary consumers. For `docs/plans/`, table of filename + title + stamp + ver 1 + seed changelog. Preserve `docs/agent-zero/INDEX.md` body.
- **Test scenarios:**
  - T3. `src/store/INDEX.md` lists `mod.rs`, `migrations.rs`, `postgres.rs`.
  - T4. `tests/fixtures/vault/` has no `INDEX.md`.
  - T5. Parent `src/INDEX.md` links every `src/*/INDEX.md`.
- **Verification:** audit script (U3) covers this unit.

### U3. Audit script + CI

- **Goal:** Map cannot silently drift.
- **Requirements:** R6
- **Files:** `scripts/audit-agent-index.py`, `.github/workflows/ci.yml`
- **Approach:** Python 3 stdlib. Walk from repo root with the skip list. Require `INDEX.md` in each in-scope dir. Require each non-index tracked file name to appear in that dir's `INDEX.md`. Require child dirs to be mentioned in the parent index. Exit 1 with a countable miss list.
- **Test scenarios:**
  - T6. Script exits 0 on the shipped tree.
  - T7. CI job `agent-index` runs the script.
- **Verification:** `python3 scripts/audit-agent-index.py` locally.

### U4. Agent pointers

- **Goal:** Discovery without reading the whole repo.
- **Requirements:** R7
- **Files:** `AGENTS.md`, `CONTRIBUTING.md`
- **Approach:** One workspace-fact bullet: start at `INDEX.md`; update the owning folder index when you change a file (protocol in `docs/agent-index.md`). CONTRIBUTING: one short "Agent index" subsection.
- **Test scenarios:**
  - T8. `AGENTS.md` links `INDEX.md` and `docs/agent-index.md`.
- **Verification:** `rg -n "INDEX.md" AGENTS.md CONTRIBUTING.md`

## Verification Contract

- `python3 scripts/audit-agent-index.py` exits 0.
- `rg -n "index: kurultai/v1" --glob INDEX.md --glob docs/agent-index.md` finds the schema.
- No `INDEX.md` under `tests/fixtures/vault` or `tests/fixtures/code_repo`.
- `cargo test --locked` is **not** required to change; do not break ingest fixtures. Run it only if a fixture path was touched (it must not be).
- Docs-only + Python script: skip `ce-simplify-code` if the non-md diff is the audit script and CI YAML only and stays small; otherwise simplify the script.

## Definition of Done

- [ ] U1–U4 on branch `cursor/agent-repo-index-7a74`
- [ ] Audit green
- [ ] New agent path documented: `AGENTS.md` → `INDEX.md` → folder `INDEX.md` → file
- [ ] No Brain UI / hub / SQLite product changes
- [ ] PR opened against `main`

## Appendix

### Skip list (canonical)

`.git/`, `target/`, `node_modules/`, `.next/`, `dist/`, `.harness/`, `.cursor/`, `ui/assets/` (list as generated in `ui/INDEX.md`), `tests/fixtures/vault/`, `tests/fixtures/code_repo/` (listed from `tests/INDEX.md`), `web/public/*.svg` may be listed in `web/public/INDEX.md` as static assets without changelog noise.

### Row template

```
| `file` | one-line does | needs paths | touches paths | YYYY-MM-DD | N | YYYY-MM-DD seed · (future) · (future) |
```
