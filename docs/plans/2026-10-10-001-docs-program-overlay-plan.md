---
title: "Kurultai Program Roadmap Overlay (difficulty, feasibility, simplification)"
type: docs
date: 2026-10-10
artifact_contract: ce-unified-plan/v1
execution: docs
depth: standard
status: proposal
---

# Kurultai Program Roadmap Overlay

## Goal Capsule

- **Objective:** add Difficulty, Feasibility and Simpler-alternative to every unit of the program roadmap in PR #420, add units found by the landscape research, and record the open owner decision on kurultai-personal `embed_dim`.
- **Relationship to #420:** this is an overlay, not a replacement. U1 to U21 keep the IDs, horizons and statuses #420 gives them (`docs/plans/2026-10-05-001-docs-program-roadmap-plan.md`, open and awaiting owner review at time of writing). Nothing here contradicts it. If #420 changes, update the matching rows here.
- **New units** use IDs U22 and up so they cannot collide with #420.
- **Research:** `docs/research/2026-10-10-landscape.md`. **Design:** `DESIGN.md`.
- **Baseline:** `origin/main` at b898c31, crate v0.7.0, PR #420 body as of 2026-10-10. Statuses are #420's; items I could not verify are marked `unverified`.

## Overlay on #420 units

Difficulty is easy / medium / hard. Feasibility lists what can block the unit. Simpler alternative names a smaller path or code to cut.

### Now

