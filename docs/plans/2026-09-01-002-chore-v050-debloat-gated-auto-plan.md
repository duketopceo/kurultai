---
title: "chore: v0.5.0 first-cut debloat + gated-auto auth nits"
date: 2026-09-01
type: chore
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
execution: code
authority: "v0.4.1→v0.5.0 code review · user scope lock 2026-09-01"
depth: standard
origin: "Perplexity Kurultai project review 2026-09-01-v041-to-v050-code-review.md"
---

# chore: v0.5.0 first-cut debloat + gated-auto auth nits

**Target repo:** `duketopceo/kurultai`  
**Base:** `main` (currently 3 commits past tag `v0.5.0` / `929523b`)  
**Process:** one PR, two commits (deletes vs gated-auto). PR-only. Never push `main` directly.

## Goal Capsule

Cut dead product surface that does nothing, without dropping live behavior. Apply four small auth nits that are locally reversible. Leave `AdminKeyStore` alone.

**Stop when:** stub connectors and unused distill/source_config are gone; marketing landing is not rust-embedded; phase closeout scripts are gone; doctor hub/port text is true; hashed env keys are not bearers; listen policy counts issued hub keys; `/api/open` is absent in hub mode; env-mutating tests this PR owns cannot flake; `cargo test --locked --all-targets` and `python3 scripts/audit-agent-index.py` pass.

**Do not:** Delete or wire `AdminKeyStore`. Delete `web/`. Touch plugin hashed bundles. Collapse INDEX protocol. Merge json/markdown connectors. Fix AE5 list/graph, team_id stamp, tailscale bind, promote policy, or namespace dual-env. Delete `docs/plans/**` or `docs/brainstorms/**`. Close GitHub #4.

## Product Contract

### Summary

v0.5.0 shipped +26k/−5k. Most of that is features. A smaller set is theater: AppFlowy polls empty, TechTracker always errors, `emit_soft_labels` has no production caller, `registry::source_config` has no caller, ~1.35MB of already-redirected landing still embeds, four one-shot `gh issue close` scripts remain. Doctor lies about hub. Separately, four gated-auto auth bugs are small enough to ride along as a second commit.

“Remove all the crap” is rejected. Isolation bugs and INDEX protocol are not this PR.

### Actors

| ID | Actor |
|----|--------|
| A1 | Solo operator on loopback |
| A2 | Hub operator (`KURULTAI_FEATURE_HUB=1`, issued keys and/or env CSV) |
| A3 | CI / implementer |

### Requirements

| ID | Requirement | Origin |
|----|-------------|--------|
| R1 | `kind = "appflowy"` and `kind = "tech_tracker"` fail closed at config/registry. No stub that returns `Ok([])`. Live kinds unchanged. | review caveman + F-stub |
| R2 | `src/distill` (`emit_soft_labels`) removed. `KnowledgeAtom.soft_labels` and store columns stay. | review caveman |
| R3 | Unused `connectors::registry::source_config` removed. `SourceConfig` helpers on the type stay. | review caveman |
| R4 | `ui/index.html`, `index.css`, `index.js`, `kurultai_logo.jpg`, `neural_tech_banner.jpg` are not in the tree and not rust-embedded. `GET /ui/` still serves Brain. `GET /ui/index.html` (and css/js) still 301 to `/ui/`. | review F landing + `ui.rs` tests |
| R5 | `scripts/phase-{1,2,4,5}-closeout.sh` deleted. | review caveman |
| R6 | `kurultai doctor` hub field reflects hub feature, not embed backend. HTTP check uses configured/daemon port, not hardcoded 8421. Unused locals gone. Ontology check must not load 100k rows into RAM just to print a count. | review F-19 / doctor |
| R7 | Env CSV: if a stored entry is 64-char lowercase hex, accept only `sha256(presented)`, never the raw presented token. Plaintext entries still match plaintext. Presenting the hex against a hex entry is reject. | review F-04 |
| R8 | Non-loopback + `HubAuth::ApiKey` start-allow when issued `HubKeyStore` rows exist even if env CSV is empty. Still refuse when both are zero. | review F-05 |
| R9 | `GET /api/open` is not mounted when hub is on. Solo loopback keeps it. Plugin kproxy already omits it. | review F-07 |
| R10 | Tests that mutate process env in this PR’s files restore previous values on Drop, including panic. Confirmed flake `open_hub_store_requires_postgres_feature` cannot race `KURULTAI_FEATURE_HUB`. | review F-11 |
| R11 | `AdminKeyStore`, `kurultai admin key`, and README CLI row stay. | user lock |
| R12 | Agent-index audit exits 0. Deleted files are removed from INDEX tables (stale rows are quality, even if the current audit is one-way). | `docs/agent-index.md` |
| R13 | No quality drop: clippy `-D warnings`, existing markdown/json/github/dayflow/pond/inbox tests still pass. | user: no quality degradation |

