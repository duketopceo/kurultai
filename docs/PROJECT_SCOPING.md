# Project scoping (`project_id`)

Implements the sequestering half of issue #184 for the case where several Claude
Code sessions share one Kurultai brain on one box as one unix user.

## This is namespacing, not isolation

Read this before relying on it for anything.

Every session runs as the **same unix user**, against the **same SQLite file**.
Any session can already read another session's files, `/proc`, and the database
directly. `project_id` therefore buys exactly one thing:

> One session's ingest does not pollute another session's recall.

It does **not** buy:

- confidentiality between sessions — any local process can pass any project
  string to `recall` or `remember`, and can read the DB with `sqlite3` anyway;
- tamper-proofing — a local process can rewrite another project's atoms;
- an audit boundary.

Per-agent ACLs on top of this would be theatre. Do not describe this feature as
isolation, sandboxing, or a security boundary in docs, UI or release notes.

## Model

- `project_id` lives in `KnowledgeAtom.metadata["project_id"]`
  (`KnowledgeAtom::project_id()`).
- Atoms with no `project_id` read as `"default"`. Legacy atoms indexed before
  this change are all `"default"` and stay recallable there.
- Names are normalized on **both** write and read (`project::normalize_project`):
  trimmed, lowercased, truncated to 64 chars. `Crew-YAM` and `crew-yam` are one
  namespace, not two.

## Setting a project

Resolution order, applied by `project::resolve_project`:

1. explicit `project` argument on the call;
2. the `KURULTAI_PROJECT` environment variable;
3. `"default"`.

Write paths that set it today:

| Path | How |
| --- | --- |
| MCP `remember` | `project` argument, else `$KURULTAI_PROJECT` |
| `BrainService::remember` | `("project_id", …)` in `metadata`, else `$KURULTAI_PROJECT` |
| `POST /ingest`, JSON dumps | `project_id` key in the atom's metadata JSON |

Connectors do **not** set `project_id` — everything a connector indexes is
`"default"`. Note `connectors/pond.rs` writes `metadata["project"]` (the raw cwd
from Pond's JSONL); that is a different key and does not feed `project_id()`.

To give each session its own namespace, set `KURULTAI_PROJECT` in that session's
environment, e.g. in the `env` block of its MCP client config:

```json
{ "mcpServers": { "kurultai": {
    "command": "kurultai", "args": ["mcp"],
    "env": { "KURULTAI_PROJECT": "crew-itdash" }
} } }
```

`kurultai mcp init` does not write that block. It runs once and cannot know the
per-session value, and writing a fixed one would clobber an exported
`KURULTAI_PROJECT`. Wiring it is currently manual per session.

## Reading

**Only `recall` is project-aware.** `search`, `ask`, `cite`, `who_knows`,
`/api/atoms` and `/api/graph` all span every project. Namespacing only holds if
sessions call `recall` for context.

- MCP tool `recall` — `{ query, project?, limit?, include_quarantine? }`
- `POST /api/recall` — `{ query, project?, limit?, include_quarantine? }`
- `BrainService::recall_for_agent(project, query, limit, include_quarantine)`

The `project_id` predicate is pushed into SQL (`SearchFilter::project`), so it
applies **before** candidate truncation. This matters: the previous
implementation searched globally, truncated to `limit * 2`, and only then
filtered in memory, so a session's own atoms could be squeezed out of the
candidate window by eight other sessions and recall returned `[]` while matching
rows sat in SQL. Regression test:
`tests/acceptance_search.rs::recall_survives_a_deep_pool_of_other_project_atoms`.

Caveat on the vector arm: sqlite-vec `vec0` KNN accepts no `WHERE` predicates, so
the project filter there is applied in Rust after an 8× widened `k`. The FTS arm
is filtered in true SQL. Postgres filters both arms in SQL.

## Interaction with quality gating

The near-duplicate pass (`quality/near_dupe.rs`) no longer pairs atoms across
projects. Two sessions recording the same fact under different projects are two
atoms on purpose; merging them would delete one namespace's copy.

## Deliberately not done

- `project_id` as a first-class column plus index (#184 §1, medium term). Still
  metadata + `json_extract`.
- LRU query cache (#184 §4).
- Sharding documentation (#184 §5) — blocked on the shared-store backend
  decision.
- Project awareness on `search` / `ask` / `who_knows`.
- `expand_markdown_context` matches neighbour chunks on
  `(source, rel_path, chunk_index)` with no project check. Low practical risk (a
  file belongs to one project) but it is an unfixed cross-project text path.

## Open decisions

Neither is settled, and neither is silently assumed by this code:

1. **Which store backend** suits one box with nine sessions. The Postgres store
   is gated behind the `postgres` cargo feature plus `KURULTAI_FEATURE_HUB=1`
   and is documented "one database per organization" — hub framing, not
   local-shared framing. Scoping works identically on both backends, so this
   choice stays open.
2. **What a namespace is** — per session, per project, or both — and whether
   cross-namespace reads should default open or closed. Today a caller reads
   exactly one namespace per `recall` and there is no "read these N namespaces"
   or "read mine plus shared" mode. `resolve_project` is the single seam where a
   different policy would land.
