# Live sources and the hosted Kurultai brain (work.shippedit.dev)

`work.shippedit.dev` (`https://api-work.shippedit.dev/mcp`) is Bartlett's single knowledge brain. There is **no local Kurultai daemon or SQLite store** on `hermes-bartlett-omarchy`.

## 1. Syncing non-repo corpora

Mail, Drive meeting notes, and Slack history are pulled daily by `/opt/data/scripts/kurultai_sync_sources.py` (cron `c8b981a9588c` at 09:00 MST) into `/opt/data/kurultai-sources/{meeting-notes,emails,slack}` and pushed directly to `https://api-work.shippedit.dev/mcp` via MCP `remember` using the host's device-flow seat key (`KURULTAI_WORK_SEAT_KEY` in `/opt/data/.env`).

Run it with `/opt/data/gvenv/bin/python3` — the host python has no `googleapiclient`.

A good run prints one JSON line and finishes in under ~15s:

```json
{"new_notes": 0, "new_emails": 3, "updated_slack_channels": 3, "pushed_to_work_shippedit_dev": 3, "hosted_atoms": 3764, "hosted_trusted": 3764, "hosted_quarantine": 0}
```

Syncing checks the `hosted_atoms` delta, not the local file counts: a file can exist in `kurultai-sources/` and still never have been pushed.

## 2. Verifying the hosted brain

```bash
curl -s -H "Authorization: Bearer $KURULTAI_WORK_SEAT_KEY" \
  https://api-work.shippedit.dev/api/status
```

Confirm `atoms`, `brain.trusted_count`, and `brain.quarantine_count` before drawing conclusions from an empty search result. If a Google-backed source returns 0 new items when new notes/emails exist, check the granted scopes in `/opt/data/google_token.json`.

## 3. Sync-run pitfalls

- **Hang guard:** set `socket.setdefaulttimeout(15)` at import time. Without it a stalled Google API call blocks forever and the whole run dies on the harness timeout instead of reporting per-source results.
- **Small pages:** Drive `pageSize=15`, Gmail `maxResults=15`. Fetching 50 of each turns one run into minutes of per-message downloads; the sync is incremental, so a small page still catches up over successive runs.
- **Dedupe digest is length-based:** entries are keyed `sha256(filename:len(summary))`. A corrected item whose summary length is unchanged will silently not re-push — force it by changing what goes into the digest or dropping the key from `.pushed_work_shippedit.json`.
- **Editing the script:** make the edit with `write_file`/`patch` against the real file, then run it as a subprocess. Building multi-line Python string literals inside an `execute_code` cell collapses the literal newlines and raises `SyntaxError: unterminated string literal`; `write_file` has no such failure mode.
- **Swallowed errors:** every per-item block is `except Exception: pass`. A silently empty result usually means a scope/auth failure, not an empty inbox — check `hosted_atoms` and run one source function directly, verbosely, when numbers look wrong.

## 4. Retiring a local daemon on a host (one-time cutover)

When a host has been given a hosted seat, remove the local stack instead of leaving a second, stale brain:

1. `systemctl --user stop kurultai.service && systemctl --user disable kurultai.service`, delete the unit, then confirm nothing answers on port 8421.
2. Remove the binary, `~/.local/share/kurultai`, `~/.config/kurultai`, `kurultai-config.toml`, and the local upkeep/purge scripts.
3. Delete the related cron entries (quarantine purge, local index refresh) — a purge job pointed at a dead store only generates noise.
4. Rewrite the skill and repo docs so they describe the hosted endpoint only; a stale "start the daemon" instruction is how a future session resurrects the retired path.
5. Prove the cutover: run the source sync, then read `/api/status` and confirm atom counts grew.
