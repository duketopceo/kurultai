#!/usr/bin/env python3
"""Generate the demo fixture corpus: ~250 tagged, cross-linked markdown atoms.

Deterministic (seeded) so the demo brain graph is stable across rebuilds.
Run from repo root: `python3 demo/generate-corpus.py` — writes demo/corpus/gen/NNN-*.md
"""

import random
import re
from pathlib import Path

random.seed(42)
OUT = Path(__file__).parent / "corpus" / "gen"
OUT.mkdir(parents=True, exist_ok=True)

# Topic clusters — each becomes a densely cross-linked neighborhood.
CLUSTERS = {
    "platform": ["architecture", "storage-engine", "retrieval-pipeline", "ingest-worker",
                 "embedding-cache", "schema-migration", "write-ahead-log", "tiered-store",
                 "sync-protocol", "conflict-resolution", "daemon-lifecycle", "config-loading",
                 "feature-flags", "metrics-pipeline", "health-probes", "session-management",
                 "connector-registry", "quarantine-lane", "promotion-flow", "atom-graph"],
    "agents": ["device-broker", "seat-tokens", "agent-onboarding", "mcp-tools",
               "hey-board", "turn-caps", "instance-claims", "agent-registry",
               "session-stamping", "relay-lane", "stdio-bridge", "board-moderation",
               "agent-keys", "device-auth", "presence-claims", "upstream-session"],
    "product": ["brain-ui", "ontology-board", "search-ux", "atom-inspector",
                "graph-rendering", "tier-badges", "dark-theme", "embed-pipeline",
                "chrome-shell", "inspector-panel", "chatboard", "settings-surface",
                "demo-mode", "landing-page", "access-gating", "hosted-instances"],
    "ops": ["deploy-runbook", "incident-001", "incident-002", "backup-restore",
            "tunnel-config", "access-policy", "disk-pressure", "memory-leak-hunt",
            "ci-failure-triage", "release-checklist", "rollback-plan", "capacity-notes",
            "monitoring-alerts", "rotation-schedule", "secrets-rotation", "migrations"],
    "research": ["knowledge-graphs", "vector-search", "bm25-baseline", "hybrid-retrieval",
                 "reciprocal-rank-fusion", "reranking", "embeddings-eval", "chunking-strategies",
                 "query-expansion", "temporal-edges", "entity-linking", "dedup-strategies",
                 "context-assembly", "memory-consolidation", "forgetting-curves", "provenance"],
    "people": ["alice-chen", "bob-okafor", "carol-nguyen", "dan-rivera",
               "erin-wallace", "frank-ito", "grace-kim", "henry-larsen",
               "irene-popov", "jake-morales"],
    "projects": ["aurora-rewrite", "beacon-launch", "cinder-migration", "delta-sprint",
                 "ember-poc", "fjord-integration", "glacier-archive", "harbor-sync",
                 "iris-dashboard", "jade-refactor", "kite-mobile", "lumen-api"],
}

ROLES = ["platform engineer", "agent infra lead", "product designer", "SRE",
         "researcher", "engineering manager"]
VERBS = ["blocked on", "depends on", "supersedes", "references", "extends",
         "replaces", "informs", "validates", "contradicts", "supports"]

def slug(s):
    return s.replace("-", " ").title()

def atom(cluster_key, topic, idx):
    """Return (filename, markdown) for one atom."""
    cluster = CLUSTERS[cluster_key]
    title = slug(topic)
    # 2-4 wiki-links: mostly same-cluster (dense neighborhoods), some cross-cluster
    links = []
    for _ in range(random.randint(2, 4)):
        if random.random() < 0.7:
            links.append(slug(random.choice(cluster)))
        else:
            other = random.choice([k for k in CLUSTERS if k != cluster_key])
            links.append(slug(random.choice(CLUSTERS[other])))
    links = [l for l in set(links) if l != title][:3]

    tags = [cluster_key, "demo"]
    if cluster_key == "people":
        tags.append("person")
    if cluster_key == "projects":
        tags.append(random.choice(["active", "archived", "planning"]))

    paras = []
    if cluster_key == "people":
        body = (f"{title} is a {random.choice(ROLES)}. Works on "
                f"{links[0] if links else 'platform'} and "
                f"{links[1] if len(links) > 1 else 'research'}.")
        paras.append(body)
    elif cluster_key == "projects":
        paras.append(f"{title} is a {random.choice(['Q1','Q2','Q3'])} initiative "
                     f"tracking {', '.join(links) if links else 'core work'}.")
    else:
        paras.append(f"{title} is part of the {cluster_key} surface. It "
                     f"{random.choice(VERBS)} {links[0] if links else 'the core store'}"
                     f" and interacts with {links[1] if len(links)>1 else 'daemon lifecycle'}.")
    paras.append(f"Key detail #{idx}: {title.lower()} {random.choice(VERBS)} "
                 f"the broader {cluster_key} workstream. Owners review it on a "
                 f"{random.choice(['weekly','monthly','per-release'])} cadence.")
    if links:
        paras.append("Related: " + ", ".join(f"[[{l}]]" for l in links))

    fm = ["---", f"title: {title}", "tags: [" + ", ".join(tags) + "]", "---"]
    return f"{idx:03d}-{topic}.md", "\n\n".join(fm) + "\n\n" + "\n\n".join(paras) + "\n"

def main():
    for old in OUT.glob("*.md"):
        old.unlink()
    idx = 0
    rows = []
    for cluster_key, topics in CLUSTERS.items():
        for topic in topics:
            idx += 1
            name, md = atom(cluster_key, topic, idx)
            (OUT / name).write_text(md)
            rows.append((name, f"{slug(topic)} atom ({cluster_key} cluster)"))
            if cluster_key != "people":
                idx += 1
                nname = f"{idx:03d}-{topic}-notes.md"
                body = (f"---\ntitle: {slug(topic)} — sync notes\ntags: [{cluster_key}, demo, notes]\n---\n\n"
                        f"Sync notes on [[{slug(topic)}]]. Action items, open questions, "
                        f"and follow-ups for the {cluster_key} lane.\n")
                (OUT / nname).write_text(body)
                rows.append((nname, f"{slug(topic)} sync-notes atom ({cluster_key})"))

    # agent-index contract: every file in the folder is listed
    table = "\n".join(
        f"| [`{n}`]({n}) | {d} | — | — | 2026-10-01 | 1 | generated |"
        for n, d in rows
    )
    (OUT / "INDEX.md").write_text(f"""---
index: kurultai/v1
folder: demo/corpus/gen
parent: demo/corpus/INDEX.md
updated: 2026-10-01
version: 1
---

# `demo/corpus/gen`

**Does:** Seeded generated fixture atoms — {idx} files across {len(CLUSTERS)} topic clusters, dense `[[wiki-links]]`
**Up:** [`demo/corpus/INDEX.md`](../INDEX.md) · **Generator:** [`../../generate-corpus.py`](../../generate-corpus.py) (regenerate = re-run it)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
{table}

## Recent

- 2026-10-01 — generated ({idx} atoms, seed 42)
""")
    print(f"wrote {idx} atoms to {OUT}")

if __name__ == "__main__":
    main()
