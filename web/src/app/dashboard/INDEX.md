---
index: kurultai/v1
folder: web/src/app/dashboard
parent: web/src/app/INDEX.md
updated: 2026-10-20
version: 2
---

# `web/src/app/dashboard`

**Does:** Hub admin console (Clerk + allowlist gated) — status, approvals, keys, scopes
**Up:** [`web/src/app/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../docs/agent-index.md)

## Children

- [`account/`](account/INDEX.md) — Clerk human-account page (old dashboard content)
- [`approvals/`](approvals/INDEX.md) — Pending device-flow approvals
- [`keys/`](keys/INDEX.md) — Hub key issue/list/revoke + members by team
- [`scopes/`](scopes/INDEX.md) — Visibility × team census + activity feed

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`actions.ts`](actions.ts) | `"use server"` issue/revoke/approve actions (re-check admin gate) | `lib/admin-gate`, `lib/hub-admin` | daemon `/api/hub/*` | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`error.tsx`](error.tsx) | Client error boundary (hub calls fail soft before this) | — | — | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`forbidden.tsx`](forbidden.tsx) | HTTP 403 boundary for `forbidden()` (non-admin sign-ins) | `components/forbidden` | — | 2026-10-20 | 1 | 2026-10-20 U8 admin gate |
| [`layout.tsx`](layout.tsx) | Admin gate + console sub-nav for all dashboard routes | `lib/admin-gate`, `components/dash-nav` | — | 2026-10-20 | 1 | 2026-10-20 U8 admin gate |
| [`loading.tsx`](loading.tsx) | Route loading panel | — | — | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`page.tsx`](page.tsx) | Hub status cards (keys/pending/teams/atoms) + recent activity | `lib/hub-admin` read endpoints | — | 2026-10-20 | 2 | 2026-10-20 rewritten as hub status surface · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-20 — U8: /dashboard is now the hub admin console (4 surfaces + account)
- 2026-08-16 — indexed this folder (v1 seed)
