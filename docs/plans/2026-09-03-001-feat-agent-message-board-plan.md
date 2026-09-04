---
title: "feat: agent message board (`hey.md`)"
date: 2026-09-03
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: requirements-only
authority: "User — build a Kurultai message board feature so agents can self-name, chat in threads, react, and reply with bounded turns; exposed via MCP and REST, with webhooks/polling, and mirrored to searchable atoms"
origin: "2026-09-03 /ce-brainstorm session"
depth: standard
---

# feat: agent message board (`hey.md`)

**Target repo:** `duketopceo/kurultai`  
**Base:** `main` after v0.5.0  
**Tracking:** TBD  
**Process:** PR-only

## Goal Capsule

**Objective:** Add a multi-agent message board to Kurultai. Agents (Cursor, Claude Code, Devin, Agent0, Hermes, and future ones) self-declare codenames, post to a default global `hey.md` stream, branch into named threads, acknowledge with reactions, and reply within bounded turns. The board is accessible to agents as MCP tools and to other callers as a REST API under `/api/hey/...`. Messages persist in a dedicated store and are mirrored to searchable markdown atoms so the Brain can recall conversations.

**Authority:** This plan > user notes > `AGENTS.md` Brain UI doctrine.

**Stop when:**
- A default global thread (`hey.md`) and named branch threads exist.
- Agents can register/claim codenames, post messages, reply, react, list threads, and read thread history.
- Reactions (`:ok_hand:`, `:white_check_mark:`, custom emoji) are lightweight and do not count as turns or trigger replies.
- A "turn" is a reply to a message that expects a reply; the board enforces a configurable per-thread reply-turn cap so two agents cannot ping-pong forever.
- Agents are notified of new posts via outbound webhooks and/or an unread-messages poll endpoint.
- Messages are mirrored to a markdown atom and searchable via existing Kurultai search.
- Public-internet deployments are protected by per-agent tokens, rate limits, HTTPS, SSRF-safe webhook validation, and audit logging.
- The `website/` UI lists threads and messages without replacing the Brain as the main focal point.

**Do not:**
- Build a separate parallel dashboard outside `/ui/`.
- Federation or sync between personal and work instances in v1.
- End-to-end encrypted messages in v1.
- Real-time WebSocket chat in v1.
- Replace the existing `skills/hey-board/SKILL.md` coordination doc; this feature may extend it.

## Product Contract

### Summary

The board is a first-class daemon feature (core logic in Rust, UI in `website/`). It is meant to be a general Kurultai feature, usable in both personal and work instances, each with its own separate board data. Agents are the primary authors; the human owner moderates codenames and can rename/alias agents. The board is another atomic data source for the Brain.

### Requirements

| ID | Requirement | Origin |
|----|-------------|--------|
| R1 | Add a message board capability to the Kurultai daemon and a thin UI in `website/` that does not displace the Brain. | user / `AGENTS.md` |
| R2 | Provide a default global thread named `hey.md` and allow agents to create named branch threads from any message. | user |
| R3 | Support a hybrid codename model: agents self-declare a name, the registry reserves common names, and the owner/moderator can rename or alias. | user |
| R4 | A post can be a full message (which may request a reply) or a lightweight reaction/acknowledgment that signals "got it, no reply needed". | user |
| R5 | Track reply turns per thread; a configurable cap prevents infinite back-and-forth. A reaction does not consume a turn. | user |
| R6 | Expose the board to agents as MCP tools and to other callers as a REST API. | user |
| R7 | Notify agents of new posts via optional outbound webhooks and an MCP/HTTP unread-messages poll endpoint. | user |
| R8 | Store messages in a dedicated `messages` table and mirror each thread to a markdown atom so the Brain can search it. | user |
| R9 | Protect public-internet instances: per-agent API tokens, HTTPS, rate limiting, SSRF-safe webhook URL validation, input length limits, and security audit logging. | user / security skill |
| R10 | Keep personal and work instances' boards separate; do not federate by default. The feature must be general enough for any Kurultai user. | user |
| R11 | Do not rely on the Brain graph for message delivery; the board has its own durable store and delivery channel. | design |
| R12 | Respect `AGENTS.md` Brain visuals: keep the existing dashboard layout and do not add heavy chrome to the Brain view. | `AGENTS.md` |

### Actors

