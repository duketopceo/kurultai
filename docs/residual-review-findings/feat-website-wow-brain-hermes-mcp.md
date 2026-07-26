## Residual Review Findings

Source: ce-code-review run `20260725-185039-0af338fe` (artifact: `/tmp/compound-engineering-502/ce-code-review/20260725-185039-0af338fe`)

Carried in PR: https://github.com/duketopceo/kurultai/pull/85

- **P2** — `docs/plans/2026-07-26-001-feat-website-wow-brain-hermes-mcp-plan.md:22` — Plan reintroduces deprecated website brain surface — settled conflict with `KTD-Brain-UI-daemon-ui-only`. The plan still directs work toward a standalone `website/` brain experience, which conflicts with the documented single-surface `GET /ui` architecture in `CONCEPTS.md` and `docs/solutions/architecture-patterns/one-brain-ui-daemon-ui-only.md`. Action: update or supersede the plan so it reflects `website/` as optional Vite preview rooted at `../ui` and daemon `GET /ui` as the only product brain dashboard.
