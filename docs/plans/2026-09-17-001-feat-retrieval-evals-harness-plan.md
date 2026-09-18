# feat: retrieval quality evals + opt-in web augmentation (`ask --web`)

## Summary

Ship a repeatable retrieval benchmark so ranking, tier-policy, and
sequestering changes get measured instead of eyeballed: a frozen golden query
set, a `kurultai eval` runner that scores `search`/`ask` over HTTP, and an
env-gated Jev judge (OpenRouter decisions API) for graded relevance and
groundedness. Second deliverable: opt-in `ask --web` ephemeral Perplexity
augmentation — REST-direct, no atom writes — plus a Jev `noul` sufficiency
gate so web calls fire only when local context is thin.

## Problem Frame

Every retrieval-affecting change so far (RRF boosts, noisy_sources denylist,
declarative tier caps) shipped with qualitative checks only. `search` returns
an RRF-fused float — there is no graded relevance, no Recall@k, no regression
gate. `scripts/recall-harness.py` measures latency/correctness, not relevance.

## Requirements

- R1. Frozen golden query set versioned in-repo, hand-labeled from real
  dogfood queries (synthetic parrot-queries inflate scores).
- R2. `kurultai eval` scores `search` and `ask` against any daemon base URL —
  local dev store and hosted Ulaanbaatar (`knowledge.shippedit.dev`) alike.
- R3. Metrics: Recall@k, Precision@k, MRR, nDCG@k (graded), answer
  groundedness, noise exclusion (pond/quarantine atoms absent).
- R4. Jev judge is optional: unset `OPENROUTER_API_KEY` → labels-only metrics
  still run; set → graded relevance/groundedness. Pin `typesafe/jev-1.13`;
  log resolved model version per run.
- R5. `ask --web` is opt-in (`PERPLEXITY_API_KEY`), ephemeral (no atom
  writes), gated by a local-sufficiency check so paid calls stay rare.
- R6. LLM judging stays scoring-side only — never in the ingest quality gate
  (prior decision, `docs/solutions/architecture-patterns/trust-lanes-quality-gate.md`).
- R7. Follows FTS-first doctrine: everything above degrades cleanly with no
  keys configured.

## Key Technical Decisions

- **KTD1 — HTTP target, not in-process.** `kurultai eval --base-url` hits
  `POST /api/search` and `POST /api/ask` on a running daemon. Covers local +
  hosted with one code path; matches `recall-harness.py` precedent. In-process
  `BrainService` tests stay in `tests/` for CI regression coverage.
- **KTD2 — Rust runner inside the CLI, not Python.** The repo's script layer
  is stdlib-Python for ops tooling, but golden-set scoring is product-adjacent
  and needs the same types (`SearchResult`, `Answer`). A `Commands::Eval`
  subcommand keeps it in-tree; `recall-harness.py` is untouched.
- **KTD3 — Jev via OpenRouter `/api/alpha/decisions`, not chat-completions.**
  Verified live: `state` + keyed `questions` (`noul`/`choice`/`score`) →
  calibrated probabilities in ~0.35s, ~$0.00002/call. Mirrors the
  `*_from_env()` + Null-impl pattern (`synthesizer_from_env`,
  `build_reranker`): `OpenRouterJudge`/`NullJudge`, key chain
  `OPENROUTER_API_KEY` → `KURULTAI_API_KEY` → keyfile. Response carries
  resolved model (`jev-1.13-20260917`) — persist it in the report.
- **KTD4 — Perplexity REST-direct, ephemeral.** `POST
  https://api.perplexity.ai/search` (`query`, `max_results`, optional
  domain/recency filters) → `{title,url,snippet,date}`. New `src/web/` module
  (trait `WebSearcher` + `NullWebSearcher`), NOT a connector — connectors are
  ingest-side. No Python SDK, no atom writes, no connector scaffolding.
- **KTD5 — Sufficiency gate before spend.** `ask --web` first runs local
  retrieval; a Jev `noul` ("do these excerpts answer the question?") decides
  whether the Perplexity call fires. Judge absent → conservative fallback:
  fire only when top-hit RRF score is below a config threshold.
- **KTD6 — Web results join as parallel context, not pseudo-atoms.** Inject
  web hits alongside `hits` in the synthesizer context block and map to
  `Citation` directly (url field already exists). Pseudo-`SearchResult`s would
  leak into `touch_access`, activity records, and quality boosts — rejected.

## High-Level Technical Design

