---
tags: [demo, mcp, agents]
---

# MCP surface

Agents wire in via the Model Context Protocol. Kurultai exposes eight
tools: `search`, `cite`, `remember`, `ask`, `who_knows`, `promote`,
`ontology_get`, `ontology_promote`.

`kurultai init --agent <cursor|claude|codex|hermes|all>` writes the MCP
config for each client. The daemon also serves MCP over HTTP/SSE at
`/mcp` when `KURULTAI_MCP_HTTP_SECRET` is set.

Agent identity is two-layer: a product-family codename plus a concurrent
`instance_id` seat — never `devin-2`-style separate codenames.
