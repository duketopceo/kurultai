---
index: kurultai/v1
folder: web/src/lib
parent: web/src/INDEX.md
updated: 2026-10-20
version: 1
---

# `web/src/lib`

**Does:** Server-side hub admin client + Clerk admin gate + display helpers
**Up:** [`web/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`admin-gate.ts`](admin-gate.ts) | Clerk allowlist/role → AdminIdentity (fail closed) | `WEB_ADMIN_EMAILS` env, Clerk `publicMetadata.role` | every dashboard page/action | 2026-10-20 | 1 | 2026-10-20 U8 admin gate |
| [`format.ts`](format.ts) | Timestamp/age/expiry display helpers (neutral) | — | dashboard pages | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`hub-admin.ts`](hub-admin.ts) | `server-only` fetch wrapper for daemon `/api/hub/*` | `KURULTAI_HUB_API_URL` + `KURULTAI_ADMIN_KEY` env | dashboard pages + actions | 2026-10-20 | 1 | 2026-10-20 U8 admin console |

## Recent

- 2026-10-20 — U8 hub admin console: server-only `/api/hub/*` wrapper + allowlist gate
