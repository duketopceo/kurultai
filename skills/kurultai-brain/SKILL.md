---
name: kurultai-brain
description: Query and write the hosted Kurultai work brain.
when_to_use:
  - "What did we change in the SQL database / schema / migrations?"
  - "Search my notes for <topic>"
  - "What do my sources know about <topic>?"
  - "Cite the source of <claim>"
  - "Remember that <fact>"
  - "Ground this answer in the local brain"
  - "Index my email / meeting notes / Slack into the brain"
  - "Seed or sync the hosted work brain"
  - "Why does the brain view look wrong / what are the lobes or sectors for?"
transport: http_mcp
setup: |
  # On this host (hermes-bartlett-omarchy) the brain is HOSTED. Nothing to install:
  #   kurultai-work:
  #     url: https://api-work.shippedit.dev/mcp
  #     headers: {Authorization: "Bearer ${KURULTAI_WORK_SEAT_KEY}"}
  # Seat key: /opt/data/.env -> KURULTAI_WORK_SEAT_KEY. Verify non-empty, never print it.
  #
  # Only on a fresh non-Bartlett dev machine, standing up a local index is:
  #   kurultai init --docs && kurultai index --full
  #   kurultai daemon --port 8421        # HTTP + Brain UI at /ui
  # Do not do this on a host that already has the hosted seat.
---

# kurultai-brain

Assemble what you know, from wherever it lives. Kurultai exposes **token-capped
excerpts + citations** through MCP tools — never full files.

On `hermes-bartlett-omarchy` the brain is **hosted** at `work.shippedit.dev`: one
shared store, no local SQLite index, no daemon to start. See *Hosted brain
operations* for the endpoint, sync script, and health check. The local-mode
description below (SQLite + FTS5 + sqlite-vec, `kurultai init`, port 8421) applies
only to a fresh dev machine that has not been given a seat yet.

## When to use

Prefer the brain over re-reading whole files when you need grounded context
that already lives in the user's knowledge: prior decisions, migration notes,
design specs, agent-session lore, or code symbol relationships. The brain
returns ranked excerpts (~400 chars) with citations back to `source` /
`source_id`, so you stay grounded without saturating the context window.

## Tools

Hermes registers these as `mcp_kurultai_<tool>` (e.g. `mcp_kurultai_search`).
Other hosts expose them under their own MCP namespace.

The tool set is **version-dependent** — enumerate the live list instead of trusting
this page (hosted returns ten: `search`, `recall`, `cite`, `ask`, `who_knows`,
`ontology_get`, `ontology_proposals`, `hey_threads`, `hey_read`, `hey_poll`):

```bash
curl -s -H "Authorization: Bearer $KURULTAI_WORK_SEAT_KEY" \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
  "$KURULTAI_MCP_URL" | python3 -c 'import json,sys; print([t["name"] for t in json.load(sys.stdin)["result"]["tools"]])'
```

### recall(topic) — reconstruct recent working context
Pull back what was recently being worked on (agent chats, session lore) when
the question is "where did I leave off" rather than "what is X".

### search(query, limit) — preferred default
Semantic + keyword search. Returns short ranked excerpts, not full documents.
Use this first for any "what do we know about X" question. Cheap; FTS-first
works with no embedding API key.

### cite(source, source_id) — one grounding slice
Returns a single citation-sized excerpt for grounding a specific claim. Use
after `search` when you need to pin one exact source, or to verify a
reference.

### ask(question) — sparingly (higher cost)
Synthesized answer with citations. Higher token cost than `search` because it
runs synthesis. Use when you need a composed answer across multiple atoms,
not when a ranked excerpt list will do.

### who_knows(topic, limit) — source coverage
Which sources know about a topic (distinct source aggregates, not full
synthesis). Use to scope a topic before diving in: "is this in my notes, my
code, or my chats?"

### ontology_get / ontology_proposals — the structured layer
`ontology_get` reads the curated concept graph; `ontology_proposals` lists
unmerged candidates (and the trusted/quarantine split) for review. Use these when
the question is about *entities and relationships*, not raw excerpts — and check
`ontology_proposals` before assuming a concept is missing from the brain.

### hey_threads / hey_read / hey_poll — the message layer
Peer/agent message threads. `hey_threads` lists them, `hey_read` opens one,
`hey_poll` fetches replies. Read-only orientation: use them only when the question
is explicitly about what another agent or peer said.

### Write path

