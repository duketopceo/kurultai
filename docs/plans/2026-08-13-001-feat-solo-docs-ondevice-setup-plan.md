---
title: "feat: solo on-device docs setup (init --docs)"
date: 2026-08-13
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "User /lfg — one-click (or close) on-device markdown ingest for solo"
depth: standard
origin: "User: easy setup so docs can be stored in the brain; CLI walkthrough; with/without API key & agents; UI matches CLI"
---

# feat: solo on-device docs setup (`init --docs`)

**Target repo:** `duketopceo/kurultai`  
**Audience:** solo operator (must not regress developers who already have config)  
**Base:** `main`  
**Process:** PR-only

## Goal Capsule

**Objective:** Make storing local markdown docs in the brain a **one-command** (or one extra flag) path: provision a folder on the device, enable `[sources.notes]`, write a **tagged** starter note, print a with/without-API-key walkthrough, and keep landing UI + README + agent prompt in lockstep with the CLI.

**Authority:** This plan > user `/lfg` this turn > README Quick start > `docs/AGENT_SETUP_PROMPT.md`.

**Stop when:** `kurultai init --docs` creates/enables an on-device markdown source; starter note has `tags:` so it is trusted; `--agent none` skips MCP; optional `--index` indexes immediately; CLI prints next steps for FTS-only vs API-key; README + agent prompt + `ui/index.html` / terminal demo match; Brain empty caption names the same command; tests cover provision + skip-agent; **no** R2/S3, **no** hub, **no** Brain 3D redesign.

**Do not:** Cloud object storage; Postgres hub; interactive TUI wizard; Windows one-click installer; changing synaptic visualization colors/layout; inferring visibility; mass-editing unrelated Brain chrome.

## Product Contract

### Summary

Today `init` writes a config **with no sources**. The user must hand-edit `config.toml` with a `root_path`, remember the ≥1-tag rule, then `index`. Closest one-click: `kurultai init --docs` (optional path, optional `--index`, optional `--agent none`).

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | `kurultai init --docs` provisions an on-device markdown folder (default: Documents/`kurultai`, else `~/kurultai`) and enables `[sources.notes]` with that absolute `root_path`. |
| R2 | `kurultai init --docs PATH` uses PATH (create if missing). Do not overwrite existing files. |
| R3 | Write a starter `.md` with YAML `tags:` so the quality gate indexes it as **trusted** (not quarantine). |
| R4 | `--agent none` skips MCP wiring; default remains `cursor`. `--agent all` unchanged. |
| R5 | `--index` runs a full index after provision. Without `--index`, print `kurultai index --full` as the next command. |
| R6 | After init, print a walkthrough covering: folder, config, MCP (or skipped), FTS **without** API key, vectors/LLM **with** `OPENROUTER_API_KEY` / `KURULTAI_API_KEY`, daemon + `http://127.0.0.1:8421/ui/`. |
| R7 | Init **without** `--docs` still works; print a one-liner pointing at `init --docs`. |
| R8 | Idempotent: second `--docs` does not wipe config/MCP/existing notes; may refresh `root_path`/`enabled`. |
| R9 | README Quick start, `docs/AGENT_SETUP_PROMPT.md`, `ui/index.html` quickstart + `ui/index.js` terminal demo show the same commands (install script / `init --docs`, not `cargo build --release` as the primary path). Fix stale GitHub org on the landing page to `duketopceo/kurultai`. |
| R10 | Brain UI empty state (0 memories caption and/or stats panel) names `kurultai init --docs` — no 3D graph redesign. |

### Actors / flows

- A1 Solo, no API key, no agent — FTS-only docs brain  
- A2 Solo + Cursor MCP  
- A3 Solo pointing `--docs` at an existing vault  
- F1 First-time `--docs` · F2 Re-run · F3 Init without `--docs`

### Scope boundaries

**In:** CLI init flags, config merge for `[sources.notes]`, starter note, walkthrough text, docs/landing/brain caption, tests.  
**Out:** R2/S3, hub, team installer, Brain visualization experiment.

### Acceptance examples

