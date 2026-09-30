---
title: feat — ontology board v2: delete entities/links + edge menu
date: 2026-09-15
issue: https://github.com/duketopceo/kurultai/issues/320
parent: docs/plans/2026-09-14-001-feat-ontology-flowsint-board-plan.md
---

# Ontology board v2 — delete entities/links + edge menu

Follow-up to #316/#319. The board can create but not remove; the plan deferred
editing/deleting to a follow-up. This slice adds **delete** on both lanes and
the board UX to reach it. Rename/attribute editing stays out (separate issue).

## Scope

1. **Store** (`src/store/mod.rs`, `src/store/postgres.rs`)
   - `delete_ontology_link(id)` — remove the link row.
   - `delete_ontology_entity(id)` — remove the entity row plus every link
     touching it (`from_id`/`to_id`), so no dangling edges remain.
   - Postgres store keeps its `ontology not on hub store yet` stubs.
   - Unit tests: delete removes rows, cascades links, missing id is a no-op
     error or silent no-op (pick: 404-style error to keep HTTP honest).

2. **HTTP** (`src/http/ontology_write.rs`, `src/http/auth.rs`, `src/http/mod.rs`)
   - `DELETE /api/ontology/entity/{id}` and `DELETE /api/ontology/link/{id}`
     in `ontology_write.rs`, same `refuse_agents` human lane.
   - Auth guard: `WRITE_ROUTES` is exact-match; these paths are dynamic, so
     extend the write-route check with a prefix predicate
     (`is_ontology_write_route`) covering `/api/ontology/entity` and
     `/api/ontology/link` (POST + DELETE alike).

3. **Board** (`website/src/api.ts`, `website/src/components/OntologyBoard.tsx`)
   - `deleteOntologyEntity(id)` / `deleteOntologyLink(id)` fetch fns.
   - Edge context menu (`onEdgeContextMenu`) → **Delete link**.
   - Node context menu → **Delete entity** with inline confirm state in the
     menu (no modal).
   - `Escape` closes menus/dialogs; successful delete refetches ontology.
   - Deleted entity id dropped from `kurultai-onto-pos`/expanded sets.

## File map

- `src/store/mod.rs` — trait methods + SQLite impl + tests
- `src/store/postgres.rs` — stubs
- `src/http/ontology_write.rs` — DELETE handlers
- `src/http/auth.rs` — prefix write-route predicate
- `website/src/api.ts` — delete fns
- `website/src/components/OntologyBoard.tsx` — edge menu, delete actions, Escape
- `website/src/styles.css` — minor menu/dialog styles
- `ui/` rebuild via `scripts/build-ui.sh`; index rows bumped.

## Verification

- `cargo test --lib` (new store tests), `clippy -D warnings`, `fmt --check`
- `tsc --noEmit`, `npm test`, `npm run build`, `build-ui.sh`
- Live smoke: DELETE entity cascades links; agent key → 403; DELETE link 404s
  on unknown id.
- Headless Chromium CDP: edge/node menus expose delete; deleted node/edge gone
  after refetch.
- `audit-agent-index.py` green.
