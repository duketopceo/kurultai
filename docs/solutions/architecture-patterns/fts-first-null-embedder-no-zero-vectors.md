---
title: FTS-first boot with NullEmbedder — never write zero vectors into sqlite-vec
date: 2026-07-21
category: architecture-patterns
module: store/embed/pipeline
problem_type: architecture_pattern
component: database
severity: high
applies_when:
  - "Indexing or searching without OPENROUTER_API_KEY"
  - "Choosing an Embedder implementation for local/dev"
  - "Upserting KnowledgeAtom rows that may lack real embeddings"
tags: [fts-first, null-embedder, sqlite-vec, zero-vector, phase-1]
---

# FTS-first boot with NullEmbedder — never write zero vectors into sqlite-vec

## Context

Phase 1 must work for a developer with no embedding API key. Early designs risked either (a) blocking `kurultai index` on OpenRouter, or (b) writing all-zero float vectors into `atoms_vec` so "semantic search" returned nonsense neighbors.

Kurultai’s doctrine is FTS-first: full-text search is always available; vectors are optional enrichment when a live embedder exists.

## Guidance

1. **Missing API key → `NullEmbedder`** (`is_live() == false`). Pipeline skips `embed_batch` and indexes FTS-only.
2. **Store guard:** only insert into `atoms_vec` when the embedding norm is above `MIN_EMBEDDING_NORM` (~1e-6). Near-zero vectors are dropped, never indexed.
3. **Search:** FTS always runs; vector arm runs only when the embedder is live and produces a non-zero query vector. Embed failures warn and fall back to FTS.
4. **Status honesty:** CLI reports `Embedder: none (FTS-only …)` when not live.

Pin **sqlite-vec `=0.1.6`** in `Cargo.toml`. Newer alphas (e.g. `0.1.10-alpha.4`) failed to compile (missing `diskann.c`) in this repo’s CI/toolchain.

## Why This Matters

- Developers are never blocked on spend or network for the core loop: markdown → atoms → FTS search → MCP excerpts.
- Zero-vector pollution permanently corrupts KNN rankings until a full re-embed; the upsert guard makes that class of bug hard to introduce.
- Agents and humans get the same FTS-first behavior locally and in CI without secrets.

## When to Apply

- Any new embedder or pipeline path that might call `upsert` with stub embeddings
- CI and smoke tests (clear ambient `OPENROUTER_API_KEY` so fixtures stay FTS-only)
- Designing Phase 2 fusion (#6): treat vector hits as optional, not required

## Examples

```rust
// App bootstrap — no key means NullEmbedder, not fake zeros
if api_key.is_none() {
    return Ok(Arc::new(NullEmbedder::new(config.embed_dim)));
}

// Store upsert — never pollute vec0
if embedding_norm(emb) >= MIN_EMBEDDING_NORM {
    // INSERT INTO atoms_vec ...
}

// Pipeline — skip network when not live
if self.embedder.is_live() {
    // embed_batch + assign
} else {
    // FTS-only index
}
```

Tests that encode the invariant:

- `store::tests::zero_vector_not_indexed_in_vec`
- `pipeline::tests::index_fixture_vault_fts_hit` (NullEmbedder)
- CLI smoke clears `OPENROUTER_API_KEY` / `KURULTAI_API_KEY`

## Related

- Doctrine: [#37](https://github.com/duketopceo/kurultai/issues/37), README “Design doctrine”
- Storage: [#1](https://github.com/duketopceo/kurultai/issues/1) / SqliteVecStore
- Embeddings: [#2](https://github.com/duketopceo/kurultai/issues/2)
- Merged Phase 1 MCP exit: PR [#46](https://github.com/duketopceo/kurultai/pull/46)
