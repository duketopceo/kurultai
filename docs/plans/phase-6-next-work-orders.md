# Phase 6 — next work orders (post–Wave B)

**Status:** Planning · ready for `/lfg`  
**Date:** 2026-08-12  
**Base:** `main` after Wave B foundation (MCP HTTP/SSE · thin metrics · soft labels · export/import · Brain UI)  
**Umbrella:** [#10](https://github.com/duketopceo/kurultai/issues/10) Open Source Launch · Milestone 6  
**Parent pack:** [`phase-6-work-orders.md`](phase-6-work-orders.md)  
**Product contract:** [`docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md`](../brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md)  
**Milestone:** [Tiered Access + Hosted Hub](https://github.com/duketopceo/kurultai/milestone/8) (#176–#181)  
**Audience spine:** developer ✅ → solo ✅ (kernel) → **team 🔜** → company ([#25](https://github.com/duketopceo/kurultai/issues/25))

## Reality check

| Fact | Implication |
|------|-------------|
| Wave B P6-1 / thin P6-2 / P6-3 **shipped** | Do not re-LFG SSE, soft labels, or thin metrics |
| Export/import packs shipped ([#103](https://github.com/duketopceo/kurultai/pull/103)) | Offline multi-device handoff exists; hub is a different mechanism ([#80](https://github.com/duketopceo/kurultai/issues/80) adjacent) |
| Tiered Access brainstorm + issues filed | **Next product phase** = Wave G (v0.5.0 Team track), still under Phase 6 Launch — **not** a Phase 7 renumber |
| Solo kernel = SQLite forever | Postgres is for **shared** team/company hub only |
| Explicit reject | Multi-tenant SaaS for many unrelated orgs on one Kurultai-operated platform |

---

## Wave G — Tiered Access + Hosted Hub (recommended queue)

Ship **visibility scopes + shared hub** before Atlas UI or enterprise connector sprawl.

| Order | ID | Work order | Issue | Size | First LFG? |
|------:|----|------------|-------|------|------------|
| 1 | **HUB-1** | **Atom visibility scope** `personal \| team \| company` + merged local query shape (solo unchanged) | [#178](https://github.com/duketopceo/kurultai/issues/178) | M | **Yes — next `/lfg`** · plan [`2026-08-12-002-feat-tiered-access-atom-scope-plan.md`](2026-08-12-002-feat-tiered-access-atom-scope-plan.md) |
| 2 | HUB-2 | Postgres + pgvector `Store` for **shared** tier | [#176](https://github.com/duketopceo/kurultai/issues/176) · legacy [#111](https://github.com/duketopceo/kurultai/issues/111) | L | After HUB-1 |
| 3 | HUB-3 | Hub mode daemon — Tailscale-only **or** public + per-device API key | [#177](https://github.com/duketopceo/kurultai/issues/177) | L | Needs HUB-2 |
| 4 | HUB-4 | Admin CLI — issue/revoke device keys; `team_id` / `org_id` boundaries | [#179](https://github.com/duketopceo/kurultai/issues/179) | M | With/after HUB-3 |
| 5 | HUB-5 | Connector ingest tags visibility scope at source (never infer later) | [#180](https://github.com/duketopceo/kurultai/issues/180) · narrows [#114](https://github.com/duketopceo/kurultai/issues/114) | M | After HUB-1 |
| 6 | HUB-6 | Acceptance suite AE1–AE5 | [#181](https://github.com/duketopceo/kurultai/issues/181) | M | Continuous; gate before calling v0.5.0 “team” |

**HUB-1 DoD (LFG capsule):** every `KnowledgeAtom` carries a visibility scope (default `personal`); solo `ask`/`search` unchanged when no hub configured; schema/migration + tests; **no** Postgres and **no** public bind in this slice.

Maps brainstorm R1–R3, R8 (local half) and AE1. Year-1 WO-201…204 → release branding remains **v0.5.0 Team**.

---

## Residual Wave B / Launch (interleave only if blocking)

| ID | Item | Issue | Notes |
|----|------|-------|-------|
| P6-2b | Full metrics + error tracking beyond thin histograms | [#102](https://github.com/duketopceo/kurultai/issues/102) · [#35](https://github.com/duketopceo/kurultai/issues/35) | Thin slice shipped ([#126](https://github.com/duketopceo/kurultai/pull/126)); GlitchTip-depth still open |
| P6-4 | Cloud Brain UI tunnel (GitHub login → local daemon) | [#101](https://github.com/duketopceo/kurultai/issues/101) | Deferred vs hub scopes; brainstorm excluded UI |
| P6-O0 | Living ROADMAP solo→team→company | [#122](https://github.com/duketopceo/kurultai/issues/122) | Docs; can parallel HUB-1 |
| Launch F | crates.io / Homebrew / Show HN packaging | [#10](https://github.com/duketopceo/kurultai/issues/10) | Non-kernel; interleave |

---

## Still later (do not steal the next LFG)

| Wave | Focus | Issues |
|------|--------|--------|
| C (legacy ids) | Prefer Wave G HUB-* over duplicate P6-T* ordering | [#111](https://github.com/duketopceo/kurultai/issues/111)–[#115](https://github.com/duketopceo/kurultai/issues/115) |
| D / E / E′ | Slack allowlist, ontology, Atlas, Notion, webhooks | [#114](https://github.com/duketopceo/kurultai/issues/114), [#116](https://github.com/duketopceo/kurultai/issues/116)–[#122](https://github.com/duketopceo/kurultai/issues/122), [#128](https://github.com/duketopceo/kurultai/issues/128)–[#135](https://github.com/duketopceo/kurultai/issues/135) |
| Ops | ARC, env hardening | [#20](https://github.com/duketopceo/kurultai/issues/20), [#29](https://github.com/duketopceo/kurultai/issues/29) |

Atlas sequencing still lives in [`phase-6-atlas-gaps.md`](phase-6-atlas-gaps.md) — do not LFG A2–A5 before O1 / structured contract.

---

## Explicitly deferred

| Item | Why |
|------|-----|
| Multi-tenant SaaS (many unrelated orgs on one platform) | Rejected in brainstorm |
| Password/session accounts for public hub | v1 = per-device API keys only |
| UI redesign for tiers | Explicitly out of brainstorm scope |
| Replacing `.kurultai` export/import | Keep for offline solo; hub supersedes only for shared tiers |

---

## `/lfg` playbook (this queue)

1. Docs (this PR): land next-queue pack + HUB-1 plan.  
2. Agent `/lfg`: **HUB-1** → [`2026-08-12-002-feat-tiered-access-atom-scope-plan.md`](2026-08-12-002-feat-tiered-access-atom-scope-plan.md).  
3. Next: **HUB-2** Postgres Store ([#176](https://github.com/duketopceo/kurultai/issues/176)).  
4. Then HUB-3 transport → HUB-4 admin → HUB-5 connector tagging; keep HUB-6 green.

### Hygiene

- Do not mass-close backlog issues while sequencing.  
- Prefer linking #111/#115 as “see Wave G” rather than rewriting issue bodies in automation.  
- Close accidental [#110](https://github.com/duketopceo/kurultai/issues/110) when a human can.