### Key flows

| ID | Flow |
|----|------|
| F1 | `kurultai index` with a toml source `kind = "appflowy"` → config/registry error, no empty success. |
| F2 | Daemon `GET /ui/` → Brain HTML. Hashed `ui/assets/*` still 200. |
| F3 | `GET /ui/index.html` → 301 `/ui/` even after files are gone. |
| F4 | Hub bind-all + `auth=api_key` + empty env CSV + at least one active issued key → listen allow. |
| F5 | Bearer equals stored sha256 hex of a different secret → 401. Bearer equals the original secret whose hash is stored → 200. |
| F6 | Hub feature off: `GET /api/open` 200. Hub on + auth none: 404 (route omitted). Hub on + ApiKey without bearer: 401. Hub on + ApiKey with bearer: 404, no spawn. |

### Acceptance examples

| ID | Example |
|----|---------|
| AE1 | Config `kind = "markdown"` still indexes. `kind = "appflowy"` errors. `kind = "tech_tracker"` errors. Covers R1, F1. |
| AE2 | `UiAssets` iter contains no `index.html` / `kurultai_logo.jpg`. `/ui/` 200. `/ui/index.css` redirect. Covers R4, F2, F3. |
| AE3 | `token_accepted(hash, &[hash]) == false`. `token_accepted(plain, &[sha256(plain)]) == true`. Covers R7, F5. |
| AE4 | Listen decision with `api_keys.len()==0` and issued-key count `>=1` allows bind-all + ApiKey. Both zero still refuses. Covers R8, F4. |
| AE-OPEN | Hub-off: `GET /api/open` 200. Hub-on + `HubAuth::None`: 404. Hub-on + ApiKey + valid bearer: 404. Hub-on + ApiKey + no bearer: 401 (middleware). Covers R9, F6. This is **not** HUB list/graph AE5. |
| AE6 | Parallel `cargo nextest run --locked` (and `--features postgres` with ambient `KURULTAI_FEATURE_HUB=1`) does not flake `open_hub_store_requires_postgres_feature`. Covers R10. |

### Scope boundaries

**In:** R1–R13. First-cut deletes + four gated-auto nits. INDEX rows for files this PR deletes. README/AGENTS one-liners that name the stubs. This plan file + `docs/plans/INDEX.md` row.

**Out:** `web/`. Plugin `webui/brainapp/assets` copies. INDEX schema/protocol change. json/markdown merge. `AdminKeyStore` delete/wire/alias. AE5 list/graph/activity. HUB-5 `team_id` stamp. Tailscale `0.0.0.0`. HTTP promote policy. `KURULTAI_NAMESPACE` vs `KURULTAI_PROJECT`. F-03 “revoke-all re-enables env CSV”. Ontology Postgres 501. Closing #4.

**Deferred follow-ups (not this PR):**

1. Wire or alias `AdminKeyStore` to `HubKeyStore`.
2. AE5: thread `MaybeHubPrincipal` through list/graph/ontology/activity.
3. INDEX audit: parse Files table, flag stale rows.
4. Plugin bundle emit from `website/` build.
5. Delete `web/` only after an explicit product decision (conflicts with `docs/multi-user-kurultai.md`).

