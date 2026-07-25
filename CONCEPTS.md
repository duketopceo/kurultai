# Concepts

Shared domain vocabulary for this project — entities, named processes, and status concepts with project-specific meaning. Seeded with core domain vocabulary, then accretes as ce-compound and ce-compound-refresh process learnings; direct edits are fine. Glossary only, not a spec or catch-all.

## Retrieval

### AgentAtomView

Token-capped read model returned by search and cite. Short excerpts by default, never full atom content unless explicitly requested.

### FTS-first

Index and search must work without an embedding API key. Full-text search is the default path; vectors are optional enrichment when a live embedder is configured.

See: [docs/solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md](docs/solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md)

### NullEmbedder

Embedder that is not live. Used when no API key is set. Pipeline skips embedding; the store must not receive stub or zero vectors.

### NullReranker

Reranker that is not live when rerank config or API key is missing. Search keeps RRF order; no network call.

### RRF (Reciprocal Rank Fusion)

Hybrid ranking that combines ordered lists without mixing incomparable raw scores. Each list contributes a reciprocal rank term; duplicate ids sum their contributions.

### hash-skip

On incremental index, if an atom’s content hash is unchanged and a vector already exists, skip re-embedding and preserve the existing vector row on upsert.

## Knowledge

### KnowledgeAtom

One structured knowledge unit: title, summary, content, tags, provenance (source and source id), optional embedding. Markdown files are one ingest source, not the system of record.

## Orchestration

### Graph orchestration (diamond)

Cut non-data “and then” waits: fan-out independent nodes with typed I/O, fan-in only at merge barriers. Loops stay inside nodes; the database is the shared state.