On the **hosted** brain use the `remember` tool with `{title, summary, tags}` —
it is exposed and it is how the daily source sync lands facts. Local/older builds
may expose no `remember` tool, so when a write returns a JSON-RPC error rather
than a stored atom, probe `tools/list` and fall back to the CLI/ingest path plus
the ontology proposal queue. Never assume a write landed: read back with
`search` or check `/api/status` atom counts before reporting success.

## Guardrails

- **Excerpts, not files.** The brain is a retrieval layer; the user's files
  remain the system of record. Edit files, not the brain.
- **FTS-first.** Search works without an embedding API key. Vectors are
  optional enrichment when a live embedder is configured.
- **Write distilled facts, not dumps.** When a build does expose a memory write
  tool, pass a title + one-line summary + 1-3 tags. Never paste a transcript or a
  whole file into the brain — distil first so it stays deduplicated.
- **`ask` costs more than `search`.** Default to `search`; escalate to `ask`
  only when you need synthesis across atoms.

## Hosted brain operations (`work.shippedit.dev`)

`work.shippedit.dev` (`https://api-work.shippedit.dev/mcp`) is the single authoritative Kurultai brain. Do not run or install a local `kurultai.service` daemon on `hermes-bartlett-omarchy`.

- **MCP endpoint:** `https://api-work.shippedit.dev/mcp` (configured as `kurultai-work` in `~/.hermes/config.yaml`, authenticated with `KURULTAI_WORK_SEAT_KEY`).
- **Health & counts:** `curl -s -H "Authorization: Bearer $KURULTAI_WORK_SEAT_KEY" https://api-work.shippedit.dev/api/status`
- **Source sync:** `/opt/data/gvenv/bin/python3 /opt/data/scripts/kurultai_sync_sources.py` pulls Drive meeting notes, Gmail work emails, and Slack messages and pushes new items directly to `https://api-work.shippedit.dev/mcp` via `remember`. A healthy run finishes in under ~15s and prints `new_notes`/`new_emails`/`updated_slack_channels`/`pushed_to_work_shippedit_dev` plus live atom counts.
- **Sync pitfalls:** keep `socket.setdefaulttimeout(15)` and small page sizes (Drive `pageSize=15`, Gmail `maxResults=15`) — the Google client blocks indefinitely without a socket timeout, and large pages push one run past the harness timeout. The push-dedupe digest is `sha256(filename:len(summary))`, so an edit that keeps the same summary length is silently not re-pushed; change the digest input (or drop the entry from `.pushed_work_shippedit.json`) when re-publishing a corrected item.
- **Details:** `references/live-sources-and-hosted-mirror.md`.

## Brain view: lobes and region labels

The 3D brain is a **client-side drawing of the atom table**, not an ontology
feature: 24 contiguous lobes are k-means chunks of the cortex volume, and each
atom's region key is its most frequent tag across the whole corpus. Server code
and the API have no sector/lobe concept — diagnosing a weird picture with API
probes alone will not find it.

When the user says the brain looks wrong (one dense blob, empty cortex, nonsense
labels like a single odd tag with `· 1`), check **tag dominance before anything
else**: fetch `/api/graph?tier=hot`, tally tags, and see whether one umbrella tag
(workspace, repo, language, file extension) is on nearly every atom. If it is,
every atom lands in lobe 0 and the leftover lobes get labelled with the handful
of atoms that lacked it. On `hermes-bartlett-omarchy`, fix this via our **local
custom sections and tags config** (`/opt/data/kurultai-sources/sections-config.json`),
consumed by `/opt/data/scripts/kurultai_sync_sources.py` (`resolve_section_and_tags`),
which strips umbrella tags (`bartlett`, `code`, `md`, `py`) and assigns specific
domain section tags (`servicetitan`, `salestrak`, `middleman`, `data-platform`,
`infrastructure`, `deployments`, `it-support`, `security`, `engineering-ops`,
`hermes-ops`, `tech_questions`, `company_wide`, `meeting-notes`, `email`, `slack`)
before pushing via MCP `remember` — keeping upstream Kurultai untouched.

Explain lobes to the user as topic territory: a region fills as data accrues
(~150 atoms saturates a lobe), labels name the biggest regions at far zoom, and
zooming in reveals individual atoms. Do not promise anatomical meaning — the
lobe geometry is aesthetic, the grouping is tags.

- **Details:** `references/brain-visualization.md`.
