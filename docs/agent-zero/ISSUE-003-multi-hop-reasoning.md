---
tags: [agent-zero, multi-hop, search, v1]
related:
  - docs/agent-zero/INDEX.md
  - docs/plans/2026-07-25-006-feat-v1-agent-zero-batch-plan.md
---
# ISSUE: Multi-hop Reasoning (Graph Orchestration)

**Labels:** feature, search, priority-high
**Tracking:** #6 (Phase 2) | Related: #7 (Synthesis)
**Status:** Open
**Created:** 2026-07-25

---

## Problem

Currently, kurultai performs **single-pass retrieval** - agents can only find information in one document:

**Current limitation:**
```bash
$ kurultai search "How do I deploy to staging?"

[0.016] ops/deploy.md — Deploy Guide — Staging deployment
  Excerpt: "For staging deployments, use the staging environment variables."
```

**Problems:**
1. **Can't answer complex questions:** Agents can't find information that's split across multiple documents
2. **No synthesis:** No reasoning across sources
3. **Perplexity advantage:** Perplexity Brain can chain information across documents
4. **Graph is flat:** Kurultai is a flat index, not a graph

**Impact:** Limited retrieval depth, can't answer "Why did we choose X?" or "What's the relationship between A and B?"

---

## Proposed Solution

### Graph Orchestration (Diamond Shape)

**Multi-hop query architecture:**
```
Query: "How do I deploy to staging and verify?"

┌─────────────────────────────────────────────────────┐
│  Multi-Hop Reasoning (Phase 2)                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Query Decomposition:                                │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │ How to  │──│ deploy  │──│ staging │            │
│  │ deploy  │  │ to  │    │ verify  │            │
│  └─────────┘  └─────────┘  └─────────┘            │
│          │              │                         │
│          ▼              ▼                         │
│  [Pass 1: Parallel Retrieval]                     │
│  ┌──────────────┐  ┌──────────────┐               │
│  │ FTS Search   │  │ Vector Search │               │
│  │ "deploy to"  │  │ embeddings    │               │
│  └──────┬───────┘  └──────┬───────┘               │
│         │                  │                        │
│         ▼                  ▼                        │
│  ┌──────────────┐  ┌──────────────┐               │
│  │ atoms: [3]   │  │ atoms: [5]   │               │
│  │ (deploy.md)  │  │ (ops.md)     │               │
│  └──────┬───────┘  └──────┬───────┘               │
│         └──────────────────┘                        │
│                    │                                 │
│                    ▼                                 │
│  [Pass 2: RRF Fusion + Rank]                       │
│  ┌───────────────────────────────────┐             │
│  │   atoms: [3, 5] (merged)           │             │
│  │   ranking: RRF(3)+RRF(5)           │             │
│  └──────────────┬────────────────────┘             │
│                 │                                    │
│                 ▼                                    │
│  [Pass 3: Graph Traversal]                         │
│  ┌───────────────────────────────────┐             │
│  │  Find edges:                         │             │
│  │  - deploy.md → staging.md (tag)    │             │
│  │  - staging.md → verify.md (tag)    │             │
│  │  - chaining: [3] → [7] → [9]      │             │
│  └──────────────┬────────────────────┘             │
│                 │                                    │
│                 ▼                                    │
│  [Pass 4: Answer Generation]                       │
│  ┌───────────────────────────────────┐             │
│  │  "To deploy to staging:"            │             │
│  │  1. Use staging env vars (ops.md)  │             │
│  │  2. Run smoke tests (verify.md)    │             │
│  │  3. Check logs at /var/log/...     │             │
│  │                                     │             │
│  │  Sources: ops/deploy.md → ops/staging.md → docs/verify.md │
│  └───────────────────────────────────┘             │
└─────────────────────────────────────────────────────┘
```

### RRF Fusion

**Reciprocal Rank Fusion for hybrid search:**
```rust
// Pseudocode
fn rr_fusion(fts_results: Vec<Atom>, vec_results: Vec<Atom>) -> Vec<Atom> {
    let mut scored: HashMap<AtomId, f64> = HashMap::new();
    
    // Score FTS results (rank 1 = 1.0, rank 2 = 0.5, etc.)
    for (i, atom) in fts_results.iter().enumerate() {
        scored.entry(atom.id).or_insert(0.0) += 1.0 / (i as f64 + 1.0);
    }
    
    // Score Vector results
    for (i, atom) in vec_results.iter().enumerate() {
        scored.entry(atom.id).or_insert(0.0) += 1.0 / (i as f64 + 1.0);
    }
    
    // Sort by fused score, return top N
    scored.into_iter()
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
        .map(|(atom, score)| atom)
        .collect()
}
```

### Graph Traversal (Edges)

**Manual tagging for now:**
```toml
# In markdown metadata (YAML frontmatter)
---
tags: ["deployment", "staging"]
related:
  - "ops/staging.md"
  - "docs/verify.md"
---
```

**Agent-learned edges (future):**
```rust
// Use embeddings to find related atoms
fn find_related_atoms(base_atoms: &[Atom], threshold: f64) -> Vec<Edge> {
    base_atoms.iter()
        .flat_map(|base| {
            vec_results.iter()
                .filter(|vec| vec.distance_to(base) < threshold)
                .map(|vec| Edge { from: base.id, to: vec.id, score: vec.distance_to(base) })
                .collect()
        })
        .collect()
}
```

