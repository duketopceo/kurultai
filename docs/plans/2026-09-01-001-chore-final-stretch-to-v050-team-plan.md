---
title: "chore: Final stretch to v0.5.0 Team — release hardening, queue cleanup, container + OS tests"
date: 2026-09-01
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "docs/plans/phase-6-next-work-orders.md · docs/plans/YEAR-1-MILESTONES.md · GitHub #180 · #181"
depth: standard
origin: "docs/plans/phase-6-next-work-orders.md"
---

# chore: Final stretch to v0.5.0 Team — release hardening, queue cleanup, container + OS tests

**Target repo:** `duketopceo/kurultai`  
**Base:** `main` after HUB-5 lands  
**Tracking:** #180 (HUB-5) · #181 (HUB-6) · phase-6-next queue · YEAR-1-MILESTONES v0.5.0 Team  
**Process:** PR-only, one logical change per PR

## Goal Capsule

Ship v0.5.0 "Team" by landing the one remaining Wave G work order (HUB-5), cleaning the stale queue/branches/issues, and hardening the release with container build and OS installer smoke tests.

**Stop when:** `v0.5.0` is tagged and released, `phase-6-next-work-orders.md` reflects Wave G done, stale branches and superseded issues are closed, and `Dockerfile` + install-script smoke tests pass in CI.

**Do not:** Start Atlas (A2–A5), Brain UI graph redesign, enterprise connectors, multi-tenant RLS, or v0.6/v0.7 work. This slice closes v0.5.0 only.

## Product Contract

### Summary

