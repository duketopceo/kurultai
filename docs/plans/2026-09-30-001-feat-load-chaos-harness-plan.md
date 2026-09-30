---
artifact_contract: ce-unified-plan/v1
execution: code
---

# Load + chaos harness for the daemon and store — plan

**Date:** 2026-09-30 · **Origin:** interactive hammer session (2026-09-30) — read ramp 10–200 conc local (0 errors, ~170 rps ceiling), hosted `knowledge.shippedit.dev` writes ~133/s with zero failures; both degrade gracefully but the harness was ad-hoc and threw away its findings.

## Goal Capsule

- **Objective:** a repeatable load/chaos suite so "can it break" is a command, not a hand-rolled session — covering stress, correctness-under-concurrency, and failure modes for the daemon, store, and MCP lane.
- **Means:** a `scripts/hammer.mjs` runner (target-selectable: local daemon or hosted via CF service token / browser-session injection) plus Rust-side chaos tests for things JS can't reach (kill-mid-write, disk pressure).
- **Stop conditions:** no hosted data mutation without a `hammer-test` namespace + cleanup path; no new deps in the daemon binary; harness lives in `scripts/`, not `src/`.

## Key technical decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Harness = **`scripts/hammer.mjs`** (Node, zero-dep fetch) | Same shape as `scripts/ui-smoke.mjs`; runs anywhere node does. Not a Rust bench — we want HTTP-surface truth, not internal microbench. |
| D2 | Target selection: `--target local` (127.0.0.1:8421) default; `--target hosted` requires an explicit auth header or the BrowserOS page-eval bridge | Hosted needs Access; the script accepts `--auth "Authorization: Bearer …"` and documents the browser-eval trick for session-cookie runs. Never hardcode a credential. |
| D3 | Write tests always write into a `hammer-test*` namespace (thread name / atom source tag) and print a cleanup recipe | Prod write tests must be self-identifying and reversible — one `DELETE … WHERE source LIKE 'hammer%'` or thread-scoped delete. |
| D4 | Chaos cases that need process control (kill mid-write, disk full) are Rust tests in `tests/`, not the JS harness | JS can't SIGKILL a child or fill a tmpfs reliably; `tests/chaos.rs` spawns a real daemon subprocess. |
| D5 | Report = compact table to stdout + optional `--json` artifact under `artifacts/` | Screenshots/tables are how we've been judging; keep it paste-able. |

## Sequencing

1. **U1** Harness core — concurrency ramp, weighted route mix, latency histogram, RSS/health watcher
2. **U2** Read-path suites — baseline ramp + adversarial search + payload ceiling
3. **U3** Write suites — burst writes, write-write races, turn-cap enforcement, board consistency
4. **U4** Chaos suite (Rust `tests/`) — kill mid-write, disk pressure, cold start
5. **U5** MCP stdio hammer — `kurultai mcp` under concurrent tool calls
6. **U6** Soak + leak watch + CI wiring decision

## Implementation Units

### U1 — Harness core

- Files: `scripts/hammer.mjs` (new), `scripts/README.md` or INDEX row update.
- CLI: `--target <url>`, `--auth <header>`, `--conc <list>` (default `10,50,100,200`), `--dur <ms>` per level, `--suite <name>` (default `read`), `--json <path>`.
- Internals: weighted route pool, worker loop, percentile stats (p50/p95/p99/max), RPS, error buckets (conn err vs non-2xx), pre/post health check + daemon RSS sample when local.
- Reuse today's proven pattern: `fetch` + `AbortSignal.timeout`, `performance.now()`, per-level `Promise.all` workers.
- **Tests:** `node scripts/hammer.mjs --target http://127.0.0.1:8421 --conc 5 --dur 2000` exits 0 with a table; `--suite bad-route` reports the 404s correctly (not as errors).

### U2 — Read suites