## Planning Contract

### Key Technical Decisions

- **KTD1. One PR, two commits.** Repo convention is one logical change per PR. User locked deletes + gated-auto together. Split commits so revert of auth nits does not restore stubs: `chore(dead): …` then `fix(auth): …`.
- **KTD2. Keep AdminKeyStore.** README advertises `admin key`. HTTP does not check it. Deleting the mint is a product break, not debloat.
- **KTD3. Fail closed on stub kinds.** Today AppFlowy “succeeds” and indexes nothing. After this PR, unknown/removed kinds error. Operators with `kind = "appflowy"` in toml will start failing index. That is intended. Do not keep a silent empty connector.
- **KTD4. Keep legacy 301, drop the bytes.** `LEGACY_PATHS` redirect stays so old `/ui/index.html` bookmarks still hit Brain. The five landing files leave the tree so rust-embed stops shipping ~1.35MB. Do not 404 those three names.
- **KTD5. `/api/open` gated by hub feature, not bind address.** Bind can be loopback while hub is on. Feature flag is the operator-visible switch. Solo default (hub off) keeps the route. Do not add `/api/open` to `WRITE_ROUTES` in this PR. Tests must **not** read ambient `KURULTAI_FEATURE_HUB` (Postgres CI exports it). Pass a bool into `router` / `build_app` so AE-OPEN is deterministic. Middleware runs first: ApiKey + no bearer is 401 even if the route is gone.
- **KTD6. Listen count is issued + env at every caller.** `hub_listen_decision` stays a pure function of `key_count`. **Both** `src/main.rs` Daemon preflight and `serve_with` must pass env CSV length **plus** active issued-key count. Patching only `serve_with` leaves `kurultai daemon` refusing bind-all with empty CSV. Do not `block_on` inside the sync helper. If main cannot see the store yet, move the listen decision until after hub store connect. Refuse text must not say env CSV is the only path. Do not change Tailscale / `ALLOW_PUBLIC_HUB` / connect-fail-open.
- **KTD7. Hex-only compare does not break stored hashes.** Existing test `token_matches_plaintext_or_sha256` (plain presented, hash stored) must remain true. The bug is presenting the hash itself.
- **KTD8. EnvGuard is named and mutexed.** User lock is `EnvGuard`. Capture previous value including unset. Restore on Drop including panic. Hold a crate-level mutex for the whole mutation, including `.await`. Drop-only without a lock does not kill the confirmed flake. Do not use `serial_test` as the only fix. Do not `remove_var` a key you did not capture. Prefer parking the type in an already-indexed test module so audit does not need a new basename.
- **KTD9. Soft-labels storage stays.** Distill hook is theater. Store columns and `KnowledgeAtom.soft_labels` are live. Do not drop M-series columns.
- **KTD10. Do not close #4.** AppFlowy remains a future connector issue. Removing the stub is honesty, not “won’t fix.”
- **KTD11. Two commits vs GitHub squash.** If the PR is squash-merged, KTD1’s revert story dies. Prefer a merge commit or rebase merge. If squash is mandatory, keep the PR body split Dead / Auth and accept one-commit revert.
- **KTD12. Unknown kinds already fail via `SourceKind::Custom`.** After the enum variants die, `parse_source_kind("appflowy")` becomes `Custom("appflowy")`, and registry already errors `unknown custom connector`. That is fail-closed. Do not add a special-case string unless a test needs a stable message. Do not keep empty-success.

### Assumptions

- No production config in the wild depends on AppFlowy returning empty success. Unknown; fail-closed anyway (KTD3).
- No operator depends on `/ui/index.html` serving the old marketing page. Redirects already exist.
- `main` is the integration branch. Tag `v0.5.0` is not retagged.
- Implementer has `gh` for the PR. Local Mac clone may still be behind; work from current `main`.

### Planning-time vs implementation-time

