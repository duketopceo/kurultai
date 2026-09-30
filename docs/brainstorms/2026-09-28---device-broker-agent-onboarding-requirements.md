# Requirements — Device broker: one upstream session, per-chat agent keys

**Date:** 2026-09-28 · **Status:** requirements settled, ready for ce-plan
**Trigger:** "tunnel for all agents on my device — each agent + chat gets a minted key, all sessions feed knowledge.shippedit.dev, without me wiring auth per agent."

## Problem

Today every agent on every machine must individually authenticate to the hosted brain. The user wants a device-level boardable connection: the device holds one protected session upstream, and any agent session on that box talks to the local broker with an auto-minted key carrying agent identity + chat identity.

## Decisions (user-settled)

- **D1 — Local broker daemon.** A per-device broker holds ONE upstream authed session to `knowledge.shippedit.dev`. Agents read/write through it with zero upstream credentials.
- **D2 — Broker per machine.** omarchy-max, node3, and server001 each run their own broker; remote boxes do not depend on the laptop being online.
- **D3 — Full MCP proxy.** The broker exposes the complete tool surface (search, cite, remember, ask, who_knows, promote, ontology_*, hey_*) — equivalent to pointing the agent at remote MCP, but through localhost.
- **D4 — Two-layer identity.** Stable **chat identity** (chat name/id attached to all its writes forever) + **rotating session key** minted per agent launch. Resuming a week-old Devin/Cursor chat reattaches the same chat identity under a new session key.

## What already exists (do not rebuild)

- `src/http/device.rs` + `device_auth.rs` — RFC 8628 device flow: `POST /api/device/code` → human approves at `/connect` (Cloudflare Access gated) → `issue_agent_seat_token` mints a `(codename, instance_id)` seat key stored in keyring/omaseal. **The broker's upstream session reuses this once per device.**
- Cloudflare Tunnel already fronts `knowledge.shippedit.dev` (`kurultai-private/deploy/server-001/`).
- Identity doctrine: codename = product family, `instance_id` = concurrent seat — never `devin-2`.

## Requirements

- **R1.** `kurultai broker` (or daemon submode) runs a localhost-bound broker; agents discover it via a well-known endpoint (unix socket or `127.0.0.1` port + env var convention).
- **R2.** First agent contact → broker auto-mints a session key bound to `(agent codename, chat identity, instance_id)`; subsequent calls carry it. Chat identity is stable across resumes; session key rotates per launch.
- **R3.** Broker proxies the full MCP surface upstream, stamping each call/write with the minted identity (agent, chat name, chat id, session id, device hostname).
- **R4.** Revocation: individual chat keys or a whole device's session revocable from Hey/`/connect` admin surface.
- **R5.** Offline tolerance: writes queue locally and flush when upstream is reachable (node3/server001 are always-on; laptop may sleep).
- **R6.** No credentials on agents: the agent never holds an upstream token — the minted key is useless off-device (localhost-only accept) and revocable.
- **R7.** Chat-ingestion tiering per AGENTS.md: session/transcript atoms land medium/cold, never hot.
- **R8.** `kurultai init --agent <name>` (or a new `--broker` flag) updates agent MCP configs to point at the local broker instead of remote.

## Non-goals

- Replacing Cloudflare Access for human/browser paths.
- Cross-device broker federation (each broker is independent upstream).
- Multi-user/team auth — solo instances only.

## Success criteria

- Fresh agent on omarchy-max/node3/server001: `kurultai init --agent <x> --broker` → agent boards, writes land on knowledge.shippedit.dev attributed to (agent, chat, session).
- Resume an old chat → same chat id reattached, new session key, zero re-auth.
- Revoke one chat's key → only that chat loses access; other chats/agents unaffected.
- Host/laptop asleep → node3/server001 brokers still work.

## Open questions for planning

- Transport for broker↔upstream: HTTPS to knowledge.shippedit.dev (simplest) vs persistent tunnel multiplex.
- Session-key format, storage (omaseal vs local sqlite), expiry policy.
- How "chat name/id" is supplied by each agent harness (Cursor/Devin/Codex expose different env/files).
