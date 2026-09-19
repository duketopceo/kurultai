# kurultai connect — Device Authorization Flow — Requirements

**Date:** 2026-09-18 · **Status:** approved for planning · **Issue:** kurultai#332

## Outcome

`kurultai connect <instance-url>` replaces manual agent-key minting. One browser login → the hosted lane mints an agent key bound to `(codename, instance_id)` → the CLI stores it in the system keyring and wires MCP configs. Nobody ever sees or pastes a token. New-machine onboarding goes from "ssh to server-001, `agent add`, copy key, `omaseal set`, edit configs" to one command.

## The flow

```
$ kurultai connect https://knowledge.shippedit.dev [--codename X] [--instance-id Y] [--agent all|cursor|claude|…] [--no-open]

  Opening browser… sign in (Cloudflare Access).
  ✓ Approved by <human identity>
  ✓ Codename "devin" claimed (or resumed — same codename, seat via instance_id)
  ✓ Agent key minted, scoped to this instance
  ✓ Stored in system keyring — never on disk
  ✓ MCP wired: cursor, claude, codex, hermes configs updated
```

1. `POST /api/device/code` → `{ code, verify_url, expires_in, interval }`. CLI prints the code, opens `verify_url`, polls `POST /api/device/token`.
2. `GET /connect?code=…` on the daemon — behind **Cloudflare Access** (settled decision: CF Access JWT is the human identity for v1; no new auth system). Approve page shows the requested codename + approve/deny.
3. On approval the daemon mints an agent key bound to `(codename, instance_id, approving human)`; `POST /api/device/token` returns `428 pending` until then, then `200 { agent_key, codename, instance_id }`.
4. CLI writes the key to the keyring (omaseal-style reference where supported, `security::write_key_file` fallback), then reuses the existing `init` MCP-wiring code so `--agent all` updates every client config.

## Hard requirements

- **Anti-overlap is the server's job.** Codename = product family, unique per lane; concurrent seats distinguished by `instance_id` (auto-derived from hostname, overridable). Collision → error suggesting `--codename`, or resume via `instance_id`. Never `devin-2`. (Doctrine: luke-agents AGENTS.md §0a.)
- **Non-TTY safe.** No browser on the box → prints code + URL and still polls; works over SSH. `--no-open` prints URL only.
- **Keys are revocable** via existing `agent` commands; a revoked key → clean 401 with "run kurultai connect" hint.
- **Zero token exposure.** Key travels only over the polling channel into the keyring. Nothing writes it to dotfiles, shell args, or logs.
- **Honest limitation documented:** v1 works only on Cloudflare-Access-fronted lanes (`knowledge.`/`work.shippedit.dev`). Self-hosted/localhost daemons are out of scope — a local-trust or OAuth fallback is deferred, not promised.

## Scope boundaries

- **In:** device code/token endpoints, CF Access-gated approve page, key minting bound to `(codename, instance_id)`, CLI `connect` subcommand, keyring storage, MCP wiring reuse, revoke + 401-hint behavior.
- **Deferred:** GitHub OAuth or any self-host approval path; multi-tenant approval UX; changes to Hey/atom auth itself.
- **Outside identity:** this mints agent keys only — it does not change what an existing key can do.

## Success criteria

- Fresh machine: `connect` → `kurultai search` via the hosted lane works, zero pastes.
- Second seat, same codename: same codename, distinct `instance_id`, both live.
- Revoked key → 401 + "run kurultai connect" hint.
- `cargo test` / `clippy` green; `INDEX.md` updated.

## Outstanding questions

- None blocking. Implementation shape (endpoint paths, poll intervals, expiry, approve-page UX) belongs to planning.

## Notes for planning

- The daemon already has the agent-key machinery (`kurultai agent add` prints a one-time key) — `connect` mints through the same path after approval, not a new key type.
- The Hey `require_writer` work (humans posting with CF-verified identity) is the adjacent auth seam — reuse its Access-JWT extraction if it landed.
- Related: kurultai#333 (CF service token + agent-token audit) stays relevant for raw-curl automation — `connect` does not eliminate service tokens.
