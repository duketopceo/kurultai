---
artifact_contract: ce-unified-plan/v1
product_contract_source: docs/brainstorms/2026-09-28---device-broker-agent-onboarding-requirements.md
execution: code
---

# Device broker: one upstream session, per-chat minted keys — plan

**Date:** 2026-09-28 · **Origin:** `docs/brainstorms/2026-09-28---device-broker-agent-onboarding-requirements.md` (decisions D1–D4 settled there)

## Goal Capsule

- **Objective:** every agent on a machine boards a local broker and reaches `knowledge.shippedit.dev` with a minted per-chat key — no per-agent upstream auth, stable chat identity across resumes, rotating session key per launch.
- **Means:** a long-lived `kurultai broker` localhost service holding one upstream device session + an auto-mint board endpoint + a full MCP proxy surface; `kurultai init --agent <x> --broker` points harnesses at it.
- **Stop conditions:** no change to human Cloudflare Access paths; no upstream token ever stored in an agent's config/env; transcript atoms never land in hot tier.

## Key technical decisions

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | Broker = **long-lived local service**, `kurultai broker` (localhost port + unix socket), NOT a per-process stdio shim | Requirement D1 settled this; a persistent broker enables queueing, dedupe, one upstream connection, and survives agent churn. |
| D2 | Upstream transport = **HTTPS to existing `/api/*`** on knowledge.shippedit.dev using the device's seat token from the existing RFC 8628 flow (`src/http/device.rs`, `issue_agent_seat_token`) | No new upstream surface except the identity-stamp fields; Cloudflare Tunnel unchanged. |
| D3 | Agent-side surface = **same stdio MCP** (`src/mcp/server.rs` `run_stdio_with`) — a new `RemoteBrainService` implementing the `BrainService` trait over broker HTTP instead of local `Store` | Tool handlers, surfaces, and tests are already built against the trait; the proxy is a backend swap, not a new protocol. `init --agent --broker` writes configs that launch `kurultai mcp --broker`. |
| D4 | Identity stamping rides **upstream API extensions**: hosted `/api` accepts `x-kurultai-agent`, `x-kurultai-chat`, `x-kurultai-session`, `x-kurultai-device` on write endpoints (remember/hey/promote); broker injects from its session registry | Requirement D4: chat identity is data attached to writes, session key is auth. Server-side change is a thin accept-and-store of headers already analogous to Hey `instance_id`. |
| D5 | Broker board: `POST /board {agent, chat_name, chat_id, instance_id}` → mints session key, returns it; agent stdio launch includes `KURULTAI_SESSION_KEY` + chat env vars injected by `init --broker` wrapper | Session key rotates per launch automatically; chat identity stable because the harness env supplies the same chat_id on resume. |
| D6 | Local queue: broker keeps a spool (SQLite table `broker_outbox`) for failed upstream writes; flush loop on reconnect | Requirement R5; node3/server001 are always-on but laptop sleeps. |

## Sequencing

1. **U1** Broker skeleton + upstream session bootstrap (device flow reuse)
2. **U2** Board endpoint + session/chat key registry
3. **U3** `RemoteBrainService` + `kurultai mcp --broker` proxy surface
4. **U4** Upstream identity stamping (hosted accept of stamped headers)
5. **U5** `init --agent --broker` wiring + offline queue
6. **U6** Remote-machine bootstrap (node3/server001), revocation, dogfood

## Implementation Units

### U1 — Broker skeleton

- Files: `src/broker/mod.rs` (new), `src/broker/server.rs` (axum localhost listener: `127.0.0.1` port + unix socket), `src/main.rs` (`kurultai broker` subcommand), `src/config/file.rs` (`[broker]` section: upstream_url, port, socket path).
- Upstream session: on start, load seat token for this device's codename (`broker@<hostname>`-style codename via existing `issue_agent_seat_token` path or stored token from a one-time `kurultai connect`-style flow); if missing, run the device flow (reuse `src/http/device.rs` client pieces, extracting them if needed).
- Follow `src/http/mod.rs` axum patterns. Keep broker listener strictly `127.0.0.1` + socket — never `0.0.0.0` (R6).
- **Tests:** broker boots, refuses bind to non-loopback, upstream session token resolves from keyring/omaseal path.

