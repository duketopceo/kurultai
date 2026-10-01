---
artifact_contract: ce-unified-plan/v1
execution: code
---

# Competitive feature sweep — gbrain / OpenViking / agent-memory lane

**Date:** 2026-10-01 · **Origin:** competitor scan (GBrain `garrytan/gbrain`, OpenViking context DB, mem0/Zep-Graphiti/Letta/Cognee/Supermemory landscape + pplx-embed release).

## Goal Capsule

- **Objective:** steal the four features competitors ship that we don't, and upgrade embeddings off NullEmbedder/3-large.
- **Means:** five right-sized units; each lands behind existing seams (no daemon re-architecture).
- **Stop conditions:** no visual changes to Brain/chrome; no external SaaS dependency as a hard requirement (pplx-embed is opt-in like OpenRouter today).

## Feature list + light plans

### U1 — pplx-embed backend (do first, unblocks everything downstream)

Perplexity's `pplx-embed-v1` (0.6B @ 1024d, $0.004/1M tok) and `pplx-embed-v1-4b` (2560d, $0.03) are MIT + ONNX-capable + API-available.

| Model | Dims | $/1M tok | Notes |
|---|---|---|---|
| openai/text-embedding-3-large (current config) | 3072 | ~$0.13 | via OpenRouter; quality ceiling |
| openai/text-embedding-3-small | 1536 | $0.02 | cheap, dated |
| **pplx-embed-v1-0.6b** | 1024 | **$0.004** | cheapest credible; MTEB/ToolRet leader at size |
| **pplx-embed-v1-4b** | 2560 | $0.03 | ~4.3x cheaper than 3-large, SOTA-ish |
| pplx-embed-context-v1-* | 1024/2560 | $0.008–0.05 | doc-context-aware chunk embeds — interesting for markdown atoms |
| Local ONNX (AllMiniLM today; pplx-0.6b via `local-embed` feature) | 384/1024 | free | pplx ships ONNX → could replace AllMiniLM as the default local |

Plan: add `PerplexityEmbedder` next to `OpenRouterEmbedder` in `src/embed/mod.rs` (~60 lines, OpenAI-compatible POST, `PERPLEXITY_API_KEY` — already a used env for ask --web); config `embed.backend = "perplexity"`. Eval on the existing `kurultai eval` path against 3-large. Also test pplx-embed-0.6b through the ONNX `local-embed` path as the new local default. **Dim change requires reindex** — keep `embed_dim` per-store, warn+reindex on mismatch.

### U2 — Gap-aware synthesis (`ask` reports what's missing)

gbrain's differentiator: synthesized answers explicitly enumerate knowledge gaps. Our `ask` is extractive/LLM with no "unknowns" honesty.

Plan: extend `src/mcp/server.rs` ask + `POST /api/ask` response with `gaps: Vec<String>` — computed cheaply from evidence thinness (query terms with zero FTS/vector hits, cited atom count < N, quarantined-only hits). LLM variant adds a one-line "the brain lacks:" to the synthesizer prompt. Ship as `ask --gaps` + response field; surface later in UI.

### U3 — Zero-LLM typed edge extraction on write

gbrain auto-links `works_at`/`invested_in` at write time with no LLM call; Graphiti does bi-temporal edges. Our ontology is manual-promote.

Plan: in `src/pipeline` (or connector ingest path), extract cheap signals at atom write: `[[wiki-links]]`, `@people`, `#tags`, frontmatter `relates_to:`/`people:`/`orgs:` — write typed edges into the ontology tables (`attends`/`works_at`/`relates_to` predicate set, source=heuristic). No LLM in the hot path; promotion still gates trusted→canonical. Test: fixture doc with `[[X]]`/`@y` produces edges.

### U4 — Corrections + supersede (bi-temporal lite)

gbrain supports corrections/withdrawal; Graphiti tracks `valid_at`/`invalid_at`. Our atoms are append-only with promote/quarantine.

Plan: `supersedes: Option<String>` on atom writes (`remember`/`ingest` accepts `supersedes=<atom_id>`); store marks old atom `invalid_at`; default search filters invalidated; `--as-of <ts>` flag on search/history queries the valid_at window. Light version: supersede chain only, no full bi-temporal algebra.

### U5 — Consolidation sweep ("sleep-time compute")

gbrain/Letta both run overnight jobs that fix citations, dedupe, merge. We have `promote` + judge but nothing scheduled on the store itself.

Plan: `kurultai sweep` subcommand + daemon nightly task (`nightly_full_sync_hour` already in config): merge dup atoms (near-identical hash/content), fix stale `source_id` links, decay unused hot-tier atoms toward warm, emit a `sweep_report` atom into Hey. Reuses existing store APIs; ~200 lines.

## Deferred (not now)

- **Connector permission levels** (Gmail read/draft/manage) — broker seat tokens already scope writes→quarantine; formal per-tool scopes belong with the broker plan's remaining units, not this sweep.
- **Multi-brain mounts / federated search** — broker gives per-device daemons; mounting `work` into `knowledge` queries is a bigger design (auth + identity across lanes).
- **viking:// filesystem-addressable context** — atoms already addressable by id/source; a tree browse API is cosmetic until ontology lands.
- **Account connectors** (Gmail/Calendar ingestion) — separate product surface.

## pplx-embed decision inputs

- API route keeps FTS+vector parity with zero infra; local-embed route removes the API dependency entirely (ONNX, MIT weights).
- Benchmarks cited by Perplexity: MTEB(Multilingual v2), BERGEN, ToolRet, ConTEB leaders — relevant axis is **ToolRet** (tool/agent retrieval ≈ our search lane).
- MRL + INT8/BINARY quantization → could shrink the vector store ~8-32x if we take binary embeds (future, not U1).

## Verification

- `kurultai eval` comparison run: NullEmbedder (baseline) vs pplx-embed-v1-0.6b vs 3-large on the dev corpus; report P@5/latency/cost.
- Each unit ships behind its existing surface (no API breakage); CI green per unit.
