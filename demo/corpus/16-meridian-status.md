---
tags: [demo, meridian, status]
---

# Meridian — September status

Shipped: zstd-3 tick compression (E13), cohort-scoped firmware rollback,
dashboard read-replica cutover.

In flight: cold-tier parquet compaction rewrite (target 2x throughput);
edge agent memory-pressure shedding.

Blocked: nothing hard. The schema-registry v2 work waits on a decision
about whether ticks should embed schema hashes — leaning yes, ADR
pending.

Health: fleet at 412 active rovers, ingest p99 at 41 ms, zero
control-path incidents since compression shipped.
