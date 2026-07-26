---
title: "feat: Quality labeling bar with trust lanes (trusted vs quarantine)"
date: 2026-07-25
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-brainstorm
execution: code
authority: "Brain quality slice · post-train-ready data · upstream ce-brainstorm Trust lanes (A)"
depth: standard
origin: "LFG /lfg after synthesis: clean multi-year brain before organizer or post-train export"
status: draft
---

# feat: Quality labeling bar with trust lanes (trusted vs quarantine)

**Target repo:** `duketopceo/kurultai`  
**Audience:** developer → team brain operators  
**Base:** `main`  
**Process:** PR-only · branch `feature/quality-labeling-trust-lanes`

## Goal Capsule

Introduce **trust lanes** on every atom write so the personal/team/company brain stays **post-train-ready**: cheap synchronous gate (tags + exact `content_hash` duplicate) on the trusted path; failures land in **quarantine** (stored, excluded from default retrieval); **promote** is explicit (human UI, MCP tool, HTTP/CLI) — never a side effect of `remember`. Near-duplicate detection and safe auto-merge run **off the agent hot path** (daemon background job).

**Stop when:** all write paths (MCP `remember`, connector `IndexPipeline`) pass the same gate; default `search` / `ask` / `who_knows` / `list_atoms` / HTTP equivalents skip quarantine; `promote` tool + HTTP route exist with audit log; near-dupe job creates merge candidates or safe auto-merges; tests cover gate, retrieval filter, promote, and background merge rules; verification contract green.

**Do not:** pathfinding brain-organizer; object storage/PDF ingest; full post-train export pipeline; WebSocket Live MCP bridge; auto-promote on tag fix via `remember`; hard-reject writes (quarantine instead); near-dupe on write hot path.

**Product Contract preservation:** Implements ce-brainstorm Trust lanes (A) with settled KTDs below.

---

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Every atom upsert (connector pipeline + MCP `remember`) runs the **same** synchronous quality gate before SQLite commit |
| R2 | **Trusted path** (gate pass): atom stored with `trust_lane = trusted`; indexed in FTS/vec as today |
| R3 | **Quarantine path** (gate fail): atom stored with `trust_lane = quarantine` + `quarantine_reason`; still persisted (agents are not blocked) but **excluded** from default retrieval |
| R4 | Gate checks (sync only, no embedding/near-dupe): (a) ≥1 non-empty trimmed tag; (b) no **exact** duplicate — another atom with same `content_hash` already in `trusted` lane |
| R5 | Default retrieval (`search`, `ask`, `who_knows`, `list_atoms`, `hybrid_search`, HTTP `/api/search`, `/search`, dashboard list) returns **trusted only** |
| R6 | Explicit opt-in to include quarantine: query param / MCP arg `include_quarantine=true` (default false) for search/list; `cite` by id still resolves quarantined atoms when addressed directly |
| R7 | **Promote** moves atom `quarantine → trusted` only after re-running gate (tags present, no trusted exact duplicate); logs actor + reason |
| R8 | Promote surfaces: MCP tool `promote`, HTTP `POST /api/promote`, CLI `kurultai promote <atom_id>` (minimal — same core fn) |
| R9 | `remember` **never** promotes; updating a quarantined atom re-evaluates gate but stays quarantined until explicit promote |
| R10 | Near-duplicate scan runs in **daemon background** (or post-index hook), not in `remember`/upsert await path |
| R11 | Near-dupe pairs: if **safe auto-merge** applies, merge automatically; else insert **merge candidate** for operator review |
| R12 | **Safe auto-merge** = union tags/metadata/resolution additively; same `content_hash` or identical normalized body; **conflicts** (both sides have non-empty disagreeing `title`, `summary`, or `resolution`) → merge candidate, never auto-merge |
| R13 | Status/dashboard exposes counts: `trusted`, `quarantine`, `merge_candidates_pending` |
| R14 | Existing stores migrate: current rows default `trust_lane = trusted` (no retroactive quarantine sweep in this slice) |

### Actors / Flows

| ID | Actor / flow |
|----|--------------|
| A1 | Agent calling MCP `remember` with distilled fact |
| A2 | Connector poll/full sync via `IndexPipeline` |
| A3 | Human operator reviewing quarantine in UI/CLI |
| A4 | Agent or human calling `promote` after fixing tags |
| A5 | Daemon nightly/incremental post-index near-dupe pass |

### Acceptance Examples

