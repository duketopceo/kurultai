## Residual Review Findings

Source run: `/tmp/ce-code-review/trust-lanes-20260725-200242` (correctness, security, testing, data-migration).
Branch: `feature/quality-labeling-trust-lanes` · Plan: `docs/plans/2026-07-25-011-feat-quality-labeling-trust-lanes-plan.md`

### Applied in working tree (step 5)

- Prefer `TrustLane::Trusted` survivor in near-dupe merge (`src/quality/merge.rs`)
- Atomic `apply_auto_merge` transaction (`src/store/mod.rs` + near_dupe)
- Migration v4 idempotent column adds via `add_column_if_missing` (`src/store/migrations.rs`)
- `TrustLane::parse` fail-closed to quarantine (`src/types.rs`)
- In-batch gate tracks all seen hashes (`src/pipeline/mod.rs`)
- `AgentAtomView` exposes `trust_lane` + `quarantine_reason` (`src/brain/mod.rs`)
- U1/U3/U4/U6-oriented store + HTTP tests; `/api/status` lane_counts error propagation

### Not applied (residual — auth posture / product)

- **P0** `src/http/mod.rs` — `POST /api/promote` has no authentication. Module documents localhost-only bind for this slice. Owner: human. Autofix: manual.
- **P1** `src/mcp/server.rs` — MCP `promote` has no capability token beyond process-local stdio. Owner: human. Autofix: manual.
- **P2** Connector re-index may flip quarantine→trusted via gate without promote audit (stable `source_id`). Deferred follow-up; KTD7 says remember never promotes — connector path is gate re-eval on upsert.

### Tracker

- no_sink: residual record file is the durable sink (tracker not filed this run).
