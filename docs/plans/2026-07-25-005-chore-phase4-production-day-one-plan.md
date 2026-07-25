---
title: "chore: Phase 4 production day-one readiness"
date: 2026-07-25
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "User /lfg phase 4 + production today; Phase 4 product closed (#8 / Milestone 4 / #62/#63/#64)"
depth: standard
origin: "User /lfg phase 4. We are going into production today I'll be using this"
---

# chore: Phase 4 production day-one readiness

**Target repo:** `duketopceo/kurultai`  
**Audience:** solo (Phase 4 exit) using shipped connectors in production today  
**Base:** `main` after Phase 4 closeout (#64) and Phase 5 poll/watch (#65/#66)  
**Process:** PR-only

## Goal Capsule

**Objective:** Make the **already-shipped Phase 4 solo stack** production-usable today: accurate config examples (markdown + Dayflow + Pond + GitHub), a copy-paste solo config, a short day-one runbook, and README status sync so the operator path is honest.

**Authority:** This plan > `phase-4-complete.md` / #64 > #27 / README.

**Stop when:** Operator can copy `examples/config.solo.toml` → `~/.config/kurultai/config.toml`, edit paths, run `index` + `daemon`/`mcp`, and README no longer advertises AppFlowy-first or stale “daemon planned” / Phase 5 “notify only planned” status.

**Do not:** Rebuild Dayflow/Pond/GitHub connectors; implement Composio, plugins (#14), AppFlowy (#4), llama.cpp, ARC (#20), GlitchTip (#35); close Milestone 5; invent new daemon features.

**Assumption (LFG headless):** “/lfg phase 4” after product closeout = **day-one production readiness for the Phase 4 stack**, not a second connector rebuild. User will use Kurultai in production today.

**Product Contract preservation:** new bootstrap.

---

## Product Contract

### Summary

Phase 4 connectors and Phase 5 daemon poll/watch are on `main`. Docs still lead with AppFlowy (deferred) and understate what is live. Ship operator-facing config + runbook so production day-one works without reading six plans.

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Add `examples/config.solo.toml` with commented, ready-to-edit sources: markdown, dayflow, pond, github (AppFlowy absent or clearly deferred comment only). |
| R2 | README Configuration section uses the solo Phase 4 shape (not AppFlowy-first). |
| R3 | Add `docs/production-day-one.md` — build → init → config → index → search/ask → mcp/daemon checklist for today. |
| R4 | README Architecture status rows honest: synthesis ✅, HTTP daemon ✅ (+ poll/watch), connectors list already OK. |
| R5 | README Phase 5 row notes poll (#65) + notify (#66) shipped; llama.cpp / ARC remain later. |
| R6 | Roadmap checklist: mark production partial (daemon poll/watch done) without claiming Milestone 5 closed. |
| R7 | Link runbook + example config from README Quick Start / Configuration. |
| R8 | Loader test (or small unit) proves example solo TOML parses with expected source kinds. |

### Actors / flows

- A1 Operator (Luke) · F1 copy example config · F2 index sources · F3 use MCP/daemon in Cursor today · F4 CI

### Scope boundaries

**In:** R1–R8 — examples, docs, README sync, one parse test.  
**Out:** New connectors, auth, remote bind, enterprise deploy (#29), coverage % gates.

### Acceptance examples

- AE1. `examples/config.solo.toml` exists; includes `kind = "markdown"`, `"dayflow"`, `"pond"`, `"github"`.  
- AE2. README config snippet does not enable AppFlowy by default.  
- AE3. `docs/production-day-one.md` lists the day-one command sequence.  
- AE4. Unit test loads the example file (or its embedded twin) and sees four source kinds.  
- AE5. CI green on the PR.

---

## Planning Contract

### Assumptions

| Decision | Class | Rejected | Why |
|----------|-------|----------|-----|
| Day-one docs/config over rebuild Phase 4 | session-settled: user-directed — chosen over rebuild connectors: Phase 4 already closed; user going into production today to use it | Re-implement #62/#63 | Product exit met |
| Prefer solo example over enterprise deploy docs | session-settled: user-directed — chosen over #29/#15 company deploy: personal production use today | Full env matrix | Audience = solo using Phase 4 |
| Keep AppFlowy deferred in examples | prior / phase-4-complete | Enable AppFlowy stub | #4 open, non-blocking |

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | Example lives in `examples/config.solo.toml` (repo), not only README | Copy-paste + testable |
| KTD2 | Paths are placeholders (`/path/to/...`); Dayflow default path documented as optional omit | Cross-machine; Dayflow resolves home default |
| KTD3 | Docs-first PR; one parser test only | No behavior change beyond docs/test |
| KTD4 | Do not change `default_config_toml()` empty-sources shape | `init` stays minimal; example is the production template |

### Sequencing

U1 example + test → U2 runbook → U3 README sync → U4 PR CI.

---

## Implementation Units

### U1. Solo example config + parse test

**Files:** `examples/config.solo.toml`, `src/config/loader.rs` (test) or `tests/config_solo_example.rs`

**Approach:** Ship commented TOML with four sources. Test reads the example path from crate root (or `include_str!`) and asserts kinds.

**Test scenarios:**
1. File parses without error.  
2. Source kinds include Markdown, Dayflow, Pond, GitHub.  
3. Markdown/github have non-empty `root_path` keys in extra.

### U2. Production day-one runbook

**Files:** `docs/production-day-one.md`

**Approach:** Short checklist: build/release install, `init --agent cursor`, copy example, enable sources, `index --full`, `search`/`ask`, `mcp` / `daemon`, FTS-without-key note, Dayflow Mac-only note, Pond binary note.

**Test scenarios:** Manual doc review — commands match README CLI surface.

### U3. README honesty sync

**Files:** `README.md`

**Approach:** Replace Configuration snippet; link example + runbook; fix Architecture status table; update Phase 5 row and roadmap checklist partial production.

**Test scenarios:** `rg` checks for links to `examples/config.solo.toml` and `docs/production-day-one.md`; AppFlowy not `enabled = true` in config snippet.

### U4. Green PR

**Verify:** fmt/clippy/tests as repo CI.

---

## Verification Contract

```bash
test -f examples/config.solo.toml
test -f docs/production-day-one.md
rg -n 'config\.solo\.toml|production-day-one' README.md
rg -n 'kind = "dayflow"|kind = "pond"|kind = "github"' examples/config.solo.toml
cargo test --locked
cargo clippy --all-targets -- -D warnings
```

---

## Definition of Done

- [ ] Example solo config + parse test  
- [ ] Day-one runbook  
- [ ] README config/architecture/Phase 5 status accurate  
- [ ] Green PR merged path ready for operator use today  
