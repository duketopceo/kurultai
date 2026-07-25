---
title: "feat: v1 Agent Zero batch (#72–#76)"
date: 2026-07-25
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "GitHub #72 #73 #74 #75 #76 · docs/agent-zero · master #27"
depth: deep
origin: "User /lfg for all those issues at once; op CLI SSH for push/PR"
---

# feat: v1 Agent Zero batch (#72–#76)

**Target repo:** `duketopceo/kurultai`  
**Audience:** developer → solo → team  
**Base:** branch including personal installer (#72 work)  
**Process:** PR-only · push via `github.com-personal` (1Password agent → duketopceo)

## Goal Capsule

Ship one PR that lands the **Agent Zero v1 pack** at a right-sized product depth:

| Issue | Slice in this PR |
|-------|------------------|
| #72 | Personal install script (already on branch; keep + polish) |
| #73 | Scheduler status + optional nightly full sync + idle-aware poll skip |
| #74 | Multi-hop expansion (tag/related chain) on `ask` + `graph_chain` |
| #75 | Citation contract fields (path, section, hash, char range) |
| #76 | Minimal local dashboard HTML + `/api/status` |

**Stop when:** units pass tests; README notes v1 batch; PR opened as duketopceo.

**Do not:** team/company installer; PostgreSQL/SSO; full knowledge-graph UI with D3; Windows installer; llama.cpp; auto-merge.

**Assumption (LFG headless):** “all those issues at once” = one implementation-ready batch PR with thin-but-real product slices per issue, not every acceptance line from Agent Zero drafts.

---

## Product Contract

### Requirements

| ID | Issue | Requirement |
|----|-------|-------------|
| R72 | #72 | Personal `scripts/install/install.sh` + tests + README (shipped; maintain) |
| R73a | #73 | Daemon records last incremental/full sync times; expose on `/api/status` |
| R73b | #73 | Optional `runtime.nightly_full_sync_hour` (0–23 local) triggers `index_all(true)` once per day |
| R73c | #73 | Optional idle: if no watch/poll activity for `inactivity_threshold_hours`, skip redundant poll cycles (log + continue) |
| R74a | #74 | After primary hybrid search, expand by shared tags (second hop) before synthesize |
| R74b | #74 | `Answer.graph_chain: Vec<String>` lists source_id path of hops used |
| R75a | #75 | `Citation` gains optional `file_path`, `section`, `title_hash`, `excerpt_start`, `excerpt_end` |
| R75b | #75 | Populate from atom metadata / source_id / content position when known |
| R76a | #76 | `GET /ui` serves a minimal HTML dashboard (status + search form) |
| R76b | #76 | `GET /api/status` JSON: health, atom count, poll/watch last times when available |

### Scope boundaries

**In:** R72–R76b above.

**Deferred**

- Team/company installers, Helm, SSO  
- Auto-learned embedding edges; deep graph viz  
- WebSocket live events (use polling in UI for this slice)  
- Per-source poll intervals in config schema beyond existing fields  

---

## Planning Contract

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | Shared `DaemonRuntime` / `Arc<DaemonStatus>` updated by poll/watch | Status without DB migration |
| KTD2 | Multi-hop = tag FTS expansion + RRF-merge with primary hits (reuse hybrid) | No Edge table yet; surgical |
| KTD3 | Citation fields optional with serde defaults | Backward compatible JSON |
| KTD4 | Dashboard = single static HTML string in `src/http/ui.html` or `include_str!` | No frontend build |
| KTD5 | Nightly full sync via tokio sleep-to-hour loop in daemon | Matches Agent Zero intent without cron crate |

### Implementation Units

### U1. Plan (this file)
### U2. #72 installer (verify present)
### U3. #75 citations contract
### U4. #74 multi-hop ask + graph_chain
### U5. #73 scheduler status + nightly + idle
### U6. #76 /api/status + /ui dashboard
### U7. Tests + README + agent-zero INDEX

---

## Verification Contract

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

## Definition of Done

- [ ] All R* covered or explicitly deferred in PR body  
- [ ] Tests green  
- [ ] PR opened with duketopceo identity (1Password SSH / token)  
- [ ] Issues #72–#76 linked  