- Files: `scripts/hammer.mjs` (suite registry).
- `read` suite: today's mix — `/api/graph`, `/api/graph?tier=hot|warm|cold`, `/api/search`, `/api/atoms`, `/api/status`, `/api/hey/threads`, `/api/ontology`, `/api/metrics`, `/ui/`.
- `search-adversarial` suite: FTS injections (`" OR "`, `NEAR/0`, `*` wildcards, 10k-char query, unicode/emoji, nested quotes, `limit=100000`). Success criterion: daemon returns 4xx or bounded results; never 500/hang/crash.
- `payload-ceiling` suite: `POST /api/hey` and `/ingest`-shaped bodies at 1KB/100KB/1MB/10MB; asserts server rejects gracefully above its limit (or accepts and stays responsive).
- **Tests:** adversarial suite against local daemon — daemon alive after, no `err>0`, latencies bounded.

### U3 — Write suites

- Files: `scripts/hammer.mjs` suites `write-burst`, `write-race`, `turn-cap`, `consistency`.
- `write-burst`: concurrent `post_message` into `hammer-test-x` thread (already exists on hosted with cap 10000) mixed with concurrent reads — today's exact protocol, now repeatable.
- `write-race`: N writers posting to the *same* thread with same `parent_id`, concurrent reacts toggling on one message, then fetch-and-verify: count messages, check no dupes by id.
- `turn-cap`: agent (not admin) auth token, post past `turn_cap` — expect 4xx once cap hit (admin posts bypass it today; that's a known gap worth a unit test documenting the behavior).
- `consistency`: interleave post + list + delete on one thread; assert list is never torn (ids unique, ordered, no partial rows).
- **Cleanup:** suite prints the SQL/endpoint to wipe `hammer-test%` rows; hosted cleanup = server-side SQL or per-message DELETE loop.
- **Tests:** local daemon run — races produce no dupes, cap test documents actual behavior, consistency never tears.

### U4 — Chaos (Rust tests)

- Files: `tests/chaos.rs` (new), plus helpers under `tests/common/` if a daemon-spawn helper doesn't exist.
- `kill_mid_write`: spawn `kurultai daemon` on a scratch store, drive writes via HTTP from the test, SIGKILL mid-burst, restart, assert store opens clean + counts are consistent (SQLite WAL recovery).
- `disk_pressure`: point storage at a small tmpfs (or ulimit the file), fill it, assert writes fail loudly (not silently swallowed) and reads still serve.
- `cold_start`: time first `/api/graph` + `/api/search` after daemon spawn on a populated store — records the cold-cache penalty number.
- **Tests:** the tests themselves; each is hermetic (scratch store in `target/`), safe in CI except `disk_pressure` (gate it `#[ignore]` if tmpfs isn't portable).

### U5 — MCP stdio hammer

- Files: `scripts/hammer-mcp.mjs` or extend `hammer.mjs` with `--lane mcp`.
- Spawn `kurultai mcp` stdio child, fire N concurrent JSON-RPC tool calls (`search`, `remember`, `who_knows`, `hey_post`), measure per-tool latency + errors.
- This is the real agent lane — HTTP numbers don't cover stdio serialization.
- **Tests:** 50 concurrent `search` calls all return valid results; `remember` under load lands in the store (verified via `/api/atoms` count delta).

### U6 — Soak + report

- `soak` suite: sustained `--conc 10 --dur 600000` (10 min) with a watcher polling RSS/fd count every 30s; flag monotonic growth (>20% over baseline = leak suspect).
- Report: markdown table to stdout + `--json` artifact; summary row per suite with pass/fail against thresholds (e.g., `err==0`, `p99 < 5s` for hosted reads).
- Decide CI wiring: nightly-only job vs manual — probably manual/on-demand, not per-PR (too slow, hits external state).
- **Tests:** soak on local daemon 10 min → RSS delta reported; JSON artifact parses.

## Non-goals

- No changes to the daemon's actual concurrency model — this is measurement, not the fix.
- No hitting hosted without auth supplied; no writes outside `hammer-test*` namespace.
- No load-balancer/tunnel changes — the CF layer is measured, not modified.

## Risks / open questions

- Hosted hammering writes real rows — even namespaced, 1.7k test messages exist on prod board right now. Decide cleanup SOP (probably: suite auto-deletes its rows at end unless `--keep`).
- `disk_pressure` portability (tmpfs on macOS vs Linux) — gate with `#[ignore]` + `--ignored` flag if flaky.
- Turn-cap behavior under admin vs agent auth needs documenting — the test may reveal the cap only binds agents, which is a finding, not a failure.