- AE1. Empty machine: `init --docs --agent none --index` → folder + tagged note + atoms searchable via FTS with no API key.  
- AE2. `init --docs /tmp/vault` on existing markdown with tags → source enabled; files not overwritten.  
- AE3. `init --agent none` without `--docs` → config exists, walkthrough mentions `--docs`.  
- AE4. Landing copy-paste block matches README (`init --docs`, then `index`/`daemon`).

## Planning Contract

### Key Technical Decisions

- KTD1. **CLI flag, not a new binary / GUI installer.**  
  Provenance: `user-directed`.  
  Rejected: Separate one-click app; cloud provision.  
  Reason: Kernel is on-device CLI; install script already exists.

- KTD2. **`--docs` optional path; default Documents/kurultai.**  
  Provenance: `user-directed` (on-device docs folder).  
  Rejected: Always require editing TOML; default `~/.config` for notes (hides user files).  
  Reason: Notes should live where humans put documents.

- KTD3. **`--index` opt-in when pointing at an existing PATH; for default new folder, still opt-in `--index` so init stays fast.**  
  Provenance: `user-approved` (headless).  
  Rejected: Always index on `--docs` (surprise on large vaults).  
  Reason: Provision quickly; one extra flag is the one-click index.

- KTD4. **Do not change 3D Brain visuals; caption/stats only when empty.**  
  Provenance: `user-directed` (UI match) + standing Brain-visual preference.  
  Rejected: Setup wizard overlay on the graph.  
  Reason: Match CLI without replacing the brain focal point.

- KTD5. **Starter note must include `tags:`.**  
  Provenance: `user-approved` (existing quality gate).  
  Rejected: Untagged welcome file.  
  Reason: Untagged markdown quarantines and is excluded from default search.

### Technical design

- Extend `Commands::Init` in `src/main.rs`: `--docs [PATH]`, `--index`, `AgentTarget` gains `none`.  
- Provision helpers beside `ensure_default_config` in `src/mcp/init.rs` (or small `src/setup.rs` if init.rs would bloat): resolve default dir via `dirs::document_dir()`, `create_dir_all`, write starter if `welcome.md` absent, upsert `[sources.notes]` in the TOML file (`toml::Value` merge; do not delete other sources).  
- Print walkthrough from one function so CLI tests can assert substrings.  
- Rebuild `website/` → `ui/` only if Brain caption/stats change (embedded assets).

### Assumptions

- A1. Users who already have `[sources.notes]` keep their files; we only set `enabled`/`root_path` when `--docs` is passed.  
- A2. `KURULTAI_CONFIG` continues to redirect config path for tests.

### Sequencing

U1 CLI provision → U2 walkthrough + `--index` → U3 docs/landing/brain empty caption → tests.

## Implementation Units

### U1 — `init --docs` / `--agent none`

**Goal:** Provision markdown source on disk and config.  
**Files:** `src/main.rs`, `src/mcp/init.rs`, `src/mcp/mod.rs`  
**Tests:** `tests/cli_smoke.rs` (temp `KURULTAI_CONFIG` + docs dir)  
**Scenarios:** default path creates starter with tags; `--docs PATH` creates dir; `--agent none` writes no MCP files; re-run does not clobber welcome.md; existing `[sources.other]` preserved.

### U2 — Walkthrough + optional index

**Goal:** Printed next steps; `--index` indexes.  
**Files:** `src/main.rs`  
**Tests:** stdout contains FTS-without-key and daemon UI URL; `--index` stdout mentions the notes source.

### U3 — Surface parity

**Goal:** README, agent prompt, landing, Brain empty caption match CLI.  
**Files:** `README.md`, `docs/AGENT_SETUP_PROMPT.md`, `AGENT_SETUP_PROMPT.md` (root pointer if needed), `ui/index.html`, `ui/index.js`, `website/src/App.tsx` and/or `website/src/components/StatsPanel.tsx`, rebuild `ui/`  
**Tests:** none beyond string presence in CLI tests; visual check not required in CI.

## Verification Contract

- `cargo test --locked` (at least `cli_smoke` + `mcp::init` unit tests)  
- `cargo fmt` / clippy as CI  
- If website changed: `cd website && npm run build` so embedded `ui/` matches source

## Definition of Done

- AE1–AE4 satisfied  
- Plan KTDs preserved  
- PR opened with walkthrough in description  
- No R2/hub/Brain-viz scope