Wave G is one merge away from the v0.5.0 gate. HUB-6 acceptance is already green. HUB-5 (#180) is implemented on `feat/hub5-connector-visibility-tagging` but un-PR'd. The `phase-6-next-work-orders.md` queue, `YEAR-1-MILESTONES.md`, and the `v0.5.0` GitHub tag/release are all out of sync with reality. This plan reconciles them, lands the final feature, and adds release-hardening tests.

### Requirements

| ID | Requirement | Origin |
|----|-------------|--------|
| R1 | HUB-5 source-level visibility scope tagging must merge to `main`. | #180 |
| R2 | Wave G queue must accurately show HUB-5 and HUB-6 complete. | phase-6-next |
| R3 | `YEAR-1-MILESTONES.md` must reflect v0.4.1 shipped and v0.5.0 as current target. | YEAR-1-MILESTONES |
| R4 | Stale branches and superseded PRs/issues must be closed. | repo hygiene |
| R5 | Container build must produce a working binary and pass a smoke test. | Dockerfile |
| R6 | Install script must dry-run successfully on representative OS targets. | tests/install_script_test.rs |
| R7 | Full Rust release verification must pass before the tag. | CONTRIBUTING.md |

### Scope boundaries

**In:** HUB-5 PR, queue/milestone doc cleanup, stale branch/PR/issue hygiene, Dockerfile smoke, install-script OS smoke, v0.5.0 tag + release.

**Out:** v0.3.1 (explicitly skipped), v0.6/v0.7, Atlas, Brain UI graph/ontology layout, connector sprawl, enterprise RLS.

**Deferred to follow-up work:** Multi-tenant RLS, Redis cache, enterprise connectors, CodeGraph, webhook runtime — all belong to v0.6+.

## Planning Contract

### Key Technical Decisions

- **KTD1. Reconcile the existing `v0.5.0` tag.** The tag already exists and is marked `Latest` on GitHub, while `Cargo.toml` is `0.4.1`. Treat the existing `v0.5.0` release as a premature/naming leftover: delete the tag and release, then recreate them from the verified merge commit. This matches the repo convention of tagging after the release PR lands.
- **KTD2. Queue-cleanup PR separate from HUB-5.** HUB-5 already has a focused diff. Queue and milestone doc cleanup should land in a follow-up `docs:` PR so HUB-5's diff stays reviewable.
- **KTD3. Container smoke uses solo build by default.** The `Dockerfile` and `docker-compose.hub.yml` exist. The smoke test will build the container and run `kurultai --version`; hub-specific Postgres paths are exercised manually, not in default CI, because live Postgres is not available in the standard CI run.
- **KTD4. Installer smoke targets the dry-run path.** The install script already has a `--dry-run` / `--help` test. Expanding it to detect OS family (macOS, Arch/Debian families) and assert package-manager presence without mutating the system is the safe first pass.

### Assumptions

- HUB-5 branch is rebased on `main` and all tests pass.
- The user can push and open PRs for HUB-5 and the cleanup PRs.
- `gh` is authenticated and can manage issues/PRs/branches.
- Docker is available locally for the container smoke test.

## Implementation Units

### U1. Open and merge HUB-5 PR

- **Goal:** Land connector ingest visibility scope tagging.
- **Requirements:** R1
- **Dependencies:** none
- **Files:** `src/types.rs`, `src/pipeline/mod.rs`, `tests/acceptance_visibility.rs`, `config.example.toml`, `src/INDEX.md`, `src/pipeline/INDEX.md`, `tests/INDEX.md`, `INDEX.md`, `config.example.toml`
- **Approach:** Push `feat/hub5-connector-visibility-tagging` to origin, open PR #180, run full verification (`cargo test`, `cargo fmt`, `cargo clippy`, `cargo build --release`, `python3 scripts/audit-agent-index.py`), merge to `main`.
- **Execution note:** The branch is already implemented; this unit is verification + merge, not new code.
- **Patterns to follow:** Existing HUB-3/HUB-4 PR convention; commit `3d7dcf3` style.
- **Test scenarios:**
  - Existing `acceptance_visibility` tests pass.
  - `cargo clippy --all-targets -- -D warnings` is clean.
  - `python3 scripts/audit-agent-index.py` exits 0.
- **Verification:** HUB-5 PR is merged and `main` includes `3d7dcf3`.

### U2. Update Wave G queue and Year-1 milestones

- **Goal:** Remove stale "next LFG" text and reflect v0.4.1 shipped, HUB-5/6 done.
- **Requirements:** R2, R3
- **Dependencies:** U1
- **Files:** `docs/plans/phase-6-next-work-orders.md`, `docs/plans/YEAR-1-MILESTONES.md`, `docs/plans/INDEX.md`, `docs/INDEX.md`, `INDEX.md`
- **Approach:** Edit queue doc to mark HUB-5 ✅ and HUB-6 ✅, and to state v0.5.0 is the current target. Update `YEAR-1-MILESTONES.md` to reflect v0.4.1 shipped and v0.5.0 in progress, plus the fact that v0.3.1 was skipped. Roll index changes through `docs/plans/INDEX.md`, `docs/INDEX.md`, and root `INDEX.md`.
- **Patterns to follow:** Agent-index ritual from `docs/agent-index.md`; use `python3 scripts/audit-agent-index.py`.
- **Test scenarios:**
  - `phase-6-next-work-orders.md` no longer lists HUB-5 as "next LFG".
  - `YEAR-1-MILESTONES.md` shows v0.4.1 as shipped and v0.5.0 as current.
  - Audit script exits 0 after index updates.
- **Verification:** Docs PR merged and `main` matches the real state.

### U3. Clean stale branches and PRs

- **Goal:** Close abandoned work and avoid confusing future agents.
- **Requirements:** R4
- **Dependencies:** U2 (queue clarity helps decide what is stale)
- **Files:** N/A (GitHub state only)
- **Approach:**
  - Delete remote branches that are clearly superseded: `cursor/hub5-connector-visibility-7a74`, `hub4-agent-ids-7a74`, `cursor/repo-hygiene-7a74` if merged/obsolete.
  - Close PR #225 and #232 if their doc changes are now covered by U2 or obsolete.
  - Close stale dependabot PRs for `web/` and `website/` if they are not mergeable as-is.
  - Close early roadmap issues (#10–#15, #29, #80, #101–#102, etc.) that have been superseded by Wave G or deferred to v0.6+; leave a short "deferred to v0.6+" comment.
- **Patterns to follow:** Use `gh` for all remote operations; never force-push `main`.
- **Test scenarios:**
  - `gh pr list` no longer shows stale cleanup PRs.
  - `gh issue list --state open` is reduced to active v0.5.0+ work orders.
- **Verification:** Open PR count ≤ 2 (HUB-5 + one active) and open issue count is trimmed.

### U4. Add container build smoke test

- **Goal:** Verify `Dockerfile` produces a runnable binary.
- **Requirements:** R5
- **Dependencies:** U1
- **Files:** `Dockerfile`, `docker-compose.hub.yml`, `.github/workflows/ci.yml`, `tests/container_smoke.rs` (new) or CI step
- **Approach:** Add a lightweight CI step (or integration test) that builds `docker build -t kurultai:smoke .` and runs `docker run --rm kurultai:smoke --version`. Keep it solo-only; hub compose is documented, not auto-tested in CI.
- **Patterns to follow:** Existing `.github/workflows/ci.yml` job structure.
- **Test scenarios:**
  - Happy path: `docker build` succeeds and `kurultai --version` prints `0.5.0` (or current `Cargo.toml` version).
  - Failure path: build fails if `Cargo.toml` is syntactically broken.
- **Verification:** New CI step passes on `main`.

### U5. Expand OS installer smoke coverage

- **Goal:** Make the install script safe to dry-run on major OS families.
- **Requirements:** R6
- **Dependencies:** none
- **Files:** `tests/install_script_test.rs`, `scripts/install.sh` (or `install.sh`)
- **Approach:** Add unit-level assertions in `tests/install_script_test.rs` that parse the install script for OS detection paths (macOS, Debian, Arch) and verify `--dry-run` exits 0 with no destructive commands. If possible, run the script in a container for each family.
- **Patterns to follow:** Existing `install_script_test.rs` patterns; no system mutation.
- **Test scenarios:**
  - `--dry-run` exits 0 on the current host.
  - Script detects `unknown` OS and exits with a clear message.
  - No `rm -rf /` or unguarded `pacman`/`apt`/`brew` commands run under dry-run.
- **Verification:** `cargo test install_script` passes.

### U6. Full release build verification

- **Goal:** Ensure `main` is releasable before tagging.
- **Requirements:** R7
- **Dependencies:** U1, U4, U5
- **Files:** `Cargo.toml`, `CHANGELOG.md`, `README.md`, `ui/`, `website/`
- **Approach:** Run the full release matrix: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`, `cargo build --release --locked`, `bash scripts/build-ui.sh`, `python3 scripts/audit-agent-index.py`. Update `Cargo.toml` version to `0.5.0` if it is not already.
- **Patterns to follow:** `CONTRIBUTING.md` verification list.
- **Test scenarios:**
  - Release binary size and `kurultai --version` are as expected.
  - Embedded Brain UI builds without stale diff.
  - `cargo build --release --locked` succeeds.
- **Verification:** All release commands exit 0 and the working tree is clean except generated `ui/` assets.

### U7. Tag and release v0.5.0

- **Goal:** Publish the v0.5.0 GitHub Release.
- **Requirements:** R1–R7
- **Dependencies:** U1–U6
- **Files:** `CHANGELOG.md`, `Cargo.toml`, GitHub releases
- **Approach:**
  1. If an existing `v0.5.0` tag/release exists, delete it (it was premature).
  2. Update `Cargo.toml` to `version = "0.5.0"` and `CHANGELOG.md` with the v0.5.0 summary.
  3. Commit on `main`.
  4. Tag `v0.5.0` on the release commit.
  5. Create a GitHub Release with notes covering HUB-1 through HUB-6.
- **Patterns to follow:** Existing `v0.4.1` release notes style.
- **Test scenarios:**
  - `git tag -l v0.5.0` resolves to the intended merge commit.
  - `gh release view v0.5.0` shows the correct notes.
- **Verification:** `v0.5.0` is the latest GitHub Release and the crate version matches.

## Open Questions

1. Should the existing `v0.5.0` tag/release be deleted and recreated, or should we keep it and add a `v0.5.0-actual` tag? (KTD1 assumes delete/recreate.)
2. Which dependabot PRs should be merged before v0.5.0 vs. closed? (Most are web/website deps and may be noise.)
3. Do we want a real Postgres-backed CI job for hub smoke tests, or is manual verification acceptable for v0.5.0? (KTD3 assumes manual.)

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| HUB-5 branch has merge conflicts after sitting un-pushed. | Rebase on `main` before opening PR. |
| Deleting the existing `v0.5.0` tag breaks external links. | The tag is already inconsistent with `Cargo.toml`; better to correct now and note in release. |
| Dependabot PRs contain useful security bumps. | Review the list in U3; merge only non-breaking ones, close the rest. |
| Container build is slow in CI. | Cache layers; make the smoke job optional if time is a constraint. |

## Documentation Plan

- `CHANGELOG.md` — v0.5.0 release notes.
- `docs/plans/phase-6-next-work-orders.md` — mark Wave G complete.
- `docs/plans/YEAR-1-MILESTONES.md` — reflect v0.4.1 shipped, v0.5.0 current.
- `README.md` — update version badge if present.

## Definition of Done

- [ ] HUB-5 merged to `main`.
- [ ] Queue and milestone docs match reality and index audit is green.
- [ ] Stale branches/PRs/issues cleaned.
- [ ] Container smoke test passes.
- [ ] OS installer smoke tests pass.
- [ ] Full release build verification passes.
- [ ] `v0.5.0` tag and GitHub Release published.
