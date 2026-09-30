---
tags: [demo, overview]
---

# What is Kurultai

Kurultai is a local-first knowledge brain: a single Rust CLI/daemon that
ingests markdown, GitHub checkouts, and connector feeds into a SQLite store,
then serves search, citations, and MCP tools to coding agents.

Core commands: `init`, `index`, `search`, `ask`, `who-knows`, `status`,
`mcp`, `daemon`. The daemon embeds the Brain UI at `/ui/` and exposes a
JSON API under `/api/*`.

This demo instance is seeded only with hand-curated public-safe fixtures —
no personal notes, no private repos, no session history.
