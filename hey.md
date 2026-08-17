# hey.md — Kurultai

**Version:** v0.4.1
**Last edit:** 2026-08-17 — repo hygiene (queue + Linear workspace)

## Current Phase: Wave G (team hub)

> Kurultai is a public repo (MIT). Rust knowledge retrieval CLI + daemon + embedded Brain UI.
> Crate on `main` is **v0.4.1**. Product flag `hub` stays **off** until a real team hub ships.
> Live queue: [`docs/plans/phase-6-next-work-orders.md`](docs/plans/phase-6-next-work-orders.md).
> Linear: [imluketheduke / Khan](https://linear.app/imluketheduke) (`KHAN-*`). Not bartlettroofs-it / `PRO-*`.

| KHAN-ID | Title | GitHub | Status | Branch/PR |
|---|---|---|---|---|
| [KHAN-255](https://linear.app/imluketheduke/issue/KHAN-255/hub-3-hub-mode-dual-transport-gh-177) | HUB-3 Railway / public or Tailscale bind | [#177](https://github.com/duketopceo/kurultai/issues/177) | **Next LFG** | — |
| [KHAN-253](https://linear.app/imluketheduke/issue/KHAN-253/hub-4-admin-api-keys-team-id-gh-179) | HUB-4 admin keys + team_id | [#179](https://github.com/duketopceo/kurultai/issues/179) | Todo after HUB-3 | — |
| [KHAN-252](https://linear.app/imluketheduke/issue/KHAN-252/hub-5-ingest-visibility-tagging-gh-180) | HUB-5 ingest visibility tagging | [#180](https://github.com/duketopceo/kurultai/issues/180) | Todo after HUB-4 | — |
| [KHAN-254](https://linear.app/imluketheduke/issue/KHAN-254/hub-1-personalteamcompany-scopes-gh-178) | HUB-1 atom scopes | [#178](https://github.com/duketopceo/kurultai/issues/178) | Done `#192` | — |
| [KHAN-256](https://linear.app/imluketheduke/issue/KHAN-256/hub-2-postgrespgvector-store-gh-176) | HUB-2 Postgres store | [#176](https://github.com/duketopceo/kurultai/issues/176) | Done `#197` | — |
| [KHAN-251](https://linear.app/imluketheduke/issue/KHAN-251/ae-suite-ae1-ae5-gh-181-pr-216) | AE suite AE1–AE5 | [#181](https://github.com/duketopceo/kurultai/issues/181) | Done `#216` (remaining AEs wait on HUB-3/4/5) | — |

## Repo Status

- **Repo:** `duketopceo/kurultai` (public, MIT)
- **Version:** v0.4.1 (GitHub Release `v0.5.0` exists but is **not** team hub shipped)
- **Stack:** Rust CLI + daemon, embedded `/ui/`, optional `web/` (Clerk)
- **Structure:** start at [`INDEX.md`](INDEX.md); plans in [`docs/plans/`](docs/plans/)

## Agents

| Agent | Status | Current Assignment |
|---|---|---|
| — | — | Next product LFG is HUB-3 only |

## Process

See `skills/hey-board/SKILL.md` in the Khan repo for the full lifecycle rules.

Key rules:
1. No self-review
2. One reviewer per issue, assigned at phase load
3. Reviewer is merge captain
4. Codegraph-first when available
5. **Kurultai is public MIT** — use PRs for all changes, including hey.md edits (no direct-to-main exemption unlike Khan)
6. Same GitHub author (duketopceo) cannot formally approve own PRs — use review comments + hey.md attribution

## Changelog

- v0.4.1 — 2026-08-17 — hygiene: current crate, Wave G queue, Linear Khan workspace
- v0.0001.0 — perplexity — Initial hey.md created, bootstrap phase
