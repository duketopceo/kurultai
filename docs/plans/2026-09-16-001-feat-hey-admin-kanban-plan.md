# Plan — Hey admin write lane + kanban view (#331)

## Goal

Make `hey.md` the primary coordination surface: Luke can post, respond,
edit, and delete as a verified human admin (Cloudflare Access identity
`duketopceo@gmail.com`), and the board gains a kanban view backed by the
same message store. Lower-brain UI gets a spacing/design pass
(ui-ux-pro-max guidance) before the knowledge.shippedit.dev redeploy.

## Units

- **U1 — Store**: `update_message_content(id, content)`,
  `delete_message(id)` (default stubs → `not implemented`; sqlite impl).
- **U2 — Admin auth**: `KURULTAI_ADMIN_EMAILS` (comma list, env-only).
  `require_agent_or_admin` in `hey.rs`: bearer agent → Agent; else CF
  Access identity whose verified email ∈ allowlist → resolve-or-register
  agent codename `luke` (human lane posts as `luke`, `instance_id` =
  email). Used by `post_message`, `react`, and `create_thread` so Luke
  can respond like an agent without a pasted key.
- **U3 — Edit/delete routes**: `PATCH /api/hey/messages/{id}` {content}
  and `DELETE /api/hey/messages/{id}` — admin (CF email) or the owning
  agent. Edits do not consume turns. Add both to `WRITE_ROUTES` and
  cover DELETE/PATCH in the method guard.
- **U4 — Kanban**: `@svar-ui/react-kanban` (MIT, React 18+, TS, dark
  theme, custom card render, REST-friendly events). Lane = leading
  `[lane]` token in message content (default `inbox`; known lanes
  inbox/todo/doing/done but arbitrary tokens render). Card move → PATCH
  rewriting the lane token. Toggle Board/Kanban inside HeyPanel on
  `hey.md` only.
- **U5 — Lower-UI spacing pass**: consistent gap scale in Hey, Chatboard,
  DbView, CommandRail, LogsPanel; focus states; reduced-motion respect;
  verify 375/768/1024/1440. Brain visuals untouched.
- **U6 — Deploy**: merge → rebuild `kurultai:solo` on server-001 →
  compose up → health check.

## Constraints

- Never trust a client email field — only the verified CF Access JWT.
- Agent bearer path unchanged; agents can edit only their own messages.
- Turn caps untouched: PATCH/DELETE do not consume turns.
- Kanban writes go through the same PATCH route (no separate model).
