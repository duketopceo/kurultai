---
tags: [demo, meridian, adr, decision]
---

# ADR-004: tick batching over per-sample streaming

**Status:** accepted · **Date:** 2026-08-14

## Context

Early prototypes streamed every sensor sample as its own message.
Throughput collapsed at ~40 rovers: per-message framing dominated, and
the ingest validator became the bottleneck.

## Decision

Batch samples into fixed 250 ms ticks at the edge. Each tick is a single
schema-validated unit: one frame, one checksum, one insert.

## Consequences

- Sustained 400+ rovers on a single ingest node (10x headroom).
- Worst-case added latency is bounded at 250 ms — acceptable for
  telemetry, unacceptable for control (control loops stay on-rover).
- Schema evolution must version the whole tick, not individual samples.
