---
tags: [demo, meridian, glossary]
---

# Meridian glossary

- **tick** — a 250 ms batch of sensor samples; the atomic unit of
  ingestion. Everything downstream reasons in ticks, never samples.
- **hot segment** — the SQLite WAL region holding the last 6 hours of
  ticks; serves all dashboard reads.
- **sampling mode** — degraded edge behavior under backpressure: send
  every Nth tick instead of all ticks.
- **cohort** — a group of rovers pinned to the same firmware version.
  Schema mismatches are always cohort-scoped.
- **mission clock** — monotonic rover time; wall-clock time is
  untrusted (rovers lose NTP in the field constantly).
