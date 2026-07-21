# Upstream inspiration & dependencies

How we use external repos while building Kurultai. Tracked in [#40](https://github.com/duketopceo/kurultai/issues/40). Master work order: [#27](https://github.com/duketopceo/kurultai/issues/27).

## How we use upstream

| Action | When | Example |
|--------|------|---------|
| **Depend** | Mature crate/extension with clear API | `sqlite-vec`, `rmcp` |
| **Inspire** | Same problem, different stack or license — port patterns | layer0 schema, mdvault chunking |
| **Integrate** | Run as sidecar/subprocess early, wrap in connector | CocoIndex for GitHub code |
| **Avoid fork** | AGPL, markdown-as-truth, or heavy infra mismatch | basic-memory, Graphiti/Neo4j |

**Doctrine filter ([#37](https://github.com/duketopceo/kurultai/issues/37)):** ingest sources stay on disk; the **brain** is SQL + `KnowledgeAtom` + `AgentAtomView` at the MCP boundary. Upstream that treats markdown/git as source of truth is inspiration for **connectors only**, not architecture.

**Cerebras reference (no repo):** [How Cerebras built a 15K-query/day KB](https://mer.vin/2026/07/how-cerebras-built-a-15k-query-day-internal-knowledge-base/) — one embeddings table, hybrid Slack ranking (FTS + embed + IDF + age decay → RRF → rerank), MCP retrieval primitives, CocoIndex for code at scale.

---

## Phase 1 — Foundation

**Status:** ✅ Complete ([phase-1-complete.md](plans/phase-1-complete.md)). Phase 2 starts at [#6](https://github.com/duketopceo/kurultai/issues/6).

### [#1 Storage: SqliteVecStore](https://github.com/duketopceo/kurultai/issues/1)

**Build:** SQLite tables for `KnowledgeAtom`, FTS5 mirror, `vec0` virtual table, migrations, content-hash dedupe skip.

| Repo | Action | Pull |
|------|--------|------|
| [asg017/sqlite-vec](https://github.com/asg017/sqlite-vec) | **Depend** | `cargo add sqlite-vec`; `vec0` cosine KNN; Rust bindings |
| [amajorai/layer0](https://github.com/amajorai/layer0) | Inspire | Schema: `documents` + `chunks` + `vec_chunks` + `documents_fts`; per-database file isolation under `~/.layer0/databases/` |
| [yun-lim/atomic](https://github.com/yun-lim/atomic) | Inspire | `atomic-core` split: store logic without HTTP/MCP framework |
| [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) | Inspire | Rust sqlite-vec + FTS5 in one binary; heading-weighted BM25 |

**SQL pattern (hybrid, from sqlite-vec + Supabase RRF):** shared `rowid` across `atoms_fts` and `vec_atoms`; fuse with `1/(k+rank_fts) + 1/(k+rank_vec)`, `k=60`.

**Do not copy:** Postgres/pgvector backends; graph tables until Phase 3+ ([#33](https://github.com/duketopceo/kurultai/issues/33)).

---

### [#2 Embeddings: OpenRouter](https://github.com/duketopceo/kurultai/issues/2)

**Build:** `Embedder` trait, OpenRouter HTTP client, batch embed, dimension config (3072 default), hash-skip when content unchanged.

| Repo | Action | Pull |
|------|--------|------|
| [amajorai/layer0](https://github.com/amajorai/layer0) | Inspire | Pluggable providers: OpenRouter, Ollama, OpenAI-compatible |
| [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) | Inspire | Local path: ONNX `nomic-embed-text-v1.5` via fastembed (Phase 5 #9) |
| [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) | Inspire | Same embedder for index + query paths; BGE normalization |
| [roboticforce/remembrallmcp](https://github.com/roboticforce/remembrallmcp) | Inspire | fastembed `all-MiniLM-L6-v2` in-process ONNX |

**Do not copy:** Hard dependency on cloud embed for dev — keep zero-vector fallback in dev env ([#29](https://github.com/duketopceo/kurultai/issues/29)).

---

### [#31 Connector: Filesystem markdown](https://github.com/duketopceo/kurultai/issues/31)

**Build:** Walk `root_path`, parse frontmatter, chunk by `##`/`###`, SHA256 incremental skip, emit `KnowledgeAtom` per chunk (distill later).

| Repo | Action | Pull |
|------|--------|------|
| [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) | **Inspire / partial port** | Heading chunks, YAML frontmatter, trigram FTS tokenizer, wikilink adjacency |
| [sderosiaux/mdvault](https://github.com/sderosiaux/mdvault) | Inspire | SHA256 change detection; `##` chunks max ~400 words + overlap; context prefix `[path > title > heading]` |
| [itkoren/sqmd](https://github.com/itkoren/sqmd) | Inspire | Hierarchical chunking preserves heading tree; token-budgeted RAG context builder |
| [pvliesdonk/markdown-vault-mcp](https://github.com/pvliesdonk/markdown-vault-mcp) | Inspire | Frontmatter-aware indexer, hash-based incremental reindex |
| [KardungLa/mdaifs](https://github.com/KardungLa/mdaifs) | Inspire | Frontmatter filters + auto link graph from `[[wikilinks]]` |

**Do not copy:** Storing chunks as the brain (mdvault/sqmd index into their own DB — we ingest into `KnowledgeAtom` only). Obsidian desktop integration.

---

### [#4 Connector: AppFlowy](https://github.com/duketopceo/kurultai/issues/4)

**Build:** REST/MCP poll → pages/databases → atoms with `source_uri`.

| Repo | Action | Pull |
|------|--------|------|
| [basicmachines-co/basic-memory](https://github.com/basicmachines-co/basic-memory) | Inspire (MCP shapes only) | `write_note` / `read_note` tool ergonomics; **AGPL — do not fork** |
| [cocoindex-io/cocoindex](https://github.com/cocoindex-io/cocoindex) | Inspire | Generic source connector pattern: declare transform, engine syncs delta |

---

### [#5 CLI: index, status, search](https://github.com/duketopceo/kurultai/issues/5)

**Build:** Wire `app::run`, connector registry poll, store upsert, human-readable search output.

| Repo | Action | Pull |
|------|--------|------|
| [pratikgajjar/recall](https://github.com/pratikgajjar/recall) | Inspire | Single binary; `index` / `search` / `sessions`; Lua plugin hooks for new sources |
| [Gentleman-Programming/engram](https://github.com/Gentleman-Programming/engram) | Inspire | `engram setup [agent]` writes MCP config per IDE |
| [amajorai/layer0](https://github.com/amajorai/layer0) | Inspire | `layer0 serve` + `layer0 config` TUI |

---

### [#11 MCP + universal installer](https://github.com/duketopceo/kurultai/issues/11)

**Build:** stdio MCP server (`search`, `cite`, `ask`, `remember`), `kurultai init` writes `~/.cursor/mcp.json` + Claude config, Smithery publish.

| Repo | Action | Pull |
|------|--------|------|
| [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) | **Inspire** | Rust MCP + `commit_to_memory`; single release binary; stdio transport |
| [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) | Inspire | MCP `search` over local index; optional HTTP transport |
| [smithery-ai/cli](https://github.com/smithery-ai/cli) | Integrate | `smithery mcp add kurultai --client cursor`; publish URL/bundle |
| [rmcp](https://crates.io/crates/rmcp) | **Depend** | Rust MCP server SDK |
| [Gentleman-Programming/engram](https://github.com/Gentleman-Programming/engram) | Inspire | Agent-specific setup matrix (Cursor, Claude, Codex, Windsurf) |

**MCP contract:** enforce `AgentAtomView` excerpts — never return full `content` by default ([#37](https://github.com/duketopceo/kurultai/issues/37)).

---

## Phase 2 — Search & retrieval

**Status:** ✅ Retrieval path shipped (RRF diamond + optional rerank). Distillation from #6 deferred to #12.

### [#6 FTS + vector + RRF + rerank](https://github.com/duketopceo/kurultai/issues/6)

**Build:** Parallel FTS5 + vec KNN → RRF (`k=60`) → optional cross-encoder rerank → `SearchResult` → `AgentAtomView`.

| Repo | Action | Pull |
|------|--------|------|
| [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) | **Inspire** | RRF fusion; heading column 2× BM25 weight; short-query vector-only fallback |
| [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) | Inspire | ONNX cross-encoder `ms-marco-MiniLM` rerank on top candidates |
| [amajorai/layer0](https://github.com/amajorai/layer0) | Inspire | `hybrid` / `vector` / `graph` search modes |
| [sderosiaux/mdvault](https://github.com/sderosiaux/mdvault) | Inspire | 7-signal re-rank after RRF (term coverage, heading, path) |
| [itkoren/sqmd](https://github.com/itkoren/sqmd) | Inspire | Token-budgeted context assembly for `ask` pipeline |
| Cerebras KB write-up | Inspire | IDF + **age decay** signals before RRF (Slack-style sources) |

**Reference SQL:** [Alex Garcia hybrid search blog](https://alexgarcia.xyz/blog/2024/sqlite-vec-hybrid-search/index.html), [sqlite-vec issue #48](https://github.com/asg017/sqlite-vec/issues/48).

---

## Phase 3 — Synthesis & interface

### [#7 Synthesis + MCP + daemon + agent capture](https://github.com/duketopceo/kurultai/issues/7)

**Build:** Planner → retrieve → synthesize with citations; HTTP daemon; ingest Cursor/Claude/Codex JSONL → distill → atoms.

| Repo | Action | Pull |
|------|--------|------|
| [imphillip/gbrain-openclaw](https://github.com/imphillip/gbrain-openclaw) | Inspire | **Compiled truth** + append-only **timeline** per entity → maps to `summary` + `resolution` + provenance |
| [keshrath/agent-knowledge](https://github.com/keshrath/agent-knowledge) | Inspire | Multi-host transcript adapters; auto-distill with secrets scrubbing |
| [pratikgajjar/recall](https://github.com/pratikgajjar/recall) | **Inspire** | Plugin registry: `line` (JSONL), `file`, `kv` (SQLite watermark); paths for Cursor/Claude/Codex |
| [Kuberwastaken/reference](https://github.com/kuberwastaken/reference) | Inspire | Cross-tool session search; `uvx` one-liner install |
| [patrykkopycinski/cursor-chat-browser](https://github.com/patrykkopycinski/cursor-chat-browser) | Inspire | `~/.cursor/projects/*/agent-transcripts/` layout + FTS index |
| [yun-lim/atomic](https://github.com/yun-lim/atomic) | Inspire | Wiki synthesis with inline citations from atoms |
| [getzep/graphiti](https://github.com/getzep/graphiti) | Inspire (semantics only) | Temporal fact invalidation — **do not fork** (Neo4j); consider `valid_from`/`valid_to` on atoms ([#33](https://github.com/duketopceo/kurultai/issues/33)) |

**Cerebras pattern:** MCP exposes **retrieval primitives** (`search`, `cite`); orchestration stays in the agent — matches `src/mcp/interface.rs`.

---

## Phase 4 — Expansion

### [#8 GitHub / Pond connectors](https://github.com/duketopceo/kurultai/issues/8)

**Build:** Tree-sitter code chunks, incremental re-embed on commit, optional symbol graph.

| Repo | Action | Pull |
|------|--------|------|
| [cocoindex-io/cocoindex](https://github.com/cocoindex-io/cocoindex) | **Integrate** | Cerebras production choice; `@coco.fn(memo=True)` delta; Tree-sitter splits; Apache-2.0 |
| [cocoindex-io/realtime-codebase-indexing](https://github.com/cocoindex-io/realtime-codebase-indexing) | Inspire | ~100-line reference pipeline; `cocoindex update -L` live watch |
| [Phoenixrr2113/codebase-graph](https://github.com/Phoenixrr2113/codebase-graph) | Inspire | tree-sitter + graph MCP; symbol `callers`/`callees` if Pond needs it |
| [roboticforce/remembrallmcp](https://github.com/roboticforce/remembrallmcp) | Inspire | `ingest_github` bulk PR import |
| [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) | Inspire | Codebase RAG + persistent `commit_to_memory` for decisions |

**Strategy:** Phase 4a — subprocess CocoIndex → ingest into Kurultai atoms; Phase 4b — native Rust tree-sitter connector.

---

### [#21 Dayflow connector](https://github.com/duketopceo/kurultai/issues/21)

**Build:** Read Dayflow timeline DB / markdown export → activity atoms (`source: dayflow`).

| Repo | Action | Pull |
|------|--------|------|
| [JerryZLiu/Dayflow](https://github.com/JerryZLiu/Dayflow) | **Integrate (read-only)** | MIT Swift app; data at `~/Library/Application Support/Dayflow/`; timeline export API |
| [imphillip/gbrain-openclaw](https://github.com/imphillip/gbrain-openclaw) | Inspire | Timeline-as-evidence, compiled summary as truth |

**Do not fork** the Dayflow app — connector only.

---

## Phase 5 — Production

### [#9 Perf + shared daemon](https://github.com/duketopceo/kurultai/issues/9)

| Repo | Action | Pull |
|------|--------|------|
| [amajorai/layer0](https://github.com/amajorai/layer0) | Inspire | Named DB isolation; OpenAI-compatible HTTP API alongside MCP |
| [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) | Inspire | Local ONNX embed + rerank — no API keys for search path |
| [edg-l/engram-mcp](https://github.com/edg-l/engram-mcp) | Inspire | Rust SQLite + ONNX; branch-scoped memory (team policy ideas) |

---

### [#20 Self-hosted CI (ARC)](https://github.com/duketopceo/kurultai/issues/20)

No direct upstream in this research set — standard [Actions Runner Controller](https://github.com/actions/actions-runner-controller) patterns apply.

---

## Cross-cutting

### [#37 Speed + token doctrine](https://github.com/duketopceo/kurultai/issues/37)

| Repo | Pull |
|------|------|
| [itkoren/sqmd](https://github.com/itkoren/sqmd) | RAG context builder with explicit token budget |
| [tinydarkforge/Engram](https://github.com/tinydarkforge/Engram) | Token-budgeted retrieval; assertion ledger confidence |
| [sderosiaux/mdvault](https://github.com/sderosiaux/mdvault) | Excerpt-first search results |
| Kurultai `src/brain/mod.rs` | `AgentAtomView`, `DEFAULT_EXCERPT_CAP` (PR [#38](https://github.com/duketopceo/kurultai/pull/38)) |

### [#33 Post-train schema](https://github.com/duketopceo/kurultai/issues/33)

| Repo | Pull |
|------|------|
| [yun-lim/atomic](https://github.com/yun-lim/atomic) | Stable atom fields for export |
| [imphillip/gbrain-openclaw](https://github.com/imphillip/gbrain-openclaw) | Entity + timeline schema for NN export |
| [getzep/graphiti](https://github.com/getzep/graphiti) | Bi-temporal edges — field ideas only |

### [#23 Testing & CI](https://github.com/duketopceo/kurultai/issues/23)

| Repo | Pull |
|------|------|
| [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) | Integration test target: hybrid search golden queries |
| [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) | MCP smoke tests against local index |

### [#29 Environments](https://github.com/duketopceo/kurultai/issues/29)

| Repo | Pull |
|------|------|
| [amajorai/layer0](https://github.com/amajorai/layer0) | Per-env SQLite file paths |

---

## Explicit avoid list

| Repo | Reason |
|------|--------|
| [basicmachines-co/basic-memory](https://github.com/basicmachines-co/basic-memory) | AGPL-3.0; markdown is source of truth |
| [getzep/graphiti](https://github.com/getzep/graphiti) | Neo4j/FalkorDB ops; Python stack |
| [mem0](https://mem0.ai) | SaaS memory; not unified local brain |
| [JerryZLiu/Dayflow](https://github.com/JerryZLiu/Dayflow) | Fork app — connector reads data only |

---

## Priority study order (implementers)

1. [alphabet-h/kb-mcp](https://github.com/alphabet-h/kb-mcp) — Rust markdown + hybrid + MCP
2. [amajorai/layer0](https://github.com/amajorai/layer0) — Rust brain schema + hybrid RAG
3. [cocoindex-io/cocoindex](https://github.com/cocoindex-io/cocoindex) — incremental ingest (Cerebras-aligned)
4. [DakodaStemen/Stratum](https://github.com/DakodaStemen/Stratum) — Rust MCP binary + rerank
5. [pratikgajjar/recall](https://github.com/pratikgajjar/recall) + [keshrath/agent-knowledge](https://github.com/keshrath/agent-knowledge) — agent transcript connectors

---

## License quick reference

| Repo | License | Kurultai use |
|------|---------|--------------|
| sqlite-vec | Apache-2.0 | Depend |
| cocoindex | Apache-2.0 | Integrate / inspire |
| Dayflow | MIT | Connector read |
| kb-mcp, layer0, Stratum | Verify before fork | Inspire / port patterns |
| basic-memory | AGPL-3.0 | MCP ideas only |
