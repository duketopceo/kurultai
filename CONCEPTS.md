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

One structured knowledge unit: title, summary, content, tags, provenance (source and source id), optional embedding, and **visibility scope** (`personal` | `team` | `company`, default `personal`). Markdown files are one ingest source, not the system of record.

### Entity

A class, instance, or metric node in the labeled property graph beside `KnowledgeAtom` (O1 / [#116](https://github.com/duketopceo/kurultai/issues/116)). Seeded classes are Memory, Note, Code, Decision, Person, and System. `ontology_promote` creates instance `ent:{atom_id}` and an `instance_of` link; it does not delete, un-index, or change the atom’s trust lane. Distinct from trust-lane `promote`.

### Link

A directed typed edge between entities: `is_a`, `instance_of`, `associates_with`, `triggered_by`, or `contradicts`. O1 stores `approved` links only. Unknown `rel` values are skipped on read. Proposed links and a write-from-inspector path are O3, not this slice.

### Visibility scope

Tiered access field on every atom (HUB-1 / [#178](https://github.com/duketopceo/kurultai/issues/178)). Solo / no-hub installs leave atoms at `personal` and search/ask behave as before. Shared hub slices (HUB-2+) will honor `team` / `company`; local search does **not** filter by scope until a hub is configured.

See: [docs/plans/phase-6-next-work-orders.md](docs/plans/phase-6-next-work-orders.md) · [docs/multi-user-kurultai.md](docs/multi-user-kurultai.md)

### Visibility scope

Atom visibility: `personal`, `team`, or `company`. Personal never leaves the originating device. Team and company live on an optional shared hub. A deployment may enable zero, one, or two shared tiers.

### Hub

Optional shared store for **one organization** (not a multi-company SaaS). Personal kernel stays local SQLite forever. HUB-2 adds `PostgresStore` behind `--features postgres` and `KURULTAI_FEATURE_HUB=1`; CLI/`open_store` still opens SQLite. HUB-3 is the transport: Tailscale bind (`auth=none` allowed) or public bind with bearer keys; Railway recipe in [`docs/deploy/railway-hub.md`](docs/deploy/railway-hub.md).

### Trust lane

Whether an atom is eligible for default retrieval. Trusted atoms are searchable; quarantine atoms stay stored and are skipped unless the caller opts in.

### Corpus tier

Two-way isolation for a shared company brain: public (everyone) vs private (IT). Not a per-person SaaS tenant. Unknown stored values fail closed to private.

### Visibility labels

Per-document access tags on an atom (for example finance vs general). Empty means public-within-tier. Distinct from corpus tier and from search tags.

### Hashtag-line tags

A markdown line made only of `#tag` tokens. Used as tags when YAML frontmatter has none, so corpora without frontmatter are not wholesale-quarantined.

## Orchestration

### Graph orchestration (diamond)

Cut non-data “and then” waits: fan-out independent nodes with typed I/O, fan-in only at merge barriers. Loops stay inside nodes; the database is the shared state.

## Interface

### Brain UI

Single product dashboard for the local brain: daemon `GET /ui`, assets under `ui/` embedded into the binary. Not a parallel Vite/`website/` product and not the Clerk `web/` auth portal.

See: [docs/solutions/architecture-patterns/one-brain-ui-daemon-ui-only.md](docs/solutions/architecture-patterns/one-brain-ui-daemon-ui-only.md)

### Yurt

CLI / brand mascot (⌂) for Kurultai. Terminal art variants live in `src/art.rs` and appear only on human TTY surfaces when banner policy allows — never on MCP stdio or plain/NO_COLOR paths.
