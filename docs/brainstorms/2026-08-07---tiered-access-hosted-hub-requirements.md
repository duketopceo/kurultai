# Tiered Access (Personal / Team / Company) + Hosted Hub

**Status:** Draft — ready for `ce-plan`
**Date:** 2026-08-07
**Tier:** Deep (cross-cutting: storage, connectors, daemon, auth)

## Context

Kurultai today is single-user, single-device, local-first: one SQLite file, one
CLI, no network backend. The originating problem: a team member needs to add
context to a shared Kurultai brain, visible to their team — but a personal
Slack DM, for example, must never reach that shared brain, while a team
channel should.

This is not a request for full RBAC/governance. It is a request for
**selective visibility across a hierarchy that varies in depth per
deployment** — a solo user has zero shared tiers, two people ("a dude and his
buddy") can stand up a `team` tier without ever needing `company`, and a real
company uses all three.

A second, later requirement was added: some deployments should be reachable
over a private Tailscale network (no public exposure), while others — run by
people other than the original author — need a public subdomain/URL, because
they won't all be on the same tailnet.

Rejected alternative: a fully multi-tenant SaaS backend serving many unrelated
companies on shared infrastructure. Ruled out explicitly — this is one
deployment per organization, not a hosted platform Kurultai's author operates
for everyone.

## Actors (A-IDs)

| ID | Actor | Description |
|----|-------|--------------|
| A1 | Solo operator | Runs Kurultai locally only. No hub. Today's default behavior, must not regress. |
| A2 | Team member | A person added to a shared `team` or `company` tier, connecting from their own device. |
| A3 | Hub admin | Provisions the hub, issues/revokes per-device API keys, defines team/org boundaries. Usually the same person as A1 in small deployments. |
| A4 | External self-hoster | Someone other than the original author running their own Kurultai deployment for their own org — needs public transport, not Tailscale. |

## Requirements (R-IDs)

- **R1** — Every knowledge atom carries a visibility scope: `personal | team | company`.
- **R2** — `personal`-scoped atoms never leave the originating device. They are never written to any shared/hosted store, under any transport mode.
- **R3** — A deployment supports zero, one, or two shared tiers. `team` without `company` must be a fully valid, first-class configuration — no forced 3-level hierarchy.
- **R4** — The shared store (team/company tier) is reachable via two selectable transport modes per deployment: (a) Tailscale-only private network, (b) public subdomain/URL.
- **R5** — Public transport mode authenticates every device via a per-device API key (bearer token), issued by an admin CLI command. No password/session accounts in v1.
- **R6** — Tailscale transport mode relies on Tailscale network membership/ACLs for access control. No separate app-level auth required in this mode.
- **R7** — The existing `Store` trait (`src/store/mod.rs`) gets a Postgres+pgvector implementation for the shared tier, without breaking the existing SQLite implementation used for the personal tier.
- **R8** — A local daemon queries personal (local SQLite) and team/company (remote hub) scopes in a single `ask`/`search` call and merges results.
- **R9** — Connectors touching shared content (e.g., a Slack channel) are configurable to tag ingested atoms with a visibility scope at ingest time. Scope is never inferred after the fact.
- **R10** — Admin CLI tooling supports: issuing/revoking device API keys, defining `team_id`/`org_id` boundaries, listing which scopes exist on a given hub.

## Key Flows (F-IDs)

- **F1** — Solo user (A1), no hub configured. `kurultai ask` behaves exactly as it does today, local SQLite only.
- **F2** — Two individuals stand up one hub, both join via Tailscale, tag a subset of atoms `team`. No `company` tier exists in this deployment.
- **F3** — A company admin (A3) provisions a hub with a public subdomain, issues per-device API keys to employees. Team A and Team B don't see each other's `team`-scoped atoms; both see `company`-scoped atoms.
- **F4** — A Slack connector ingests a public team channel tagged `team` at ingest. A DM to the same connected account is tagged `personal` and is never sent to the hub.
- **F5** — A device's API key is revoked. Its next hub query is denied. Its local, personal-tier queries (local SQLite) are unaffected.

## Acceptance Examples (AE-IDs)

- **AE1** — Fresh install, no hub configured → `kurultai ask "..."` returns results from local SQLite only, identical to pre-existing behavior.
- **AE2** — Hub reachable only via Tailscale → a device off the tailnet cannot reach it (connection refused/timeout); a device on the tailnet queries team-scoped content successfully.
- **AE3** — Hub in public mode → a request with a missing or revoked API key returns 401/403 and never returns team/company-scoped data.
- **AE4** — An atom tagged `personal` never appears in any table on the hub's Postgres instance — verified by direct inspection, not just by API behavior.
- **AE5** — Two teams (`team_id=eng`, `team_id=sales`) on one company hub → a member of `eng` querying `ask` never receives `sales`-tagged atoms, but does receive `company`-tagged atoms.

## Explicitly Out of Scope (Deferred)

- Multi-tenant SaaS across unrelated companies on shared infrastructure — ruled out; one deployment per org.
- Full user accounts (login/password/session) for public mode — v1 is API-key only.
- UI changes of any kind — explicitly excluded from this brainstorm.
- Exact ACL config file format — left to `ce-plan`.
- Which connector ships first for team-tier ingestion — Slack was the working example throughout this brainstorm, not a committed decision.
- Tailscale Funnel as a later escape hatch for exposing a tailnet-only hub publicly without re-architecting — noted, not built now.

## Architecture Notes for `ce-plan`

- Reuse the existing `Store` trait (`src/store/mod.rs`) — add a Postgres+pgvector implementation alongside the existing SQLite implementation. Do not modify existing personal-tier code paths.
- The existing `.kurultai` export/import pack mechanism ([PR #103](https://github.com/duketopceo/kurultai/pull/103)) is prior art for cross-device data movement; the hub approach likely supersedes it for team/company tiers specifically, but should be reviewed before duplicating effort.
- The existing axum HTTP daemon (`src/http/`, `src/daemon/`) is a strong candidate to extend into "hub mode" rather than rewrite — same server, different bind target (Tailscale interface vs. public) plus an auth middleware layer for public mode only.
- Related existing issues this brainstorm touches or supersedes parts of (none should be closed without review — see relabeled backlog):
  - [#111](https://github.com/duketopceo/kurultai/issues/111) — Postgres + pgvector store (this doc's R7 is a scoped version of this)
  - [#114](https://github.com/duketopceo/kurultai/issues/114) — Slack connector with ACL (this doc's F4/R9 narrows this)
  - [#115](https://github.com/duketopceo/kurultai/issues/115) — personal/team/company RBAC (this doc replaces "RBAC" framing with scope-tagging + transport-level auth, per R1–R6)
  - [#80](https://github.com/duketopceo/kurultai/issues/80) — opt-in cloud sync (adjacent; this doc's hub is a different mechanism, not sync)
  - [#101](https://github.com/duketopceo/kurultai/issues/101) — cloud-hosted UI (out of scope here — UI explicitly excluded)
  - [#79](https://github.com/duketopceo/kurultai/issues/79) — sign in with Anthropic (superseded by the API-key model in R5; "sign in" style auth deferred)

## Next Step

Recommended: `ce-plan` against this doc, or convert R1–R10 / F1–F5 / AE1–AE5 directly into scoped GitHub issues under a new milestone (see companion issues opened alongside this doc).
