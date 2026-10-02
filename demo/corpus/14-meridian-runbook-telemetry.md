---
tags: [demo, meridian, runbook, incident]
---

# Runbook — telemetry pipeline stall

**Symptom:** dashboard tick age exceeds 60 s; ingest queue depth climbs.

1. Check `/health` on ingest — if down, restart and let WAL replay run
   (expect ~90 s for a full hot segment).
2. If queue depth > 10k, edges should already be in sampling mode —
   verify with `meridianctl edge mode`. If not, force it.
3. Check the schema registry: a rover firmware push with a mismatched
   tick version stalls validation. Roll the offending cohort back via
   `meridianctl fleet rollback <version>`.
4. If the stall is downstream (parquet roll), pause compaction and let
   hot drain — cold can lag safely for 24 h.

Escalate to the on-call engineer only after step 3 fails; most stalls
are schema-version mismatches from unannounced firmware pushes.
