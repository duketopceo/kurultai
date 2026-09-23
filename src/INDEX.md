---
index: kurultai/v1
folder: src
parent: INDEX.md
updated: 2026-09-16
version: 7
---

# `src`

**Does:** Rust CLI + daemon (main product)
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`app/`](app/INDEX.md) — CLI application orchestration
- [`brain/`](brain/INDEX.md) — Agent read-model + graph doctrine
- [`config/`](config/INDEX.md) — config.toml load/merge
- [`connectors/`](connectors/INDEX.md) — Source ingest adapters
- [`daemon/`](daemon/INDEX.md) — Poll + fs watch
- [`distill/`](distill/INDEX.md) — Index-time distill
- [`embed/`](embed/INDEX.md) — Embeddings (cloud / local / null)
- [`export/`](export/INDEX.md) — Export/import packs
- [`hub/`](hub/INDEX.md) — HUB-4 issued keys + write activity (postgres)
- [`http/`](http/INDEX.md) — Daemon HTTP + Brain UI + MCP SSE
- [`ingest/`](ingest/INDEX.md) — Ingest jobs
- [`mcp/`](mcp/INDEX.md) — MCP stdio + agent init
- [`memory/`](memory/INDEX.md) — Memory / tier helpers
- [`ontology/`](ontology/INDEX.md) — Typed property graph helpers
- [`pipeline/`](pipeline/INDEX.md) — Index pipeline
- [`quality/`](quality/INDEX.md) — Trust lanes, promote, near-dupe
- [`query/`](query/INDEX.md) — FTS/hybrid search + RRF
- [`rerank/`](rerank/INDEX.md) — Optional rerank
- [`security/`](security/INDEX.md) — Paths, redaction, hub keys
- [`store/`](store/INDEX.md) — SQLite kernel + optional Postgres hub store
- [`synthesize/`](synthesize/INDEX.md) — ask / who-knows
- [`eval/`](eval/INDEX.md) — retrieval eval runner + metrics + Jev judge
- [`web/`](web/INDEX.md) — ephemeral Perplexity search (`ask --web`)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`activity.rs`](activity.rs) | In-memory activity events for Brain UI / status | — | `src/mcp/brain.rs` | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`art.rs`](art.rs) | Yurt terminal banner art | — | `src/config/file.rs` · `src/config/loader.rs` | 2026-07-26 | 1 | 2026-08-16 indexed (v1 seed) |
| [`doctor.rs`](doctor.rs) | kurultai doctor PASS/FAIL/WARN diagnostics | `src/config` · `src/embed` · `src/environment` · `src/error` · `src/mcp` | — | 2026-08-13 | 1 | 2026-08-16 indexed (v1 seed) |
| [`environment.rs`](environment.rs) | KURULTAI_ENV paths (dev/staging/prod store locations) | `src/error` | `src/app/context.rs` · `src/config/loader.rs` · `src/doctor.rs` · `src/logging.rs` · `src/types.rs` | 2026-08-01 | 1 | 2026-08-16 indexed (v1 seed) |
| [`error.rs`](error.rs) | KurultaiError and Result | — | `src/app/context.rs` · `src/config/loader.rs` · `src/config/mod.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` | 2026-07-18 | 1 | 2026-08-16 indexed (v1 seed) |
| [`features.rs`](features.rs) | Versioned feature flags (fts, brain_ui, mcp_http, hub, web_search) | — | — | 2026-09-17 | 3 | 2026-09-17 `web_search` flag (ask --web) · 2026-08-29 hub summary names HUB-3 transport · 2026-08-16 indexed (v1 seed) |
| [`hashutil.rs`](hashutil.rs) | Content hashing for incremental index skip | — | `src/connectors/dayflow.rs` · `src/connectors/github.rs` · `src/connectors/pond.rs` · `src/http/auth.rs` · `src/http/ingest.rs` | 2026-07-21 | 1 | 2026-08-16 indexed (v1 seed) |
| [`lib.rs`](lib.rs) | Crate root: module graph and public error/env re-exports | — | — | 2026-09-17 | 5 | 2026-09-17 `eval` + `web` module exports · 2026-09-16 `connect` module export · 2026-09-15 `webui` module export (#329) |
| [`logging.rs`](logging.rs) | tracing-subscriber setup | `src/environment` · `src/error` | — | 2026-07-21 | 1 | 2026-08-16 indexed (v1 seed) |
| [`connect.rs`](connect.rs) | `kurultai connect` device-authorization CLI — code → poll → omaseal/key-file → `wire_agent`; persistent seat-id + seat-scoped credential names | `src/error` · `src/mcp` · `src/security` · `src/webui` · `reqwest` | — | 2026-09-19 | 2 | 2026-09-19 seat file `<config>/seat-id` (`{hostname}-{rand}`) + `{lane}-{codename}-{seat}-agent-token` cred names · 2026-09-16 added |
| [`login.rs`](login.rs) | `kurultai login` device-code flow for hosted agent tokens | `src/error` · `reqwest` · `dirs` | — | 2026-09-06 | 1 | 2026-09-06 added |
| [`main.rs`](main.rs) |  CLI entry: init, index, search, ask [--web], eval, daemon, mcp, connect, login, export  | — | — | 2026-09-18 | 12 | 2026-09-18 judge_from_config in brain_from_app + `status` judge line · 2026-09-18 `review` cmd — pre-merge Jev commit gate, exit 1 on hard flags · 2026-09-18 print judge-disabled reason in eval output · 2026-09-17 `eval` cmd + `ask --web` + brain_from_app web/judge wiring · 2026-09-16 `connect` cmd + `agent revoke` · 2026-09-15 `webui` cmd + init key prompt + `daemon --bind` (#329) |
| [`metrics.rs`](metrics.rs) | Prometheus text for GET /api/metrics + client-perf sample store | — | `src/http/mod.rs` | 2026-09-20 | 2 | 2026-09-20 client family: per-metric bounds (fps vs ms), `observe_client` + `ClientReport` for POST /api/metrics/client (#102) · 2026-08-16 indexed (v1 seed) |
| [`project.rs`](project.rs) | project_id namespacing for shared-store sessions (#184) | — | `src/mcp/server.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`types.rs`](types.rs) | KnowledgeAtom, Config, search/ask types, visibility scope, OntologyProposal | `src/environment` | `src/brain/mod.rs` · `src/config/loader.rs` · `src/config/mod.rs` · `src/connectors/appflowy.rs` · `src/connectors/dayflow.rs` · `src/pipeline/mod.rs` · `tests/acceptance_visibility.rs` · `src/ontology/mod.rs` | 2026-09-18 | 3 | 2026-09-18 `Config.judge_enabled`/`judge_model` ([judge]) · 2026-09-15 `Config.tier_policy` runtime field (#325) |
| [`webui.rs`](webui.rs) | `kurultai webui` — probe/spawn daemon, print URL, open browser | `src/error` | `src/main.rs` · `src/connect.rs` | 2026-09-16 | 2 | 2026-09-16 `open_browser` pub(crate) for `connect` · 2026-09-15 added (#329) |
| [`write_policy.rs`](write_policy.rs) | Write provenance + SharedClosed quarantine containment | — | `src/mcp/server.rs` · `src/quality/promote.rs` | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-09-23 — prepared `/api/graph` payload (#324): `Store::atom_epoch` mutation counter + `http::graph_cache` serve param-keyed snapshots byte-for-byte; Postgres hub (untrackable epoch) assembles live
- 2026-09-20 — `metrics.rs` gains the client family (browser-reported nav/tier-load/fps/long-task/heap samples → `kurultai_client_*` series + `metrics.client` in `/api/status`); `daemon/mod.rs` gains `WATCH_MIN_INTERVAL` (30s floor between watch-triggered index cycles — sustained inotify streams could no longer hot-loop)
- 2026-09-18 — `eval/review.rs` + `Commands::Review`: pre-merge Jev commit gate (secrets/security/mismatch → exit 1)
- 2026-09-18 — `eval/mod.rs` judge circuit breaker (dead key / insufficient credits → labels-only, flagged); `main.rs` prints disable reason
- 2026-09-17 — `eval/` (runner + metrics + Jev judge), `web/` (Perplexity searcher), `brain.rs` `ask_with_web` + sufficiency gate, `main.rs` `eval` cmd + `ask --web`, `features.rs` `web_search` flag
- 2026-09-16 — `connect.rs` + `main.rs`: `kurultai connect <url>` (RFC 8628 device flow, omaseal/0600 key storage, `wire_agent` reuse) and `agent revoke [--instance-id]`
- 2026-09-15 — `main.rs`/`webui.rs`: `kurultai webui`, init `--key/--key-file/--no-key`, `daemon --bind` (#329)
- 2026-09-15 — `types.rs`: `Config.tier_policy` (serde-skipped, loader-populated) (#325)
- 2026-09-15 — `main.rs`: `brain_from_app` applies `config.tier_policy` (#325)
- 2026-09-15 — board v2 deletes (#320): store `delete_ontology_entity`/`delete_ontology_link`, DELETE routes, write guard covers DELETE
- 2026-09-14 — ontology board human write lane (#316): `http/ontology_write.rs` routes, `ontology/create_entity`+`create_link`, WRITE_ROUTES + agent refusal
- 2026-09-11 — O3 ontology proposal queue (#118): `types.rs` OntologyProposal; `store/` schema v15 + proposal CRUD; `ontology/` submit/decide; `http/proposals.rs` REST; `mcp/server.rs` propose tools
- 2026-09-06 — add `login.rs` and `kurultai login` subcommand; `http/device_auth.rs` device-code endpoints; schema v14
- 2026-09-04 — `http/`, `mcp/`, `ontology/`: message board REST, hey_* MCP tools, schema v12 sync, HubGate fixture updates
- 2026-09-01 — `types.rs`: HUB-5 `SourceConfig::default_visibility_scope` helper (personal/team/company)
- 2026-08-31 — `main.rs`: `init --doctor` diagnostic toggle reuses `doctor` spine
- 2026-08-31 — review fixes: auth DB 500s, reason length, team atom validation, shared hub DDL
- 2026-08-31 — HUB-4: ensure hub_activity table migrates alongside hub_api_keys
- 2026-08-31 — HUB-4: `src/hub/` issued keys, AE5 filter, write activity log
- 2026-08-29 — HUB-3: listen start-fail, hub Postgres when flag on, `PORT` env
- 2026-08-16 — indexed this folder (v1 seed)
