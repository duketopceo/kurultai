# Phase 6 — next work orders (post–Wave B)

**Status:** crate **v0.4.1** on `main` · Wave G hub still **v0.5.0** (flag `hub` **off**)  
**Date:** 2026-08-12 · **Updated:** 2026-08-17  
**Base:** `main` after Wave B foundation + HUB-1/2/6 + O1/O2 + agent index  
**Umbrella:** [#10](https://github.com/duketopceo/kurultai/issues/10) Open Source Launch · Milestone 6  
**Parent pack:** [`phase-6-work-orders.md`](phase-6-work-orders.md)  
**Product contract:** [`docs/brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md`](../brainstorms/2026-08-07---tiered-access-hosted-hub-requirements.md)  
**Milestone:** [Tiered Access + Hosted Hub](https://github.com/duketopceo/kurultai/milestone/8) (#176–#181)  
**Linear:** [imluketheduke / Khan](https://linear.app/imluketheduke) — `KHAN-*` only. Ignore bartlettroofs-it / `PRO-*`.  
**Audience spine:** developer ✅ → solo ✅ (kernel) → **team 🔜** → company ([#25](https://github.com/duketopceo/kurultai/issues/25))

## Version flags

Override with `KURULTAI_FEATURE_<ID>=0|1`. Catalog: `src/features.rs` · `kurultai status`.

| Flag | Since | Default | Work |
|------|-------|---------|------|
| `fts` | v0.3.0 | on | Kernel search (shipped) |
| `brain_ui` | v0.4.0 | on | Embedded `/ui/` (shipped) |
| `mcp_http` | v0.4.0 | on | Daemon MCP HTTP/SSE (shipped) |
| `local_embed` | v0.3.0 | cargo feature | `--features local-embed` |
| `hub` | **v0.5.0** | **off** | Wave G remaining: HUB-3…5 — do not treat as shipped |

**v0.3.1 was skipped.** Production crate tag is [v0.4.1](https://github.com/duketopceo/kurultai/releases/tag/v0.4.1). A GitHub Release named `v0.5.0` already exists (2026-08-14) — that is a **naming leftover**, not “team hub shipped.” Team cashflow still waits on HUB-3+.

## Reality check

| Fact | Implication |
|------|-------------|
| Wave B P6-1 / thin P6-2 / P6-3 **shipped** | Do not re-LFG SSE, soft labels, or thin metrics |
| Export/import packs shipped ([#103](https://github.com/duketopceo/kurultai/pull/103)) | Offline multi-device handoff exists; hub is a different mechanism ([#80](https://github.com/duketopceo/kurultai/issues/80) adjacent) |
| HUB-1 `#192` · HUB-2 `#197` · AE suite `#216` · O1+O2 `#201`/`#202` · agent index `#224` | On `main`. Do not re-LFG. GitHub [#116](https://github.com/duketopceo/kurultai/issues/116)/[#117](https://github.com/duketopceo/kurultai/issues/117) are tracker leftovers (human close). |
| **Next LFG** = HUB-3 | [#177](https://github.com/duketopceo/kurultai/issues/177) · [KHAN-255](https://linear.app/imluketheduke/issue/KHAN-255/hub-3-hub-mode-dual-transport-gh-177) |
| Solo kernel = SQLite forever | Postgres is for **shared** team/company hub only |
| Explicit reject | Multi-tenant SaaS for many unrelated orgs on one Kurultai-operated platform |

---

## Wave G — Tiered Access + Hosted Hub (recommended queue)

Ship **visibility scopes + shared hub** before Atlas UI or enterprise connector sprawl.

| Order | ID | Work order | Issue | Linear | Size | Status |
|------:|----|------------|-------|--------|------|--------|
| 0 | **REL-1** | **v0.4.1 production tag** | this tree | — | S | ✅ `#196` |
| 1 | **HUB-1** | Atom visibility `personal \| team \| company` | [#178](https://github.com/duketopceo/kurultai/issues/178) | [KHAN-254](https://linear.app/imluketheduke/issue/KHAN-254/hub-1-personalteamcompany-scopes-gh-178) | M | ✅ `#192` |
| 2 | **HUB-2** | Postgres + pgvector `Store` for **shared** tier | [#176](https://github.com/duketopceo/kurultai/issues/176) | [KHAN-256](https://linear.app/imluketheduke/issue/KHAN-256/hub-2-postgrespgvector-store-gh-176) | L | ✅ `#197` |
| 3 | **HUB-3** | Hub mode daemon — Tailscale-only **or** public + per-device API key | [#177](https://github.com/duketopceo/kurultai/issues/177) | [KHAN-255](https://linear.app/imluketheduke/issue/KHAN-255/hub-3-hub-mode-dual-transport-gh-177) | L | **Next LFG** (API-key scaffold `#190`) |
| 4 | HUB-4 | Admin CLI — issue/revoke device keys; `team_id` / `org_id` | [#179](https://github.com/duketopceo/kurultai/issues/179) | [KHAN-253](https://linear.app/imluketheduke/issue/KHAN-253/hub-4-admin-api-keys-team-id-gh-179) | M | After HUB-3 |
| 5 | HUB-5 | Connector ingest tags visibility at source | [#180](https://github.com/duketopceo/kurultai/issues/180) | [KHAN-252](https://linear.app/imluketheduke/issue/KHAN-252/hub-5-ingest-visibility-tagging-gh-180) | M | After HUB-4 (`#193` closed unmerged) |
| 6 | HUB-6 | Acceptance suite AE1–AE5 | [#181](https://github.com/duketopceo/kurultai/issues/181) | [KHAN-251](https://linear.app/imluketheduke/issue/KHAN-251/ae-suite-ae1-ae5-gh-181-pr-216) | M | ✅ `#216` in tree; remaining AEs wait on HUB-3/4/5 |

**2026-08-15 sequence:** [`2026-08-15-000-chore-wave-g-railway-sequence-plan.md`](2026-08-15-000-chore-wave-g-railway-sequence-plan.md) (001 Railway → 002 agent IDs → 003 desktop wrap). **Next LFG is 001 / HUB-3 only.**

HUB-1 shipped (`#192`). Next capsule is HUB-3 — see [`2026-08-15-001-feat-hub3-railway-transport-plan.md`](2026-08-15-001-feat-hub3-railway-transport-plan.md). Year-1 WO-201…204 branding remains **v0.5.0 Team**.

---

## Residual Wave B / Launch (interleave only if blocking)

| ID | Item | Issue | Notes |
|----|------|-------|-------|
| P6-2b | Full metrics + error tracking beyond thin histograms | [#102](https://github.com/duketopceo/kurultai/issues/102) · [#35](https://github.com/duketopceo/kurultai/issues/35) | Thin slice shipped ([#126](https://github.com/duketopceo/kurultai/pull/126)); GlitchTip-depth still open |
| P6-4 | Cloud Brain UI tunnel (GitHub login → local daemon) | [#101](https://github.com/duketopceo/kurultai/issues/101) | Deferred vs hub scopes; brainstorm excluded UI |
| P6-O0 | Living ROADMAP solo→team→company | [#122](https://github.com/duketopceo/kurultai/issues/122) | Docs; can parallel HUB-3 |
| Launch F | crates.io / Homebrew / Show HN packaging | [#10](https://github.com/duketopceo/kurultai/issues/10) | Non-kernel; interleave |

---

## Still later (do not steal the next LFG)

| Wave | Focus | Issues |
|------|--------|--------|
| C (legacy ids) | Prefer Wave G HUB-* over duplicate P6-T* ordering | [#111](https://github.com/duketopceo/kurultai/issues/111)–[#115](https://github.com/duketopceo/kurultai/issues/115) |
| D / E / E′ | Slack allowlist, ontology, Atlas, Notion, webhooks | [#114](https://github.com/duketopceo/kurultai/issues/114), [#116](https://github.com/duketopceo/kurultai/issues/116)–[#122](https://github.com/duketopceo/kurultai/issues/122), [#128](https://github.com/duketopceo/kurultai/issues/128)–[#135](https://github.com/duketopceo/kurultai/issues/135) |
| Ops | ARC, env hardening | [#20](https://github.com/duketopceo/kurultai/issues/20), [#29](https://github.com/duketopceo/kurultai/issues/29) |

Atlas sequencing still lives in [`phase-6-atlas-gaps.md`](phase-6-atlas-gaps.md) — O1 is on `main`; do not steal A2–A5 into the HUB-3 LFG.

Brain dual-mode (brain-shape FDG + algorithmic ontology; galaxy out): research [`docs/brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md`](../brainstorms/2026-08-13---brain-shape-algorithmic-ontology.md); plan [`2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md`](2026-08-13-004-feat-brain-shape-algorithmic-ontology-plan.md). Slice A `#200`, O1 `#201`, Sugiyama O2 `#202` are on `main`. Do not LFG O3.

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

1. ~~HUB-1 atom scope~~ ✅ `#192` · [KHAN-254](https://linear.app/imluketheduke/issue/KHAN-254/hub-1-personalteamcompany-scopes-gh-178).  
2. ~~v0.4.1 prep~~ ✅ `#196` / `#195`. Tag **`v0.4.1`** is on `main`.  
3. ~~HUB-2 Postgres Store~~ ✅ `#197` · [KHAN-256](https://linear.app/imluketheduke/issue/KHAN-256/hub-2-postgrespgvector-store-gh-176). SQLite stays the solo kernel.  
4. **HUB-3** Railway / public or Tailscale bind ([#177](https://github.com/duketopceo/kurultai/issues/177) · [KHAN-255](https://linear.app/imluketheduke/issue/KHAN-255/hub-3-hub-mode-dual-transport-gh-177)) — **this is the next LFG.** API-key scaffold is `#190`.  
5. Then HUB-4 admin ([#179](https://github.com/duketopceo/kurultai/issues/179) · [KHAN-253](https://linear.app/imluketheduke/issue/KHAN-253/hub-4-admin-api-keys-team-id-gh-179)) → HUB-5 connector tagging ([#180](https://github.com/duketopceo/kurultai/issues/180) · [KHAN-252](https://linear.app/imluketheduke/issue/KHAN-252/hub-5-ingest-visibility-tagging-gh-180); `#193` did not land). AE suite `#216` is already in tree; re-run remaining AEs after 3/4/5.

### Hygiene

- Do not mass-close backlog issues while sequencing.  
- Prefer linking #111/#115 as “see Wave G” rather than rewriting issue bodies in automation.  
- Close accidental [#110](https://github.com/duketopceo/kurultai/issues/110) when a human can.