```
kurultai eval --base-url <daemon> [--judge] [--k 10]
   │
   ├─ load evals/golden.json  (query → expected atom ids/title hashes + negatives)
   ├─ for each query: POST /api/search → label metrics (Recall@k, P@k, MRR)
   │                  optional judge: nDCG via score-rubric per (q,hit) pair
   ├─ for each ask query: POST /api/ask → judge groundedness + citation support
   ├─ noise assertions: pond/quarantine ids absent from results
   └─ report: JSON + console; stores config hash, resolved judge model, git sha

kurultai ask "..." --web
   │
   ├─ normal local retrieval (unchanged)
   ├─ sufficiency check (Jev noul, else score-threshold fallback)
   │     sufficient → answer locally, zero web spend
   │     thin → POST api.perplexity.ai/search (top 3)
   │            → web hits appended to synthesizer context + citations
   └─ Answer unchanged shape; web cites carry url + source="web"
```

## Output Structure

```
evals/
  INDEX.md              # index ritual
  golden.json           # frozen query set (v1: ~30 queries)
  reports/              # gitignored run outputs (or stdout-only)
src/
  eval/                 # runner + metrics + report types
    mod.rs
    metrics.rs
    judge.rs            # trait Judge + OpenRouterJudge + NullJudge
  web/
    mod.rs              # trait WebSearcher + PerplexitySearcher + NullWebSearcher
tests/
  evals_search.rs       # in-process regression suite on fixture vault
  fixtures/evals/       # tiny golden subset for CI (no network)
```

## Implementation Units

### U1. Golden set + metrics core

**Goal:** Frozen query set and pure-metric functions, no network.
**Requirements:** R1, R3.
**Dependencies:** none.
**Files:** `evals/golden.json`, `evals/INDEX.md`, `src/eval/metrics.rs`,
`tests/fixtures/evals/`, `tests/evals_search.rs`.
**Approach:** golden.json entries: `{id, query, relevant: [atom-id or
title-hash], negative_sources?: ["pond"], kind: "search"|"ask"}`. Seed ~30
queries from real dogfood history (Hey/pond sessions, deploy notes) plus
known-phrase fixture queries. Metrics: `recall_at_k`, `precision_at_k`,
`mrr`, `ndcg_at_k` (graded gains vector), pure fns over id lists.
**Test scenarios:** metric math on hand-computed lists (perfect ranking →
1.0; miss → 0; graded nDCG ordering); golden.json schema parse rejects
missing fields; fixture subset runs against `tests/fixtures/vault` and
asserts `KNOWN_PHRASE_KURULTAI_42` doc recalled at k=5.
**Verification:** `cargo test` green; metrics unit tests pass.

### U2. `kurultai eval` runner over HTTP

**Goal:** CLI subcommand executing the golden set against a live daemon.
**Requirements:** R2, R3, R6.
**Dependencies:** U1.
**Files:** `src/main.rs` (Commands::Eval + dispatch), `src/eval/mod.rs`
(runner, HTTP client, report), `evals/golden.json`.
**Approach:** `eval [--base-url http://127.0.0.1:8421] [--k 10] [--judge]
[--json out.json]`. Reuse reqwest patterns from `src/rerank/mod.rs`. Report:
per-query rows + aggregate + config fingerprint (tier policy hash, noisy
sources, embedder/synthesizer live flags, git sha, resolved judge model).
**Test scenarios:** against in-process axum router (stress_http.rs pattern):
full run emits valid report; missing daemon → clean error; `--k` respected;
pond-sourced atoms flagged when present in results (noise-exclusion metric).
**Verification:** run against local daemon + fixture store, report written.

### U3. Jev judge (`src/eval/judge.rs`)

**Goal:** Optional graded relevance + groundedness via OpenRouter decisions.
**Requirements:** R4, R6.
**Dependencies:** U1 (metrics consume judge scores), U2 (runner wires it).
**Files:** `src/eval/judge.rs`, `src/eval/mod.rs`, `src/main.rs`,
`.env.example`, `CONCEPTS.md` (judge term), relevant `INDEX.md` rows.
**Approach:** `trait Judge { is_live(); grade(state, questions) }` +
`OpenRouterJudge` posting to `api/alpha/decisions`, model default
`typesafe/jev-1.13` (config `[eval] judge_model`), `NullJudge` when no key.
Question batches: per (query, hit) `score` rubric 0–3 → nDCG gains; per ask
answer: `score` groundedness + `noul` citation support. Record resolved
`model` + `usage.cost` in report. Alpha endpoint — assert response shape,
fail soft to labels-only metrics.
**Test scenarios:** mock HTTP (stub Judge impl per retrieval_hybrid.rs
pattern): nDCG consumes judge gains; NullJudge → report omits graded
metrics; malformed response → soft-fail; resolved model logged.
**Verification:** `--judge` run on local store produces graded metrics and
logged model version.

### U4. `ask --web` ephemeral Perplexity augmentation

