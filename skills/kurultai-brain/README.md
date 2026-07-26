# kurultai-brain skill

A portable [agentskills.io](https://agentskills.io)-compatible skill that
teaches any MCP-capable agent (Hermes Agent, Claude Code, Cursor, Codex, …)
how to use the [Kurultai](https://github.com/duketopceo/kurultai) knowledge
brain.

## What it gives the agent

Five MCP tools, served by the kurultai stdio MCP server (`kurultai mcp`):

| Tool | Use |
|------|-----|
| `search` | Ranked excerpts + citations (the default) |
| `cite` | One grounding slice for a claim |
| `ask` | Synthesized answer with citations (higher cost) |
| `who_knows` | Which sources cover a topic |
| `remember` | Store a distilled fact for future sessions |

Hermes Agent registers them as `mcp_kurultai_search`, `mcp_kurultai_cite`, …
per its `mcp_<server>_<tool>` convention.

## Install

```bash
# 1. Install kurultai (no binary release yet — build from source)
cargo install --git https://github.com/duketopceo/kurultai --locked

# 2. Wire the kurultai MCP server into your agent
kurultai init --agent hermes      # or: cursor | claude | codex | all

# 3. Index sources + run the brain
kurultai index --full
kurultai daemon --port 8421       # optional: HTTP + poll + watch, UI at /ui

# 4. Restart the agent so the kurultai MCP tools load
```

Then point your agent at this `SKILL.md` (e.g. Hermes loads skills from
`~/.hermes/skills/`; copy or symlink this directory there, or add it via your
skill registry).

## License

MIT.
