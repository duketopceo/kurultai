---
tags: [agent-zero, dashboard, http, v1]
related:
  - docs/agent-zero/INDEX.md
  - docs/plans/2026-07-25-006-feat-v1-agent-zero-batch-plan.md
---
# PR: Dev Dashboard (HTTP Daemon + WebSocket)

**Tracking:** #42 (Phase 1 complete)
**Master Plan:** [#27](https://github.com/duketopceo/kurultai/issues/27)
**Audience:** Developer → Solo → Team
**Complexity:** High (Phase 3 component)

---

## Goal

Add a local dev dashboard for Kurultai with:
- Live knowledge graph visualization
- Search results history with citations
- Settings and API key management
- MCP connection status
- Knowledge growth timeline

**Inspired by:** Perplexity Brain UI and Computer dashboard.

---

## Problem

Currently, kurultai is CLI-only for developers:
- No way to see what the brain knows (stale data problem)
- No dashboard to monitor activity
- Can't verify what was indexed or search performance
- No admin interface for MCP connections

**Impact:** Harder to debug, iterate, and convince team members to adopt.

---

## Proposed Solution

### Architecture

**HTTP Daemon on port 8421:**

```text
┌─────────────────────────────────────────────────┐
│ Kurultai HTTP Daemon (v1 #76 slice)             │
├─────────────────────────────────────────────────┤
│ TCP: 8421                                       │
│ Proto: HTTP/1.1 (WebSocket deferred)            │
│ Auth: None (local dev)                          │
│ Routes (shipped):                               │
│   GET  /health            - liveness            │
│   GET  /api/status        - atoms + scheduler   │
│   GET  /api/search        - search              │
│   GET  /ui                - dashboard HTML      │
└─────────────────────────────────────────────────┘
```

### UI Components

1. **Dashboard Home:**
   - Knowledge stats (total atoms, sources, update cadence)
   - Recent activity timeline
   - Quick search bar
   - MCP connections status

2. **Knowledge Graph:**
   - Nodes = documents/sources
   - Edges = relationships (via embeddings or manual tagging)
   - Click node → view associated atoms

3. **Search History:**
   - Last 100 queries with timestamps
   - Citations for each result
   - Performance metrics (ms latency)

4. **Settings:**
   - Source enable/disable
   - Update interval configuration
   - Embedder/Reranker settings
   - API keys management

### API Specification

```json
// GET /api/status
{
  "environment": "dev",
  "storage": {"path": "/root/.local/share/kurultai/dev/store.db", "schema": 2},
  "embedder": {"name": "openrouter/text-embedding-3-large", "dimension": 3072},
  "reranker": {"name": "none", "disabled": true},
  "sources": [
    {
      "id": "notes",
      "kind": "markdown",
      "enabled": true,
      "atoms": 424,
      "last_sync": "2026-07-25T20:42:21Z",
      "poll_interval_secs": 60
    }
  ],
  "mcp": [
    {"agent": "cursor", "connected": true, "file": "/root/.cursor/mcp.json"}
  ]
}
```

```json
// GET /api/knowledge
{
  "total_atoms": 424,
  "total_sources": 1,
  "by_source": [...],
  "knowledge_delta": {
    "added_today": 42,
    "added_yesterday": 128,
    "total_lifetime": 1247
  }
}
```

```json
// GET /api/search
[
  {
    "id": "evt_abc123",
    "timestamp": "2026-07-25T20:42:31Z",
    "query": "database migration",
    "results": [
      {
        "source_id": "notes",
        "file_path": "ops/deploy.md",
        "title": "Deploy Guide — Database migration",
        "excerpt": "Always run the database migration scripts before cutting traffic.",
        "snippet": "[ops/deploy.md > Deploy Guide > Database migration]",
        "rank": 1,
        "latency_ms": 16
      }
    ]
  }
]
```

```json
// WS /ws/events - Live Activity Stream
{
  "type": "index_complete",
  "source": "notes",
  "fetched": 4,
  "indexed": 4,
  "duration_ms": 71,
  "timestamp": "2026-07-25T20:42:21Z"
}

{
  "type": "search",
  "query": "database migration",
  "results_count": 2,
  "latency_ms": 16,
  "timestamp": "2026-07-25T20:42:31Z"
}

{
  "type": "mcp_connect",
  "agent": "cursor",
  "file": "/root/.cursor/mcp.json",
  "timestamp": "2026-07-25T20:41:48Z"
}
```

---

## Implementation Plan

### Phase 3: Dev Dashboard (High Priority)

**Week 1: HTTP Server Foundation**
- [ ] Add `src/http/mod.rs` with Actix-web (or Warp)
- [ ] Implement `/api/status`, `/api/knowledge`, `/api/search` routes
- [ ] Add liveness probe `/api/health`
- [ ] Test all endpoints with curl/Postman

**Week 2: WebSocket & Live Updates**
- [ ] Add WebSocket endpoint `/ws/events`
- [ ] Create event emitter in `src/app/context.rs`
- [ ] Push index events, search events, MCP events
- [ ] Frontend: simple HTML+JS client for demo

**Week 3: Dashboard UI**
- [ ] Create HTML templates in `templates/` (simple React or vanilla JS)
- [ ] Implement Knowledge Graph visualization (D3.js or Plotly)
- [ ] Add Search History with pagination
- [ ] Settings form for source management

**Week 4: Polish & Tests**
- [ ] Add authentication for team mode (Basic Auth)
- [ ] Write integration tests for HTTP server
- [ ] Add performance metrics endpoint
- [ ] Documentation updates

### Out of Scope (Later Phases)

- [ ] RBAC for team/company (Phase 4+)
- [ ] Multi-tenant database (PostgreSQL instead of SQLite)
- [ ] Admin portal UI (Phase 5+)

---

## Testing

**Manual Tests:**

```bash
# 1. Start HTTP daemon
./target/debug/kurultai daemon --port 8421

# 2. Test status endpoint
curl http://localhost:8421/api/status

# 3. Open dashboard
open http://127.0.0.1:8421/ui
```

**Integration Tests:**
- [ ] Test `/api/status` and `/api/search` with valid/invalid input
- [ ] Test daemon restart and recovery
- [ ] WebSocket event streaming — deferred beyond v1 #76

---

## Acceptance Criteria

1. ✅ HTTP daemon starts on port 8421 without errors
2. ✅ `/api/status` returns accurate system state
3. ✅ `/api/knowledge` shows all atoms and sources
4. ✅ `/api/search` returns search history with citations
5. ✅ WebSocket pushes live events (index, search, MCP)
6. ✅ Dashboard HTML loads and renders correctly
7. ✅ Knowledge Graph visualization shows nodes and edges
8. ✅ Settings form can enable/disable sources
9. ✅ All API endpoints have proper error handling
10. ✅ Daemon can be restarted gracefully

---

## Success Metrics

- **Developer time:** 2-4 hours to build initial dev dashboard
- **Team adoption:** 50% of team members install Kurultai (from CLI only)
- **MCP connections:** Auto-discovered and displayed in dashboard
- **Search performance:** <100ms for API endpoints
- **WebSocket latency:** <50ms for event pushing

---

## Open Questions

1. **Auth:** Should dev dashboard require authentication?
   - Decision: No for local dev, Basic Auth for team mode (Phase 4)

2. **Frontend:** React or vanilla JS?
   - Decision: Vanilla JS initially for speed, consider React in Phase 4

3. **Knowledge Graph:** Generate automatically or manual tagging?
   - Decision: Manual tagging first (Phase 2), auto-edge generation later (Phase 3+)

4. **Port:** Use 8421 or dynamic port?
   - Decision: 8421 (well-known), optional custom port via `--port`

---

## References

- Master plan: [#27 — Work Order: Master phase plan](https://github.com/duketopceo/kurultai/issues/27)
- Phase 3 tracking: [#7 Synthesis & Interface](https://github.com/duketopceo/kurultai/issues/7)
- Perplexity Brain UI inspiration: https://www.perplexity.ai/help-center/en/articles/19700001-what-is-brain
- Perplexity Computer dashboard: https://www.perplexity.ai/products/computer