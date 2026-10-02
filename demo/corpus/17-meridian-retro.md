---
tags: [demo, meridian, retro]
---

# Retro — the September 12 load test

We pushed the ingest pipeline to 800 synthetic rovers and it held, but
two things bent: WAL checkpointing spiked CPU at roll boundaries, and
the dashboard replica lag alarm fired at 90 s during compaction.

Went well: sampling mode engaged automatically at the 10k queue
threshold — the backpressure design did its job without operator input.

Fixes landed: checkpoint pacing (PR merged), alarm threshold moved to
5 min to match the compaction window.

Still open: whether hot segment should pre-warm the replica. Deferred —
current lag is acceptable and pre-warming adds a failure mode.
