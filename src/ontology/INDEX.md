---
index: kurultai/v1
folder: src/ontology
parent: src/INDEX.md
updated: 2026-10-02
version: 6
---

# `src/ontology`

**Does:** Typed property graph helpers
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`mod.rs`](mod.rs) | Entity/Link helpers + O3 proposal queue (submit/decide) + human-lane direct writes + re-exports `extract` | `src/error` · `src/store` · `src/types` · `src/ontology/extract.rs` | `src/mcp/server.rs` · `src/http/proposals.rs` · `src/http/ontology_write.rs` · `src/pipeline/mod.rs` | 2026-10-02 | 6 | 2026-10-02 `pub mod extract` + re-exports (U3) · 2026-09-14 `create_entity`/`create_link` human lane (#316) · 2026-09-11 O3 `submit_proposal`/`decide_proposal` (#118) · 2026-09-06 sync schema version assertion to v14 · 2026-09-04 sync schema version assertion to v12 · 2026-08-14 indexed (v1 seed) |
| [`extract.rs`](extract.rs) | Zero-LLM edge extraction: `[[wiki-links]]`, `@mentions`, frontmatter `related:`/`depends_on:` → `references` links + entity stubs at index time | `src/error` · `src/store` · `src/types` | `src/pipeline/mod.rs` | 2026-10-02 | 1 | 2026-10-02 indexed (U3 zero-LLM edge extraction) |

## Recent

- 2026-10-02 — U3 zero-LLM edge extraction: `extract.rs` (`extract_references` + `apply_extracted_edges`), new `OntologyLinkType::References` wire value; pipeline upserts edges post-`upsert_batch` (failure non-fatal)
- 2026-09-15 — delete coverage test: entity delete cascades links, missing ids err (#320)
- 2026-09-14 — `create_entity`/`create_link`: human-lane writes reusing proposal validation, apply immediately as `approved` (#316)
- 2026-09-11 — O3 proposal queue: `submit_proposal` (validate + dedupe, no mutation) / `decide_proposal` (approve applies entity/link, reject no-ops); kinds promote_atom/new_link/new_entity (#118)
- 2026-09-06 — sync schema version assertion to v14
- 2026-09-04 — sync schema version assertion to v12
- 2026-08-16 — indexed this folder (v1 seed)
