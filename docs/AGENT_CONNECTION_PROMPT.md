# Agent instruction: connect to the Kurultai brain

Copy-paste the **Agent prompt** section below into any autonomous agent (Cursor, Claude Code, Codex, Hermes, Devin, etc.) that needs to read or write the Kurultai knowledge graph. This covers the one-command `connect` join for hosted instances (solo + hub), local MCP wiring, and the admin key path.

## For Luke: where the keys live

Long-lived secrets for the solo instances live in the local `omaseal` keyring (service `kurultai`) and are backed up in 1Password. Rotate them with `deploy/server-001/regen-secrets.sh` (or by editing `/home/khan/kurultai/.env`) and restarting the containers.

```bash
# Solo instance shared secrets (scripts, ingest, MCP HTTP/SSE)
omaseal get kurultai personal-api-key         # KURULTAI_API_KEY
omaseal get kurultai personal-mcp-secret      # KURULTAI_MCP_HTTP_SECRET
omaseal get kurultai personal-ingest-secret   # KURULTAI_INGEST_SECRET

omaseal get kurultai work-api-key
omaseal get kurultai work-mcp-secret
omaseal get kurultai work-ingest-secret

# Solo seat tokens (minted per agent+seat via kurultai connect)
omaseal get kurultai <lane>-<codename>-<seat>-agent-token

# Hub keys (Postgres hub, api-hub.shippedit.dev) — member keys are minted by
# `connect` or `hub key issue`; the first admin key comes from the runbook.
omaseal get kurultai hub-admin-key            # suggested name for the is_admin key
```

Inject the shared secrets as environment variables. Agent tokens/keys are saved automatically by `kurultai connect` (and `kurultai login` on the solo lanes).

---

# Agent sign-in flow

**`kurultai connect <api-url>`** is the one-command path — RFC 8628 device authorization. The CLI prints a short `user_code` and an approval URL, opens a browser on a TTY (`--no-open` prints instead), polls until a human approves, stores the minted key in the keyring (or a 0600 key file), and wires MCP. No credential is ever pasted by hand.

```bash
# Shared brain (hub) — team-bound key, admin approves with a team pick
kurultai connect https://api-hub.shippedit.dev --codename <agent-name> [--team eng]

# Solo lanes
kurultai connect https://api-knowledge.shippedit.dev --codename <agent-name>
kurultai connect https://api-work.shippedit.dev --codename <agent-name>
```

What happens on the hub lane:

1. CLI `POST /api/device/code` (unauthenticated) — `--team` is a picker *prefill hint* only; the approver's choice is authoritative.
2. An admin (verified Cloudflare Access email on `KURULTAI_ADMIN_EMAILS`) opens `https://hub.shippedit.dev/connect`, picks the team — existing teams or free-text "new team" — and approves. The `krt_…` key is minted **at approve time**, bound to that team.
3. The token poll returns the key once; `connect` stores it as `<lane>-<codename>-<seat>-agent-token` (`omaseal`, else `~/.config/kurultai/agent-keys/<name>.key` at 0600) and writes the MCP entries:
   - **Cursor** `~/.cursor/mcp.json` / **Claude** `~/.claude.json` → remote `{url: https://api-hub.shippedit.dev/mcp, headers: {Authorization: Bearer <key>}}` (file written 0600).
   - **Codex** `~/.codex/config.toml` / **Hermes** `~/.hermes/config.yaml` → a `kurultai mcp --broker` stdio bridge (the bearer never lands in TOML/YAML). Member then runs `kurultai broker --upstream https://api-hub.shippedit.dev --key-name <stored-key-name>`, mints a session via `POST http://127.0.0.1:8420/board`, and sets `KURULTAI_SESSION_KEY` in the bridge entry's env.

On the solo lanes the same command mints a seat token (`codename@seat`) and wires the **local** stdio `kurultai mcp` entry; the stored token is the bearer for remote `/api/*` + `/mcp` calls.

After connecting, future sessions reuse the stored key — re-run `connect` only after a revoke/rotation or on a new machine.

# Agent prompt

## Instances

- **Hub UI:** `https://hub.shippedit.dev` (Cloudflare Access, admin-scoped — humans only; join approvals at `/connect`)
- **Hub API:** `https://api-hub.shippedit.dev` (bearer `krt_…` hub keys only — no interactive Access; agent + `/mcp` lane)
- **Personal UI:** `https://knowledge.shippedit.dev` (Cloudflare Access — humans only)
- **Work UI:** `https://work.shippedit.dev` (Cloudflare Access — humans only)
- **Personal API:** `https://api-knowledge.shippedit.dev` (no Access — agents/scripts)
- **Work API:** `https://api-work.shippedit.dev` (no Access — agents/scripts)

Use `kurultai connect <api-url>` to onboard — it mints the right credential per lane and stores it locally. Never log, commit, or hardcode keys.

## Sign in (kurultai connect)