**Goal:** Opt-in web fallback in `ask`, no atom writes.
**Requirements:** R5, R7.
**Dependencies:** none (soft-depends U3 for sufficiency gate; ships with
score-threshold fallback regardless).
**Files:** `src/web/mod.rs`, `src/mcp/brain.rs` (`ask_with_team` injection),
`src/synthesize/mod.rs` (context block accepts web hits), `src/main.rs`
(`ask --web`), `src/features.rs` (`web_search` flag), `src/config/file.rs`
(`[web]` non-secret settings), `.env.example`.
**Approach:** `trait WebSearcher` + `PerplexitySearcher` (POST
`api.perplexity.ai/search`, `max_results=3`) + `NullWebSearcher`;
`web_searcher_from_env()` gated on `PERPLEXITY_API_KEY` AND
`KURULTAI_FEATURE_WEB_SEARCH`. Sufficiency: Jev `noul` when live, else
top-RRF-score threshold (`[web] min_local_score`). Web hits appended to
synthesizer context as `source=web` excerpts + citations (url from hit);
excluded from `touch_access`, activity, quality boosts.
**Test scenarios:** NullWebSearcher → `--web` warns and answers locally;
stub searcher returns hits → citations include url + source=web;
sufficient local context → searcher not called (assert via stub counter);
web hits absent from `touch_access` side effects.
**Verification:** `ask --web` on thin query calls Perplexity once and cites
URLs; on fat query spends zero calls.

### U5. Docs, index ritual, CI wiring

**Goal:** Documentation, INDEX.md rows, optional CI lane.
**Requirements:** all.
**Dependencies:** U1–U4.
**Files:** `evals/README.md` (labeling guide), `docs/eval/INDEX.md`,
`tests/INDEX.md`, `src/INDEX.md`, root `INDEX.md`, `CONCEPTS.md`,
`.github/workflows/` (optional evals job — fixture-only, no secrets).
**Approach:** labeling guide covers hand-labeling from real queries,
negative/noise cases, refresh cadence. CI job runs `tests/evals_search.rs`
offline only (no Jev/Perplexity — secrets stay out of CI initially).
**Test scenarios:** `audit-agent-index.py` green; offline CI suite passes.
**Verification:** full check suite green.

## Scope Boundaries

- LLM judging never enters the ingest quality gate (R6).
- No Python SDK / `pplx-srch-sdk` dependency — REST-direct only.
- No atom writes from web results; no web connector.
- No hosted-store eval in CI (auth + flake surface); hosted runs are manual.

### Deferred to Follow-Up Work

- Jev tier/lane classification — only if the harness shows declarative
  tier rules are measurably imprecise.
- `content.snippets` URL-fetcher path — only if `ask` needs page bodies.
- Perplexity MCP server wiring — agent-facing, zero integration work.
- Cached judge labels / judgment store — add if re-runs get expensive.

## Risks & Dependencies

- **Alpha endpoint drift** (`api/alpha/decisions`): shape-assert + soft-fail
  to labels-only metrics; pin `typesafe/jev-1.13`, not the `~latest` alias.
- **Data egress**: judge sends query+excerpt text to OpenRouter; web sends
  queries to Perplexity. Both env-gated, documented in `.env.example`.
- **Golden-set rot**: store corpus evolves → labels go stale. Mitigation:
  labeling guide + `--json` diffs across runs; refresh cadence documented.
- **Perplexity pricing** (~$5/1k req, third-party figure — confirm on docs
  pricing page before shipping U4): sufficiency gate keeps volume trivial.
- **Secrets**: `OPENROUTER_API_KEY`/`PERPLEXITY_API_KEY` via existing
  key-file + env conventions; never in config.toml, fixtures, or CI logs.

## Open Questions

- Golden set source: seed from pond session history queries vs. hand-write
  fresh — resolve at U1 implementation time (both acceptable).
- Whether `eval` should also expose an MCP tool — default no (CLI/manual).

## Sources & Research

- Verified live: `POST api/alpha/decisions` `typesafe/jev-1.13` — 0.35s,
  calibrated `noul`/`score` answers, resolved model `jev-1.13-20260917`,
  `usage.cost` ≈ $0.000019/call.
- `docs.perplexity.ai/api-reference/search-post` — request/response schema.
- `github.com/perplexityai/search_evals` — shape borrowed (system config +
  frozen tasks + grader + stored traces); retrieval metrics per standard
  offline-eval practice, not its answer-f1 approach.
- Repo: `scripts/recall-harness.py` (HTTP precedent), `tests/retrieval_hybrid.rs`
  (stub pattern), `src/synthesize|rerank|embed` (env-gated client pattern),
  `docs/eval/gate0-rewrite.md` (freeze+score template).
