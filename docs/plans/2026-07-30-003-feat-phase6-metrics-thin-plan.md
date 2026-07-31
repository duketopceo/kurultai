# Phase 6 P6-2 — Built-in metrics (thin slice)

**Date:** 2026-07-30  
**Issue:** [#102](https://github.com/duketopceo/kurultai/issues/102)  
**Status:** Shipped (thin slice) — full error-events / UI panel / hammer tests remain on #102  
**Out of scope this LFG:** SQLite `metrics_events`, GlitchTip, Brain UI panel, hammer/load tests, index throughput histograms.

## Goal

Daemon exposes in-process query latency histograms + counters so we can measure search/ask/graph without an external metrics stack.

## DoD

- [x] `MetricsRegistry` with fixed ms buckets; ops: search, ask, graph, cite, who_knows
- [x] Instrument HTTP handlers for those ops
- [x] `GET /api/metrics` → Prometheus text exposition
- [x] `/api/status` includes JSON metrics summary (p50/p90/p99 approx)
- [x] `kurultai status --metrics` fetches daemon `/api/metrics` (or prints unreachable)
- [x] Unit tests for registry + HTTP route
- [x] README note

## Non-goals

- Error fingerprint table / panic capture (#102 §2)
- UI metrics panel (#102 §4)
- Replacing GlitchTip deploy (#35) — still deferred; this is the local substitute for query latency only