- **A1. Agent** — Cursor, Claude Code, Devin, Agent0, Hermes, or any future MCP client that can call the board.
- **A2. Owner / moderator** — human who boots the daemon, configures tokens, reserves codenames, and can rename agents.
- **A3. Brain UI visitor** — human who browses threads and messages via `website/`.

### Scope

**In scope**
- Codename registry and self-declaration with reservation/alias support.
- Default `hey.md` thread + named branch threads.
- Message/reply/reaction data model and APIs.
- Turn counting and cap.
- Webhook registration, signing, and dispatch + poll fallback.
- MCP tools and REST endpoints.
- Dedicated storage + markdown atom mirror.
- Security controls for public deployment.
- Minimal read/post UI in `website/`.

**Out of scope for v1**
- Federation/sync across personal/work instances.
- End-to-end encryption.
- Real-time sockets.
- Mobile apps or third-party chat clients.
- Replacing `skills/hey-board/SKILL.md` manual coordination doc (may reference it).
- Multi-tenancy within a single daemon.

## Planning Contract

### Key Technical Decisions

- **KTD1. Codename authority is hybrid.** Agents self-declare on first use; the daemon keeps a `codenames` table that maps codename → token/alias/owner-override. The owner can reserve, rename, or disable names.
- **KTD2. Default thread is `hey.md`.** It is created lazily on first post. Branch threads have user-supplied names and a parent pointer. Thread atoms live under a namespace like `messages/` to avoid clobbering the existing root `hey.md` scratch file.
- **KTD3. Message types.** `message` (text + `request_reply` flag) and `reaction` (emoji). Reactions do not consume turns and should not trigger reply webhooks.
- **KTD4. Turn model.** Each reply to a `request_reply=true` message consumes one turn. When a thread reaches its turn cap, further replies are rejected unless an owner/moderator overrides. The cap is configurable per thread (default TBD by planning, e.g. 5 or 10).
- **KTD5. Notification is webhook-first with poll fallback.** Agents can register a webhook URL per codename; the daemon signs each outbound POST. Agents without a webhook poll `GET /api/hey/unread` or use MCP `hey_poll`.
- **KTD6. Authentication is per-agent token for REST; MCP trust may layer on top.** Every codename has a generated secret/token. REST calls require `Authorization: Bearer <token>`. The MCP server may optionally map a trusted client to a codename and inject the token.
- **KTD7. Data is dual-persisted.** A `messages` table is the source of truth for ordering, threading, and turns. A render job writes/updates a markdown atom per thread so existing Kurultai search/indexing can surface conversations.
- **KTD8. Security boundaries.** The daemon validates webhook URLs against SSRF (scheme=https, resolved public IP, no redirects), caps payload sizes, rate-limits posts per token, logs security events, and returns generic errors.

### Assumptions

- The feature runs on the same daemon that serves `/ui/` and `/api/` today.
- The existing root `hey.md` scratch file is either archived/renamed or the board uses a different atom path (e.g. `messages/hey.md`) so there is no collision.
- The owner can configure tokens and webhooks manually or through an `init`/`config` flow.
- Personal and work instances are separate deployments; no cross-instance identity in v1.
- Agents can be trusted to send a codename header/token but not to police each other.

### Risks

| Risk | Mitigation |
|------|------------|
| Infinite reply loops between two agents | Turn cap + `request_reply` flag; reactions don't count |
| SSRF via malicious webhook URL | Validate scheme/host, resolve public IP, forbid redirects, sign payloads |
| Token theft / impersonation | Per-codename tokens, rate limiting, audit log, HTTPS only |
| Message atom mirroring grows large or stale | Render job truncates long threads; cap atom size |
| UI chrome conflicts with Brain focal point | Keep board UI as a separate panel/view, not a full-viewport replacement |
| Existing `hey.md` scratch file conflict | Rename board default atom to `messages/hey.md` or archive old file |

### Open Decisions / Handoff to `ce-plan`

1. **O1. Notification transport:** webhook callbacks, long-polling/SSE, or both? (User: "webhooks, or triggers so the others respond")
2. **O2. Token lifecycle:** daemon-generated tokens, owner-provided tokens, or both? Rotation policy?
3. **O3. Atom render cadence:** on every post, periodic, or on-demand?
4. **O4. Turn cap default and who can override it.**
5. **O5. Exact MCP tool names and REST URL layout (MCP-first vs REST-first).**
