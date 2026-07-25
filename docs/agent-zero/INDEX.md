# Kurultai Agent Zero drafts → GitHub (v1)

**Last Updated:** 2026-07-25  
**Source zip:** `Downloads/agent-zero-selected-6-20260725-150923.zip`  
**Repo:** [duketopceo/kurultai](https://github.com/duketopceo/kurultai)

---

## GitHub issues created

| Draft | GitHub | Priority | Title |
|-------|--------|----------|-------|
| ISSUE-004-agent-installer.md | [#72](https://github.com/duketopceo/kurultai/issues/72) | P0 | Agent-level CLI installer (personal) — **in progress:** `scripts/install/install.sh` + plan `docs/plans/2026-07-25-005-feat-v1-personal-installer-plan.md` |
| ISSUE-001-scheduled-background-indexing.md | [#73](https://github.com/duketopceo/kurultai/issues/73) | P0 | Scheduled background indexing |
| ISSUE-003-multi-hop-reasoning.md | [#74](https://github.com/duketopceo/kurultai/issues/74) | P1 | Multi-hop reasoning (graph orchestration) |
| ISSUE-002-query-result-citations.md | [#75](https://github.com/duketopceo/kurultai/issues/75) | P1 | Complete query result citations contract |
| PR-001-dev-dashboard-http-daemon.md | [#76](https://github.com/duketopceo/kurultai/issues/76) | P2 stretch | Dev dashboard (HTTP UI + WebSocket) |

Master plan: [#27](https://github.com/duketopceo/kurultai/issues/27)

---

## Recommended v1 execution order

1. **#72** Agent installer — barrier to entry  
2. **#73** Scheduled indexing — stale data (builds on Phase 5 daemon poll / notify)  
3. **#74** Multi-hop — multi-doc answers (RRF already shipped)  
4. **#75** Citations — complete provenance contract (partially shipped)  
5. **#76** Dashboard — stretch after P0/P1  

---

## Local draft files

Full Agent Zero specs (problem, architecture, AC, open questions):

- [ISSUE-001-scheduled-background-indexing.md](./ISSUE-001-scheduled-background-indexing.md)
- [ISSUE-002-query-result-citations.md](./ISSUE-002-query-result-citations.md)
- [ISSUE-003-multi-hop-reasoning.md](./ISSUE-003-multi-hop-reasoning.md)
- [ISSUE-004-agent-installer.md](./ISSUE-004-agent-installer.md)
- [PR-001-dev-dashboard-http-daemon.md](./PR-001-dev-dashboard-http-daemon.md)

---

## Auth note

Issues were created with `LukeDuke-Bartlett` (can open/close issues). Milestone + custom label create returned 404 (needs `duketopceo` admin). Re-auth when ready:

```bash
gh auth login -h github.com -u duketopceo
gh api repos/duketopceo/kurultai/milestones -f title="v1 Release" -f state=open
```

---

**Created by:** Agent Zero drafts → imported for duketopceo/kurultai v1  
**Date:** 2026-07-25
