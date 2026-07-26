---
title: "feat: web/ GitHub login + multi-user single Kurultai model"
date: 2026-07-25
type: feat
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-plan-bootstrap
execution: code
authority: "User /lfg — fork vs smart add + GitHub login + users in one kurultai"
depth: standard
origin: "Fast path Clerk+Kiranism pattern; #81 GitHub login; #80 sync deferred design"
---

# feat: web GitHub login + multi-user single Kurultai model

**Target repo:** `duketopceo/kurultai`  
**Audience:** team (website auth) · personal kernel unchanged  
**Base:** `main`  
**Process:** PR-only

## Goal Capsule

**Smart-add** a lean `web/` Next.js app with **Clerk + Sign in with GitHub** (Kiranism/Clerk pattern). Keep existing Vite marketing site in `website/` untouched. Document how **multiple users and devices** share one Kurultai instance without full cloud sync (#80) yet.

**Stop when:** `web/` runs locally with Clerk GitHub env stubs; docs explain multi-user model; Rust CI unaffected; #81 advanced; no Clerk secrets committed.

**Do not:** vendor entire Kiranism starter; replace `website/`; cut `v*` release; implement atom sync; Anthropic login (#79); rewrite Rust daemon auth this PR.

### Settled decisions (session-settled)

| KTD | Decision | Provenance | Rejected | Reason |
|-----|----------|------------|----------|--------|
| KTD1 | **Smart-add `web/`** monorepo app, not fork-as-root / not full starter vendoring | user-directed (“fork or add smart”) | Dump Kiranism into repo root | Keeps Rust crate clean; CI stays green |
| KTD2 | Auth = **Clerk + GitHub** (fast path that scales personal→team→company orgs) | user-approved | Better Auth only / custom OAuth | Fast; orgs map to team/company |
| KTD3 | Multi-user “single Kurultai” = **one deployment + org membership + promote-to-shared-index**; device sync design-doc only (#80) | user-directed | Build full sync now | Phase 6; need auth identity first |
| KTD4 | Personal kernel stays `cargo install` / local `store.db` | user-approved | Force website login for CLI | Local-first doctrine |

---

## Product Contract

### Requirements

| ID | Requirement |
|----|-------------|
| R1 | `web/` Next.js (App Router) app with Clerk provider and Sign in with GitHub enabled via env. |
| R2 | `.env.example` documents `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY`, `CLERK_SECRET_KEY`, GitHub connection note. |
| R3 | `web/README.md` — install, `npm run dev`, Clerk dashboard steps (enable GitHub). |
| R4 | Doc `docs/multi-user-kurultai.md` — personal kernel vs team instance vs company; how users/devices attach; sync options pointing at #80. |
| R5 | Root README links `web/` + multi-user doc; CI ignores `web/node_modules` (gitignore). |
| R6 | No secrets in git; Rust tests unchanged. |

### Actors / flows

- A1 Operator sets Clerk keys · F1 `npm run dev` · F2 Sign in with GitHub · F3 Sees signed-in shell  
- A2 Team admin · F4 Reads multi-user doc for org = one Kurultai  

### Scope boundaries

**In:** `web/` lean scaffold, Clerk GitHub, docs, README link, gitignore.  
**Out:** Full shadcn dashboard pages, billing, atom sync, distillation gate, Rust multi-tenant storage.

### Acceptance examples

- AE1. `web/package.json` + Clerk dependency; `npm install` works.  
- AE2. Without keys, app documents failure mode; with keys, GitHub sign-in button present.  
- AE3. `docs/multi-user-kurultai.md` explains one instance = org + shared index; personal store separate.  
- AE4. `cargo test` / CI Lint still pass (web not in cargo).

---

## Planning Contract

### Multi-user model (design — ship as doc)

```text
Personal kernel (device)     Team Kurultai (one deploy)      Company
─────────────────────       ──────────────────────────      ────────
local store.db              shared store + daemon           multi-tenant orgs
no Clerk required           Clerk org members               same + compliance
promote atoms ──opt-in──►   shared index                    ◄── promote / policy
```

- **Users in a single Kurultai** = Clerk Organization members on one web+API deploy talking to one shared store.  
- **Multiple devices** (same user) = same Clerk user; local kernels sync later via #80; until then export/import or “promote to team”.  
- **Not** one SQLite file magically shared over Dropbox.

### Sequencing

1. U1 — Scaffold `web/` + Clerk GitHub  
2. U2 — Multi-user doc + README links  
3. U3 — gitignore / CI hygiene  

---

## Implementation Units

### U1. Lean `web/` Clerk + GitHub

**Files:** `web/package.json`, `web/src/app/…`, `web/src/proxy.ts`, `web/.env.example`, `web/README.md`

**Work:** Minimal Next App Router; `@clerk/nextjs`; sign-in/sign-up routes; home shows user when signed in; GitHub as social (Clerk dashboard). Inspired by Kiranism pattern — do not copy entire starter.

**Scenarios:** package installs; env example complete; sign-in page references GitHub.

### U2. Multi-user + sync model doc

**Files:** `docs/multi-user-kurultai.md`, root `README.md`

**Work:** Document tiers, one-instance users, device story, link #80/#81/#79.

**Scenarios:** doc exists; README links it and `web/`.

### U3. Hygiene

**Files:** `.gitignore`, optionally `web/.gitignore`

**Work:** ignore `web/node_modules`, `.next`, `.env*`; ensure Cargo CI unaffected.

---

## Verification Contract

- `git ls-files -- 'web/node_modules/**' 'web/.next/**'` must be empty  
- `rg Clerk web/`  
- `cargo test`  
- `cargo clippy --all-targets -- -D warnings`  
- Manual: `cd web && npm install && npm run build` if network allows  

---

## Definition of Done

- [ ] U1–U3 landed  
- [ ] Plan linked from PR  
- [ ] #81 referenced  
- [ ] CI green on Rust jobs  
