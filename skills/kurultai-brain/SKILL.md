---
name: kurultai-brain
description: Query and contribute to the local Kurultai knowledge brain — a SQLite-vec + FTS5 index of notes, agent chats, Dayflow, and code checkouts. Use when the agent needs grounded excerpts + citations from the user's local knowledge instead of guessing or dumping whole files into context.
when_to_use:
  - "What did we change in the SQL database / schema / migrations?"
  - "Search my notes for <topic>"
  - "What do my sources know about <topic>?"
  - "Cite the source of <claim>"
  - "Remember that <fact>"
  - "Ground this answer in the local brain"
transport: stdio_mcp
setup: |
  # Wire the kurultai MCP server into your agent (pick one):
  kurultai init --agent hermes      # NousResearch Hermes Agent -> ~/.hermes/config.yaml
  kurultai init --agent cursor      # Cursor -> ~/.cursor/mcp.json
  kurultai init --agent claude      # Claude Code -> ~/.claude.json
  kurultai init --agent codex       # Codex -> ~/.codex/config.toml
  kurultai init --agent all         # all four

  # Index sources + run the brain, then restart the agent so tools load:
  kurultai index --full
  kurultai daemon --port 8421       # optional: HTTP + poll + watch, UI at /ui
---

# kurultai-brain

Assemble what you know, from wherever it lives. Kurultai indexes the user's
markdown notes, agent transcripts, Dayflow timeline, and local code checkouts
into one SQLite + FTS5 + sqlite-vec store, then exposes **token-capped
excerpts + citations** through five MCP tools — never full files.

## When to use

Prefer the brain over re-reading whole files when you need grounded context
that already lives in the user's knowledge: prior decisions, migration notes,
design specs, agent-session lore, or code symbol relationships. The brain
returns ranked excerpts (~400 chars) with citations back to `source` /
`source_id`, so you stay grounded without saturating the context window.

## Tools

Hermes registers these as `mcp_kurultai_<tool>` (e.g. `mcp_kurultai_search`).
Other hosts expose them under their own MCP namespace.

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

### remember(title, summary, tags) — write path
Store a **distilled** fact for future sessions. Prefer the distilled fields
(`summary`, `tags`) over raw dumps — the brain stays clean, deduplicated, and
structured. Never dump raw chat or whole files into `remember`; distill first.

## Guardrails

- **Excerpts, not files.** The brain is a retrieval layer; the user's files
  remain the system of record. Edit files, not the brain.
- **FTS-first.** Search works without an embedding API key. Vectors are
  optional enrichment when a live embedder is configured.
- **`remember` is for facts.** Distill before writing. A good `remember` is a
  title + a one-line summary + 1-3 tags, not a transcript paste.
- **`ask` costs more than `search`.** Default to `search`; escalate to `ask`
  only when you need synthesis across atoms.

## Setup

See the `setup` block in the frontmatter above. After `init`, restart the
agent so the kurultai MCP server (`kurultai mcp`, stdio) is discovered.

## License

MIT — same as the kurultai project.