| Decide now | Defer to implementer |
|------------|----------------------|
| What is deleted vs kept | Exact match arms / error strings |
| Fail closed on stub kinds | Whether parse rejects in loader vs registry |
| 301 kept without files | How rust-embed folder listing is asserted |
| Hub feature gates `/api/open` | How the router is built in tests |
| Listen counts issued keys | Sync vs async count at `serve_with` |
| Hex-only when stored entry looks like sha256 hex | Exact hex detection |
| Drop-safe env restore | Helper location / mutex vs task-local |

## Implementation Units

### U1. **Remove AppFlowy and TechTracker stubs**

- **Goal:** No fake connectors. Unknown kinds fail. Live connectors unchanged.
- **Requirements:** R1, R12, R13. **Flows:** F1. **Covers AE1.**
- **Dependencies:** none
- **Commit:** `chore(dead):` (first commit)
- **Files:** `src/connectors/appflowy.rs` (delete), `src/connectors/mod.rs`, `src/connectors/registry.rs` (including `from_config_rejects_unimplemented_kinds`), `src/connectors/INDEX.md` (drop the `appflowy.rs` row **and** sibling Touches cells that name it), `src/types.rs`, `src/config/loader.rs` (`parse_source_kind` arms), `src/INDEX.md`, `AGENTS.md` (`tech_tracker` example), `README.md` (today says AppFlowy is **registered** but not implemented — that becomes a lie)
- **Patterns:** After deletion, Custom/unknown error is enough. Update `from_config_rejects_unimplemented_kinds` so it no longer requires `"not implemented"`. Do not replace stubs with new TODOs. README: deferred #4 / not implemented — not “removed from roadmap”. Leave #4 open.
- **Test scenarios:**
  - **Happy:** markdown/json/inbox/dayflow/pond/github still register when enabled. A **disabled** `appflowy` source does not error (only enabled sources hit `from_config`).
  - **Error:** enabled `kind = "appflowy"`, `kind = "tech_tracker"`, and alias `techtracker` fail at registry (Custom/unknown). No `Ok([])` poll path. No `AppFlowyConnector` type left.
  - **Edge:** `SourceKind` match is exhaustive after variants are gone. Clippy `-D warnings` stays green.
  - **Integration:** connector/registry/loader tests pass. Agent-index passes after INDEX rows for `appflowy.rs` are removed.

### U2. **Remove unused distill hook and `source_config`**

- **Goal:** Dead functions gone. Soft-label data model untouched.
- **Requirements:** R2, R3, R12, R13.
- **Dependencies:** none
- **Commit:** `chore(dead):`
- **Files:** `src/distill/mod.rs` (delete), `src/distill/INDEX.md` (delete), `src/lib.rs`, `src/INDEX.md`, `src/connectors/registry.rs` (`source_config` only), `src/connectors/INDEX.md` if it names the fn, `src/distill/` folder gone
- **Patterns:** `pub mod distill` is only a stub + unit test that asserts empty vec. `tests/acceptance_visibility.rs` `source_config_*` tests are **SourceConfig methods**, not `registry::source_config`. Do not touch those tests.
- **Test scenarios:**
  - **Happy:** crate builds without `distill`. Soft-label fields on atoms still compile.
  - **Error:** `rg emit_soft_labels` empty. `rg 'fn source_config'` limited to `src/connectors/registry.rs` is empty (do not hit `source_config_default_*` in acceptance tests).
  - **Edge:** `src/lib.rs` module list has no empty hole that clippy flags.
  - **Integration:** `cargo nextest run --locked` (and postgres job) still pass.

### U3. **Stop embedding the marketing landing**

