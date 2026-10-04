---
title: Competitive sweep complete — phase wrap-up
tags:
  - competitive-sweep
  - complete
  - docs
---

# Competitive sweep — phase wrap-up

**Status:** ✅ All five units merged to `main` (2026-10-01 → 2026-10-04)
**Plan:** [2026-10-01-001-feat-competitive-feature-sweep-plan.md](2026-10-01-001-feat-competitive-feature-sweep-plan.md)
**Origin:** competitor scan — GBrain (`garrytan/gbrain`), OpenViking, mem0 / Zep-Graphiti / Letta / Cognee / Supermemory + the pplx-embed release.

## Units shipped

| # | Feature | Stolen from | PR | Landing |
|---|---------|-------------|----|---------|
| U1 | Perplexity embedding backend (`embed.backend = "perplexity"`) | Perplexity `pplx-embed-v1` release | [#400](https://github.com/duketopceo/kurultai/pull/400) | `src/embed/perplexity.rs` — int8-base64 → f32 decode; Matryoshka `dimensions` via `embed.dimension`; fails loud on missing key |
| U2 | Gap-aware `ask` | gbrain gap analysis | `2f428be` | `Answer.gaps` (zero/thin/quarantine-only coverage + unmatched terms), both ask paths + CLI |
| U3 | Zero-LLM typed edge extraction | gbrain self-wiring graph | [#403](https://github.com/duketopceo/kurultai/pull/403) | `ontology/extract.rs` — `[[links]]` / `@mentions` / frontmatter rels → `references` edges + stubs at index time; 363 edges / 310 entities on demo corpus |
| U4 | Supersede / bi-temporal-lite | gbrain + Graphiti | [#405](https://github.com/duketopceo/kurultai/pull/405) | `supersedes:` frontmatter → `superseded_at/by` (schema v17); excluded from default search/ask; `--as-of` + `--include-superseded`; `fm_*` metadata preserves stripped frontmatter |
| U5 | Consolidation sweep | gbrain + Letta sleep-time | [#407](https://github.com/duketopceo/kurultai/pull/407) | `src/sweep.rs` + `kurultai sweep [--dry-run]`; content-hash dedupe → supersede, `prune_stale_ontology`, tier/quarantine census → Hey `kurultai-sweep` thread; runs after nightly full sync |

## Supporting work landed this phase

- Demo corpus 6 → 235 atoms ([#391](https://github.com/duketopceo/kurultai/pull/391)) — seeded generator, self-indexing `gen/INDEX.md`
- Canonical OpenRouter attribution (`HTTP-Referer` + `X-Title`) on every call site ([#404](https://github.com/duketopceo/kurultai/pull/404))
- `ui/` embed determinism fix — stamp = deterministic subtree SHA ([#402](https://github.com/duketopceo/kurultai/pull/402))
- Deploy hardening + default route → new UI (#386, #387, #388)
- `remote_ingest` feature flag (#406); Meridian demo-seed corpus (parallel PR)

## Verified live

- pplx-embed: real API, 208 vectors @ dim 1024 in `atoms_vec` (dev store)
- Extraction: 363 `references` edges / 310 instance entities on demo reindex
- Supersede: default/`--include-superseded`/`--as-of` windows all correct
- Sweep: demo pass posts to Hey; census correct (220 hot / 15 quarantined)
- 361 lib tests green; `clippy --all-targets --all-features -D warnings` clean; agent-index audit green

## Deferred (deliberate, from the plan)

- `pplx-embed-context-v1` — not exposed on the embeddings endpoint (GA models only)
- ONNX local 0.6B — viable privacy/offline lane, not urgent with API working
- Connector permission tiers → belongs to broker seat-token plan
- Multi-brain mounts / `viking://` browse → bigger design, later

## Carry-over into next phase

1. **Hosted pplx flip** — `PERPLEXITY_API_KEY` + `embed.backend` on server-001, rebuild + full reindex (dim 3072→1024 rebuilds `atoms_vec`). SSH access restored 2026-10-03.
2. **Ontology 2D board** — U3 gave it real typed edges to render.
3. **UI heavy rewrite** — `design-lab/` + `ui-batch.mjs` batch-generation track.
4. **Scheduled sweep enablement** — set `nightly_full_sync_hour` on hosted configs so U5 actually fires.
5. Hygiene: ~1.7k `hammer-test*` messages on hosted; Hey admin `turn_cap` bypass gap documented.
