---
tags: [agent-zero, scheduler, daemon, v1]
related:
  - docs/agent-zero/INDEX.md
  - docs/plans/2026-07-25-006-feat-v1-agent-zero-batch-plan.md
---

# Scheduled Background Indexing

> **Shipped contract (v1 batch #73):** config uses `[runtime] nightly_full_sync_hour` and `inactivity_threshold_hours` (not `[brain]`).  
> `GET /api/status` returns `{ ok, service, atoms, scheduler }` where `scheduler` has  
> `last_incremental_unix`, `last_full_unix`, `last_client_activity_unix`, `poll_enabled`, `watch_enabled`, `nightly_full_sync_hour`.  
> Idle skip keys off client query/ask activity (not indexing alone).  
> Per-source `next_scheduled` / dashboard graph are deferred.

**Labels:** feature, infrastructure, priority-high
**Tracking:** #7 (Phase 3) | Related: #27 (Master Plan)
**Status:** Open
**Created:** 2026-07-25

---

## Problem

Currently, kurultai requires manual indexing:
```bash
kurultai index --full
```

This creates a **stale data problem**:
- Agents search based on last-indexed knowledge
- No overnight context update (unlike Perplexity Brain)
- Developers must remember to run `index` commands
- No automatic cleanup of old/removed content

**Impact:**
- Reduced user trust: "Why doesn't kurultai know about this?"
- Higher cognitive load: remembering to reindex after changes
- Poor developer experience (CLI-only, no background task)

---

## Proposed Solution

### Background Scheduler

**Cron-like scheduling with SQLite backend:**
```
┌─────────────────────────────────────────────────┐
│ Scheduled Indexing Service (Phase 3)            │
├─────────────────────────────────────────────────┤
│ Sources with poll_interval_secs=60              │
│   → check mtime every 60s                       │
│   → if changed, run incremental sync            │
│                                                 │
│ Nightly cron (at 2am)                           │
│   → run FULL_SYNC for all sources               │
│   → update knowledge_delta in db                │
│   → trigger analytics events                   │
│                                                 │
│ Manual triggers                                  │
│   → `kurultai index --full` (on-demand)         │
│   → API: POST /api/reindex                      │
│                                                 │
│ Idle detection                                   │
│   → if source unchanged for N hours, skip       │
│   → preserve token budget (index-time heavy)    │
└─────────────────────────────────────────────────┘
```

### Configuration

```toml
# ~/.config/kurultai/config.toml
[sources.marks_notes]
enabled = true
kind = "markdown"
root_path = "~/notes"
poll_interval_secs = 60  # Check for changes every 60s

[brain]
enabled = true
nightly_update = "02:00"  # Daily full sync at 2am
check_interval_minutes = 1  # Background checker freq
inactivity_threshold_hours = 2  # Skip if no changes
```

### Scheduling Options

| Interval | Use Case | Example | Update Type |
|----------|----------|---------|-------------|
| `poll_interval_secs = 60` | Active development | Markdown vault with 100 files | Incremental (mtime check) |
| `poll_interval_secs = 300` | Team repository | Shared GitHub PR | Incremental (pull latest) |
| `nightly_update = "02:00"` | Context awareness | Project specs, docs | Full sync (0h overnight) |
| `inactivity_threshold_hours = 24` | Low-traffic | Personal notes | Skip (no changes) |

### Status API

```json
// GET /api/status
{
  "sources": [
    {
      "id": "marks_notes",
      "enabled": true,
      "poll_interval_secs": 60,
      "last_full_sync": "2026-07-25T20:42:21Z",
      "last_incremental_sync": "2026-07-25T21:30:15Z",
      "next_scheduled": "2026-07-25T22:00:00Z",
      "changed_files": 3,
      "status": "healthy"
    }
  ],
  "scheduler": {
    "enabled": true,
    "next_run": "2026-07-25T22:00:00Z",
    "next_nightly": "2026-07-26T02:00:00Z"
  }
}
```

---

## Implementation Plan

### Week 1: Scheduler Core
- [ ] Add `src/scheduler/mod.rs` with cron-like scheduler
- [ ] Add `schedules` table to schema v3
- [ ] Implement `SourcePoller`: check mtime, hash, detect changes
- [ ] Test incremental indexing under load

### Week 2: Background Worker
- [ ] Add background task runner (tokio::task)
- [ ] Run `SourcePoller` every N minutes
- [ ] Trigger `IndexPipeline` for changed sources only
- [ ] Add idle detection to skip unchanged sources

### Week 3: Nightly Full Sync
- [ ] Add cron scheduler for nightly update
- [ ] Run `full_sync` for all enabled sources
- [ ] Update `knowledge_delta` tracking
- [ ] Trigger analytics events

### Week 4: API & Dashboard Integration
- [ ] Add GET /api/status endpoint with scheduler info
- [ ] Add POST /api/reindex trigger endpoint
- [ ] Display scheduler status in dashboard
- [ ] Write integration tests

---

## Testing

**Manual Tests:**
```bash
# 1. Start kurultai with scheduler enabled
KURULTAI_SCHEDULER_ENABLED=true ./target/debug/kurultai status

# 2. Wait for scheduled check (or force incrementally)
sleep 65 && ./target/debug/kurultai index --incremental

# 3. Check next scheduled time
curl http://localhost:8421/api/status | jq '.sources[].next_scheduled'

# 4. Mock time changes to test idle detection
# (advanced: patch mtime and verify no re-index)
```

**Integration Tests:**
- [ ] Test incremental sync detects changed files
- [ ] Test idle detection skips unchanged sources
- [ ] Test nightly cron runs at correct time
- [ ] Test scheduler can be enabled/disabled via config
- [ ] Test background task survives app restart

---

## Acceptance Criteria

1. ✅ Scheduler can be enabled/disabled via config flag
2. ✅ Incremental sync detects changed files via mtime
3. ✅ Idle detection skips sources unchanged for N hours
4. ✅ Nightly cron runs at configured time (e.g. 02:00)
5. ✅ Status API shows next scheduled time for each source
6. ✅ Background worker survives app restart (in-database state)
7. ✅ Can trigger manual reindex via API or CLI
8. ✅ Knowledge delta tracking updates nightly
9. ✅ Scheduler doesn't burn token budget (index-time heavy)
10. ✅ All sources sync concurrently (not blocking)

---

## Success Metrics

- **Stale data problem:** Reduced from manual reindex to automatic overnight update
- **Developer time:** 0 extra time to maintain knowledge (background sync)
- **Token efficiency:** 90% reduction in re-indexing (only changed files)
- **Satisfaction:** 80% of users report "it just works" without manual commands
- **Adoption:** 50% of team members adopt scheduled syncing

---

## Open Questions

1. **What triggers a scheduled index?**
   - Option A: Cron (time-based only)
   - Option B: Event-based (inotify on markdown files)  ← **RECOMMENDED**
   - Decision: Hybrid - inotify for dev, cron for team overnight

2. **How do we handle concurrent updates?**
   - Option A: Lock store during index (simple but slow)
   - Option B: Batch index with upsert (faster but complex conflict resolution)
   - Decision: Batch index with optimistic concurrency

3. **Should we delete removed files from DB?**
   - Option A: Yes, clean up stale entries
   - Option B: No, keep them with deleted flag (audit trail)
   - Decision: Deleted flag + TTL cleanup (weekly)

4. **What if source is unavailable during scheduled sync?**
   - Option A: Skip source, continue others
   - Option B: Retry N times, then mark unhealthy
   - Option C: Slack/notify user
   - Decision: Retry N=3, then mark unhealthy, send webhook if configured

---

## References

- Master plan: [#27 — Work Order: Master phase plan](https://github.com/duketopceo/kurultai/issues/27)
- Phase 3 tracking: [#7 Synthesis & Interface](https://github.com/duketopceo/kurultai/issues/7)
- Perplexity Brain inspiration: [Perplexity Brain overview](https://www.perplexity.ai/help-center/en/articles/19700001-what-is-brain)
- Tech: `tokio::time::Interval` for scheduling, `chrono` for time handling