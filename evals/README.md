# Retrieval evals

`kurultai eval` runs a frozen golden query set against a live daemon and
scores `search` / `ask` — Recall@k, Precision@k, MRR, nDCG@k (judged),
groundedness, and noise-source violations. Use it to measure ranking,
tier-policy, and sequestering changes instead of eyeballing results.

## Run

```bash
# labels-only metrics against a local daemon
kurultai eval --base-url http://127.0.0.1:8421 --k 10

# with the Jev judge (graded nDCG + groundedness; needs OpenRouter key)
OPENROUTER_API_KEY=… kurultai eval --judge --json evals/reports/run.json

# against the hosted personal brain
kurultai eval --base-url https://knowledge.shippedit.dev
```

The report prints aggregate metrics plus a JSON dump (`--json`) that records
the resolved judge model (`jev-1.13-…`), judge cost, and `KURULTAI_EVAL_GIT_SHA`
when set — keep reports when comparing before/after a retrieval change.

## The golden set (`evals/golden.json`)

- Hand-label queries from **real dogfood usage** — Hey threads, pond session
  history, deploy notes. Synthetic questions that parrot document wording
  inflate scores.
- Matchers hit `title` / `title_contains` / `source_id` /
  `source_id_contains` / `title_hash` — never atom `id` (content-hash,
  changes on edit). Atoms are section-split: prefer `source_id_contains`
  (file path) or `title_contains` over exact `title`.
- `deny_sources` asserts noise exclusion — `"pond"` should never appear.
- `kind: "ask"` + `judge_answer: true` grades groundedness when `--judge` runs.
- ~30 queries is the right starting size; refresh labels when the corpus
  changes materially (new connector, big ingest).

## The judge

`--judge` uses TypeSafe Jev via OpenRouter's decisions API
(`api/alpha/decisions`), pinned to `typesafe/jev-1.13` — the `~latest` alias
moves, and reports are only comparable within a model version. Each hit gets
a 0–3 `score` rubric (→ nDCG gains); each ask answer gets a groundedness
rubric + a correctness `noul`. ~$0.00002/call, ~0.35s per query.

Without `--judge` or a key, the run is labels-only — still useful for
recall/precision/MRR regression checks. CI runs `tests/evals_search.rs`
offline (fixture vault, no secrets, no network).

## `ask --web`

`kurultai ask "…" --web` augments thin local context with one Perplexity
`/search` call — ephemeral: web results join the answer context as
`source=web` citations (with URLs) but are **never** stored, embedded, or
counted in access/tier stats.

Gate: `KURULTAI_FEATURE_WEB_SEARCH=1` + `PERPLEXITY_API_KEY`. The sufficiency
check is Jev `noul` when a judge is configured, else a minimum-hit floor —
sufficient local context spends zero paid calls.

## Pre-merge commit review

`kurultai review <range>` grades every non-bot commit in a range with the
same Jev judge — 11 typed questions per commit (`git show` → decisions):
secrets, security risk, unauth-write / network-exposure / XSS / CSRF
surfaces, message-vs-diff match, test adequacy, follow-up needed, plus
quality and risk rubrics.

```bash
kurultai review origin/main..HEAD            # pre-merge check
kurultai review HEAD~5..HEAD --json out.json # report artifact
```

Hard flags (probability ≥ 0.5): `leaks_secret`, `security_risk`,
`unauth_write`, `network_exposure`, `injection_xss`, `csrf`, and
`matches_message` < 0.5. Any hard flag → the commit prints `FAIL` and the
command exits 1. Advisory flags (`needs_followup` ≥ 0.7, `tests_adequate`
< 0.3) print `warn` but don't fail. No OpenRouter key → skips with a
message and exits 0, so it's safe to wire into local hooks/CI that may
lack secrets. Cost is ~$0.0002/commit.
