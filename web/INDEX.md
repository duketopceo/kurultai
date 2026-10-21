---
index: kurultai/v1
folder: web
parent: INDEX.md
updated: 2026-10-20
version: 2
---

# `web`

**Does:** Next.js + Clerk hub admin console (status, approvals, keys, scopes)
**Up:** [`INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../docs/agent-index.md)

## Children

- [`public/`](public/INDEX.md) — Static SVGs
- [`src/`](src/INDEX.md) — Next app source (app/, components/, lib/)

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`.env.example`](.env.example) | Clerk + hub console env placeholders | — | — | 2026-10-20 | 2 | 2026-10-20 U8 console envs (`KURULTAI_HUB_API_URL`, `KURULTAI_ADMIN_KEY`, `WEB_ADMIN_EMAILS`) · 2026-08-16 indexed (v1 seed) |
| [`.gitignore`](.gitignore) | Next ignore rules | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`README.md`](README.md) | Clerk-gated hub admin console notes | — | — | 2026-10-20 | 2 | 2026-10-20 U8 console env/surface docs · 2026-08-16 indexed (v1 seed) |
| [`eslint.config.mjs`](eslint.config.mjs) | ESLint | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |
| [`next.config.ts`](next.config.ts) | Next config (turbopack.root pinned to web/) | — | — | 2026-10-20 | 2 | 2026-10-20 silence multi-lockfile root inference · 2026-08-16 indexed (v1 seed) |
| [`package-lock.json`](package-lock.json) | npm lockfile | — | — | 2026-10-20 | 2 | 2026-10-20 `server-only` dep · 2026-08-16 indexed (v1 seed) |
| [`package.json`](package.json) | Next.js hub admin console | — | — | 2026-10-20 | 2 | 2026-10-20 `server-only` dep · 2026-08-16 indexed (v1 seed) |
| [`tsconfig.json`](tsconfig.json) | TS config | — | — | 2026-07-25 | 1 | 2026-08-16 indexed (v1 seed) |

## Recent

- 2026-10-20 — U8 hub admin console: 4 surfaces under /dashboard, server-only `/api/hub/*` client, WEB_ADMIN_EMAILS gate
- 2026-08-16 — indexed this folder (v1 seed)
