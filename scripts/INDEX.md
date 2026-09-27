---
index: kurultai/v1
folder: scripts
parent: INDEX.md
updated: 2026-09-04
version: 2
---

# `scripts`

**Does:** Install, UI build, closeout, index audit
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`install/`](install/INDEX.md) — Install helpers

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`audit-agent-index.py`](audit-agent-index.py) | CI audit: INDEX.md coverage vs git ls-files | — | — | 2026-09-13 | 2 | 2026-09-13 skip `website/public/` — Vite public assets are cataloged in `website/INDEX.md` · 2026-08-16 indexed (v1 seed) |
| [`audit-ui.py`](audit-ui.py) | Audit built `ui/` for stale/dead assets and outdated deps | — | `website/` · `ui/` | 2026-09-04 | 1 | 2026-09-04 add dead-code/old-version UI audit · 2026-08-16 indexed (v1 seed) |
| [`recall-harness.py`](recall-harness.py) | Randomized recall/search load + correctness suite (volume, deep, same/mixed-path concurrency, failover, p50/95/99) | `/api/search` · `/api/recall` · `/api/atoms` | — | 2026-09-08 | 1 | 2026-09-08 added |
| [`build-ui.sh`](build-ui.sh) | website/ → ui/ production copy for rust-embed | — | `website/` · `ui/` | 2026-09-04 | 2 | 2026-09-04 prune legacy files and stale hashed bundles before rebuild · 2026-08-16 indexed (v1 seed) |
| [`ui-batch.mjs`](ui-batch.mjs) | Batch UI design generator — OpenRouter chat completions over `design-lab/prompts/` → gitignored `design-lab/out/` variants | OPENROUTER_API_KEY · `--model` (Kimi/GPT) | `design-lab/` | 2026-09-26 | 1 | plan 2026-09-25-001 U3 scaffold |
| [`install.ps1`](install.ps1) | Windows installer | — | — | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |
| [`install.sh`](install.sh) | Unix installer | — | — | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase-1-closeout.sh`](phase-1-closeout.sh) | Phase 1 closeout helper | — | — | 2026-07-24 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase-2-closeout.sh`](phase-2-closeout.sh) | Phase 2 closeout helper | — | — | 2026-07-24 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase-4-closeout.sh`](phase-4-closeout.sh) | Phase 4 closeout helper | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase-5-closeout.sh`](phase-5-closeout.sh) | Phase 5 closeout helper | — | — | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-26 — `ui-batch.mjs`: OpenRouter batch UI design generator (plan 2026-09-25-001 U3) — reads `design-lab/prompts/*.md`, writes `--model`/`--n` variants to gitignored `design-lab/out/`
- 2026-09-08 — `recall-harness.py`: randomized recall suite — shallow/deep queries from live corpus, volume, same-path + mixed-path concurrency, correctness probes, failover timing; console summary + `--json` report

- 2026-09-04 — add `audit-ui.py`; update `build-ui.sh` to prune stale assets
- 2026-08-16 — `audit-agent-index.py` coverage check for CI
- 2026-08-16 — indexed this folder (v1 seed)
