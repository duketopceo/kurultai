---
title: "feat: Phase 5 multi-agent MCP init (Cursor / Claude Code / Codex)"
date: 2026-07-25
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "Phase 5 production readiness · audience #25 developer → solo · upstream #11 MCP installer residual"
depth: standard
origin: "LFG after status inventory: MCP server ships; only Cursor auto-wire exists; Claude/Codex init is the 1.0 agent-surface gap"
---

# feat: Phase 5 multi-agent MCP init (Cursor / Claude Code / Codex)

**Target repo:** `duketopceo/kurultai`  
**Audience:** developer → solo  
**Base:** `main`  
**Process:** PR-only · branch `cursor/phase5-multi-agent-mcp-init-f07c`

## Goal Capsule

Extend `kurultai init --agent <target>` so one command wires the **existing** stdio MCP server (`kurultai mcp`) into **Cursor**, **Claude Code**, and **Codex** user-scope configs — same tools (`search`, `cite`, `ask`, `who_knows`, `remember`), different config files.

**Stop when:** each target merges idempotently; tests cover JSON + TOML writers; README/mac-dev document the matrix; PR green.

**Do not:** reinvent MCP transport; Smithery publish; Claude Desktop-only path as primary; team/company installer (ISSUE-004 stretch); local embeddings; cloud sync; Pond ingest changes.

**Product Contract preservation:** N/A (bootstrap plan — no prior requirements-only artifact).

---

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | `kurultai init --agent cursor` keeps current behavior: merge `kurultai` into `~/.cursor/mcp.json` under `mcpServers` |
| R2 | `kurultai init --agent claude` merges user-scope Claude Code MCP into `~/.claude.json` under top-level `mcpServers` with stdio `command` + `args: ["mcp"]` (optional `"type": "stdio"`) |
| R3 | `kurultai init --agent codex` merges `[mcp_servers.kurultai]` into `~/.codex/config.toml` with `command` + `args = ["mcp"]` |
| R4 | `kurultai init --agent all` wires cursor + claude + codex; returns/prints each path written |
| R5 | Malformed existing JSON/TOML → refuse overwrite of sibling servers (match Cursor refuse semantics) |
| R6 | Idempotent: second run updates `kurultai` entry only; other MCP servers preserved |
| R7 | Binary path resolution reuses existing `resolve_kurultai_bin()` |
| R8 | README Agents section + `docs/mac-dev.md` list the agent matrix and restart note |

### Actors / Flows

| ID | Actor / flow |
|----|----------------|
| A1 | Developer with Cursor and/or Claude Code and/or Codex CLI |
| F1 | Install binary → `init --agent all` → restart agents → tools appear |
| F2 | Init one agent without touching others' config files |

### Acceptance Examples

| ID | Example |
|----|---------|
| AE1 | Fresh home: `init --agent claude` creates `~/.claude.json` with `mcpServers.kurultai` |
| AE2 | Existing `~/.claude.json` with other servers: kurultai added; others unchanged |
| AE3 | Fresh `~/.codex/config.toml`: `[mcp_servers.kurultai]` present with command/args |
| AE4 | Existing Codex TOML with unrelated keys: preserved; kurultai table upserted |
| AE5 | `init --agent bogus` errors with supported list including cursor/claude/codex/all |

### Scope boundaries

**In:** R1–R8; unit tests under `src/mcp/init.rs` (temp HOME via env if already used, else tempfile + injected path helpers).

**Deferred (non-blocking)**

- Claude Desktop `claude_desktop_config.json` (different app; same JSON shape — follow-up)
- Project-scope `.mcp.json` / `.codex/config.toml` (user-scope only this slice)
- Smithery / marketplace one-click
- Calling `claude mcp add` / `codex mcp add` subprocesses (direct file merge is durable + testable)
- Local embeddings, ARC, GlitchTip, AppFlowy

---

## Planning Contract

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | Reuse stdio `kurultai mcp`; no new transport `(session-settled: user-approved — chosen over new HTTP/client-specific protocols: MCP tools already ship)` | Surgical; matches doctrine |
| KTD2 | Claude Code target = `~/.claude.json` user-scope `mcpServers`, not `~/.claude/settings.json` or Desktop path `(session-settled: user-directed — chosen over Desktop-first / settings.json: developer audience uses Claude Code; settings.json does not load MCP)` | Docs 2026 consensus |
| KTD3 | Codex target = TOML merge into `~/.codex/config.toml` `[mcp_servers.kurultai]` `(session-settled: user-directed — chosen over inventing mcp.json for Codex: official Codex MCP config is TOML)` | Official Codex shape |
| KTD4 | Add `AgentTarget::{Claude, Codex, All}`; `All` calls the three writers sequentially | Clear CLI; `clap` ValueEnum / FromStr |
| KTD5 | Extract shared JSON merge helper used by Cursor + Claude; separate TOML path for Codex | DRY without wrong abstraction |
| KTD6 | Prefer testable path injection (`wire_*_at(path)`) over only HOME mutation | Deterministic unit tests |
| KTD7 | This LFG ships agent init only — not local embeddings / sync `(session-settled: user-approved — chosen over embeddings-first Phase 5: inventory named Claude/Codex init as highest-leverage agent gap)` | Matches /lfg after inventory |

