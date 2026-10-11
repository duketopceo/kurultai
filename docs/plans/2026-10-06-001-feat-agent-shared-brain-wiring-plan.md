# Plan: Remote agent wiring to the shared personal brain

## Problem

Every agent on Luke's machines (local omarchy-max: cursor, claude, codex, devin,
opencode, agy, hermes — plus agents on the ssh mini) should read and write the
**shared** Kurultai personal brain (`knowledge.shippedit.dev`) without any
per-session manual instruction. Today each agent would have to be told
individually, and there is no per-chat identity carried on writes.

## Settled decisions (KTDs)

- **KTD-1 — Remote MCP, not local stores.** Agents connect to the hosted
  personal lane over HTTP MCP; no per-machine SQLite brain.
  *Provenance: user-directed. Rejected alternative: local `kurultai mcp`
  instances — private brains defeat "shared memory for agent fleets."*
- **KTD-2 — Per-codename seat identity, shared transport secret.**
  The agent's `agent_key`/seat token (already minted per codename, stored in
  omaseal as `kurultai/personal-<codename>-agent-token`) is the bearer on
  `POST /mcp` → full read+write tool surface. The shared
  `personal-mcp-secret` remains the read-only lane, available as a fallback.
  *Provenance: user-directed. Rejected alternative: one shared write key —
  collapses attribution; rejected: minting a dozen new CF access tokens —
  unnecessary, bearer lane already exists.*
- **KTD-3 — Use the existing `api-*` hostnames.** `api-knowledge.shippedit.dev`
  already terminates bearer-auth'd traffic without Cloudflare Access
  (verified: `POST /mcp` returns 415, not a 302 Access redirect). No Access
  policy changes required.
  *Provenance: discovered fact, recorded during planning.*
- **KTD-4 — `instance_id` is agent-minted per chat.** Format
  `<codename>-<yyyymmdd>-<slug>` (≤64 chars). The rule tells agents to state
  it in their first status block so it survives compaction; a dropped
  instance_id is minted again — seats are cheap, never an error.
  *Provenance: user-directed. Rejected: deterministic re-derivation —
  impossible across compaction.*
- **KTD-5 — Secrets resolve through OmaSeal at runtime.** OmaSeal's new MCP
  server is already installed in all 7 agents on this machine, and the
  kurultai secrets are manifest-ALLOW. Agents resolve
  `omaseal://kurultai/personal-<codename>-agent-token` at session start;
  no secret is written into any dotfile or MCP config.
  *Provenance: user-directed ("pickup an access token"). Rejected: literal
  tokens in `~/.cursor/mcp.json` etc.*

## Scope

Primary repo: `duketopceo/luke-agents` (rules + scripts + docs).
Secondary: `duketopceo/kurultai` (doc updates only).
Runtime changes: CF/server config is already correct — nothing to redeploy
except optionally enabling `shared_write` (Unit 4).

## Implementation units

### U1 — `kurultai-mcp` stdio bridge script (luke-agents)

New `scripts/mcp/kurultai/` in luke-agents, matching the existing
`scripts/mcp/hetzner/` convention (wrapper scripts + README, symlinked into
`~/.cursor/bin/` by dotfiles `link-agent-skills.sh`; also install as
`~/bin/kurultai-mcp` for non-cursor harnesses):

- `kurultai-mcp.sh` — resolves `kurultai/personal-mcp-secret` (read lane) and
  the caller's `kurultai/personal-<codename>-agent-token` (write lane) from
  omaseal at spawn; bridges stdio → `https://api-knowledge.shippedit.dev/mcp`
  via `mcp-remote` (`npx -y mcp-remote`) with
  `Authorization: Bearer <seat-token>`, falling back to the shared secret
  (read-only surface) when no codename token exists. Codename comes from
  `--codename` arg or `KURULTAI_AGENT_CODENAME`.
- `README.md` — usage, codename list, instance_id convention.

Test scenarios: script exits non-zero with a clear message when omaseal
denies/misses a secret; resolves + connects for a known codename; read-only
fallback works without a seat token.

### U2 — Per-harness MCP config entries (host wiring, documented)

Each harness's MCP config gets a `kurultai` stdio server running the U1
script. Per omaseal `mcp status` guidance, wrap with `omaseal run` where a
harness's env block is literal-only (codex). Document the exact config block
per harness in the U1 README and in `docs/` of luke-agents; apply on
omarchy-max and document the mini's equivalent. Only `opencode`/`devin`/
`agy` codenames may need `kurultai agent add` on the server — mint only the
missing ones (one-time admin action, not per-chat).

### U3 — Canonical rule text (luke-agents AGENTS.md §0a)

Amend §0a (canonical identity doctrine) with the write contract:
*resolve `personal-<codename>-agent-token` via omaseal at session start;
mint `instance_id` once, restate it in the first status block so it survives
compaction; include it on every `hey_post`; `search`/`cite` the brain before
re-investigating; `remember` durable findings upstream.* Because every
per-harness file is already a thin pointer to AGENTS.md, this one edit
propagates to every harness without touching per-agent files.

### U4 — Enable `shared_write` on the personal instance

Set `KURULTAI_FEATURE_SHARED_WRITE=1` for `kurultai-personal` so all
agent-reachable writes land in quarantine (`agent_write_containment`) with
`metadata.agent_id`/`write_transport` stamps; promotion stays CLI-gated.
Requires `KURULTAI_ADMIN_TOKEN` set (write_route_guard fails closed without
it). Apply via `~/kurultai/.env` + `deploy/server-001/redeploy.sh` env-only
recreate. Verify a `remember` over seat-token `/mcp` lands quarantined.

### U5 — Kurultai repo doc updates

Update `docs/AGENT_CONNECTION_PROMPT.md` section B — it currently documents
remote MCP as read-only; seat-token bearer on `api-*` hostnames grants the
full surface. Add the U1 bridge + omaseal resolve recipe so strangers get the
same wiring. Note the known gap: `kurultai agent list` inside the container
fails with `embed_dim mismatch` on a `dimension = 1024` config — the command
appears to bypass the file `[embed]` block; investigate during
implementation, fix if trivial, else record as a follow-up.

## Risks / notes

- A leaked seat token grants write (quarantined) + read on the personal lane —
  revocable via `kurultai agent revoke <codename> [--instance-id]`.
- `/connect` and `/api/device/*` are open on `api-*` hostnames by design
  (self-gating device flow); confirm acceptable, else add hub-auth coverage —
  out of scope unless implementation reveals it blocks nothing.
- ssh mini uses the identical recipe (same URL, same omaseal flow, its own
  codenames or `instance_id` like `mac-mini`).

## Verification

1. `kurultai-mcp` bridge up under one harness config; `tools/list` over `/mcp`
   returns the full 8+hey tool surface on a seat token, read-only surface on
   the shared secret.
2. `remember` from agent A (seat token, `X-Kurultai-Agent` stamp) → lands
   quarantined with `agent_write_containment` under shared_write.
3. `search`/`cite` from agent B sees prior trusted atoms; quarantined write
   invisible to default search.
4. `hey_post` with `instance_id` shows correct `codename@instance_id`.
5. `python3 scripts/audit-agent-index.py` green in kurultai; luke-agents
   skills-index/link checks green.
