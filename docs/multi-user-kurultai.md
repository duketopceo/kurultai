# Multi-user & multi-device Kurultai

How people and machines relate to **one** Kurultai deployment — without making cloud the default for solo use.

Related: GitHub login [#81](https://github.com/duketopceo/kurultai/issues/81) · Anthropic login [#79](https://github.com/duketopceo/kurultai/issues/79) · Cloud sync [#80](https://github.com/duketopceo/kurultai/issues/80) · Audience [#25](https://github.com/duketopceo/kurultai/issues/25)

## What ships today vs roadmap

| Capability | Status |
|------------|--------|
| Personal kernel (`cargo install`, local `store.db`) | **Shipped** |
| `web/` Clerk + GitHub sign-in shell | **Shipped** (this PR) |
| One team deploy + one Clerk Organization | **Design + partial UI** — auth only; shared store/API enforcement not in this PR |
| Promote-to-shared-index, device sync (#80) | **Roadmap** |
| Multi-tenant / VPC, many Orgs / RBAC, audit, retention | **Roadmap / design only** — not shipped |

## Three layers

```text
Personal kernel              Team instance (shipped auth)   Company (roadmap — not shipped)
(local Mac / store.db)       (one shared deploy)            (multi-tenant / VPC)
─────────────────────        ───────────────────            ────────────────────
cargo install                web/ + Clerk GitHub login      same + compliance (future)
no website login required    one Clerk Org (design)         many Orgs / RBAC (future)
full personal fidelity       promote atoms (future)         policy + audit (future)
```

## Users in a single Kurultai

**One team Kurultai** (target model) = one deployment (web app + API/daemon + one shared `store.db` or managed DB) + **one Clerk Organization**.

**Today:** `web/` authenticates users via Clerk and can display organization context. It does **not** yet enforce org-scoped access to a shared store or API.

| Concept | Meaning |
|---------|---------|
| **User** | Clerk identity (Sign in with GitHub) |
| **Member** | User invited into the Org |
| **Shared index** | Atoms everyone in the Org can search (after promote / shared connectors) — **future** |
| **Personal kernel** | Still local on each laptop until they promote |

Do **not** share one SQLite file via Dropbox/iCloud. That corrupts WAL and races writers.

## Multiple devices (same person)

Until [#80](https://github.com/duketopceo/kurultai/issues/80) ships:

1. **Same Clerk user** on the website from any browser/device  
2. **Local kernels** per machine stay separate — export/backup `store.db` or re-index sources  
3. **Promote** important atoms to the team instance (future: distillation gate + API)

Later (#80): opt-in encrypted sync personal ↔ team / device ↔ device.

## Sync options (roadmap)

| Option | When | Notes |
|--------|------|-------|
| Promote-only | Team (future) | Push selected atoms to shared index |
| Backup blob | #80 thin slice | Encrypted `store.db` / pack upload |
| Incremental sync | #80 later | Atom-level push/pull + conflict policy |
| Managed bind | Company (future) | Local daemon mirrors hosted Kurultai |

## Auth

- Website / team: **Clerk + GitHub** (`web/`) — fast path for all tiers (user → Org → enterprise SSO later)
- Optional later: Anthropic ([#79](https://github.com/duketopceo/kurultai/issues/79))
- CLI local: env keys only; no Clerk required

Authentication (signed-in user) is **not** the same as tenant authorization (org-scoped store/API access). The latter is future work once shared storage lands.

## Operator checklist (IT team on cloud)

Design reference for a future team deployment — not a fully executable workflow until shared store/API enforcement ships.

1. Stand up one Kurultai host (daemon + store)  
2. Deploy `web/` with Clerk; enable GitHub; create Org “your-team”  
3. Invite engineers as Org members  
4. Each person keeps a personal kernel; promote into the shared index under policy (future)  
5. **Future:** company tier = more Orgs + audit/retention when you need them  