| ID | Example |
|----|---------|
| AE1 | `remember(title, summary, tags=[])` → returns atom id; atom in quarantine with reason `untagged`; `search(summary)` does not return it |
| AE2 | `remember` with tag `["ops"]` and unique content → trusted; searchable via MCP `search` |
| AE3 | Second `remember` with same content hash as existing trusted atom → quarantined with reason `exact_duplicate:<id>`; first atom unchanged |
| AE4 | Markdown file without frontmatter tags → connector indexes atom as quarantined `untagged`; vault FTS reindex does not pollute default search |
| AE5 | `promote(atom_id)` on quarantined untagged atom after manual tag fix in DB is **not** valid — promote requires atom record updated with tags first, then promote re-runs gate |
| AE6 | `promote(atom_id)` after tags added via `remember` update path still quarantined until promote → trusted |
| AE7 | Near-dupe job finds two trusted atoms with Jaccard ≥0.92 on normalized content, conflicting titles → `merge_candidates` row, both atoms remain |
| AE8 | Near-dupe job finds quarantine duplicate of trusted, identical body → auto-merge deletes quarantine copy, logs merge |
| AE9 | `search(..., include_quarantine=true)` returns quarantined atom with reason in view metadata |
| AE10 | MCP `remember` latency p95 unchanged vs baseline (gate adds ≤2 SQLite lookups, no embed) |

### Scope boundaries

**In:** R1–R14; migration v4; `src/quality/*`; store/query/brain/MCP/HTTP/daemon hooks; tests; README + `docs/solutions` note.

**Out (explicit)**

- Pathfinding brain-organizer / graph layout
- Object storage, PDF connectors
- Full post-train export pipeline
- WebSocket Live MCP bridge
- Retroactive mass quarantine of legacy untagged atoms (follow-up script optional, not blocking)
- Human-only merge UI (merge candidates list + API stub acceptable; full UI polish deferred)

---

## Planning Contract

### Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | **Quality bar slice, not organizer** `(session-settled: user-directed — rejected pathfinding brain-organizer for this slice: multi-year brain needs clean data first)` | Focus; avoids scope creep |
| KTD2 | **Same gate for humans + agents** on every write — `IndexPipeline::index_connector` and `BrainService::remember` both call `quality::gate::evaluate` `(session-settled: user-directed — rejected human-only or agent-only bars)` | One brain, one standard |
| KTD3 | **Quarantine on fail; default retrieval skips** `(session-settled: user-directed — rejected hard reject and warn-and-allow)` | Agents keep writing; bad data isolated |
| KTD4 | **`trust_lane` column on `knowledge_atoms`** (`trusted` \| `quarantine`) + `quarantine_reason TEXT` via migration v4; filter in store search/list SQL | Single table; FTS/vec rows only for atoms that pass gate on trusted path — quarantine atoms still get FTS rows for explicit include_quarantine queries |
| KTD5 | **Exact duplicate via existing `content_hash`** (`sha256_hex(content)` already in `src/store/mod.rs` upsert + `idx_atoms_content_hash`) `(session-settled: user-directed — primary crap includes duplicates)` | Reuses `src/hashutil.rs` + store index; no new hash |
| KTD6 | **Trusted path = cheap sync only**: tag check + `SELECT id FROM knowledge_atoms WHERE content_hash = ? AND trust_lane = 'trusted' LIMIT 1` `(session-settled: user-directed — rejected admit-all-then-grade-async for trusted; must not delay agent DB use)` | Sub-ms SQLite; no embed on write |
| KTD7 | **Promote is explicit** — new MCP/HTTP/CLI; writes `quality_audit` row; `remember` never flips lane `(session-settled: user-directed — rejected auto-promote-on-fix and human-only)` | Auditable, agent-callable |
| KTD8 | **Near-dupe off hot path** — `src/quality/near_dupe.rs` invoked from `src/daemon/mod.rs` after incremental/full index (debounced) `(session-settled: user-directed — rejected near-dupe wait on write)` | Hot path stays fast |
| KTD9 | **Exact + near duplicate with merge** — near via normalized token Jaccard + optional vector distance when embed live; exact via hash `(session-settled: user-directed — rejected exact-only)` | Handles paraphrase dupes |
| KTD10 | **Safe auto-merge additive only; conflicts → merge candidate** `(session-settled: user-directed — rejected human-only merge; conflicts never auto-merge)` | Defined below in U5 |
| KTD11 | **Default hybrid search unchanged architecturally** — add trusted filter to store ID queries before hydrate (`src/query/hybrid.rs` passes filter to store) | FTS-first doctrine preserved per `docs/solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md` |
| KTD12 | **Legacy atoms trusted by default** on migration | Avoid breaking existing installs; forward-only bar |