### U2 — Board + key registry

- Files: `src/broker/board.rs`, `src/broker/registry.rs`, migration for `broker_sessions` + `broker_chats` tables (local broker sqlite, NOT the knowledge store — broker state is device-local).
- `POST /board` mints `sess_<rand>` keyed to `(agent_codename, chat_id, chat_name, instance_id, hostname)`; `chat_id` provided by caller or derived `sha(chat_name)` fallback; same `(agent, chat_id)` re-board → same chat row, new session key (D4).
- `GET /board/whoami` returns the minted identity (debug/verify path).
- Session keys: opaque, stored hashed (sha256), expiring on broker restart or explicit revoke (R4).
- **Tests:** mint once/re-mint new session key same chat_id; revoke single chat leaves others working; unkeyed request → 401.

### U3 — MCP proxy surface

- Files: `src/mcp/remote_brain.rs` (new — `BrainService` impl over broker HTTP), `src/mcp/mod.rs`, `src/main.rs` (`kurultai mcp --broker` flag).
- `run_stdio_with` already carries `WriteContext` (`src/mcp/server.rs:150`) — extend WriteContext to carry `(agent, chat_id, session_key)`; RemoteBrainService forwards each tool call to `POST http://127.0.0.1:<port>/mcp` which the broker translates into upstream `/api/*` calls.
- Tool coverage = full surface (requirement D3): reuse `tool_defs_for`/`call_tool` unchanged — the trait boundary makes this mechanical.
- **Tests:** existing `src/mcp/server.rs` tests run against a mock RemoteBrainService; new test asserts stamped headers on write calls.

### U4 — Upstream identity stamping

- Files: `src/http/mod.rs` (extract `X-Kurultai-*` headers on write routes), `src/http/device.rs` or a thin `src/http/stamp.rs`, `src/store/` persistence of chat/session metadata on `remember`/`hey` writes (column or tags — prefer metadata field, not tag spam).
- Keep it additive: missing headers → behaves exactly as today (no auth weakening).
- **Tests:** remember atom carries `agent+chat+session` metadata; hey post carries instance_id + chat; absent headers unchanged.

### U5 — Agent wiring + offline queue

- Files: `src/mcp/init.rs` (`wire_agent` gains `--broker` mode writing `kurultai mcp --broker` command + env block `KURULTAI_CHAT_ID`/`KURULTAI_CHAT_NAME` where harness supplies it, else placeholder prompting `kurultai board --chat`), `src/broker/outbox.rs` (spool + flush loop).
- Chat-id discovery per harness is the known-thin spot: Cursor exposes chat context differently than Devin/Codex. Rule: harness env > `KURULTAI_CHAT_ID` env in MCP config > `board --chat` command > `default` chat. Document in plan notes.
- **Tests:** `init --agent cursor --broker` writes broker-pointing config; outbox survives broker restart and flushes on reconnect.

### U6 — Remote bootstrap + revocation

- Files: `deploy/` notes or `docs/HANDOFF-broker-node3-server001.md`; `src/broker/revoke.rs` (`kurultai broker revoke --chat <id>` / `--device`); systemd user unit example for node3/server001.
- Dogfood: run broker on omarchy-max; board this Devin session; verify a `remember` write lands on knowledge.shippedit.dev stamped with chat identity.

## Test scenarios (per unit, enumerated in each unit above)

Cross-cutting: `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`; broker + mcp integration test spawning broker on ephemeral port and one stdio client end-to-end (board → remember → assert upstream stub received stamped call).

## Risks

- **Harness chat-id discovery** (U5) is the fragile dependency — degrade to `default` chat rather than block.
- **Upstream single-token blast radius** — mitigated by revocation (U6) + the token living only in the broker's keyring, never in agent configs (R6).
- **Store schema churn** — U4's metadata column is additive-only; no migration of existing atoms.
