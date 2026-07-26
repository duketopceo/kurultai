---
title: "chore: Jul 25–26 PR window cleanup + review leftovers"
date: 2026-07-26
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
origin: "LFG pipeline — audit PRs created/merged/closed 2026-07-25..2026-07-26; ship surgical follow-up (ce-simplify + still-valid review leftovers)"
authority: "Settled-decisions brief (user-directed) · AGENTS.md Brain UI prefs · origin/main@v0.3.0 (#96) · open #97/#98 coordination"
---

# chore: Jul 25–26 PR window cleanup + review leftovers

**Target repo:** `duketopceo/kurultai`
**Base:** current `origin/main` (post-[#96](https://github.com/duketopceo/kurultai/pull/96) / tag `v0.3.0`)
**Audience:** `ce-work` → open PR via LFG

## Goal Capsule

**Objective:** After auditing merged and open PRs from **2026-07-25** and **2026-07-26**, land one focused follow-up PR that applies **ce-simplify-style cleanup** and **still-valid** CodeRabbit / review leftovers — especially where LFG/simplify was skipped on large merges (#96, #90, #84, #93).

**Authority:** This plan > settled-decisions brief > AGENTS.md (Brain visuals) > residual findings on merged PR threads > open #97/#98 (coordinate, do not collide).

**Stop when:** Listed units land on a green PR against `main`; each claimed leftover was re-verified still present before editing; Brain visuals unchanged except proven broken behavior / a11y-focus / GPU-leak / hover-perf; README / `kind = "json"` loader left to #97/#98 (or rebased after they merge).

**Do not:** Full rewrite of v0.3.0 features; Phase 6 product work; Brain palette/layout redesign; pile commits onto open #97/#98; “fix” stale review comments already invalidated by later merges.

**Assumption:** Solo research during `ce-plan` confirmed a concrete leftover set on `main@9d2ae12`; `ce-work` must **re-verify** before each edit (main may move).

**Product Contract preservation:** new bootstrap (`product_contract_source: ce-plan-bootstrap`).

**Execution profile:** code; research-first in `ce-work`; prefer mechanical simplify + quick-win correctness; open one follow-up PR.

**Tail ownership:** LFG / `ce-work` after this plan.

---

## Product Contract

### Summary

Two days of shipping left mechanical debt: ephemeral agent receipts on `main`, false “no unresolved findings” residual docs, CodeRabbit threads still true after #96, and a few correctness/security quick wins (#90 touch 404 mapping, #84 empty `embed.backend`, #93 release workflow token scope). Open [#97](https://github.com/duketopceo/kurultai/pull/97) (Phase 5 closeout) and [#98](https://github.com/duketopceo/kurultai/pull/98) (README + `json` source kind) already own README/tracker surfaces — this PR must not fight them.

### Problem Frame

Without a surgical pass, `main` keeps quarantinable root markdown, dead review artifacts, known UI interaction bugs (search clear leaves dropdown), GPU leaks on timeline drag, and release-workflow over-privileged checkout — while a full “fix everything from #90–#96” rewrite would thrash Brain UX against AGENTS.md preferences.

### Requirements

#### Research discipline

- R1. `ce-work` starts with a short re-audit of the candidate leftover list against current `origin/main` (and whether #97/#98 merged). Fix only items still valid; record skips with reason in the PR body.
- R2. Scope window is PRs created/merged/closed on 2026-07-25 and 2026-07-26 only — not whole-repo history.

#### Simplify + hygiene

- R3. Remove ephemeral agent artifacts from the repo root: `RETURN_RECEIPT_BACKEND.md`, `RETURN_RECEIPT_DB.md`, `RETURN_RECEIPT_UI.md`, and `residual-review-findings/` (or relocate under an ignored/local-only path). Do **not** “fix” them by adding `tags:` so they enter default search.
- R4. Fix the stale checklist path in `docs/plans/2026-07-26-002-feat-v0.3.0-unification-plan.md` (`plans/...` → `docs/plans/...`) or delete the obsolete checklist item if the plan is historical-only.
- R5. Prefer deleting unused / lying leftovers over adding comments or dual paths (ce-simplify posture).

#### Still-valid correctness / security leftovers

- R6. `/api/touch` must map missing atoms to **404**, not **400**, when the store reports not-found (today `SqliteVecStore::touch_access` returns `Err`, so `Ok(None)` is effectively unreachable).
- R7. Touch / promote HTTP responses used by the Brain UI must not dump full atom `content` to the browser when a lean status payload suffices (align with CONTRIBUTING “never full content by default”).
- R8. Explicit empty/whitespace `embed.backend = ""` must fail config validation (not silently become `None`).
- R9. Local-embed commented template in default config must not leave active OpenRouter `model`/`dimension` lines that conflict when users uncomment local settings (document-correct pairing).
- R10. Release workflow: narrow `contents: write` to the publish job; set `persist-credentials: false` on build checkout (still-valid #93 findings).

#### Brain UI — broken behavior only

- R11. Clearing the search box hides `#search-dropdown` (still broken: `search("")` → `loadAtoms()` without `hidden = true`).
- R12. Timeline/graph rebuild disposes Three.js geometries/materials (or equivalent) before `nodes.clear()` / `edges.clear()`.
- R13. Hover path does not rebuild inspector / full recolor on every `pointermove` over the **same** node (gate on hover-id change).
- R14. Search input has a visible focus indicator (outline was removed with no replacement).
- R15. Landing `ui/index.js` theme `localStorage` access is guarded so init cannot abort; copy buttons use `type="button"`; canvas particle resize clamps positions (still-valid #96 findings).
- R16. No Brain visual redesign (palette, layout modes, chrome, camera framing) unless a fix above unavoidably requires a minimal style token — prefer CSS focus ring only.

#### Open PR coordination

- R17. Do **not** edit `README.md` in this PR while #97/#98 are open (or land after them and rebase). Do **not** add `kind = "json"` to `src/config/loader.rs` — owned by #98.
- R18. Ship as a **new branch + new PR** from `main`, not commits onto #97/#98.

### Actors

- A1. Implementing agent (`ce-work` / LFG) — research, fix, open PR
- A2. Maintainer — merge order vs #97/#98
- A3. CI — `cargo fmt` / `clippy -D warnings` / `cargo test --locked` / audit

### Flows

- F1. Research gate
  - **Trigger:** `ce-work` starts
  - **Steps:** Fetch `origin/main`; check #97/#98 state; re-verify each R* target; drop stale items
  - **Outcome:** Working leftover checklist in PR description
- F2. Surgical PR
  - **Actors:** A1, A3
  - **Outcome:** Green PR with simplify + leftover fixes; Brain visuals preserved
- F3. Merge coordination
  - **Actors:** A2
  - **Outcome:** If #97/#98 merge first → rebase this PR; if this merges first → those PRs rebase (README-only conflict expected)

### Acceptance Examples

- AE1. Covers R3, R5 — Given the follow-up PR, When grepping the tree, Then no `RETURN_RECEIPT_*.md` or `residual-review-findings/` remain tracked.
- AE2. Covers R6 — Given a missing `atom_id`, When `POST /api/touch`, Then response is 404 with JSON error (not 400).
- AE3. Covers R11 — Given an open search dropdown, When the search input is cleared, Then `#search-dropdown` is hidden.
- AE4. Covers R10 — Given `.github/workflows/release.yml`, When inspecting jobs, Then build checkout has `persist-credentials: false` and workflow-level `contents: write` is not granted to build-only steps.
- AE5. Covers R17 — Given the follow-up PR diff, When listing files, Then `README.md` is absent.

### Scope Boundaries

**In:** Ephemeral artifact removal; plan checklist path; touch/embed/config leftovers; release.yml token hygiene; Brain/landing **broken-behavior** UI leftovers from #96/#90/#92 that remain on `main`.

**Deferred for later:**

| Item | Why |
|------|-----|
| README rewrite / install tag `v0.2.0` → `v0.3.0` | #98 |
| `SourceKind::Json` in config loader | #98 (connector exists; loader still maps unknown → `Custom`) |
| Phase 5 closeout docs/script/README ✅ | #97 |
| Full keyboard orbit/zoom for `role="application"` | Heavy a11y; out of surgical scope |
| CDN SRI for unpkg Three.js | Heavy; cloud egress already blocks unpkg |
| Batch `touch_access_many` single SQL transaction | Valid #90 perf; larger store change — optional stretch only if time |
| Success-path `request_id` on every bare `Vec` handler | Partial already on some routes; widen later |
| Stylelint on `ui/index.css` | Not a CI gate today |
| `#api/open` localhost abuse hardening | Product/security design needed; not a drive-by |
| Camera preserve on resize (#92 Codex) | Only if still trivial; defer if it fights framing prefs |
| Phase 6 features | Explicitly rejected |

**Outside this product's identity for this PR:** Redesigning Brain HUD/palette; rebuilding ingestion/embeddings; editing open #97/#98 branches.

### Dependencies

- `origin/main` includes #96 (`v0.3.0`)
- Open #97, #98 may merge during execution — rebase discipline required
- AGENTS.md Brain visual prefs bind R16

### Sources

- Merged window: #63–#66, #70, #77, #82–#96 (plus closed supersessions #67–#69)
- Open: #97, #98
- CodeRabbit unresolved threads on #96, #90, #84, #83, #93 (spot-checked against `main@9d2ae12`)
- Prior pattern: [2026-07-25-008-fix-pr77-coderabbit-followup-plan.md](2026-07-25-008-fix-pr77-coderabbit-followup-plan.md)
- AGENTS.md; CONTRIBUTING.md quality gates

---

## Planning Contract

### Key Technical Decisions

- KTD1. Window-scoped audit only (session-settled: user-directed — chosen over unlimited historical audit: “all from today… and all from yesterday.”)
- KTD2. Priority is ce-simplify + review/fix leftovers, not new features (session-settled: user-directed — chosen over Phase 6 / redoing v0.3.0 unification.)
- KTD3. Execute full LFG to an open PR (session-settled: user-directed — chosen over report-only review.)
- KTD4. New branch from `main`; do not push onto #97/#98 (session-inferred headless default for open-area “fix open PRs vs main” — coordinate/rebase instead of hijacking those branches.)
- KTD5. Delete ephemeral receipts rather than tag them for FTS (session-inferred headless default — tags would index noise; simplify removes the artifacts.)
- KTD6. Brain UI changes limited to broken behavior / focus / dispose / hover gate (session-inferred headless default for open-area UI aggressiveness — honors AGENTS.md “ask before changing Brain visuals”; no evidence user wants redesign.)
- KTD7. Map store not-found errors to HTTP 404 in `api_touch` (and promote if same pattern) rather than widening `Store::touch_access` to `Result<bool>` unless a clean Option API is already nearby — prefer smallest diff that makes 404 reachable.
- KTD8. Prefer lean JSON for touch success (`ok`, `request_id`, `atom_id`, tier/title fields) over returning full `KnowledgeAtom`. Planning-time grep: `ui/brain.js` has **no** `/api/touch` caller — treat lean payload as API hygiene with low UI regression risk; still re-grep in U1 before shipping.
- KTD9. Reject explicit empty/whitespace `embed.backend` in the loader (before the empty→`None` filter), not only in later `validate` — omitted key stays `None`; present-but-blank is a config error. *(doc-review resolution — one clear correct place)*

### Assumptions

- A1. #97 and #98 remain the owners of README + Phase 5 tracker hygiene + `json` loader wiring; this PR skips those files even if README still says `v0.2.0` on `main`. *(headless default for open-area conflict)*
- A2. CodeRabbit “Stored XSS via onclick in `src/http/mod.rs`” from #84 is **stale** on current `main` (embedded HTML dashboard removed); do not reintroduce dashboard HTML to “fix” it.
- A3. `website/` Vite root already points at `../ui` — no duplicate Brain dashboard to sync beyond editing `ui/` sources.
- A4. Docs-adjacent checklist/path fixes do not require browser verification; Brain interaction fixes should be sanity-checked in Vite preview or daemon `/ui/` when the environment allows (unpkg may be blocked — SVG/fallback path still exercises search dropdown DOM).
- A5. Stretch items (batch touch SQL, resize camera preserve) are optional and must not block the PR if they expand scope.
- A6. `api_promote` already returns a lean success body and is **out of U3** unless U1 finds a not-found→400 twin; do not expand touch work into promote.

### High-level design

```
origin/main
    │
    ├─ U1 re-verify leftovers + skip stale
    ├─ U2 delete receipts / residual / checklist path
    ├─ U3 HTTP touch 404 + lean payload + embed.backend "" reject + config template
    ├─ U4 release.yml permissions / persist-credentials
    ├─ U5 Brain/landing interaction leftovers (no visual redesign)
    └─ U6 (optional) mcp atomic_write mode + chmod after write
         │
         └─ open PR (rebase if #97/#98 landed)
```

### Sequencing

U1 → U2 → U3 → U4 → U5 → U6(optional). U5 may run parallel to U4 after U1. Do not touch README at any step.

### Risks

| Risk | Mitigation |
|------|------------|
| Merge conflict on README with #97/#98 | Never edit README here |
| Touch payload shape breaks Brain UI | Grep `api/touch` / response fields in `ui/brain.js` before shrinking |
| Dispose logic regresses WebGL | Dispose only owned geometries/materials; smoke-load graph once |
| Over-fixing #90 perf (N writes) balloons PR | Defer batch SQL unless trivial |

---

## Implementation Units

### U1. Research re-verification gate

**Goal:** Confirm each leftover against live `main` and open-PR state before editing.

**Files:** none (read-only): use `gh pr view 97/98`, `git fetch`, spot-check paths listed in R3–R15.

**Requirements:** R1, R2, R17

**Approach:** Produce a short “still valid / stale / deferred” table in the PR body. Drop A2-class stale items (XSS onclick) without code churn.

**Test scenarios:** N/A (process gate).

**Verification:** PR description contains the re-verification table.

**Dependencies:** none

---

### U2. Ephemeral artifact + plan-path simplify

**Goal:** Remove agent receipt noise and fix the unification plan checklist path mismatch.

**Files:**
- delete: `RETURN_RECEIPT_BACKEND.md`, `RETURN_RECEIPT_DB.md`, `RETURN_RECEIPT_UI.md`, `residual-review-findings/release-v0.3.0.md` (and directory if empty)
- edit: `docs/plans/2026-07-26-002-feat-v0.3.0-unification-plan.md` (checklist path)

**Requirements:** R3, R4, R5

**Approach:** `git rm` the receipts/residual dir. Fix checklist to `docs/plans/2026-07-26-002-feat-v0.3.0-unification-plan.md` or remove the obsolete unchecked item if the release already shipped.

**Test scenarios:** N/A (docs/hygiene) — verify files untracked after commit.

**Verification:** `git ls-files 'RETURN_RECEIPT*' 'residual-review-findings/*'` empty; checklist path grep clean.

**Dependencies:** U1

---

### U3. Touch semantics + embed config leftovers

**Goal:** Correct `/api/touch` not-found mapping and harden empty `embed.backend`; fix local-embed template pairing.

**Files:**
- `src/http/mod.rs` (`api_touch` + tests only)
- `src/mcp/brain.rs` (only if lean payload needs a dedicated method)
- `src/config/loader.rs` (reject explicit empty backend per KTD9)
- `src/config/mod.rs` (commented local-embed template coherence)
- tests adjacent in those modules

**Requirements:** R6, R7, R8, R9

**Approach:**
1. When `touch_access` errors with atom-not-found, return 404; keep other store errors as 5xx or 400 consistently with nearby handlers.
2. Return lean success JSON (`ok`, `request_id`, `atom_id`, plus any tier/title fields already cheap to include). No `/api/touch` caller in `ui/brain.js` at planning time — still re-grep in U1.
3. Per KTD9: present-but-blank `embed.backend` → loader config error; omitted → `None`.
4. In default config template, group local-embed overrides so uncommenting local `backend`/`model`/`dimension` does not leave conflicting OpenRouter lines active — comments-only change preferred.
5. Do not change `api_promote` unless U1 finds a separate still-valid leftover.

**Test scenarios:**
- Missing atom → `POST /api/touch` → 404 + JSON `error` + `request_id`.
- Existing atom → 200; body lacks full raw `content` (or only fields UI needs).
- Config `backend = ""` → loader/`validate` error.
- Config omitted backend → `None` (unchanged).
- Config `backend = "local"` still loads.

**Verification:** `cargo test --locked` for http + config tests touching these cases; `cargo clippy --all-targets -- -D warnings`.

**Dependencies:** U1

---

### U4. Release workflow credential hygiene

**Goal:** Close still-valid #93 security quick wins without redesigning release packaging.

**Files:** `.github/workflows/release.yml`

**Requirements:** R10

**Approach:** Move `permissions: contents: write` to the publish job (build job `contents: read` or default). Add `with: persist-credentials: false` on build `actions/checkout`. Keep artifact upload/publish behavior intact. Pinning `macos-14` vs `macos-latest` from #94 is optional/deferred unless trivial.

**Test scenarios:** N/A unit — verify YAML structure by review; CI workflow lint if present.

**Verification:** Workflow file review; ensure publish job still can create releases.

**Dependencies:** U1

---

### U5. Brain / landing interaction leftovers (no visual redesign)

**Goal:** Fix still-valid #96 UI correctness/stability leftovers without changing Brain look-and-feel.

**Files:**
- `ui/brain.js` (search clear; dispose; hover id gate)
- `ui/index.css` (search focus-within ring only)
- `ui/index.js` (localStorage try/catch; particle clamp on resize)
- `ui/index.html` (`type="button"` on copy buttons; optional banner width/height attrs)

**Requirements:** R11–R16

**Approach:**
1. On empty query: hide dropdown, clear query state, then `loadAtoms()`.
2. Before clearing node/edge groups, dispose geometries/materials (and maps if any).
3. Track `hoveredId`; skip `showHover` work when unchanged.
4. Add `.search-control:focus-within` border/outline using existing electric token — no palette inventing.
5. Guard theme storage; clamp particles on resize; `type="button"` on copy controls.

**Do not:** Change B/W/purple theme, camera whole-brain framing, layout modes, or add control chrome.

**Test scenarios:**
- Manual or DOM-level: clear search → dropdown `hidden === true`.
- Hover same node across moves → inspector not thrashing (logic gate present).
- Theme init with localStorage throwing → remaining listeners still bind (try/catch present).

**Verification:** Grep/assert code paths; optional Vite preview `website/` → `ui/` on `:5174` if daemon/CDN constraints allow.

**Dependencies:** U1

---

### U6. Optional — MCP atomic write permissions

**Goal:** If still cheap after U2–U5, preserve restrictive mode on MCP config rewrites (#83).

**Files:** `src/mcp/init.rs` (`atomic_write`)

**Requirements:** none mandatory (stretch from #83)

**Approach:** After writing temp file, `chmod` to owner-only before rename when platform supports it; add a unit test on Unix. Skip on Windows if awkward.

**Test scenarios:** Unix: rewritten file mode is `0o600` (or prior mode preserved if stricter semantics chosen — document choice).

**Verification:** `cargo test --locked mcp::` (or init module tests).

**Dependencies:** U3 (ordering only — avoid parallel conflict-free; may skip)

---

## Verification Contract

| Gate | Command / check | Applies |
|------|-----------------|---------|
| Format | `cargo fmt --all -- --check` | U3, U6 |
| Lint | `cargo clippy --all-targets -- -D warnings` | U3, U6 |
| Tests | `cargo test --locked` | U3, U6; full suite before PR |
| Hygiene | `git ls-files 'RETURN_RECEIPT*' 'residual-review-findings/*'` empty | U2 |
| Diff discipline | PR file list excludes `README.md` and does not add `json` to `parse_source_kind` | R17 |
| UI smoke (best-effort) | Clear-search dropdown hide; no Brain visual change | U5 |
| Workflow | Visual review of `.github/workflows/release.yml` permissions | U4 |

`release:validate` not required (no version bump in this PR).

---

## Definition of Done

**Global**

- [ ] U1 re-verification table in PR body; stale items explicitly skipped
- [ ] U2 receipts/residual removed; checklist path fixed
- [ ] U3 touch 404 + lean payload + empty backend reject + template coherence
- [ ] U4 release credential hygiene
- [ ] U5 Brain/landing interaction leftovers without visual redesign
- [ ] CI green on the follow-up PR
- [ ] Conflicts with #97/#98 named in PR body (rebase plan)

**Per unit:** each unit’s Verification subsection satisfied; no unit expands into Phase 6 or README ownership.

---

## Appendix — Research snapshot (planning-time, re-verify in U1)

Planning base: `origin/main@9d2ae12` (2026-07-26).

| PR | Finding | Status on main | Plan disposition |
|----|---------|----------------|------------------|
| #96 | RETURN_RECEIPT_* lack tags | Present, no frontmatter | **Delete** (U2) |
| #96 | residual “Unresolved: None” | Present / misleading | **Delete** (U2) |
| #96 | checklist `plans/` path | Present line 216 | **Fix** (U2) |
| #96 | search clear leaves dropdown | Confirmed `brain.js` | **Fix** (U5) |
| #96 | `nodes.clear` without dispose | Confirmed | **Fix** (U5) |
| #96 | hover every pointermove | Confirmed | **Fix** (U5) |
| #96 | search focus outline missing | Confirmed | **Fix** (U5) |
| #96 | unguarded localStorage | Confirmed | **Fix** (U5) |
| #96 | copy buttons no `type` | Confirmed | **Fix** (U5) |
| #96 | particle resize stranding | Likely still valid | **Fix** (U5) |
| #96 | keyboard orbit for `role=application` | Valid / heavy | **Defer** |
| #96 | span not `.instrument`ed | Present | **Defer** / optional micro |
| #90 | touch 404 unreachable | Confirmed store `Err` | **Fix** (U3) |
| #90 | touch returns full atom | Confirmed | **Fix** (U3) |
| #90 | graph default 10k | Present | **Defer** (product knobs) |
| #90 | N serial touch writes | Present | **Defer** stretch |
| #84 | empty `embed.backend` → None | Confirmed | **Fix** (U3) |
| #84 | local template conflict | Confirmed comments | **Fix** (U3) |
| #84 | XSS onclick in http HTML | **Stale** | Skip |
| #83 | atomic_write perms | Confirmed bare write | Optional U6 |
| #93 | workflow `contents: write` + persist creds | Confirmed | **Fix** (U4) |
| #98 | README + json kind | Open PR | **Defer** to #98 |
| #97 | Phase 5 closeout | Open PR | **Defer** to #97 |

External research: not load-bearing (repo/PR audit only).
