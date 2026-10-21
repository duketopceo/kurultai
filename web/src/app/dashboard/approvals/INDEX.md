---
index: kurultai/v1
folder: web/src/app/dashboard/approvals
parent: web/src/app/dashboard/INDEX.md
updated: 2026-10-20
version: 1
---

# `web/src/app/dashboard/approvals`

**Does:** Pending `kurultai connect` device-flow approvals (mint-at-approve w/ team pick)
**Up:** [`../INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`page.tsx`](page.tsx) | Pending flows table → `POST /api/hub/device/approve`; recent decisions | `GET /api/hub/device/pending`, scopes/keys for team datalist | `ApproveFlowForm` | 2026-10-20 | 1 | 2026-10-20 U8 approvals surface |

## Recent

- 2026-10-20 — U8 approvals surface
