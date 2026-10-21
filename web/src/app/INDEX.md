---
index: kurultai/v1
folder: web/src/app
parent: web/src/INDEX.md
updated: 2026-10-20
version: 2
---

# `web/src/app`

**Does:** App router pages
**Up:** [`web/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

- [`dashboard/`](dashboard/INDEX.md) — Hub admin console (status/approvals/keys/scopes/account)
- [`sign-in/`](sign-in/INDEX.md) — Clerk sign-in
- [`sign-up/`](sign-up/INDEX.md) — Clerk sign-up

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`favicon.ico`](favicon.ico) | favicon.ico | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`icon.svg`](icon.svg) | App icon (Council Ring token pass) | — | — | 2026-10-13 | 1 | 2026-10-13 indexed (PR #441) |
| [`globals.css`](globals.css) | Global CSS + admin console styles (cards/tables/badges/forms) | — | — | 2026-10-20 | 3 | 2026-10-20 U8 console styles · 2026-09-04 human vs agent auth layout · 2026-08-16 indexed (v1 seed) |
| [`layout.tsx`](layout.tsx) | Root layout + Clerk provider (sign-up button removed — invite-only) | — | — | 2026-10-20 | 2 | 2026-10-20 U8 invite-only header · 2026-08-16 indexed (v1 seed) |
| [`page.tsx`](page.tsx) | Marketing/home — human GitHub vs agent keys | — | — | 2026-10-20 | 3 | 2026-10-20 console CTA label · 2026-09-04 clarify human vs agent access · 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-20 — U8 hub admin console: dashboard subtree, console CSS, invite-only header
- 2026-09-04 — human login copy + account settings clarity
- 2026-08-16 — indexed this folder (v1 seed)
