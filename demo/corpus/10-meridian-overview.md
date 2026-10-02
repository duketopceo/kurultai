---
tags: [demo, meridian, overview]
---

# Project Meridian — overview

Meridian is a fictional fleet-telemetry platform used to seed this demo
brain. It ingests sensor readings from autonomous rovers, normalizes them
into a canonical tick format, and serves them to a mission dashboard.

The project exists only as fixture data — names, numbers, and decisions
are synthetic but internally consistent, so search, who-knows, and the
brain graph behave the way they would on a real knowledge store.

Components: `meridian-edge` (on-rover agent), `meridian-ingest`
(tick batching + validation), `meridian-dash` (operator dashboard),
`meridian-core` (shared schema crate).
