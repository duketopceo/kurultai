---
title: "chore: clean terminal install + README redesign"
date: 2026-07-25
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "User /lfg those — install cleanup + README design; no v* cut"
depth: standard
origin: "User asked to clean terminal install, /ce-plan design README, package release?/v#?; then /lfg those"
---

# chore: clean terminal install + README redesign

**Target repo:** `duketopceo/kurultai`  
**Audience:** developer / solo (Mac laptop, public README)  
**Base:** `cursor/mac-dev-install-f07c` (#70) on top of `main`  
**Process:** PR-only (update #70)

## Goal Capsule

Make public install and README **honest and quiet** for Mac/dev loading: cargo-first install that does not pretend a GitHub Release exists, a redesigned README that matches shipped Phase 1–5 reality, and release **plumbing kept** without cutting a `v*` tag.

**Stop when:** install script is quiet/clear with cargo fallback; README is redesigned (install → config → status); `docs/mac-dev.md` aligned; release workflow remains for later; **no git tag / no `gh release create`**; CI green on PR.

**Do not:** cut `v0.1.0` or any GitHub Release; publish crates.io; merge closed PRs #67–#69; rewrite connectors/daemon code; AppFlowy enablement.

### Settled decisions (session-settled KTDs)

| KTD | Decision | Provenance | Rejected | Reason |
|-----|----------|------------|----------|--------|
| KTD1 | Do **not** cut a `v*` tag or GitHub Release in this run | user-approved (LFG of lean) | Cut `v0.1.0` now | No binary assets yet; curl→binary would fail; Cargo.toml already `0.1.0` |
| KTD2 | Keep release workflow plumbing for later | user-directed | Delete release.yml | Prep-only; tag when ready |
| KTD3 | Install path is **cargo-first**; curl script is optional wrapper | user-directed | Advertise curl as primary “npm install” that needs a release | Honesty for Mac load today |
| KTD4 | README redesign is docs/UX only; stay on `environment=dev` + debug | user-directed | Prod-first day-one defaults | User still in debug/Mac load |

---

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | `scripts/install.sh` is quiet, detects missing `cargo`, clear PATH hints; prefers release binary only when latest release asset exists; otherwise `cargo install --git`. |
| R2 | `scripts/install.ps1` same honesty for Windows (cargo message if no release). |
| R3 | README redesigned: brand → one-line value → **Install** → **Mac/dev** → short Why → honest Status → Config (dev, real connectors, AppFlowy off/deferred) → Agents/MCP → Environments (compressed) → Roadmap (compressed) → Contributing/License/Name. |
| R4 | README status matches `main`: markdown/Dayflow/Pond/GitHub FS, RRF search, synthesis/ask, CLI+MCP+daemon poll+watch; AppFlowy deferred; no “daemon planned”. |
| R5 | `docs/mac-dev.md` points at cargo-first install; no lie that `main` curl works before merge (branch-aware note OK until merge). |
| R6 | `.github/workflows/release.yml` kept (tag-triggered); README notes “releases not published yet”. |
| R7 | No `git tag`, no `gh release create`, no crates.io publish. |

### Scope boundaries

**In:** `scripts/install.sh`, `scripts/install.ps1`, `README.md`, `docs/mac-dev.md`, optional tiny note in release workflow comments, this plan.

**Out:** binary builds, tagging, crates.io, closed PR content (#67–#69), code changes outside docs/scripts/workflow.

### Acceptance examples

- AE1. With no GitHub Release, `install.sh` runs `cargo install --git` (or exits with clear “install Rust” if no cargo).  
- AE2. README Install section leads with cargo; curl is secondary / “when releases exist”.  
- AE3. README does not enable AppFlowy; lists live connectors; mentions daemon.  
- AE4. Repo has no new `v*` tags after this PR.

---

## Planning Contract

### Key technical decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Cargo-first install | Works today without release assets |
| D2 | Compress phase tables | README was a work-order dump; keep checklist short |
| D3 | Leave Cargo.toml at `0.1.0` | Already set; tag later when releasing binaries |

### Sequencing

1. U1 — Clean install scripts  
2. U2 — README redesign  
3. U3 — mac-dev + release note sync  

---

## Implementation Units

### U1. Clean terminal install scripts

**Files:** `scripts/install.sh`, `scripts/install.ps1`

**Work:** Quiet progress lines; check `command -v cargo` before fallback; fail with rustup URL if missing; keep asset names; fix REPO default `duketopceo/kurultai`.

**Tests:** Manual / script syntax (`bash -n`); no Rust unit tests required.

**Scenarios:**
1. No release → cargo path message  
2. No cargo → non-zero exit + rustup hint  
3. `bash -n scripts/install.sh` passes  

### U2. README redesign

**Files:** `README.md`

**Work:** Restructure per R3–R4; keep ASCII yurt brand; drop AppFlowy `enabled = true`; sync Phase 5 daemon status; point install to cargo-first.

**Tests:** Doc review against AE2–AE3 (`rg` checks).

**Scenarios:**
1. `rg -n 'cargo install --git' README.md`  
2. `rg -n 'enabled = true' README.md` does not include appflowy  
3. `rg -n 'daemon' README.md` present; no “daemon later” as only status  

### U3. mac-dev + release honesty

**Files:** `docs/mac-dev.md`, `.github/workflows/release.yml` (comment only if needed)

**Work:** Align mac-dev with cargo-first; note releases unpublished; workflow stays tag-gated.

**Scenarios:**
1. mac-dev does not claim release binary required  
2. No `v*` tag created by this work  

---

## Verification Contract

- `bash -n scripts/install.sh`  
- `rg` acceptance checks above  
- CI on PR (fmt/clippy/tests unchanged for docs-heavy PR)  

---

## Definition of Done

- [ ] U1–U3 complete  
- [ ] No git tag / no GitHub Release created  
- [ ] PR #70 updated (or successor) with plan link  
- [ ] CI decided green or babysit residuals surfaced  