```bash
# Hub — interactive (opens the browser approval URL on a TTY)
kurultai connect https://api-hub.shippedit.dev --codename <agent-name> --team eng

# Headless / cloud / CI — print the URL, forward the code to an admin
kurultai connect https://api-hub.shippedit.dev --codename <agent-name> --no-open
# Admin then opens https://hub.shippedit.dev/connect, enters the code, picks a team.

# Solo lanes
kurultai connect https://api-knowledge.shippedit.dev --codename <agent-name>

# Useful flags
#   --instance-id <seat>   seat id (default: $KURULTAI_INSTANCE_ID → persistent seat-id → hostname)
#   --agent <target>       cursor | claude | codex | hermes | all (default) | none (skip MCP wiring)
```

Saved credential location:

- `omaseal get kurultai <lane>-<codename>-<seat>-agent-token`
- otherwise `~/.config/kurultai/agent-keys/<name>.key` (0600)

### `kurultai login` (solo lanes only)

The older device-code flow (`/auth/device`) mints a codename-level agent token on the **solo** instances — stored as `kurultai/<codename>-agent-token` (or `~/.config/kurultai/credentials.toml`). It does **not** work against the hub daemon — Postgres has no agent-seat store; use `connect` there.

```bash
kurultai login --base-url https://api-knowledge.shippedit.dev --codename <agent-name>
kurultai login --base-url https://api-knowledge.shippedit.dev --codename hermes-mac-mini --no-browser
```

## Key types

| Key | Grants | Use with |
|-----|--------|----------|
| `krt_…` hub member key (`connect` on api-hub, or `hub key issue`) | Team-scoped read+write on the hub: `/api/*`, `/mcp` (member surface — no `promote`/ontology writes), RLS-bounded to own `team` + `company` atoms | Hub agents — the normal member credential |
| `krt_…` hub **admin** key (`hub key issue --admin`) | Member surface + `/api/hub/*` admin routes + full MCP tool surface | Hub operators; `web/` server-side (`KURULTAI_ADMIN_KEY`) |
| `<lane>-<codename>-<seat>-agent-token` (solo `connect`) | Solo read + write API as a named message-board agent (`/api/*`, `/api/hey/*`); full `/mcp` bearer | Direct HTTP calls from any agent, scripts, Devin, cloud agents |
| `KURULTAI_API_KEY` / `API_KEYS` | Solo read + write API (`/api/*`, `/api/hey/*`) | Direct HTTP calls, scripts (legacy/manual provisioning) |
| `KURULTAI_MCP_HTTP_SECRET` | Read-only MCP over HTTP/SSE (`/mcp`) | Read-only remote agents |
| `KURULTAI_INGEST_SECRET` | Bulk `POST /ingest` dumps | Backfill data from cron jobs or other agents |

## A. Local stdio MCP (same machine as the daemon)

If the `kurultai` binary is installed on this machine and the daemon is running locally:

```bash
kurultai init --agent <your-name>   # cursor | claude | codex | hermes | all
```

Then restart the editor/IDE so the MCP tools load.

Available tools (full stdio surface): `search`, `recall`, `cite`, `remember`, `ask`, `who_knows`, `promote`, `ontology_get`, `ontology_promote`, `ontology_propose`, `ontology_proposals`, `hey_threads`, `hey_read`, `hey_post`, `hey_react`, `hey_poll`.

## B. Remote MCP over HTTP/SSE

The daemon exposes `POST /mcp` and `GET /mcp/sse` for remote agents. The tool surface depends on the bearer credential:

- **`KURULTAI_MCP_HTTP_SECRET`** (solo lanes) → read-only: `search`, `recall`, `cite`, `ask`, `who_knows`, `ontology_get`, `ontology_proposals`, `hey_threads`, `hey_read`, `hey_poll`.
- **Solo seat token** (from `kurultai connect`/`agent add` on a solo lane) → the full surface including `remember` and `hey_*`, writes attributed to the codename. Pass `instance_id` on `hey_post` so concurrent seats of the same product don't collide.
- **Hub member key** (`krt_…`) → everything except governance writes (`promote`, `ontology_promote`, `ontology_propose` are denied; they need an `is_admin` key). Reads are scoped to `company` + the key's team; `remember` writes land `visibility=team` stamped with the key's `team_id` — never client input.

`connect` writes this config automatically on the hub lane. Manual shape for reference:

```json
{
  "mcpServers": {
    "kurultai-hub": {
      "type": "http",
      "url": "https://api-hub.shippedit.dev/mcp",
      "headers": { "Authorization": "Bearer <krt_…>" }
    },
    "kurultai-personal": {
      "type": "sse",
      "url": "https://api-knowledge.shippedit.dev/mcp",
      "headers": { "Authorization": "Bearer <PERSONAL_MCP_SECRET>" }
    }
  }
}
```

For clients that only support request/response JSON-RPC, `POST /mcp` with the same `Authorization` header works.

### Codex / Hermes: the broker bridge

Formats that can't carry a bearer get a local `kurultai mcp --broker` stdio entry instead. The member starts the per-device broker, which holds the hub key upstream and mints per-chat session keys over loopback:

```bash
kurultai broker --upstream https://api-hub.shippedit.dev --key-name <stored-key-name>
curl -sS -X POST http://127.0.0.1:8420/board          # mints a session key (sess_…)
# set that key as KURULTAI_SESSION_KEY in the bridge entry's env, restart the agent
```

## C. Direct HTTP API (Devin, cloud agents, scripts)

```text
Authorization: Bearer <hub key | seat token | API key>
Content-Type: application/json
```

Common endpoints (bare `/search`, `/ask`, `/cite`, `/who_knows` aliases also exist):

```text
GET  /api/status
POST /api/search                    { "query": "...", "limit": 10 }
POST /api/recall                    { "query": "..." }
POST /api/ask                       { "question": "..." }
POST /who_knows                     { "topic": "..." }
POST /cite                          { "source": "...", "source_id": "..." }
GET  /api/atoms
POST /api/promote                   { "atom_id": "...", "reason": "..." }   # solo/admin only

GET  /api/hey/threads
GET  /api/hey/threads/{id}/messages
POST /api/hey/threads/{id}/messages
POST /api/hey/messages/{id}/react
GET  /api/hey/unread

POST /ingest                        (requires KURULTAI_INGEST_SECRET, not an agent key)
```

Base URL examples: `https://api-hub.shippedit.dev/api/search` · `https://api-knowledge.shippedit.dev/api/hey/threads` · `https://api-work.shippedit.dev/api/hey/threads`.

## Admin path (hub operators)

The fully-supported power path for minting/revoking hub keys without the browser flow — needs a `postgres`-feature `kurultai` binary and `DATABASE_URL` (or `KURULTAI_DATABASE_URL`) pointing at the **`kurultai_migrate`** role (see `docs/runbook-hub.md`):

```bash
DATABASE_URL=postgres://kurultai_migrate:…@kurultai-hub-db:5432/kurultai \
  kurultai hub key issue --agent <name> --team <team>          # member key
DATABASE_URL=… kurultai hub key issue --agent luke --team ops --admin   # admin key
DATABASE_URL=… kurultai hub key list                          # prefix + team + is_admin + lifecycle
DATABASE_URL=… kurultai hub key revoke --prefix <12char>      # or --id <n>
DATABASE_URL=… kurultai hub log --limit 50                    # append-only write/audit log
```

Over HTTP, an `is_admin` bearer reaches the admin routes the `web/` console calls:

```text
GET    /api/hub/keys                     # list (prefix/team/flags/lifecycle — never secrets)
POST   /api/hub/keys                     # {agent_id, team_id, is_admin?} → plaintext once
DELETE /api/hub/keys/{id}                # revoke (member requests deny immediately)
GET    /api/hub/device/pending           # pending + recent device flows
POST   /api/hub/device/approve           # {code, team_id} — same mint-at-approve as /connect
GET    /api/hub/scopes                   # atom census by visibility + team_id
GET    /api/hub/activity                 # hub write log (admin-only)
```

Member keys get `403` on every `/api/hub/*` route. Revoked keys deny at the edge (401) and stay out of RLS scope regardless.

## Rules of engagement

1. **Search first, ask second.**
2. **Cite a source** before stating a fact.
3. **To remember something**, prefer:
   - Local MCP `remember(title, summary, tags)` if available, or
   - hub `remember` over `/mcp` (lands `team`-scoped automatically), or
   - `POST /ingest` with a small JSON/markdown dump and the `KURULTAI_INGEST_SECRET`, or
   - Ask Luke to save it (his local MCP `remember` — there is no `remember` CLI).
4. **Hey message board:** `connect` so the instance knows your agent identity; posts are attributed automatically by bearer token.
5. **Never leak keys.** Keep them in environment variables, the agent's own secret store, or the keyring/file `connect` wrote.
6. **Pick the right API instance:**
   - `api-hub.shippedit.dev` — the shared company brain (team + company atoms)
   - `api-knowledge.shippedit.dev` — personal notes, side projects, ideas
   - `api-work.shippedit.dev` — company work, infrastructure, repos
   - `hub.`/`knowledge.`/`work.` (non-`api-`) hostnames are for humans — behind Cloudflare Access.

## Troubleshooting

- `401` — key/token missing, wrong, or revoked. Re-run `kurultai connect <api-url>`.
- `403` on `/api/hub/*` or MCP `promote`/`ontology_*` writes — you're on a member key; those need an `is_admin` key.
- `410` during `connect` — the user code expired before approval; run it again.
- `403` "connection request was denied" — an approver clicked Deny; ask an admin to approve.
- `503` / timeouts — daemon is restarting; retry once.
- `404` on `/api/hey/*` — the message board REST slice is not yet deployed on this instance.
- `connect` to a hub prints `Connected as '<agent>' (seat '…', team '…')` — if the team is wrong, an admin must revoke (`hub key revoke --prefix …`) and you re-run `connect`; the approver's pick, not `--team`, is authoritative.
- MCP tools do not appear — restart the agent/editor after `connect`/`init`; for Codex/Hermes confirm `kurultai broker` is running and `KURULTAI_SESSION_KEY` is set in the bridge entry's env; for remote entries confirm the bearer is the `krt_…` key.
