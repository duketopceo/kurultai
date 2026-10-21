---
index: kurultai/v1
folder: src/mcp
parent: src/INDEX.md
updated: 2026-10-20
version: 4
---

# `src/mcp`

**Does:** MCP stdio + broker stdio relay + agent init
**Up:** [`src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`brain.rs`](brain.rs) | BrainService implementing AgentRead; `ask_with_web` ephemeral web augmentation + Jev sufficiency gate; `remember_with` stamps member `team_id`/`visibility=team` from `PgScope` (KTD5); `*_hub` variants take an optional `hub_team` → `SearchFilter::with_hub_team` app-level team predicate | `src/activity` · `src/brain` · `src/embed` · `src/eval` · `src/web` · `src/store/scope.rs` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-10-21 | 6 | 2026-10-21 hub adoption U5: `search_views_scoped_hub` + `recall_for_agent_hub` thread the resolved hub `team_id` into `SearchFilter::with_hub_team` (member key's reads constrained to own team + company; RLS still enforces underneath) · 2026-10-20 hub adoption U2: `remember_with` member stamping from `PgScope::current()` (visibility=team + server team_id, client team_id overwritten); `multi_hop_expand` re-enters scope inside `JoinSet` spawns (task-locals don't cross spawn) · 2026-10-02 ask paths set `answer.gaps` |
| [`broker_stdio.rs`](broker_stdio.rs) | `mcp --broker` stdio↔broker relay: frames → `POST /mcp` with `X-Kurultai-Session` | `src/mcp/server.rs` · `reqwest` | `src/main.rs` | 2026-09-30 | 1 | 2026-09-30 added — agent lane holds no upstream credentials (plan 2026-09-28-001 U3) |
| [`init.rs`](init.rs) | kurultai init --agent cursor/claude/codex/hermes; `RemoteWire`/`wire_agent_remote` — Cursor/Claude get remote `{url, Authorization: Bearer}` HTTP entries (0600), Codex/Hermes get `mcp --broker` stdio bridges (bearer stays in omaseal, never in TOML/YAML) | `src/config` · `src/error` · `src/security` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` · `src/connect.rs` | 2026-10-21 | 2 | 2026-10-21 hub adoption U5: `RemoteWire`+`WireOutcome`+`wire_agent_remote` — remote-URL JSON entries for cursor (`~/.cursor/mcp.json`) / claude (`~/.claude.json`, `type:"http"`, siblings preserved, atomic 0600); codex (`config.toml`) / hermes (`config.yaml`) get broker-bridge entries + stored-key notes · 2026-08-16 indexed (v1 seed) |
| [`interface.rs`](interface.rs) | AgentRead trait | `src/synthesize` · `src/types` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` | 2026-07-23 | 1 | 2026-08-16 indexed (v1 seed) |
| [`mod.rs`](mod.rs) | MCP module: stdio server + init wiring; re-exports `RemoteWire`/`WireOutcome`/`wire_agent_remote` | — | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/server.rs` · `src/connect.rs` | 2026-10-21 | 2 | 2026-10-21 hub adoption U5: export remote-wire surface (`RemoteWire`, `WireOutcome`, `wire_agent_remote`) · 2026-08-16 indexed (v1 seed) |
| [`server.rs`](server.rs) | MCP tool dispatch (search, ask, ontology_*, hey_*); `handle_message_scoped` threads `HubPrincipal` team/admin into hub-scoped reads + member governance denies; frame/error/response helpers pub(crate) for broker_stdio | `src/error` · `src/mcp` · `src/ontology` · `src/project` · `src/write_policy` · `src/store/scope.rs` | `src/daemon/mod.rs` · `src/doctor.rs` · `src/http/mcp.rs` · `src/http/mod.rs` · `src/mcp/broker_stdio.rs` | 2026-10-21 | 6 | 2026-10-21 hub adoption U5: `handle_message_scoped(scope)` — member principals get `ToolSurface::HubMember` (governance tools absent + JSON-RPC denied), `hub_team` → `search_views_scoped_hub`/`ask_with_team`/`who_knows_with_team`/`cite` filters; admin principals keep `Full` · 2026-09-30 `read_stdin_frame`/`rpc_error`/`write_response`/`StdinFrame` → pub(crate) for broker relay · 2026-09-18 ask tool `web` param → ask_with_web · 2026-09-11 `ontology_propose` (full surface) + `ontology_proposals` (read) (#118) · 2026-09-04 hey_* message board tools + clamp + readonly list update · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-21 — U5 hub-member MCP + remote wiring (plan 2026-10-19-2053): `server.rs` `handle_message_scoped` takes the `HubPrincipal` resolved at `http/mcp.rs` — member keys get `ToolSurface::HubMember` (governance writes JSON-RPC-denied + omitted from tools/list) and `hub_team` → `search_views_scoped_hub`/`ask_with_team`/`who_knows_with_team`/`cite` filters; `brain.rs` `*_hub` variants stamp `SearchFilter::with_hub_team`; `init.rs` `wire_agent_remote` — cursor/claude write `{url,Authorization:Bearer}` entries (0600), codex/hermes write `mcp --broker` bridge entries whose bearer lives only in omaseal; `mod.rs` re-exports the remote-wire API
- 2026-10-20 — `brain.rs` (hub adoption U2, KTD5): `remember_with` reads `PgScope::current()` — a member scope forces `visibility=team` + `metadata.team_id=scope.team_id` (overwrites client-supplied value); `multi_hop_expand` captures the scope and re-enters it inside `JoinSet` spawned hop queries so member asks can't merge other teams' atoms
- 2026-10-02 — `brain.rs`: `ask_with_team`/`ask_with_web` populate `Answer.gaps` (local hits only)

- 2026-09-30 — `broker_stdio.rs` (U3): `mcp --broker` relays JSON-RPC frames to the device broker `POST /mcp` with `X-Kurultai-Session`; `server.rs` frame helpers → pub(crate)
- 2026-09-17 — `brain.rs`: `ask_with_web` + `with_web_searcher`/`with_judge` — Perplexity augmentation behind sufficiency gate
- 2026-09-15 — `brain.rs`: `BrainService.tier_policy` + `with_tier_policy`; tier calls use configured policy (#325)
- 2026-09-11 — `ontology_propose`/`ontology_proposals` tools: agents submit drafts, humans decide via REST/UI (#118)
- 2026-09-04 — `server.rs` adds `hey_threads/read/poll` tools and read-only tool list
- 2026-08-16 — indexed this folder (v1 seed)