### CLI Output

```bash
$ kurultai ask "How do I deploy to staging and verify?"

🧠 Answer:
To deploy to staging:

1. **Use staging environment variables**
   From ops/deploy.md: "For staging deployments, use the staging environment variables."

2. **Run smoke tests**
   From docs/verify.md: "After deployment, run smoke tests on /api/health."

3. **Check logs**
   From ops/staging.md: "Logs are at /var/log/kurultai/staging.log."

🔗 Chain:
  ops/deploy.md → ops/staging.md → docs/verify.md

⏱️ Search: 234ms
```

### MCP Tool

**`full_ask` tool:**
```json
{
  "result": {
    "answer": "To deploy to staging: 1. Use staging env vars, 2. Run smoke tests, 3. Check logs.",
    "sources": [
      {
        "file": "ops/deploy.md",
        "excerpt": "For staging deployments, use the staging environment variables."
      },
      {
        "file": "docs/verify.md",
        "excerpt": "After deployment, run smoke tests on /api/health."
      },
      {
        "file": "ops/staging.md",
        "excerpt": "Logs are at /var/log/kurultai/staging.log."
      }
    ],
    "graph_chain": ["ops/deploy.md", "ops/staging.md", "docs/verify.md"]
  },
  "latency_ms": 234
}
```

---

## Implementation Plan

### Phase 2: Basic Graph (Week 1-2)
- [ ] Add `Edge` struct to schema v3 (from, to, score)
- [ ] Implement `rr_fusion()` function
- [ ] Add `full_ask()` CLI command (search → fuse → chain → answer)
- [ ] Add `full_ask` MCP tool
- [ ] Test RRF fusion with FTS + Vector results

### Phase 3: Graph Visualization (Week 3-4)
- [ ] Add HTTP API endpoint `/api/search/query` (multi-hop)
- [ ] Implement `graph_traverse()` function
- [ ] Return graph chain in search results
- [ ] Add Knowledge Graph visualization in dashboard

### Phase 4: Learned Edges (Later)
- [ ] Auto-generate edges from embedding similarity
- [ ] Add confidence scores to edges
- [ ] Support edge pruning (low-confidence links)
- [ ] Add interactive graph UI (click node → see edges)

---

## Testing

**Manual Tests:**
```bash
# 1. Test multi-hop search
kurultai ask "How do I deploy to staging and verify?"

# 2. Verify graph chain is returned
kurultai search "staging deploy" --format=json | jq '.graph_chain'

# 3. Test RRF fusion manually
echo '[{"query":"deploy staging","sources":[{"title":"deploy.md","content":"deploy staging"},{"title":"ops.md","content":"ops deploy"}]}]'
```

**Integration Tests:**
- [ ] Test RRF fusion scores correctly
- [ ] Test graph traversal finds all related documents
- [ ] Test multi-hop answer includes all relevant sources
- [ ] Test edge generation with similar embeddings

---

## Acceptance Criteria

1. ✅ `rr_fusion()` merges FTS and Vector results with proper scores
2. ✅ `full_ask()` command returns answer + sources + graph chain
3. ✅ MCP `full_ask` tool returns answer + sources + graph chain
4. ✅ HTTP API `/api/search/query` supports multi-hop queries
5. ✅ Graph chain is returned in search results
6. ✅ CLI output formats graph chain clearly
7. ✅ Dashboard shows Knowledge Graph visualization
8. ✅ Edge generation works with embedding similarity
9. ✅ RRF fusion improves ranking over single-pass search
10. ✅ Graph traversal handles cycles gracefully

---

## Success Metrics

- **Retrieval depth:** 3+ documents per query (vs 1 currently)
- **Answer accuracy:** 70% of multi-hop questions answered correctly
- **Query latency:** <500ms for multi-hop queries
- **User satisfaction:** 60% report improved agent capabilities

---

## Open Questions

1. **Should graph edges be labeled?**
   - Option A: Yes, manual labels (recommended)
   - Option B: Auto-labels from embedding similarity
   - Decision: Manual labels first (Phase 2), auto later (Phase 4)

2. **How deep should graph traversal be?**
   - Option A: 3 hops max (current)
   - Option B: Dynamic based on query length
   - Option C: Unlimited (performance risk)
   - Decision: 3 hops max (configurable via `--max-hops=3`)

3. **What if graph is disconnected?**
   - Option A: Return partial answer
   - Option B: Error out (forces more tagging)
   - Option C: Try single-pass search as fallback
   - Decision: Try single-pass search as fallback

---

## References

- Master plan: [#27 — Work Order: Master phase plan](https://github.com/duketopceo/kurultai/issues/27)
- Phase 2 tracking: [#6 Search & Retrieval](https://github.com/duketopceo/kurultai/issues/6)
- RRF algorithm: Cormack et al. "Reciprocal Rank Fusion outperforms Condorcet and individual Rank Learning methods" (ACL 2009)
- Perplexity Brain multi-hop: https://www.perplexity.ai/help-center/en/articles/19700001-what-is-brain