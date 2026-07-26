---
title: One brain UI surface — daemon /ui only
date: 2026-07-25
category: architecture-patterns
module: http/ui
problem_type: architecture_pattern
component: tooling
severity: high
applies_when:
  - "Adding or redesigning a Kurultai brain dashboard / explorer"
  - "Introducing Vite, Next.js, or other frontend apps in this repo"
  - "Running ce-compound-refresh on UI or http-related learnings"
tags: [duplicate-ui, website, http-ui, brain-ui, architecture]
---

# One brain UI surface — daemon `/ui` only

## Context

Kurultai briefly had three overlapping UIs: an embedded `DASHBOARD_HTML` string on daemon `GET /ui`, a standalone Vite app under `website/`, and a separate Next.js + Clerk app under `web/` (auth portal PR). Agents and humans treated each as “the” dashboard, so features forked and README entrypoints drifted.

## Guidance

1. **Canon:** Brain UI = daemon `GET /ui` only. README entry remains `http://127.0.0.1:8421/ui`.
2. **Assets:** Editable source lives in `ui/` and is **embedded** into the binary (`src/http/ui.rs` via `rust-embed`). Do not reintroduce a giant `DASHBOARD_HTML` constant in `src/http/mod.rs`.
3. **`website/`:** Optional Vite **preview** of `ui/` (API proxy only). Not a product; no second brain explorer.
4. **`web/`:** Out of scope for brain dashboard work — auth / multi-user portal is a different product surface. Do not fold Clerk into `/ui` without an explicit project decision.
5. **Refresh audit signal:** When refreshing learnings tagged `duplicate-ui`, `website`, or `http-ui`, verify there is still only one brain UI root (`ui/` + daemon routes). Flag regressions: new `website/` product app, new `web/` brain dashboard, or a re-forked embedded HTML dashboard.

## Why This Matters

- One URL for humans and agents; no “which UI is current?” thrash.
- Binary ships the UI; no Node runtime required for the product path.
- Compound refresh can detect duplicate UI roots via tags before a third surface appears again.

## When to Apply

- Any PR that adds HTML/JS under a new top-level folder for “the brain”
- Dashboard / synapse / explorer feature work
- `ce-compound-refresh` passes touching HTTP or frontend docs

## Examples

```text
# Product path
kurultai daemon --port 8421
open http://127.0.0.1:8421/ui

# Optional design preview only
cd website && npm run dev   # serves ../ui with /api → :8421
```

## Prevention

- Before adding a frontend tree: search for tags `duplicate-ui` / `brain-ui` and read this learning + CONCEPTS **Brain UI**.
- Refuse PRs that introduce a second brain dashboard without demoting the loser.
- Keep `/api/*` as the data plane; UI is presentation only.

## Related

- [CONCEPTS.md — Brain UI](../../CONCEPTS.md#brain-ui)
- README daemon / Status UI one-liner
- PR #82 `web/` (Clerk) — separate auth surface, not brain UI
