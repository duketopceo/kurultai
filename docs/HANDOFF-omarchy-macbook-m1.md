# Handoff — omarchy-macbook-m1 → omarchy-max

**Date:** 2026-09-19 · **Machine wiped after this.** Pick up on `omarchy-max`.

## Machine identity changed — seat-id will differ

The old machine's persistent seat was `lukekimball-736544` (`~/.config/kurultai/seat-id`,
minted by `connect.rs` → `persistent_seat_id`). On the new Mac the hostname differs, so a
fresh seat-id (`<new-host>-<rand>`) mints on first `kurultai connect`. That is intended —
seats are per-machine, credentials are per-seat (`{lane}-{codename}-{seat}-agent-token` in
omaseal). Old seats (`omarchy`, `lukekimball-736544`, `devin@pace-server`) stay healthy
server-side; revoke them via `agent revoke` if the machine is gone for good.

## Shipped today (all on `main`, deployed to knowledge.shippedit.dev)

- **#356** Hey whole-thread view + id-vs-name resolution (thread-hop fix)
- **#357** Mobile horizontal overflow fix (dead `.nav-links` selector + cascade order)
- **#358** `connect` persistent seat-ids + seat-scoped credential names — fixed the
  omaseal clobber that caused seat-token 401s (`dev-devin-agent-token` overwritten by a
  second seat). Prod store also purged of `luke.k@bartlettroofs.com` approver rows →
  `duketopceo@gmail.com` (agent_seats + device_flows).
- **#359** Brain `max` tier renders full cortex (sprite cap 20k, density-shrunk nodes)

## Open items

1. **Watcher hot-loop (unfixed, real bug):** the dev daemon's notify watcher on
   `~/Documents/kurultai` fired ~2,000 index cycles in 46 min (824% CPU) — `watch_session`
   in `src/daemon/mod.rs` debounces bursts but has **no minimum interval between
   watch-triggered cycles**. Proposed fix: `WATCH_MIN_INTERVAL ≈ 30s` floor so sustained
   event streams can't hot-loop. Trigger source unidentified (external mass-rewrite of the
   notes dir; daemon was killed before the writer could be caught — use
   `inotifywait -m -r <root>` live if it recurs).
2. **Argus PR lane:** commit `1181d32` (`feat: wire Argus reviewer PR lane`) sits on
   `feat/argus-pr-review` unmerged; also `stash@{0}` on that branch (unknown WIP — inspect
   `git stash show -p` before it ages). Its `run:'false'` needs flipping to ship; repo still
   needs `OPENROUTER_API_KEY` secret added by Luke for Jev/Argus lanes.
3. **Pending devin presence:** `devin@lukekimball-736544` seat verified 200 + presence
   posted to hey.md on knowledge.shippedit.dev. New machine should `kurultai connect`
   fresh — do NOT copy the old seat-id file or omaseal entries.
4. **Jev/ontology state (hosted):** 11,568 atoms scanned ($0.17), 8,986 judged
   ontology-worthy, 3,180 docs batch-promoted → 3,181 `instance_of` links live. Listing
   artifacts were `/tmp/jev_ontology_listing.md` + `/tmp/jev_onto_scan.json` — **both wiped
   with this machine**; regenerate via the Jev scan if needed.
5. **Review-runner TODOs** (from earlier session, partially shipped in #352): offline
   tests + docs rows for `src/eval/review.rs` were still open when the session moved on.

## New-machine bootstrap

```
cargo build --release && cp target/release/kurultai ~/.local/bin/kurultai
kurultai connect https://api-knowledge.shippedit.dev --codename devin   # approve in browser
```

Mints `devin@<new-host>-<rand>` seat → stores `dev-devin-<seat>-agent-token` in omaseal.
