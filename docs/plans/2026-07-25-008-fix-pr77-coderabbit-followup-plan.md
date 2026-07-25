---
title: "fix: PR #77 remaining CodeRabbit follow-up"
date: 2026-07-25
type: fix
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "GitHub PR #77 unresolved CodeRabbit threads · parent plan docs/plans/2026-07-25-006-feat-v1-agent-zero-batch-plan.md"
depth: standard
origin: "LFG after Codex credit limit on PR #77; user approved remaining review fixes on feat/v1-agent-zero-batch"
---

# fix: PR #77 remaining CodeRabbit follow-up

**Target repo:** `duketopceo/kurultai`  
**Branch:** `feat/v1-agent-zero-batch` (existing open PR #77)  
**Audience:** implementer (`ce-work`) → CI → CodeRabbit re-review

## Goal Capsule

Close the **still-valid** unresolved CodeRabbit findings on [PR #77](https://github.com/duketopceo/kurultai/pull/77) with surgical fixes on the same branch — no new product surface beyond what #72–#76 already shipped.

**Stop when:** listed units land; `cargo fmt` / `clippy -D warnings` / `cargo test --locked` pass; follow-up commit(s) pushed to the PR branch.

**Do not:** reopen #72–#76 scope; add WebSocket/graph UI; invent an Edge table; open a second PR; auto-merge.

**Assumption (LFG headless):** “yes for kurultai phase” = finish PR #77 review leftovers, not start a new Agent Zero phase.

---

## Product Contract

### Requirements

| ID | Source | Requirement |
|----|--------|-------------|
| R1 | CR docs | `docs/agent-zero/INDEX.md` has one row per issue `#72`–`#76`; `#76` is HTTP dashboard/status only (no WebSocket claim) |
| R2 | CR docs | Citation / `/api/search` examples in ISSUE-002 match shipped `Citation` + `SearchResult` JSON |
| R3 | CR docs | PR-001 markdownlint: blank lines around fences, diagram fence `text`, single trailing newline |
| R4 | CR types | `title_hash` uses stable SHA-256 (via `hashutil`), not `DefaultHasher` |
| R5 | CR types | `excerpt_start` / `excerpt_end` are **character** offsets, not byte indices |
| R6 | CR http | `/api/status` must not report store failures as `ok: true` + `atoms: 0` |
| R7 | CR config | Invalid `nightly_full_sync_hour` (>23) is a **config error**, not silent disable |
| R8 | CR daemon | Idle skip tracks **client** inactivity separately from indexing timestamps; query/ask refresh client activity |
| R9 | CR brain | Second-hop tag searches fan out concurrently (bounded ≤4), merge/dedupe, tolerate per-tag failure |
| R10 | CR synthesize | `graph_chain` reflects primary seeds + hop inserts (not every distinct score-ordered hit as a fake path) |
| R11 | CR install test | Dry-run test uses plain `--dry-run` with isolated `HOME`/`XDG_CONFIG_HOME` and asserts no config mutation |

### Scope boundaries

**In:** R1–R11; minimal README `[runtime]` commented scheduler keys if still missing.

**Out / skip with reason**

| Finding | Disposition |
|---------|-------------|
| install.sh pipe-to-shell rustup | **Skip** — `30442ee` already downloads to temp file then `sh "$rs"`; no pipe-to-shell remains |
| install.sh dry-run log on stdout | **Skip** — `resolve_src` dry-run already uses `log` (stderr) + `echo "$dest"` |
| Full parent/child Edge walk for `graph_chain` | **Deferred** — no Edge store; R10 is the right-sized fix |
| Docstring coverage 80% gate | **Out** — advisory CodeRabbit pre-merge warning, not a CI gate |

### Actors & acceptance

- **Developer:** pushes follow-up commit(s) on PR #77; CodeRabbit threads become resolvable.
- **CI:** green Lint & Test + macOS smoke.
- **AE1:** `title_hash` for the same title is identical across runs and uses hex SHA-256 prefix.
- **AE2:** After idle threshold, a `/search` or `/ask` request resumes polling (client activity touch).
- **AE3:** Two tag hops run concurrently; a failing hop does not fail `ask`.

---

## Planning Contract

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 `session-settled:` | Stay on `feat/v1-agent-zero-batch` / PR #77 | User approved follow-up on this branch; rejected new PR |
| KTD2 `session-settled:` | Scope = remaining CR threads only | Conversation was unfinished review work, not a new phase |
| KTD3 | Separate `last_client_activity_unix` from index marks | Fixes CR idle semantics without changing nightly/full timestamps |
| KTD4 | `SearchResult.matched_by` gains `"multi_hop"` (or equivalent) for hop inserts; `graph_chain` = ordered unique source_ids of primary then hop | Addresses fake-path complaint without Edge table; preserves Answer docstring honesty |
| KTD5 | Concurrent hops via `futures`/`tokio::join` over `Arc<BrainService>` (Clone cheap Arcs) | BrainService is already Arc-backed; sequential 4× search is the latency bug |
| KTD6 | Invalid nightly hour → `Err` from loader | Latest CR review prefers fail-fast over warn+disable; omitted hour stays `None` |
| KTD7 | `/api/status` on count failure → HTTP 503 JSON `{ ok: false, error, scheduler }` | Dashboard can distinguish broken vs empty store |

### Assumptions

- A1. First follow-up commit `30442ee` already fixed DaemonStatus injection, idle wedge-on-failure (`touch_activity` each cycle), nightly catch-up, install stderr/rustup download, and some docs frontmatter — re-verify before re-implementing.
- A2. `sha2` / `hashutil::sha256_hex` already in-tree — reuse, do not add blake3.
- A3. CodeRabbit “processing” pending state clears after push; unresolved threads may need manual resolve after fix.

### Sequencing

U1 (docs) → U2 (citation integrity) → U3 (status/config) → U4 (idle) → U5 (concurrent hop + graph_chain) → U6 (install test). U5 depends on understanding `expand_with_tags` in `src/mcp/brain.rs`; U4 touches daemon + http handlers together.

---

## Implementation Units

### U1. Agent Zero / README doc alignment

**Goal:** Remove duplicate INDEX rows and align citation/dashboard docs with shipped types.

**Files:** `docs/agent-zero/INDEX.md`, `docs/agent-zero/ISSUE-002-query-result-citations.md`, `docs/agent-zero/PR-001-dev-dashboard-http-daemon.md`, `README.md` (scheduler comments if missing)

**Requirements:** R1, R2, R3

**Test scenarios:** N/A (docs-only) — verify by reading examples against `Citation` / search JSON shapes in `src/types.rs` and `src/http/mod.rs`.

---

### U2. Stable title hash + character excerpt offsets

**Goal:** Citation provenance fields are stable and Unicode-safe.

**Files:** `src/types.rs`, prefer reuse `src/hashutil.rs`; unit tests in `src/types.rs` or adjacent test module

**Requirements:** R4, R5

**Test scenarios:**
- Same title → identical `title_hash` across calls; hash equals prefix of `sha256_hex(title)`.
- Multi-byte content (e.g. `"café…"`) with excerpt mid-string → `excerpt_start`/`excerpt_end` are char counts, not byte indices.
- Empty / not-found excerpt → `(None, None)` unchanged.

---

### U3. Status honesty + nightly hour hard error

**Goal:** Status and config fail loudly instead of lying or silently disabling.

**Files:** `src/http/mod.rs`, `src/config/loader.rs`, loader tests

**Requirements:** R6, R7

**Test scenarios:**
- Loader rejects hour `24` (and preferably `255`) with error; accepts `0`/`23`; omitted → `None`.
- Status handler: when `atom_count` fails, response is non-200 or `ok: false` (not healthy empty).

---

### U4. Client-activity idle tracking

**Goal:** Idle skip reflects user inactivity; indexing alone does not keep the daemon “active.”

**Files:** `src/daemon/mod.rs`, `src/http/mod.rs` (search/ask handlers), `src/types` status snapshot if field renamed

**Requirements:** R8

**Pattern:** Keep `last_incremental_unix` / `last_full_unix` for index times. Add/use `last_client_activity_unix` for idle. `mark_incremental`/`mark_full` must **not** refresh client activity. Touch client activity from `/search`, `/ask`, and preferably `/api/search` UI path. Idle threshold compares against client activity (and still allow `touch_activity` on failed poll attempts so wedge-on-failure stays fixed).

**Test scenarios:**
- Unit/daemon test: after threshold with no client touches, poll cycle skips; after `touch` from handler helper, skip lifts.
- Index-only marks do not prevent idle.

---

### U5. Concurrent tag hops + honest graph_chain

**Goal:** Faster multi-hop and a chain that matches hop provenance.

**Files:** `src/mcp/brain.rs`, `src/synthesize/mod.rs`, tests in those modules

**Requirements:** R9, R10

**Approach:**
1. Fan out ≤4 tag `search` calls concurrently; merge by atom id; failures → empty for that tag.
2. Tag hop inserts mark `matched_by` with `"multi_hop"` (preserve existing matchers on primary).
3. `graph_chain_from_hits`: unique `source_id`s — all primary hits (no multi_hop marker) in rank order, then multi_hop inserts in rank order. Do **not** claim Edge-walk semantics in docs/comments.

**Test scenarios:**
- Concurrent bound: four tags → at most four searches; dedupe overlapping atoms.
- One tag search error → overall ask/expand still Ok.
- Unrelated primary + hop insert → chain lists primary seed(s) then hop source_id; does not invent edges.

---

### U6. Install dry-run isolation test

**Goal:** Prove `--dry-run` alone creates no config/MCP state.

**Files:** `tests/install_script_test.rs`

**Requirements:** R11

**Test scenarios:**
- Run with `--dry-run` only (no `--no-init`), temp `HOME` + `XDG_CONFIG_HOME`, assert success + DRY-RUN on stderr, assert temp dirs have no new kurultai config/MCP files.

---

## Verification Contract

Repo gates (must pass before DONE):

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Manual (optional, non-blocking for this follow-up): `kurultai daemon` → `GET /api/status` shows scheduler fields including client activity; idle behavior after threshold.

---

## Definition of Done

- [ ] U1–U6 complete or explicitly skipped with reason in the commit body
- [ ] Verification commands above green
- [ ] Changes committed and pushed to `feat/v1-agent-zero-batch` (PR #77)
- [ ] No new secrets; no scope expansion beyond R1–R11

---

## Appendix

### Unresolved CR thread map (pre-fix)

| Path | Severity | Plan unit |
|------|----------|-----------|
| `docs/agent-zero/INDEX.md` | minor | U1 |
| `docs/agent-zero/ISSUE-002-…` | major | U1 |
| `docs/agent-zero/PR-001-…` | trivial | U1 |
| `src/types.rs` (hash + offsets) | minor/major | U2 |
| `src/http/mod.rs` status | minor | U3 |
| `src/config/loader.rs` | minor (may be outdated) | U3 |
| `src/daemon/mod.rs` idle | major | U4 |
| `src/mcp/brain.rs` concurrency | major | U5 |
| `src/synthesize/mod.rs` graph_chain | major | U5 |
| `tests/install_script_test.rs` | minor | U6 |
| `scripts/install/install.sh` | skip | already fixed |
