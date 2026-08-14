# Kurultai Feature Matrix (KHAN-251)

Comprehensive inventory of every feature the acceptance/integration suite
covers. Each row maps to one or more tests under `tests/acceptance_*.rs`.

Status legend:

- ✅ **working** — exercised by a passing acceptance test.
- ⚠️ **partial** — core path works but a documented gap remains (see
  `ACCEPTANCE_REPORT.md`).
- ❌ **broken** — declared in the milestone but not implemented / not
  wired; covered by an `#[ignore]`'d test plus a report entry.

## A0 — Original features

| # | Feature | Module | Test file | Status |
|---|---------|--------|-----------|--------|
| A0-1 | Markdown ingest (frontmatter + heading chunks) | `connectors/markdown`, `ingest/dump` | `acceptance_ingest.rs` | ✅ |
| A0-2 | FTS5 full-text search (bm25 + stopword sanitization) | `store/mod` | `acceptance_search.rs` | ✅ |
| A0-3 | Vector search (sqlite-vec kNN) | `store/mod` | `acceptance_search.rs` | ✅ |
| A0-4 | Hybrid search (FTS + vector RRF) | `query/hybrid` | `acceptance_search.rs` | ✅ |
| A0-5 | Ask (RAG — extractive synthesis + citations) | `synthesize`, `mcp/brain` | `acceptance_mcp.rs`, `acceptance_http.rs` | ✅ |
| A0-6 | MCP tool `search` | `mcp/server` | `acceptance_mcp.rs` | ✅ |
| A0-7 | MCP tool `cite` | `mcp/server`, `mcp/brain` | `acceptance_mcp.rs` | ✅ |
| A0-8 | MCP tool `remember` (agent write path) | `mcp/brain` | `acceptance_mcp.rs` | ✅ |
| A0-9 | MCP tool `ask` | `mcp/server` | `acceptance_mcp.rs` | ✅ |
| A0-10 | MCP tool `who_knows` | `mcp/brain`, `synthesize` | `acceptance_mcp.rs` | ✅ |
| A0-11 | MCP tool `promote` (quarantine → trusted) | `quality/promote` | `acceptance_mcp.rs` | ✅ |
| A0-12 | HTTP API (search/ask/cite/who_knows/atoms/graph/status) | `http/mod` | `acceptance_http.rs` | ✅ |
| A0-13 | Brain UI (`GET /ui`) | `http/ui` | `acceptance_http.rs` | ✅ |
| A0-14 | MCP stdio JSON-RPC server | `mcp/server` | `acceptance_mcp.rs` | ✅ |
| A0-15 | Quality gate (trust lanes: trusted / quarantine) | `quality/gate` | `acceptance_ingest.rs`, `acceptance_visibility.rs` | ✅ |

## Tiered Access milestone — added features

| # | Feature | Module | Test file | Status |
|---|---------|--------|-----------|--------|
| TA-1 | Visibility scope `personal` / `team` / `company` (round-trips in store) | `types`, `store/mod` | `acceptance_visibility.rs` | ✅ |
| TA-2 | Corpus tiers `public` / `private` (type + parse) | `types` | `acceptance_visibility.rs` | ✅ |
| TA-3 | Corpus tier persistence in SQLite store | `store/mod`, `store/postgres` | `acceptance_visibility.rs` | ✅ |
| TA-4 | KTD15 visibility labels (per-document) | `store/mod`, `store/postgres` | `acceptance_visibility.rs` | ✅ |
| TA-5 | `SourceConfig::default_corpus_tier()` / `default_visibility_labels()` | `types`, `pipeline/mod` | `acceptance_visibility.rs` | ✅ |
| TA-6 | Hashtag-line ingest (whitespace `#tag` lines) | `ingest/dump` | `acceptance_ingest.rs` | ✅ |
| TA-7 | Project-scoped recall API (`POST /api/recall`, `recall_for_agent`) | `http/mod`, `mcp/brain` | `acceptance_search.rs`, `acceptance_http.rs` | ✅ |
| TA-8 | Hub API-key auth middleware (`HubGate`, `hub_api_auth`) | `http/auth` | `acceptance_http.rs` | ✅ |
| TA-9 | Config-not-code adapters: inbox connector | `connectors/inbox` | `acceptance_ingest.rs` | ✅ |
| TA-10 | Config-not-code adapters: loopback `POST /ingest` | `http/ingest` | `acceptance_ingest.rs`, `acceptance_http.rs` | ✅ |
| TA-11 | Postgres + pgvector hub store (`open_hub_store`) | `store/postgres` | `acceptance_visibility.rs` (feature-gated note) | ⚠️ |
| TA-12 | Ontology O1 entities (class / instance / metric) | `ontology`, `store/mod` | `acceptance_ontology.rs` | ✅ |
| TA-13 | Ontology O1 typed links (`is_a`, `instance_of`, …) | `types`, `store/mod` | `acceptance_ontology.rs` | ✅ |
| TA-14 | MCP tool `ontology_get` | `mcp/server` | `acceptance_mcp.rs` | ✅ |
| TA-15 | MCP tool `ontology_promote` (atom → instance entity) | `mcp/server`, `ontology` | `acceptance_mcp.rs` | ✅ |
| TA-16 | Seeded class tree (memory, note, code, decision, person, system) | `ontology`, `store/migrations` | `acceptance_ontology.rs` | ✅ |

## Summary

_Last verified against HEAD (`110f371`, PR #217) — see `ACCEPTANCE_REPORT.md` for detail._

- **Working features exercised by passing tests:** 30
- **Partial features:** 1 (Postgres/pgvector hub store gated behind a
  compile-time feature flag + runtime env toggle — deliberate, by design,
  not a bug)
- **Broken / missing features:** 0

Corpus-tier persistence, visibility-labels persistence, hashtag-line ingest,
and SourceConfig tier/label defaults at ingest (TA-2 through TA-6) were all
fixed in commit `110f371` (#217), which landed after the acceptance suite in
`7cdeb36` (#216) first surfaced them as broken. See `ACCEPTANCE_REPORT.md`
for the broken-feature detail as it stood before that fix, and the "Fixed"
notes added to each section for where the fix actually lives.