### Safe auto-merge definition (normative)

Merge atom **B** into **A** (A = older `indexed_at` or lower id) when **all** hold:

1. Same `content_hash` **OR** normalized-body Jaccard ≥ 0.95
2. **No conflict:** not (both have non-empty `title` and titles differ case-insensitively); not (both have non-empty `summary` and summaries differ); not (both have `Some(resolution)` with different text)
3. **Additive union:** tags = union; metadata keys missing on A filled from B; `resolution` on A empty → take B's
4. Delete B's FTS/vec rows and atom row; log `quality_audit` action `auto_merge`

Otherwise: insert `merge_candidates(a_id, b_id, reason)` with status `pending`.

### Assumptions

- `content_hash` column populated on all upserts today (`src/store/mod.rs:153`).
- Atom id stability uses `atom_id(source, source_id, content)` (`src/hashutil.rs`) — duplicate **content** from different sources gets different ids but same hash; gate catches via hash not id.
- Quarantine atoms remain in `atoms_fts` so `include_quarantine` search works without reindex.
- Near-dupe job budget: scan atoms with `indexed_at` in last 24h + all quarantine (cap 500/run).

### Dependencies

| Dep | Notes |
|-----|-------|
| SQLite schema v3 → v4 | `src/store/migrations.rs` |
| Daemon poll loop | `src/daemon/mod.rs` — hook after `IndexPipeline::index_connector` |
| MCP tool surface | `src/mcp/server.rs`, `src/mcp/interface.rs` |
| HTTP brain API | `src/http/mod.rs` |
| Existing hash-skip embed | `src/pipeline/mod.rs:80-113` — unchanged; gate runs before upsert |

### Risks

| Risk | Mitigation |
|------|------------|
| Connector flood quarantines untagged markdown | Expected; document tag frontmatter; dashboard quarantine count |
| Gate SQL on every upsert adds latency | Indexed `content_hash` + `trust_lane`; benchmark in tests |
| Near-dupe false-positive merge | Conservative Jaccard threshold; conflict rules block auto-merge |
| `include_quarantine` leaks crap into agent answers | Default false; tool schema documents danger |
| Migration on large DB | Single ALTER + index; offline OK for personal brain sizes |

### Pattern references

- Atom model: `src/types.rs` (`KnowledgeAtom`)
- Write path (agent): `src/mcp/brain.rs` (`remember` → `store.upsert`)
- Write path (connectors): `src/pipeline/mod.rs` → `store.upsert_batch`
- Hash/id: `src/hashutil.rs` (`sha256_hex`, `atom_id`)
- Store upsert + `content_hash`: `src/store/mod.rs` (`upsert_sync`)
- Hybrid retrieval: `src/query/hybrid.rs` (`hybrid_search`)
- FTS-first doctrine: `docs/solutions/architecture-patterns/fts-first-null-embedder-no-zero-vectors.md`
- Daemon background work: `src/daemon/mod.rs`

---

## Implementation Units

### U1. Schema migration v4 — trust lanes + audit + merge candidates

**Goal:** Persist lane, reasons, audit trail, merge queue.  
**Files:** `src/store/migrations.rs`, `src/types.rs` (optional `TrustLane` enum), `src/store/mod.rs` (hydrate columns)  
**Approach:**

- Bump `CURRENT_SCHEMA_VERSION` to 4.
- `ALTER TABLE knowledge_atoms ADD COLUMN trust_lane TEXT NOT NULL DEFAULT 'trusted'`
- `ADD COLUMN quarantine_reason TEXT`
- `CREATE INDEX idx_atoms_trust_lane ON knowledge_atoms(trust_lane)`
- New tables:
  - `quality_audit(id, ts, action, atom_id, actor, detail_json)`
  - `merge_candidates(id, atom_a, atom_b, reason, status, created_at)` with `status IN ('pending','merged','dismissed')`
- Extend `ATOM_COLUMNS` / `row_to_atom` to load `trust_lane`, `quarantine_reason`.
- Add `TrustLane` parse/display helpers.

**Test scenarios:**

