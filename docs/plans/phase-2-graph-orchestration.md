# Phase 2 note: Graph orchestration (not linear agent chains)

**Tracking:** [#6](https://github.com/duketopceo/kurultai/issues/6) Search & Retrieval · [#7](https://github.com/duketopceo/kurultai/issues/7) Synthesis  
**Source:** Graph-engineering framing (Codez / industry discourse, 2026) — parked for Kurultai, not a product pivot.  
**Status:** Design note only. Does not block Phase 2 planning.

---

## Thesis

Most linear agents form degenerate chains with wasteful waits. Redraw by cutting non-data “and then” edges so independent work can run in parallel.

- **Nodes** — bounded jobs with input/output contracts and schemas  
- **Edges** — carry validated data shapes (not vibes or chat turns)  
- **Fan-out** — `parallel()` independent work  
- **Fan-in** — barriers only when full results are needed → diamond: split → work → merge  
- **Runtime** — routing, adversarial verifiers on edges, loop-until-dry, model tiering, node isolation  

Orchestrators that spawn coordinated subagent fleets are one *delivery* of this idea. Kurultai’s durable edge is **SQL as shared state + typed MCP/CLI contracts**, not becoming a second agent-runtime product.

---

## Map onto Kurultai

| Graph idea | Phase 1 (done) | Phase 2+ (#6 / #7) |
|------------|----------------|---------------------|
| Node I/O contracts | `AgentAtomView`, cite, remember payloads | Keep schemas stable; expand only with typed fields |
| Cut non-data waits | FTS ∥ embed already overlapped in search | Diamond for FTS + vector (+ later rerank) → fuse only at merge |
| Fan-in barrier | Naive score merge today | RRF / fusion at one barrier; no serial “FTS then vector then …” waits |
| Adversarial edge | CI + store zero-vector guard | Separate verifier for answer/cite consistency (don’t trust the writer node) |
| Shared state | SQLite atoms | Same brain; graph converges *on* SQL, not chat history |
| Isolation | Process-local MCP stdio | Worktree/subagent isolation when fan-out mutates code; read-only search stays cheap |

### Steal

1. Treat retrieval as a **diamond**, not a chain: fan-out FTS + vector (+ optional rerank) → barrier at rank/fusion only.  
2. Keep MCP/tool results as **validated shapes** (`AgentAtomView`, `Citation`).  
3. Prefer a **verifier node** (tests, cite checks) distinct from the node that produced the answer.  
4. Loop-until-dry only *inside* a node (e.g. embed retry); don’t serialize the whole pipeline on one retry.

### Don’t overbuild

- Free-JS “dynamic workflows” spawning fleets are harness-specific. Kurultai ships the **brain + contracts**; clients (Cursor, Claude, CE) own fleet orchestration.  
- Never `parallel()` across true data deps (e.g. remember → cite that id; index → search that corpus).  
- No second graph DB in Phase 2 — SQL atoms + edges-as-tool-IO is enough until a real multi-hop product need appears.

---

## One-line doctrine add-on

**Orchestrate as a dataflow graph; loops live inside nodes; SQL holds the shared state the graph converges on.**

Aligns with existing doctrine ([#37](https://github.com/duketopceo/kurultai/issues/37)): index-time heavy, read-time light, structured atoms — graph orchestration is *how agents call that brain*, not a replacement for it.

---

## When to deepen

- Implementation plan: [2026-07-21-001-feat-search-retrieval-rrf-plan.md](2026-07-21-001-feat-search-retrieval-rrf-plan.md) (`/ce-plan` for #6).  
- Only promote “knowledge graph” storage if multi-hop entity edges become a product requirement (not for fusion alone).