- **Goal:** ~1.35MB out of the binary. Brain UI unchanged. Old landing URLs still redirect.
- **Requirements:** R4, R12, R13. **Flows:** F2, F3. **Covers AE2.**
- **Dependencies:** none
- **Commit:** `chore(dead):`
- **Files:** `ui/index.html`, `ui/index.css`, `ui/index.js`, `ui/kurultai_logo.jpg`, `ui/neural_tech_banner.jpg` (delete), `src/http/ui.rs` (keep `LEGACY_PATHS` 301), `ui/README.md` (drop “Landing showcase”), `ui/INDEX.md` (five rows)
- **Patterns:** Existing `ui_serves_css_asset` already asserts `/ui/index.css` redirects and hashed `assets/*.css` is 200. Keep both. Canon: `docs/solutions/architecture-patterns/one-brain-ui-daemon-ui-only.md` — do not add a new UI root.
- **Test scenarios:**
  - **Happy:** `/ui/` brain HTML 200. Hashed CSS 200 + immutable cache.
  - **Edge:** `/ui/index.html`, `/ui/index.js`, `/ui/index.css` still redirect to `/ui/` (today only css is asserted).
  - **Error:** requesting `kurultai_logo.jpg` is 404 (file gone). Not a product URL.
  - **Integration:** `UiAssets::iter()` has no `index.html` and no `*.jpg` at `ui/` root. Do not edit `website/` (TopBar `/ui/index.html` still 301s).

### U4. **Delete phase closeout scripts**

- **Goal:** No one-shot `gh issue close` scripts in tree.
- **Requirements:** R5, R12.
- **Dependencies:** none
- **Commit:** `chore(dead):`
- **Files:** `scripts/phase-1-closeout.sh`, `scripts/phase-2-closeout.sh`, `scripts/phase-4-closeout.sh`, `scripts/phase-5-closeout.sh` (delete), `scripts/INDEX.md` (four rows + **Does** line that currently says “closeout”)
- **Patterns:** Historical closeout docs under `docs/plans/phase-*-closeout.md` stay (protected plans).
- **Test scenarios:**
  - **Happy:** `scripts/install.sh` and `scripts/audit-agent-index.py` still present.
  - **Error:** `test ! -e scripts/phase-{1,2,4,5}-closeout.sh`. Do not repo-wide `rg` those names (protected `docs/plans/phase-*.md` keep them).
  - **Integration:** agent-index pass.

### U5. **Doctor tells the truth**

- **Goal:** Diagnostics match reality. No extra RAM tax.
- **Requirements:** R6, R13.
- **Dependencies:** U9 if tests still flip process env for the hub label.
- **Commit:** `chore(dead):` (doctor honesty is delete-adjacent, not auth)
- **Files:** `src/doctor.rs`. There is no `#[cfg(test)]` today. This unit **adds** unit tests for extracted helpers. Do not invent a full doctor suite. Do not add a `Store` count method.
- **Patterns:** Hub label = `features::enabled("hub")` (on/off), not `embed_backend`. HTTP port = clap’s `PORT` env then `8421`. Do not invent a config port key. Ontology: `COUNT(*)` (or equivalent) on the sqlite handle already in `check_ontology`. Drop `let _ = environment`. Extract tiny pure helpers (`hub_status_label`, `daemon_port`, ontology count) and unit those. Do not boot a live daemon.
- **Test scenarios:**
  - **Happy:** helper reports hub on/off from a bool or guarded env. Helper reads `PORT` then 8421.
  - **Edge:** missing config still reports, does not panic.
  - **Error:** ontology store error is a failed check, not a hang/OOM. Count path does not call `list_ontology_entities(100_000)`.
  - **Integration:** new doctor helper tests pass. No live HTTP daemon required.

### U6. **Hex env keys are not bearers**

- **Goal:** Stored sha256 hex cannot be replayed as the token.
- **Requirements:** R7, R13. **Flows:** F5. **Covers AE3.**
- **Dependencies:** none
- **Commit:** `fix(auth):` (second commit)
- **Files:** `src/http/auth.rs`
- **Patterns:** Keep plaintext-stored keys matching plaintext. Keep plaintext presented against stored hash. Reject presented==stored when stored looks like sha256 hex (detect 64-hex **case-insensitively**). Do not change `HubKeyStore::resolve_token`. Add a **new** test `token_accepted(hash, &[hash]) == false` — existing `token_matches_plaintext_or_sha256` does not cover hash-as-bearer.
- **Test scenarios:**
  - **Happy:** `token_accepted("secret-token", &[sha256("secret-token")])` true. Plaintext list still matches plaintext. Existing `token_matches_plaintext_or_sha256` stays green.
  - **Error:** new assert `token_accepted(hash, &[hash]) == false`. Wrong secret false.
  - **Edge:** mixed list `[plain, hash_of_other]` does not accept `hash_of_other` as bearer. Uppercase stored hex is not a bearer. 64-char hex that is an actual plaintext key is an accepted risk — document in the test name.
  - **Integration:** unit policy is enough (same function `hub_api_auth` uses). HTTP 401/200 for F5 is optional.

