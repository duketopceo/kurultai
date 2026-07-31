# Phase 6 P6-3 — Soft multi-label scores (thin slice)

**Date:** 2026-07-31  
**Issue:** [#113](https://github.com/duketopceo/kurultai/issues/113)  
**Status:** Shipped (thin slice) — LLM distillation emit remains stub (#12)

## Goal

Keep hard `tags` for the trust gate. Add vocabulary + per-atom scored soft labels and a post-RRF search boost when the query mentions a label name/alias.

## DoD

- [x] Schema v7: `label_vocab` + `atom_soft_labels`
- [x] `KnowledgeAtom.soft_labels` round-trip on upsert/get (preserve when empty)
- [x] Hard tags still gate trust / promote
- [x] Hybrid search soft-label boost (`α = 0.5`)
- [x] `distill::emit_soft_labels` stub for #12
- [x] Tests: round-trip, vocab share, gate ignore, boost reorder

## Out of scope

- MCP/HTTP vocab CRUD
- Brain UI soft-label colouring
- Real LLM distillation (#12)
