# Multi-user & multi-device Kurultai

How people and machines relate to **one** Kurultai deployment — without making cloud the default for solo use.

Related: GitHub login [#81](https://github.com/duketopceo/kurultai/issues/81) · Anthropic login [#79](https://github.com/duketopceo/kurultai/issues/79) · Cloud sync [#80](https://github.com/duketopceo/kurultai/issues/80) · Audience [#25](https://github.com/duketopceo/kurultai/issues/25)

## Three layers

```
Personal kernel              Team instance                 Company
(local Mac / store.db)       (one shared deploy)           (multi-tenant / VPC)
─────────────────────        ───────────────────           ────────────────────
cargo install                web/ + daemon + store         same + compliance
no website login required    Clerk users in one Org        many Orgs / RBAC
full personal fidelity       scrubbed / promoted atoms     policy + audit
```

## Users in a single Kurultai

**One team Kurultai** = one deployment (web app + API/daemon + one shared `store.db` or managed DB) + **one Clerk Organization**.

| Concept | Meaning |
|---------|---------|
| **User** | Clerk identity (Sign in with GitHub) |
| **Member** | User invited into the Org |
| **Shared index** | Atoms everyone in the Org can search (after promote / shared connectors) |
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
| Promote-only | Team now | Push selected atoms to shared index |
| Backup blob | #80 thin slice | Encrypted `store.db` / pack upload |
| Incremental sync | #80 later | Atom-level push/pull + conflict policy |
| Managed bind | Company | Local daemon mirrors hosted Kurultai |

## Auth

- Website / team: **Clerk + GitHub** (`web/`) — fast path for all tiers (user → Org → enterprise SSO later)
- Optional later: Anthropic ([#79](https://github.com/duketopceo/kurultai/issues/79))
- CLI local: env keys only; no Clerk required

## Operator checklist (IT team on cloud)

1. Stand up one Kurultai host (daemon + store)  
2. Deploy `web/` with Clerk; enable GitHub; create Org “your-team”  
3. Invite engineers as Org members  
4. Each person keeps a personal kernel; promote into the shared index under policy  
5. Company tier = more Orgs + audit/retention when you need them  