1. Fresh store opens at v4; columns exist
2. v3 store migrates; existing rows `trust_lane = trusted`
3. `row_to_atom` round-trips quarantine atom with reason

---

### U2. Quality gate module (sync write barrier)

**Goal:** Single `evaluate(atom, store) -> GateOutcome` used by all writers.  
**Files:** `src/quality/mod.rs`, `src/quality/gate.rs`, `src/quality/mod.rs` exports  
**Approach:**

```rust
pub enum GateOutcome {
    Trusted,
    Quarantine { reason: String },
}

pub async fn evaluate(store: &dyn Store, atom: &KnowledgeAtom) -> Result<GateOutcome>
```

Checks (order):

1. `non_empty_tags(atom.tags)` — else `Quarantine { reason: "untagged" }`
2. `store.find_trusted_by_content_hash(&sha256_hex(&atom.content))` — if Some(existing) and existing.id != atom.id → `Quarantine { reason: format!("exact_duplicate:{existing}") }`
3. Else `Trusted`

Apply outcome in upsert path: set `trust_lane` + `quarantine_reason` on atom before SQL.

Add `Store` methods: `find_trusted_by_content_hash`, `set_trust_lane`, `count_by_lane`, `list_quarantine(limit)`.

**Wire into:**

- `src/mcp/brain.rs::remember` — evaluate before upsert; return id + lane in MCP text (`trusted` / `quarantined:untagged`)
- `src/pipeline/mod.rs::index_connector` — map `evaluate` over `enriched` before `upsert_batch` (batch evaluate OK; no near-dupe)

**Test scenarios:**

1. Untagged atom → quarantine reason `untagged`
2. Tagged unique content → trusted
3. Same content_hash as trusted atom, different id → quarantine `exact_duplicate:*`
4. Same id re-upsert (update) with unchanged hash → stays trusted if tags OK
5. Gate completes without calling embedder (mock panics on embed)

---

### U3. Retrieval filter — trusted-by-default

**Goal:** Default search/ask/list skip quarantine.  
**Files:** `src/store/mod.rs`, `src/query/hybrid.rs`, `src/mcp/brain.rs`, `src/http/mod.rs`  
**Approach:**

- Add `SearchFilter { trusted_only: bool }` default `true`.
- Modify `fts_search_ids`, `vector_search_ids` SQL: `JOIN knowledge_atoms a ... WHERE (?trusted_only = 0 OR a.trust_lane = 'trusted')`
- `list_atoms(limit, filter)` — dashboard default trusted only
- `hybrid_search(..., filter)` — pass through from brain
- `BrainService::search`, `ask`, `who_knows`, `list_atoms` accept optional `include_quarantine: bool` (default false)
- `cite(source, source_id)` — unchanged (direct lookup by key, any lane)

**Test scenarios:**

1. Quarantined atom not in default `search` results
2. `include_quarantine=true` returns quarantined hit
3. `ask` citations exclude quarantine atoms from synthesis input
4. Trusted + quarantine with same FTS token — default returns trusted only
5. `list_atoms(10)` omits quarantine

---

### U4. Promote tool (MCP + HTTP + CLI)

**Goal:** Explicit lane transition with audit.  
**Files:** `src/quality/promote.rs`, `src/mcp/interface.rs` (trait method), `src/mcp/brain.rs`, `src/mcp/server.rs`, `src/http/mod.rs`, `src/main.rs`  
**Approach:**

- `promote_atom(store, id, actor) -> Result<PromoteResult>`:
  - Load atom; must be `quarantine`
  - Re-run gate; if still fails, return error with reason (no partial promote)
  - Set `trust_lane = trusted`, clear `quarantine_reason`
  - Insert `quality_audit` row `{ action: "promote", actor }`
  - `activity.record("promote", id, ...)`
- MCP tool `promote` args: `{ "atom_id": "...", "reason": "optional note" }`
- HTTP `POST /api/promote` JSON body same shape
- CLI: `kurultai promote <atom_id> [--reason]`

**Test scenarios:**

1. Promote quarantined untagged → error
2. Promote after tags added (via store update in test) → trusted + searchable
3. Promote duplicate-of-trusted → error still duplicate
4. Promote writes audit row with actor `mcp` / `http` / `cli`
5. `remember` on quarantined id does not auto-promote

---

### U5. Near-duplicate background job + safe auto-merge

**Goal:** Off hot path dedupe and merge.  
**Files:** `src/quality/near_dupe.rs`, `src/quality/merge.rs`, `src/daemon/mod.rs`, `src/store/mod.rs`  
**Approach:**

