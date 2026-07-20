# feat: Kurultai Phase 1 Foundation

**Date:** 2026-07-19  
**Target repo:** `duketopceo/kurultai`  
**Authority:** GitHub `#27` + ideation `docs/ideation/kurultai-future-directions-2026-07-18.html`  
**Depth:** Deep · **Readiness:** `implementation-ready` · `product_contract_source: ce-plan-bootstrap`

## Decisions locked

1. **Object storage:** Not in Phase 1. Hot medium = single SQLite (+ FTS5 + vec0). MinIO/S3 is a later cold/archive tier (#34).
2. **Post-train readiness:** Phase 1 only adds durable labels/metadata, stable content-addressed IDs, and an export-friendly atom shape (#33). No cold archive or labeling pipeline yet.
3. **Infra (self-hosted GHA, GlitchTip, env hardening):** Deferred (#20, #29, #35). Phase 1 ships GitHub-hosted CI + a light `dev|staging|prod` config switch only.

## Goal Capsule

A developer can install/build Kurultai, index a local markdown vault, search via CLI and MCP with citations, and run a golden-path CI gate—without API keys for FTS-only mode. Schema choices leave a clean path to multi-year labeled export for company post-training later.

**Stop when:** fixture vault → index → FTS hit (vector hit when key present); MCP `tools/list` exposes the four read tools; `status` is honest.

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | Schema-at-open: atoms + FTS5 + vec0 (shared rowid); `store_meta` fingerprints `embed_model` + `embed_dim`. |
| R2 | FTS-first search; refuse zero-vector upserts; OpenRouter when keyed. |
| R3 | Raw-first atoms (no distillation write path); filesystem `.md` connector; Obsidian = `vault_path` alias. |
| R4 | Shared `kurultai::app` runtime; thin CLI; `init` writes Rust `Config` shape; honest connector registry. |
| R5 | MCP stdio: `search`, `read_atom`, `status`, `reindex` + `install`/`doctor`. |
| R6 | Golden-path CI: fixture vault integration test + clippy on GitHub-hosted runners. |
| R7 | Post-train prep: durable `tags`/provenance/`source_uri` + documented export contract (no object storage). |
| R8 | Light `KURULTAI_ENV` / `--env` paths for `dev\|staging\|prod` local store roots. |

### Actors / flows

- **A1** Developer (local) · **F1** init → index → search · **F2** MCP agent search · **F3** CI golden path

### Scope boundaries

**In:** Phase 1 vertical slice above.  
**Deferred:** AppFlowy depth (after FS works), distillation, RRF polish beyond basic fuse, object storage/cold tier, GlitchTip projects, ARC/self-hosted runners, multi-tenant RBAC, HTTP daemon productization.  
**Outside identity:** Chat UI wrapper, Luke-specific defaults in core, Neo4j/AST-in-core.

## Key Technical Decisions

| KTD | Decision | Why |
|-----|----------|-----|
| KTD1 | SQLite hot only in P1 | Laptop/local-first; object storage for cold later |
| KTD2 | FTS before vectors | Day-one value without API spend; zero-vector fail-loud |
| KTD3 | Raw atoms; distillation later | Unblocks connectors; fields stay nullable for future labels |
| KTD4 | FS connector first; Obsidian alias | Shortest path to atoms; AppFlowy follows |
| KTD5 | One runtime for CLI+MCP | No dual query stacks |
| KTD6 | Labels/provenance now, cold later | Stable IDs + labels without rewriting hot schema |
| KTD7 | Hosted CI in P1 | `#20` Phase 2/5 is self-hosted; don’t block foundation |

## Work orders

| Issue | Title | Milestone | Maps to |
|-------|-------|-----------|---------|
| #31 | Connector: Filesystem markdown (Obsidian alias) | Phase 1 | R3, U3 |
| #32 | Runtime: `kurultai::app` + `init` + env paths | Phase 1 | R4, R8, U4 |
| #33 | Schema: post-train prep fields + export contract | Phase 1 | R7, U1 |
| #34 | Roadmap: Hot/cold tiers + object storage | Phase 3 | deferred |
| #35 | Roadmap: GlitchTip `kurultai-{dev,staging,prod}` | Phase 5 | deferred |
| #20 comment | Docker-ephemeral-first; ARC later | Phase 5 | N6 |

Existing Phase 1 issues linked via comments: #1 (U1), #2 (U2), #3/#4 (U3), #5 (U4), #11 (U5), #23 (U6). Milestone #29 → Phase 5.

## Implementation Units

### U1. Schema-at-open + embedding contract + post-train fields
**Goal:** Working `SqliteVecStore` with hybrid tables and export-friendly metadata.  
**Reqs:** R1, R7 · **Deps:** none  
**Files:** `src/store/mod.rs`, `src/store/schema.sql`, `src/types.rs`, `tests/store_schema_test.rs`  
**Approach:** Idempotent DDL; `store_meta`; refuse dim/model mismatch; keep `tags`/provenance/`source_uri` stable for future export.  
**Tests:** open empty DB creates tables; upsert+fts roundtrip; zero-dim mismatch errors; meta fingerprint rejects mixed dims.  
**Verify:** unit tests pass; fresh DB self-creates.

### U2. FTS-first embedder + OpenRouter
**Goal:** Real embeddings when keyed; FTS search without key.  
**Reqs:** R2 · **Deps:** U1  
**Files:** `src/embed/mod.rs`, `tests/embed_guard_test.rs`  
**Approach:** HTTP OpenRouter; reject all-zero vectors; search path FTS-only until vectors exist.  
**Tests:** stub/zero rejected; FTS works with no key; vector path with mock/key.  
**Verify:** `kurultai search` returns fixture hits offline.

### U3. Filesystem connector + Obsidian alias + hash upsert
**Goal:** Index real `.md` trees.  
**Reqs:** R3 · **Deps:** U1  
**Files:** `src/connectors/filesystem.rs`, `src/connectors/obsidian.rs`, `src/connectors/mod.rs`, `tests/fixtures/vault/`, `tests/connector_fs_test.rs`  
**Approach:** glob + content-hash ids; Obsidian wraps FS; AppFlowy stays stub/honest-unimplemented.  
**Tests:** 3-file fixture indexes; edit one file → one re-embed; deleted file orphan policy documented.  
**Verify:** `index --full` then FTS hit on known phrase.

### U4. Runtime kernel + CLI wire + env paths
**Goal:** Binary uses library; init/status honest.  
**Reqs:** R4, R8 · **Deps:** U1–U3  
**Files:** `src/app/mod.rs`, `src/config/`, `src/main.rs`, `tests/cli_init_test.rs`  
**Approach:** `kurultai::app::run`; `init` emits Config; `KURULTAI_ENV` / `--env` store roots; registry no fake sources.  
**Tests:** init writes config; status lists only registered kinds; env isolates store paths.  
**Verify:** end-to-end CLI without println stubs.

### U5. MCP + install/doctor
**Goal:** Agents can query Kurultai.  
**Reqs:** R5 · **Deps:** U4  
**Files:** `src/mcp/`, `src/main.rs` (`mcp` subcommand), `tests/mcp_smoke_test.rs`  
**Approach:** stdio; 4 tools; share app kernel; install writes Cursor/Claude mcp.json.  
**Tests:** tools/list = 4; search tool returns citation-shaped payload.  
**Verify:** MCP smoke in CI or local script.

### U6. Golden-path CI (hosted)
**Goal:** Phase 1 DoD automated.  
**Reqs:** R6 · **Deps:** U1–U4 (U5 optional smoke)  
**Files:** `.github/workflows/ci.yml`, `rust-toolchain.toml`, `tests/golden_path_test.rs`  
**Approach:** fmt/clippy/test on `ubuntu-latest`; fixture vault golden path; no self-hosted yet.  
**Tests:** CI green on PR.  
**Verify:** workflow runs on push/PR.

## Verification Contract

- `cargo test` including golden path
- `cargo clippy -- -D warnings`
- Manual: `init` → `index` → `search` on fixture; MCP tools/list
- Schema: documented export field list in README (Post-train export contract)

## Definition of Done

- `#27` Phase 1 exit criteria met (index + search E2E; MCP search responds)
- Work orders #31–#33 open and linked; #34–#35 + #20 comment filed as roadmap
- This plan committed under `docs/plans/`
- No object storage / ARC / GlitchTip code in Phase 1 PRs

## Export contract (post-train prep)

Atoms persist these fields for future cold-tier / labeling export (no object storage in P1):

| Field | Purpose |
|-------|---------|
| `id` | Content-addressed stable ID |
| `source` / `source_id` / `source_uri` | Provenance join keys |
| `title` / `content` / `summary` | Text corpus |
| `tags` | Durable labels (JSON array) |
| `provenance` | Free-form / JSON provenance blob |
| `source_updated_at` / `indexed_at` | Temporal ordering |
| `content_hash` | Change detection |
| `metadata` | Source-specific extras |
| `embedding` (optional) | Vector when keyed |

Future cold tier (#34) should preserve these fields unchanged when archiving off hot SQLite.
