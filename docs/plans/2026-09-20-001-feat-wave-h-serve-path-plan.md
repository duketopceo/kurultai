# Wave H + client perf telemetry — phase plan

**Date:** 2026-09-20 · **Status:** active · **Milestone:** Wave H serve-path hardening
**Parent:** `phase-6-next-work-orders.md` (Wave H section) · **Trigger:** "lfg next phase" + user ask to track load times / lag / FPS for optimization work.

## Ordering

Instrument before optimizing — felt-performance baselines first, then serve-path work measured against them.

| Seq | Item | Issue | Size | Owner |
|----:|------|-------|------|-------|
| 1 | **Watcher min-interval floor** — `watch_session` debounces bursts (300ms) but has no floor between cycles; a sustained inotify stream ran 2,000 index cycles / 46 min at ~824% CPU. Add `WATCH_MIN_INTERVAL` between `run_poll_cycle` calls. | (bug, unnumbered) | S | Devin |
| 2 | **Client perf telemetry** — browser reports felt perf to the daemon: nav/tier-load ms (per `LOAD_TIER_CAPS` tier), FPS EMA + long-task count (PerformanceObserver), heap where exposed. `POST /api/metrics/client` batch + `client_*` ops in `MetricsRegistry` → `/api/metrics` + `/api/status`. Numbers + enum labels only. | #102 | M | Devin |
| 3 | **#325 declarative tier policy** — hot/medium/cold membership as data, not call-site code. Unblocks #324 shape. | #325 | M | A0 (server-001) — design + first cut |
| 4 | **#324 prepared hot-tier payload** — publish-on-mutation serve records for Brain loads; serve byte-for-byte. Needs #325's tier definition + step-2 baselines to prove the win. | #324 | L | TBD after 2+3 |
| 5 | **#323 embed/chunk versioning** · **#326 durable reindex outbox** | #323/#326 | L | later |

## Telemetry design (step 2)

- **Questions it answers:** which tier loads slow (p50/p99 per tier)? does FPS degrade with node count? long tasks during interaction? regression after each deploy?
- **Client sample:** `{nav_ms, tier_load_ms{tier,nodes,ms}, fps_ema, long_tasks, heap_mb?, viewport, tier}` — batched, sent on tier change + every ~60s, `sendBeacon` on unload.
- **Server:** `POST /api/metrics/client` → `MetricsRegistry.observe_client(metric, ms)`; reuses histogram machinery; exposed as `kurultai_client_*` series + `client` block in `summary_json`.
- **Cardinality:** labels are enum only (`metric`, `tier`). No node ids, no queries, no URLs.

## Out of scope this phase

- GlitchTip/external APM (#102's deep half) — in-process histograms suffice at solo scale.
- Atlas projections (#128–129), connector sprawl — post-v1.