### Assumptions

- Claude Code reads top-level `mcpServers` in `~/.claude.json` for user scope (project `.mcp.json` deferred).
- Codex CLI + IDE share `~/.codex/config.toml`.
- `toml` crate already in workspace — use for parse/serialize; preserve comments only best-effort (lossy TOML round-trip accepted; document in risks).

### Risks

| Risk | Mitigation |
|------|------------|
| TOML round-trip drops comments | Upsert only `[mcp_servers.kurultai]` keys via parse→edit→emit; note in README if comments lost |
| Claude Code also has project-local entries in `~/.claude.json` | Only touch top-level `mcpServers`; do not rewrite project maps |
| HOME resolution in CI | Tests use temp dirs + `*_at` helpers |

### Pattern references

- Existing: `src/mcp/init.rs` (`wire_cursor`, refuse-malformed, atomic rename)
- CLI: `src/main.rs` `Commands::Init`
- Docs: `README.md` Agents table; `docs/mac-dev.md`
- Inspiration: engram-style per-agent setup (`docs/upstream-inspiration.md` #11)

---

## Implementation Units

### U1. AgentTarget + JSON merge helpers

**Goal:** Extend enum/parse; share Cursor/Claude JSON upsert.  
**Files:** `src/mcp/init.rs`, `src/main.rs` (help text)  
**Approach:** `AgentTarget::{Cursor, Claude, Codex, All}`; `wire_json_mcp_servers(path, entry)`; `wire_cursor` / `wire_claude` call it with correct paths; `wire_agent` match + All aggregates.  
**Test scenarios:**
- Parse accepts cursor/claude/codex/all (case-insensitive); rejects unknown with list
- JSON merge creates file when missing
- JSON merge preserves sibling servers
- Malformed JSON refuses overwrite

### U2. Codex TOML writer

**Goal:** Idempotent `[mcp_servers.kurultai]` upsert.  
**Files:** `src/mcp/init.rs`  
**Approach:** Read/parse TOML table; set `command` + `args` array; write atomically (temp + rename). Create parent `~/.codex` as needed.  
**Test scenarios:**
- Missing file → creates with kurultai table
- Existing unrelated keys preserved
- Second run updates command path only
- Invalid TOML → error, file unchanged

### U3. Docs + CLI surface

**Goal:** Operator-visible matrix.  
**Files:** `README.md`, `docs/mac-dev.md`, clap help on `--agent`  
**Approach:** Document `cursor` / `claude` / `codex` / `all`; note restart; link that MCP tools are unchanged.  
**Test scenarios:** N/A (docs) — verify strings in README by review

### U4. Verification gate

**Goal:** fmt/clippy/test green.  
**Files:** (none new beyond tests in U1–U2)  
**Approach:** Run repo standard gates.  
**Test scenarios:** Full suite passes including new init tests

---

## Verification Contract

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

Focus: `mcp::init` unit tests; no MCP server behavior changes expected.

---

## Definition of Done

- [ ] R1–R8 satisfied or explicitly deferred in PR body
- [ ] U1–U3 landed with tests for JSON + TOML writers
- [ ] Verification Contract green
- [ ] PR opened on `cursor/phase5-multi-agent-mcp-init-f07c`

---

## Open Questions

| Q | Status |
|---|--------|
| Also wire Claude Desktop path? | **Deferred** — follow-up if users ask |
| Project-scope `.mcp.json`? | **Deferred** — user-scope first |
| Prefer subprocess `claude mcp add`? | **Deferred** — file merge preferred for tests |

---

## Appendix: Research breadcrumbs (2026)

- Claude Code user MCP: `~/.claude.json` → `mcpServers` (not `~/.claude/settings.json`)
- Claude Desktop: OS-specific `claude_desktop_config.json` — out of scope
- Codex: `~/.codex/config.toml` → `[mcp_servers.NAME]` with `command` / `args`
- Existing Kurultai MCP tools unchanged in `src/mcp/server.rs`