### U7. **Listen policy counts issued keys**

- **Goal:** Issued-key-only hub can bind non-loopback.
- **Requirements:** R8, R13. **Flows:** F4. **Covers AE4.**
- **Dependencies:** none (postgres feature for issued keys)
- **Commit:** `fix(auth):`
- **Files:** `src/http/hub_listen.rs`, `src/http/mod.rs` (`serve_with` call site), **`src/main.rs` (Daemon preflight)**. Look at `src/daemon/mod.rs` (second `serve_with`); do not redesign it. Read `src/hub/keys.rs` `has_active_keys`; do not change its semantics.
- **Patterns:** `hub_listen_decision` remains pure. **Both** callers pass env CSV length + issued active count. `main.rs` must not stub issued=0. Count in async callers; do not `block_on` in the sync helper. If main cannot connect the store yet, move the listen decision until after hub store connect. Update refuse reason so env CSV is not the only allowed path. Do not change Tailscale / `ALLOW_PUBLIC_HUB` / connect-fail-open.
- **Test scenarios:**
  - **Happy:** bind-all + ApiKey + env key count 0 + issued count 1 → Allow. Covers AE4. Tests lock the **sum**, not only `hub_listen_decision(key_count=1)`.
  - **Error:** bind-all + ApiKey + both counts 0 → Refuse. bind-all + auth none → Refuse (existing AE11).
  - **Edge:** loopback + auth none + zero keys → Allow (solo). Unicast uses the same match arm as bind-all.
  - **Integration:** without postgres feature, issued count is zero. Existing hub_listen unit table still passes. A wiring test (or argument on `resolve_listen_socket`) proves `main`/`serve_with` pass the sum. Connect-fail leaving `key_store=None` stays issued=0 (out of scope to fail-closed).

### U8. **Drop `/api/open` on hub**

- **Goal:** No process spawn with attacker argv on the hub router.
- **Requirements:** R9, R13. **Flows:** F6. **Covers AE-OPEN.**
- **Dependencies:** U9 if any AE-OPEN test still touches process env. Prefer a bool so U9 is not required.
- **Commit:** `fix(auth):`
- **Files:** `src/http/mod.rs` (route table + `api_open`), `tests/acceptance_http.rs`, `tests/stress_http.rs` (`build_app`). Not `plugin/api/kproxy.py` (already omitted).
- **Patterns:** Solo hub-off keeps `GET /api/open`. Hub-on **omits** the route; do not 404 inside `api_open` after `Command::new`. Do not read ambient `KURULTAI_FEATURE_HUB` inside `router()` in tests. Pass a bool into `router` / `build_app` (set from `features::enabled("hub")` only in `serve_with`). Middleware runs first: ApiKey + no bearer is 401 even if the route is gone.
- **Test scenarios:**
  - **Happy:** hub-off `GET /api/open` returns 200 (today’s contract, including missing file). Tests pass hub-off explicitly so Postgres CI ambient flag cannot flip this.
  - **Error:** hub-on + `HubAuth::None` → 404. Hub-on + ApiKey + valid bearer → 404 (not 200, not spawn). Hub-on + ApiKey + no bearer → 401.
  - **Edge:** `/api/open` is still not a `WRITE_ROUTES` member (out of scope to expand the write list).
  - **Integration:** kproxy tests unchanged. Do not thread `MaybeHubPrincipal` (that is HUB AE5, out of scope).

### U9. **EnvGuard for known racers**

