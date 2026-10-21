# Kurultai web app (Clerk + GitHub)

Authenticated app for **team / company** tiers. Marketing site stays in `../website/` (Vite). Personal CLI stays `cargo install` on your Mac.

Pattern inspired by [Kiranism/next-shadcn-dashboard-starter](https://github.com/Kiranism/next-shadcn-dashboard-starter) (Clerk + orgs) — **smart-add**, not a full fork.

## Setup

```bash
cd web
cp .env.example .env.local
# paste Clerk keys
npm install
npm run dev
```

Open http://localhost:3000 → **Continue with GitHub**.

### Clerk dashboard

1. [Create application](https://dashboard.clerk.com)
2. **User & Authentication → Social connections → GitHub** (enable)
3. **Disable public sign-up** — the admin console is invite-only
4. Optional: **Organizations** on — one org ≈ one shared Kurultai instance
5. Copy publishable + secret keys into `.env.local`

## Hub admin console

`/dashboard/*` is the admin console for a hub daemon (hub mode + Postgres).
It calls the daemon's `/api/hub/*` routes **server-side only** — the admin
key never reaches the browser.

| Env | Purpose |
|-----|---------|
| `KURULTAI_HUB_API_URL` | Hub daemon base URL (`https://api-hub.shippedit.dev`, `http://127.0.0.1:8423` in dev) |
| `KURULTAI_ADMIN_KEY` | `is_admin` hub key — mint via `kurultai hub key issue --agent web-admin --team ops --admin` (`HUB_ADMIN_KEY` is a fallback name) |
| `WEB_ADMIN_EMAILS` | Comma-separated Clerk emails allowed in (fail closed when unset; `publicMetadata.role="admin"` also passes) |

Surfaces:

- `/dashboard` — status cards (active keys, pending approvals, teams, atoms) + recent activity
- `/dashboard/approvals` — pending `kurultai connect` device flows → approve with a team pick
- `/dashboard/keys` — issue (plaintext shown once), list, revoke; members grouped by team
- `/dashboard/scopes` — visibility × team census + hub activity feed
- `/dashboard/account` — your Clerk identity

Signed-in non-admins get a 403 page; signed-out users are redirected to
Clerk sign-in by `src/proxy.ts`.

## Multi-user / devices

See [docs/multi-user-kurultai.md](../docs/multi-user-kurultai.md).

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run dev` | Local app |
| `npm run build` | Production build |
| `npm run start` | Serve build |