| Unit | Difficulty | Feasibility | Simpler alternative |
|---|---|---|---|
| **U1** Wave H serve-path closeout (#323, #326) | medium: #326 outbox touches the ingest write path and needs crash-replay tests; #323 adds a schema version to chunks | Needs a SQLite migration that older stores accept; no external dependency. Risk: embedder change cost if the re-embed path hits a paid API | Land #326 only; defer #323 until the next embedder switch actually happens (#400 already reindexed once). Reuse `tests/chaos.rs` instead of a new harness |
| **U2** Device broker U4-U6 (+#413) | medium: three sub-units, some may already exist (`src/broker/board.rs` has `/revoke`) | Depends on the hosted hub and Cloudflare Access behaviour; #426 (no re-key for a revoked codename) blocks clean revocation tests | First diff plan vs code and ship only the gap. Fix #413 by deleting the Python bridge in favour of `src/mcp/broker_stdio.rs` if both exist (see U24) |
| **U3** Docs drift correction | easy: text edits across ROADMAP, AGENTS, SECURITY, README, INDEX | None. Needs an owner call on which count is canonical (16 tools) | Generate the MCP tool list from `src/mcp/server.rs` in a script and have `scripts/audit-agent-index.py` fail on drift, so it cannot recur |
| **U4** Plan frontmatter and duplicate hygiene | easy: mechanical, 11 rows owned | None; `plans/` (6 tracked files) vs `docs/plans/` (86) | Delete the `plans/` duplicates and the retired `artifact_readiness` field with one script rather than editing 49 files by hand |
| **U5** Work-instance dogfood, token hygiene (#333, #362, #413) | medium: operational, touches deploy and Cloudflare | Needs owner access to Cloudflare and server-001; cannot be done from CI alone. Security: the repo has an outside collaborator, so token scope review belongs here (owner action) | Skip a separate work instance until a second user exists; run one instance with scopes |
| **U6** Evals, load, chaos upkeep | easy to medium | `kurultai eval` is local and free in FTS mode; vector evals cost embedding calls (bill only via the `orch` key per AGENTS.md) | Keep FTS-mode golden set in CI; run vector evals manually before releases |

### Next

| Unit | Difficulty | Feasibility | Simpler alternative |
|---|---|---|---|
| **U7** Team RBAC, claim-level permissions (#115, #188) | hard: enforcement at ingest and query across SQLite and Postgres stores | Needs Postgres hub (`--features postgres`) and a decision on identity (Clerk vs device keys). Claim-level (#188) is a data-model change | Ship #115 (scope filter on search/ask using the existing `personal/team/company` tag) and defer #188 |
| **U8** Connector expansion (#114, #134, #135, #133, #130, #121) | medium each, hard as a set | Slack and Notion need third-party API terms, tokens and rate limits | Implement #133 (the structured source contract) and the inbox/webhook runtime (#134) only; let external automation (Zapier, n8n) push into the inbox instead of writing native Slack/Notion connectors |
| **U9** Ontology primitives and atlas (#116, #132, #131, #128, #129) | hard: typed objects, edge index, audit | Depends on whether the ontology is core value or a side feature (open owner question) | `src/ontology/mod.rs` (952 lines) already has class tree plus zero-LLM edges; stop at #131 typed-edge index and drop #128/#129 until a user asks |
| **U10** Multi-hop retrieval (#120) | medium: edges exist (#403), traversal and fusion are new | Needs eval proof it beats hybrid search, else it adds latency for nothing | Add one-hop neighbour expansion to `recall` behind a flag and measure on `evals/golden.json` first |
| **U11** Agent Zero plugin parity | easy: add 3 tools to the in-repo `plugin/` or vice versa | Two repos (`plugin/` here, `duketopceo/kurultai_people`) | Delete one copy. `plugin/` (5 tools) is shipped here; archive `kurultai_people` and point docs at the in-repo plugin |

### Later

| Unit | Difficulty | Feasibility | Simpler alternative |
|---|---|---|---|
| **U12** Versioned definitions and Redis L2 cache (#119, #112) | medium (#119), medium (#112) | Redis adds an operational dependency to a single-binary product | Drop #112: SQLite hot-tier already exists; measure first. #119 can be plain git history on the definitions file |
| **U13** Business dashboard, Explorer to 100% (#138, #137, #139, #140, #117) | hard: 3D graph perf at corpus scale | Browser WebGL performance; depends on U1 telemetry (#364/#365 shipped) | Cap the Explorer at the layouts that already work; delete `ui-next`/`brain.html` duplication between `website/` and `ui/` (see U25) |
| **U14** Desktop Brain wrap | medium: Tauri or similar shell around `/ui/` | Not started, signing and notarisation per OS | Do not build; `kurultai daemon` + browser tab is the same product. Remove the unit unless a user asks |
| **U15** Distribution, auth, speculative connectors (#79, #78, #4) | easy (crates.io, Homebrew), hard (#79) | #79 Sign in with Anthropic depends on an external programme that may not exist; AppFlowy fetcher needs a tool that failed to connect in this environment | Ship crates.io and the install script only; close #79 and #4 as won't-do until demand |
| **U16 to U21** Delivered baselines | n/a (done) | n/a | Maintenance only. Candidate trims are tracked as U24 and U25 below |

## New units from research and this pass

| Unit | Horizon | Status | Evidence | Difficulty | Feasibility | Simpler alternative |
|---|---|---|---|---|---|---|
| **U22** Council Ring design tokens | Next | proposal | `DESIGN.md`; current palette is dark purple (`ui/favicon.svg`, `docs/design/ui-definitions.md`) | easy: token swap in `website/src/next/` and the Brain node colours | Owner must sign off palette and logo first | Keep purple, apply only the new README/brand assets |
| **U23** Publish retrieval benchmark | Next | not started | Mem0 and Zep publish LoCoMo / LongMemEval / DMR numbers (research doc); Kurultai has `kurultai eval` and `evals/golden.json` but no public results | medium: adapt the LongMemEval dataset to the eval runner | Benchmark data licences; embedding spend (use the `orch` key, Grok-4.7-class ceiling) | Publish the in-repo golden-set results with method, labelled as not comparable to vendor numbers |
| **U24** Collapse duplicate stdio bridges | Now | not started, `unverified` | #413 names `kurultai-mcp-bridge.py`; `src/mcp/broker_stdio.rs` is the Rust path from #381. The Python bridge is not tracked in this repo (`git ls-files` finds none) so it may live in `deploy/` or a home directory | easy | None | Delete the Python bridge once the Rust one covers it |
| **U25** Trim UI duplication | Later | not started | `website/` (Vite) builds `ui/`; `web/` is a second Next.js app; `website/` and `ui/` both contain `brain.html` and `ui-next.html`; `design-lab/` generation scripts are scratch | medium | Owner decides whether `web/` (Clerk team app) survives | Freeze `web/` until U7 needs it; keep one UI build path |
| **U26** Split `src/store/mod.rs` and `src/http/mod.rs` | Later | not started | 4,810 and 3,160 lines (`wc -l`); `src/store/postgres.rs` 1,665 | medium: pure refactor, behaviour tests exist | None; merge-conflict risk with concurrent agents | Leave as is until a unit touches them; split only the part being changed |
| **U27** `forget` and feedback loop | Later | not started | Cognee ships `forget` and `improve` (research doc); Kurultai has supersede and sweep but no explicit delete-with-audit tool in the MCP list | medium | Needs write-audit design; must not delete from `quarantine` silently | `supersedes:` plus sweep prune already covers most cases; add only an audited delete |
| **U28** Owner decision: kurultai-personal `embed_dim` | Now | **open, owner decides** | The personal lane runs with `dimension = 1024` while code defaults to 3072 (`src/config/loader.rs:82,140`); `docs/plans/2026-10-06-001-feat-agent-shared-brain-wiring-plan.md` records `kurultai agent list` failing with `embed_dim mismatch` on a 1024 config, and `deploy/server-001/INDEX.md` records an earlier crash loop from the same mismatch. `docs/plans/2026-10-01-001-...competitive-feature-sweep-plan.md` notes a dimension change requires a reindex | n/a (decision), then medium for reindex | Options for the owner: keep 1024 (pplx-embed-0.6b, cheaper) and fix the CLI to read the `[embed]` block; or move to another dimension and reindex. Cost and downtime are the constraints | This overlay does not choose. Whichever is chosen, the CLI bypass of the file `[embed]` block is a separate small bug worth fixing |

## Dependencies and order

U3 and U4 first (cheap, remove contradictions), U28 decision before U1 #323 (embedding version work should know the target dimension), U24 with U2, U22 after the owner signs off assets, U23 after U6.

## Verification

For each unit, the verification in #420 stands. For this overlay: `python3 scripts/audit-agent-index.py`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `cd website && npm test`. Docs and assets only in this PR.

## Notes

- Security observation for the owner (nothing changed here): the repository lists an outside collaborator, `LukeDuke-Bartlett`. AGENTS.md's repo standard says no outside collaborators without a reason.
- Unverified: I did not read every issue body; unit evidence beyond what #420 states comes from `git ls-files`, `wc -l` and the files cited.
