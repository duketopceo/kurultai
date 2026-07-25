---
tags: [agent-zero, citations, search, v1]
related:
  - docs/agent-zero/INDEX.md
  - docs/plans/2026-07-25-006-feat-v1-agent-zero-batch-plan.md
---

> **Shipped contract (v1 batch #75):** `Citation` fields are `source`, `source_id`, `title`, `url`, `excerpt`, optional `file_path`, `section`, `title_hash`, `excerpt_start`, `excerpt_end`.  
> `GET /api/search?q=` returns `Vec<SearchResult>` (atom + score + rank), not a wrapped `{ query, results }` envelope.

# ISSUE: Query Result Citations

**Labels:** feature, UX, priority-medium
**Tracking:** #7 (Phase 2) | Related: #37 (Doctrine)
**Status:** Open
**Created:** 2026-07-25

---

## Problem

Currently, kurultai returns search results with **excerpts but no citations**:

**Current output:**
```
[0.016] notes — Deploy Guide — Database migration
  [ops/deploy.md > Deploy Guide > Database migration]
  Always run the database migration scripts before cutting traffic.
  KNOWN_PHRASE_KURULTAI_42 is the golden search token for fixture tests.
```

**Problems:**
1. **No verified source:** How do we know this excerpt came from `ops/deploy.md`?
2. **Trust issues:** Can't verify the agent read what it claims
3. **Debugging pain:** Can't inspect the original document when result is wrong
4. **Workflow disruption:** Manual file lookup required to verify context

**Impact:** Reduced user trust, harder to debug, poor agent verification.

---

## Proposed Solution

### Citation Structure

**Add source metadata to every result:**

```json
// Current AgentAtomView (simplified)
{
  "id": "atom_abc123",
  "title": "Deploy Guide — Database migration",
  "content": "Always run the database migration scripts before cutting traffic.",
  "rank": 1
}
```

```json
// New AgentAtomView with citations
{
  "id": "atom_abc123",
  "title": "Deploy Guide — Database migration",
  "excerpt": "Always run the database migration scripts before cutting traffic.",
  "snippet": "[ops/deploy.md > Deploy Guide > Database migration]",
  "citations": [
    {
      "source_id": "notes",
      "source_kind": "markdown",
      "file_path": "ops/deploy.md",
      "title": "Deploy Guide",
      "section": "Database migration",
      "title_hash": "sha256(deploy.md first 100 chars)",
      "excerpt_start": 42,
      "excerpt_end": 120,
      "full_text": "Always run the database migration scripts before cutting traffic."
    }
  ],
  "rank": 1,
  "metadata": {
    "indexed_at": "2026-07-25T20:42:21Z",
    "version": "1.0"
  }
}
```

### CLI Search Output

```bash
$ kurultai search "database migration" --limit 2

[0.016] ops/deploy.md — Deploy Guide — Database migration
  💬 Excerpt: "Always run the database migration scripts before cutting traffic."
  📄 Source: ops/deploy.md
  📊 Match: title_hash=3a5f... (exact match)
  ⏰ Indexed: 2026-07-25T20:42:21Z
```

### MCP Tools Output

**`search` tool:**
```json
{
  "results": [
    {
      "id": "atom_abc123",
      "title": "Deploy Guide — Database migration",
      "excerpt": "Always run the database migration scripts before cutting traffic.",
      "citations": [
        {
          "file_path": "ops/deploy.md",
          "section": "Database migration",
          "excerpt_start": 42,
          "excerpt_end": 120
        }
      ]
    }
  ]
}
```

### HTTP API Response

```json
// GET /api/search?query="database migration"
{
  "query": "database migration",
  "results": [
    {
      "id": "evt_abc123",
      "timestamp": "2026-07-25T20:42:31Z",
      "results": [
        {
          "id": "atom_abc123",
          "source_id": "notes",
          "file_path": "ops/deploy.md",
          "title": "Deploy Guide — Database migration",
          "section": "Database migration",
          "excerpt": "Always run the database migration scripts before cutting traffic.",
          "citations": [
            {
              "file_path": "ops/deploy.md",
              "title_hash": "3a5f...",
              "start_char": 42,
              "end_char": 120
            }
          ],
          "rank": 1,
          "latency_ms": 16
        }
      ]
    }
  ]
}
```

### Integration with Perplexity Brain

**Perplexity Brain citation style:**
```
> "Always run the database migration scripts before cutting traffic."
> 
> — ops/deploy.md, section "Database migration"
```

**Kurultai citation style (to match):**
```
> "Always run the database migration scripts before cutting traffic."
> 
> 💬 ops/deploy.md → Deploy Guide → Database migration (exact match)
```

---

## Implementation Plan

### Phase 2: Basic Citations (Week 1-2)
- [ ] Add `citations` field to `AgentAtomView` struct
- [ ] Store `file_path`, `title`, `section`, `title_hash` in schema v3
- [ ] Update `search()` command to output citations
- [ ] Update MCP `search` tool to return citations

### Phase 3: Enhanced Citations (Week 3-4)
- [ ] Add `excerpt_start`, `excerpt_end` character positions
- [ ] Add `full_text` for complete context
- [ ] Improve CLI formatting with colored citations
- [ ] Add HTTP API endpoint `GET /api/search/<id>` with full citation

### Phase 4: Multi-source Citations (Later)
- [ ] Support multiple citations per result (cross-source)
- [ ] Add citation provenance (indexed_at, source_version)
- [ ] Implement citation deduplication (same content from different sources)

---

## Testing

**Manual Tests:**
```bash
# 1. Search and verify citations
kurultai search "kurultai" --limit 3

# Expected output includes:
# - file_path: ops/deploy.md
# - section: Database migration
# - title_hash: exact hash

# 2. Check MCP output
echo '[{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search","arguments":{"query":"kurultai","limit":3}}}]' \
  | kurultai mcp | jq '.result.content[0].text'

# Verify citations array present
```

**Integration Tests:**
- [ ] Test citation structure in `AgentAtomView`
- [ ] Test CLI search output formatting
- [ ] Test MCP `search` tool returns citations
- [ ] Test HTTP API returns citations
- [ ] Test citation deduplication (same content from different files)

---

## Acceptance Criteria

1. ✅ `AgentAtomView` includes `citations` array with at least: `file_path`, `title`, `section`
2. ✅ CLI `search` command outputs citations in readable format
3. ✅ MCP `search` tool returns citations in tool response
4. ✅ HTTP API `/api/search` returns citations for each result
5. ✅ Citations show exact source document (file_path)
6. ✅ Citations show section (title_hash)
7. ✅ Citations show character range (start, end)
8. ✅ MCP `cite` tool returns full citation with source
9. ✅ CLI formatting is consistent across all output modes
10. ✅ Citations can be verified by manually reading the source file

---

## Success Metrics

- **Trust:** 80% of users can verify search results by clicking citation
- **Debugging:** 90% reduction in manual file lookup time
- **Agent verification:** 100% of agents can trace answer to source
- **User satisfaction:** 75% report improved trust in search results

---

## Open Questions

1. **Should citations be clickable in CLI?**
   - Option A: No (terminal limitation)
   - Option B: Yes, hyperlink via ANSI escape codes (partial support)
   - Decision: Highlight with ANSI colors, no click (terminal limitation)

2. **How should we handle duplicate content?**
   - Option A: Return all citations
   - Option B: Return unique citation by title_hash
   - Decision: Return all, deduplicate at UI layer

3. **What about synthetic citations (generated by ask)?**
   - Option A: Include generated citation
   - Option B: Mark as "synthetic" (not from source)
   - Decision: Include citation, mark type in metadata

---

## References

- Master plan: [#27 — Work Order: Master phase plan](https://github.com/duketopceo/kurultai/issues/27)
 - Phase 2 tracking: [#6 Search & Retrieval](https://github.com/duketopceo/kurultai/issues/6)
 - Doctrine: [#37 — Index-time heavy, read-time light](https://github.com/duketopceo/kurultai/issues/37)
 - Perplexity Brain citation style: https://www.perplexity.ai/help-center/en/articles/19700001-what-is-brain