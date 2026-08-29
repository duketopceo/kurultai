---
index: kurultai/v1
folder: tests
parent: INDEX.md
updated: 2026-08-29
version: 2
---

# `tests`

**Does:** Rust integration + acceptance tests
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`fixtures/`](fixtures/INDEX.md) — Ingest corpora (do not add INDEX.md inside vault/code_repo)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`acceptance_concurrency.rs`](acceptance_concurrency.rs) | Acceptance tests — multi-process SQLite safety (Track A: shared crew brain). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`acceptance_http.rs`](acceptance_http.rs) | Acceptance tests — HTTP API surface (KHAN-251). | — | — | 2026-08-29 | 2 | 2026-08-29 unprefixed alias auth tests · 2026-08-16 indexed (v1 seed) |
| [`acceptance_ingest.rs`](acceptance_ingest.rs) | Acceptance tests — ingest surface (KHAN-251). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`acceptance_mcp.rs`](acceptance_mcp.rs) | Acceptance tests — MCP tool surface (KHAN-251). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`acceptance_ontology.rs`](acceptance_ontology.rs) | Acceptance tests — ontology O1 (KHAN-251). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`acceptance_search.rs`](acceptance_search.rs) | Acceptance tests — search surface (KHAN-251). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`acceptance_visibility.rs`](acceptance_visibility.rs) | Acceptance tests — visibility / tiered access (KHAN-251). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`acceptance_write_policy.rs`](acceptance_write_policy.rs) | Acceptance tests — shared-store write containment (Track A / A2). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`cli_smoke.rs`](cli_smoke.rs) | Phase 1 CLI smoke (#5 / #23) — binary against fixture vault. | — | — | 2026-08-29 | 2 | 2026-08-29 unset ambient hub flag in bin() · 2026-08-16 indexed (v1 seed) |
| [`inbox_adapter_test.rs`](inbox_adapter_test.rs) | Inbox tray + dump format parity + gate heuristics (AE1–AE7). | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |
| [`install_script_test.rs`](install_script_test.rs) | Personal installer script smoke (#72) — bash syntax + dry-run / help. | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`json_ingestion_test.rs`](json_ingestion_test.rs) | Integration tests for U1.1–U1.3: schema migration v006, ingestion_jobs store | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase3_ask_test.rs`](phase3_ask_test.rs) | Phase 3 integration: extractive ask + who_knows on fixture vault. | — | — | 2026-07-23 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase4_connectors_test.rs`](phase4_connectors_test.rs) | Phase 4: Dayflow fixture index → FTS hit. | — | — | 2026-08-12 | 1 | 2026-08-16 indexed (v1 seed) |
| [`phase5_daemon_test.rs`](phase5_daemon_test.rs) | Phase 5: daemon poll_once indexes without full sync. | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`retrieval_hybrid.rs`](retrieval_hybrid.rs) | Phase 2 (#23) hybrid retrieval integration — FTS ∥ vector → RRF → stub rerank. | — | — | 2026-08-14 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-08-29 — acceptance tests for unprefixed query alias auth under `HubAuth::ApiKey`
- 2026-08-29 — CLI smoke unsets ambient `KURULTAI_FEATURE_HUB` (Postgres CI job)
- 2026-08-16 — indexed this folder (v1 seed)

