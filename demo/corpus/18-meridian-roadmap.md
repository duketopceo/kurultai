---
tags: [demo, meridian, roadmap]
---

# Meridian roadmap — Q4

1. **Schema registry v2** — ticks carry a schema hash; ingest validates
   per-version instead of per-cohort guesswork. Unblocks mixed-version
   fleets during rollouts.
2. **Cold compaction rewrite** — parallel parquet writers; target 2x
   roll throughput before fleet growth hits 1k rovers.
3. **Edge OTA hardening** — signed firmware bundles + staged cohort
   rollout gates. Currently the riskiest path in the system.
4. **Replay tooling** — `meridianctl replay <range>` to re-ingest a time
   window from cold storage for debugging. Frequently requested by ops.

Explicitly out: real-time control over the telemetry path (ADR-004 drew
that line and it holds).
