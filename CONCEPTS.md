# Kurultai concepts

Domain vocabulary for agents and humans. Keep entries short; link to `docs/solutions/` for deep learnings.

## AgentAtomView

Token-capped read model returned by MCP/CLI search and cite. Excerpts (~400 chars by default), never full atom `content` unless explicitly requested. See `src/brain/`.

## FTS-first

Index and search must work without an embedding API key. Full-text (FTS5) is the default path; vectors are optional when a live embedder is configured.

See: [docs/solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md](docs/solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md)

## NullEmbedder

`Embedder` implementation with `is_live() == false`. Used when no API key is set. Pipeline skips embedding; store must not receive stub/zero vectors.

## KnowledgeAtom

One SQL row of structured knowledge: title, summary, content, tags, provenance (`source`, `source_id`), optional embedding. Markdown files are one ingest source, not the system of record.

## hash-skip

On incremental index, if an atom’s `content_hash` is unchanged and a vector already exists, skip `embed_batch` and let upsert preserve the existing `atoms_vec` row.

## Graph orchestration (diamond)

Cut non-data “and then” waits: fan-out independent nodes with typed I/O, fan-in only at merge barriers. Loops stay inside nodes; SQL is the shared state. See [docs/plans/phase-2-graph-orchestration.md](docs/plans/phase-2-graph-orchestration.md).

## RRF (Reciprocal Rank Fusion)

Hybrid ranking that combines ordered lists without mixing incomparable raw scores. Contribution per list: `1 / (k + rank)` with `k=60` and 1-based ranks; sums merge duplicate ids. Phase 2 search plan: [docs/plans/2026-07-21-001-feat-search-retrieval-rrf-plan.md](docs/plans/2026-07-21-001-feat-search-retrieval-rrf-plan.md).
