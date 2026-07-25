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
3. Optional: **Organizations** on — one org ≈ one shared Kurultai instance
4. Copy publishable + secret keys into `.env.local`

## Multi-user / devices

See [docs/multi-user-kurultai.md](../docs/multi-user-kurultai.md).

## Scripts

| Command | Purpose |
|---------|---------|
| `npm run dev` | Local app |
| `npm run build` | Production build |
| `npm run start` | Serve build |