- **Goal:** Parallel tests cannot leak `KURULTAI_FEATURE_HUB` / secrets. Confirmed flake dies.
- **Requirements:** R10, R13. **Covers AE6.**
- **Dependencies:** none (U5/U8 may depend on this)
- **Commit:** `fix(auth):` (same second commit; no third commit)
- **Files:** `src/store/mod.rs` (`open_hub_store_requires_postgres_feature`), **`src/store/postgres.rs`** (`open_hub_store_connects_when_flag_and_url_set`), `src/features.rs`, `src/app/context.rs`, `src/http/hub_listen.rs` env tests, `src/http/ingest.rs`, `src/http/mcp.rs`, `tests/acceptance_visibility.rs`. Do **not** drag in `tests/acceptance_concurrency.rs` (BUSY_TIMEOUT, not HUB) unless it actually mutates the same vars. If EnvGuard is a **new** tracked file, add its INDEX basename in this commit (audit is presence-only). Prefer an already-indexed `#[cfg(test)]` module.
- **Patterns:** Type named **`EnvGuard`**. Capture previous including unset. Restore on Drop including panic. Hold a crate-level mutex for the mutation **including `.await`**. Never `remove_var` a key you did not capture. Prefer constructor injection when the test does not need env. Do not use `serial_test` as the only fix.
- **Test scenarios:**
  - **Happy:** `open_hub_store_requires_postgres_feature` is stable under nextest / `--test-threads=8`.
  - **Edge:** panic inside the test still restores env (Drop). Mutex is held across await.
  - **Error:** ingest/MCP capture previous (including None) and restore; they currently clobber.
  - **Integration:** `cargo nextest run --locked --features postgres` with ambient `KURULTAI_FEATURE_HUB=1` still passes tests that set the flag to 0 (restore previous, not blindly remove).

### U10. **Index and honesty docs**

- **Goal:** Audit green. Docs do not advertise deleted stubs as code.
- **Requirements:** R12, R11.
- **Dependencies:** U1–U5 (deleted paths known). Auth INDEX stamps ride on commit 2 with U6–U9. **No third commit.**
- **Commit:** `chore(dead):` for delete INDEX/README/this plan row. `fix(auth):` for http/store INDEX if those files change.
- **Files:** `docs/plans/INDEX.md` (this plan row, commit 1), `src/connectors/INDEX.md` (`appflowy.rs` + sibling Used-by), `ui/INDEX.md` (five landing files), `scripts/INDEX.md` (four closeouts + Does line), `src/INDEX.md` (`distill/` child), `src/distill/INDEX.md` (gone with folder), `AGENTS.md`, `README.md`. Stamp `src/http/INDEX.md` / `src/store/INDEX.md` on commit 2 if auth/listen/open/EnvGuard files change. Do not edit `docs/plans/phase-*.md` bodies except a broken relative link to a deleted script — retarget to the markdown closeout, do not delete the plan. Do not change INDEX schema or the audit script.
- **Test scenarios:**
  - **Happy:** `python3 scripts/audit-agent-index.py` exits 0.
  - **Error:** no INDEX Files table cell points at a deleted path as if it still exists (quality; audit may not catch stale rows — still do it).
  - **Integration:** README still lists `admin key`. AppFlowy is deferred #4 / not implemented, not “registered”, not “removed from roadmap.”

## Risks and dependencies

