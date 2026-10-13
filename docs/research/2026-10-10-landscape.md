# Kurultai landscape: agent memory systems (2026-10-10)

Read with `docs/plans/2026-10-10-001-docs-program-overlay-plan.md`. Every external claim cites the page it came from; fetched 2026-10-10. Facts about Kurultai come from this repo at `origin/main` (v0.7.0).

## What Kurultai is

A local-first knowledge store written in Rust: it indexes markdown, JSON dumps, Dayflow, Pond and local git checkouts into one SQLite store (FTS5 + sqlite-vec), and serves search, cited `ask`, and MCP tools over CLI, a daemon with an embedded Brain UI, and HTTP (README; `src/mcp/server.rs`). The MCP surface includes `search`, `recall`, `cite`, `remember`, `ask`, `who_knows`, `promote`, `ontology_*` and the `hey_*` message-board tools.

## Comparable systems

### Mem0
- Open-source memory layer plus managed cloud; Apache 2.0. Retrieval fuses semantic, BM25 and entity matching; memory scoped at user, session and agent level; extraction is single-pass and add-only. Self-host via Docker Compose; library for Python and npm. The README does not mention an MCP server, only a CLI and agent skills. Default LLM `gpt-5-mini`, default embeddings `text-embedding-3-small`. Source: https://github.com/mem0ai/mem0
- Claims 92.5 on LoCoMo and 94.4 on LongMemEval for its managed platform (April 2026), noting open-source numbers will differ. Same source. These are vendor numbers.
- Better than Kurultai: managed hosting, SDKs in two languages, published benchmarks, user/session/agent scoping as a first-class API.
- Worse: add-only accumulation (no supersede or time-travel described), requires an LLM at write time.

### Zep / Graphiti
- Graphiti is a temporal knowledge graph engine: facts carry validity windows and are invalidated rather than deleted; Apache-2.0; needs Neo4j, FalkorDB or Neptune (Kuzu support deprecated), Python 3.10+ and by default an OpenAI key; ships an MCP server. Source: https://github.com/getzep/graphiti
- The Zep paper reports 94.8% vs MemGPT's 93.4% on DMR and up to 18.5% accuracy gain with 90% lower latency on LongMemEval. Source: https://arxiv.org/abs/2501.13956
- Better: real temporal reasoning over extracted entities and facts; benchmark evidence.
- Worse: heavy infrastructure (a graph database), LLM extraction cost per episode. Kurultai's `supersedes:` frontmatter and `--as-of` queries cover a cheaper slice of the same idea with zero LLM calls.

### Letta
- Agent runtime built around memory blocks (labelled `human` / `persona` values) with archival memory passages and a git-tracked "MemFS" in the newer SDK. Source: https://docs.letta.com/concepts/memgpt (the page is thin on archival and recall details; I could not fetch the GitHub README, HTTP 504).
- Better: the agent owns and edits its own core memory; memory is part of the runtime.
- Worse for Kurultai's use: it is an agent framework, so you adopt the runtime to get the memory. Kurultai is runtime-agnostic.

### Cognee
- Open-source memory platform: documents, code and conversations become a graph plus vectors; operations are `remember`, `recall`, `improve`, `forget`; can run on one Postgres; local GLiNER extraction and local embeddings work without an API key; Apache-2.0; ships an MCP server. The Postgres-as-graph path is described as demo, production being a licensed product. Source: https://github.com/topoteretes/cognee
- Better: code-to-graph, feedback loop (`improve`), `forget`.
- Worse: part of the production path is commercial.

### basic-memory
- Notes are plain Markdown on disk with a local SQLite index (Postgres also supported); MCP tools read, write, edit, move, delete and search notes; works with Obsidian; AGPL-3.0; optional hosted cloud at $15/month. Source: https://github.com/basicmachines-co/basic-memory
- Closest in spirit to Kurultai's solo mode. Better: Markdown is the source of truth and editable by hand; Obsidian-compatible; simple mental model. Worse for team use: AGPL, single-user shape.

## What users of these systems expect (from the above)

1. An MCP endpoint that works with Claude, Cursor and Codex without glue (Graphiti, Cognee, basic-memory all ship one).
2. Memory that changes over time without silent loss: invalidation or supersede (Graphiti), not append-only (Mem0).
3. Published retrieval numbers (Mem0, Zep). Kurultai has `kurultai eval` and `evals/golden.json` but no published results.
4. Works without a paid key (Cognee local path, basic-memory). Kurultai FTS mode does.
5. Hosted option or one-command self-host (Mem0, basic-memory).

## What is unique about Kurultai

- Multi-agent coordination in the same store: the Hey board, per-codename seat identity and `who_knows` routing. None of the five above lists an agent message board (this is an absence in the pages I read, not proof).
- Trust lanes (`trusted` vs `quarantine`) with a tag gate at write time, and visibility scopes (`personal`/`team`/`company`) that exist in the schema.
- Gap-aware `ask`: answers say what the brain does not know.
- Zero-LLM index-time edge extraction (`[[links]]`, `@mentions`, frontmatter) and zero-LLM supersede/time-travel.
- One Rust binary, SQLite by default; Postgres only for the hub tier.

## Where Kurultai is weaker

- No published benchmark numbers against LoCoMo/LongMemEval (all peers publish or cite one). 
- Surface area: 16+ MCP tools plus a Next.js app, a Vite UI, a broker and a Python bridge, versus peers' narrower scope.
- Multi-hop retrieval is an open issue (#120); Graphiti and Cognee have graph traversal today.
- Hosted/team RBAC is open (#115, #188).
