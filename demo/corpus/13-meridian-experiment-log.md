---
tags: [demo, meridian, experiment]
---

# Experiment log — ingest compression

| Run | Codec | Ratio | p99 encode (ms) | Verdict |
|-----|-------|-------|------------------|---------|
| E12 | none | 1.0x | 0.0 | baseline |
| E13 | zstd-3 | 4.1x | 1.8 | winner |
| E14 | zstd-19 | 5.6x | 14.2 | too slow at edge |
| E15 | lz4 | 2.9x | 0.4 | candidate for edge fallback |

E13 shipped: ticks are zstd-3 compressed on the rover before QUIC send.
CPU cost on the rover ARM board measured at 3% of one core — inside the
5% budget set by ADR-002. E15 is kept as the degrade-mode codec when a
rover reports thermal throttling.