| Risk | Why | Mitigation |
|------|-----|------------|
| AppFlowy toml in the wild starts failing | KTD3 changes empty-success to error | Fail closed. Mention in PR body. Leave #4 open. |
| rust-embed compile misses remaining assets | Deleting files changes the embed set | U3 tests: hashed CSS still embedded; `/ui/` 200 |
| Listen count needs async `has_active_keys` | `resolve_listen_socket` is sync today | Implementer may count before the call. Do not block the runtime on a nested runtime. Tests lock the decision, not the plumbing. |
| Hex heuristic false-negative | A 64-char hex plaintext env key would stop matching itself | Unlikely for operator secrets. Test documents the heuristic. Prefer issued keys. |
| Env mutex deadlocks | Nested guards | One process-wide lock; Drop order. Hold across the test body including await. |
| CI nextest is parallel | Default CI will flake without U9 | U9 is not local-only. Must be in the PR, not “fix later.” |
| Squash merge | KTD1 two commits vanish | KTD11. PR body: **do not squash**. |
| `parse_source_kind` leftover arms | Compile fail after enum variants die | Exhaustive match is the safety net. |
| rust-embed still ships JPGs | Forgot to delete files under `ui/` | U3 embed listing assertion. |
| Two concerns in one PR | Review noise | KTD1 two commits. PR body sections: Dead / Auth. |
| Quality drop | User constraint | R13: CI-shaped fmt/clippy/nextest + index audit. No new `allow(dead_code)`. |
| Scope creep into HUB AE5 | Same files (`http/mod.rs`); this plan’s old AE5 name | U8 is AE-OPEN only. Do not thread principals. |
| CLI daemon preflight ignores issued keys | `main.rs` resolve before store connect | U7 files include `src/main.rs`. Both callers pass the sum. |
| Hub-on `/api/open` is 401 under ApiKey | `path_requires_hub_auth` is true; middleware first | AE-OPEN matrix: 401 without bearer, 404 with bearer. |
| Postgres CI `KURULTAI_FEATURE_HUB=1` | Ambient env flips AE-OPEN / doctor / feature tests | Inject bool or EnvGuard. Restore previous, not remove. |
| EnvGuard without mutex | Drop-only does not serialize `set_var` | KTD8. |
| New EnvGuard file without INDEX row | Agent-index CI red | Prefer already-indexed test module. |
| Ontology COUNT via new Store method | Pulls postgres + mocks | COUNT stays inside doctor. |
| Doctor invents config port | Daemon uses `PORT` then 8421 | U5. |
| Repo-wide `rg` closeout / `source_config` | Hits protected plans and acceptance helpers | Narrow verify commands. |
| README still says AppFlowy “registered” | Honesty lock | U1 owns README. |
| Uppercase hex env entry | Heuristic miss; hash still a bearer | Case-insensitive detect (U6). |
| `has_active_keys` connect failure | `serve_with` warn+continues, issued=0 | Out of scope to fail-closed. Do not “fix” here. |
| Double `resolve_listen_socket` | main then serve_with can disagree | Same sum at both callers. |

**Blockers:** none to start. U1–U4 parallel. U6 independent. U7 independent of U8. U5/U8 depend on U9 **only if** they still mutate process env. U10: delete honesty on commit 1; auth INDEX on commit 2.

## Verification (whole PR)

Match CI, not a private cargo-test ritual.

1. `python3 scripts/audit-agent-index.py` → 0
2. `cargo fmt --all -- --check`
3. `cargo clippy --all-targets -- -D warnings`
4. `cargo nextest run --locked` (fallback: `cargo test --locked --all-targets -- --test-threads=2` twice)
5. `cargo clippy --all-targets --features postgres -- -D warnings` and `cargo nextest run --locked --features postgres` (job exports `KURULTAI_FEATURE_HUB=1`)
6. AE1, AE2, AE3, AE4, AE-OPEN, AE6 named tests pass
7. `rg AppFlowyConnector` empty. `rg emit_soft_labels` empty. `test ! -e scripts/phase-{1,2,4,5}-closeout.sh`
8. `AdminKeyStore` and `kurultai admin key` still compile. README CLI row still lists `admin key`
9. rust-embed listing has no `neural_tech_banner` / `kurultai_logo`

## Out-of-scope reminders for the implementer

If a later agent “helpfully” deletes `admin_keys.rs`, `web/`, or 75 INDEX files, they have left this plan. Stop.

deepened: 2026-09-01

Confidence pass 2: U7 `main.rs` both callers; AE-OPEN 401/404 matrix + inject bool; EnvGuard named+mutex; TechTracker assert string; README “registered”; narrow `rg`; CI-shaped verify; exactly two commits. No scope expansion.
