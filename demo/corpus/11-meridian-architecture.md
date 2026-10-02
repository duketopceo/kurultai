---
tags: [demo, meridian, architecture]
---

# Meridian architecture

Rovers run `meridian-edge`, which batches sensor samples into 250 ms
ticks and ships them over QUIC to `meridian-ingest`. Ingest validates
each tick against the `meridian-core` schema, writes it to the hot
segment (SQLite WAL), and hands off to cold parquet on a 6-hour roll.

`meridian-dash` queries a read replica of the hot segment; it never
touches ingest directly. Backpressure rule: if the ingest queue depth
exceeds 10k ticks, edge agents drop to sampling mode rather than
blocking rover control loops.

Failure domains are deliberately separate: a dashboard outage cannot
stall ingestion, and an ingest stall degrades capture rate but never
rover safety.