- `normalize_body(content) -> String` (lowercase, collapse whitespace, strip punctuation)
- `jaccard(a, b) -> f64` on word tokens
- `run_near_dupe_pass(store, embedder)`:
  - Candidates: quarantine atoms + trusted atoms indexed in last 24h (cap 500)
  - For each pair with same hash OR jaccard ≥ 0.92 (or cosine ≥ 0.95 when both have vectors and embedder live):
    - Try `try_safe_auto_merge(a, b)` per KTD10 definition
    - On success: audit `auto_merge`, delete loser
    - On conflict: `INSERT merge_candidates ... pending`
- Daemon: after each incremental/full index batch, `tokio::spawn` near-dupe pass (do not block poll loop completion beyond spawn)

**Test scenarios:**

1. Identical content trusted + quarantine → auto-merge removes quarantine
2. Same hash, conflicting titles → merge_candidate, both remain
3. Paraphrase above threshold, compatible metadata → auto-merge tag union
4. Job does not run during `remember` unit test (no daemon) — invoke pass directly in test
5. Near-dupe processes 0 atoms when store empty

---

### U6. Observability + docs

**Goal:** Operators see lane health.  
**Files:** `src/http/mod.rs` (`/api/status`), `src/http/ui.rs` (optional badge), `README.md`, `docs/solutions/architecture-patterns/trust-lanes-quality-gate.md` (new)  
**Approach:**

- Extend status JSON: `brain.trusted_count`, `brain.quarantine_count`, `brain.merge_candidates_pending`
- Document tag requirement for connectors; quarantine behavior; promote workflow
- MCP `tools/list` documents `include_quarantine` on search when U3 adds it

**Test scenarios:**

1. Status endpoint returns counts after mixed ingest
2. N/A docs — human review

---

## Verification Contract

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked quality::
cargo test --locked store::tests::
cargo test --locked mcp::brain::tests::
cargo test --locked mcp::server::tests::
cargo test --locked pipeline::tests::
```

Manual smoke (optional):

```bash
kurultai index --source notes   # fixture vault
kurultai promote <id>           # after quarantine scenario
```

Focus: gate unit tests, retrieval filter integration, promote audit, near-dupe merge rules.

---

## Definition of Done

- [ ] R1–R14 satisfied or explicitly deferred in PR body
- [ ] U1–U6 landed with enumerated tests passing
- [ ] All KTD session-settled annotations preserved in PR description
- [ ] No embed/near-dupe on `remember` await path (verify with test / timing guard)
- [ ] Verification Contract green
- [ ] PR opened on `feature/quality-labeling-trust-lanes`

---

## Open Questions

| Q | Status |
|---|--------|
| Retroactive quarantine sweep CLI? | **Deferred** — migration defaults legacy to trusted |
| Full merge-candidate review UI? | **Deferred** — API + table sufficient |
| Tag inference from path/heading for markdown? | **Deferred** — explicit tags only this slice |
| Vector-only near-dupe when FTS-only? | **Resolved** — Jaccard fallback always available |

---

## Appendix: Research breadcrumbs (codebase)

| Area | Path | Finding |
|------|------|---------|
| Atom shape | `src/types.rs:11-38` | No lane field yet; `tags: Vec<String>`, `content` canonical body |
| Agent write | `src/mcp/brain.rs:240-298` | `remember` → `atom_id` + optional embed + `store.upsert`; no tag gate |
| Content hash | `src/hashutil.rs:6-19` | `sha256_hex`, `atom_id` / `atom_id_from_hash` |
| Store upsert | `src/store/mod.rs:148-251` | Computes `content_hash`, hash-skip vec; index on `content_hash` |
| Hash-skip embed | `src/pipeline/mod.rs:80-113` | Unchanged by this slice |
| Hybrid search | `src/query/hybrid.rs:14-102` | FTS ∥ vector → RRF; no lane filter yet |
| Migrations | `src/store/migrations.rs:4-48` | Current v3; v2 added `content_hash` |
| MCP tools | `src/mcp/server.rs:224-295` | Five tools; promote not present |
| FTS-first | `docs/solutions/.../fts-first-null-embedder-no-zero-vectors.md` | Retrieval stays FTS-capable without embed key |
| Daemon | `src/daemon/mod.rs` | Poll/watch loops — hook for near-dupe pass |
