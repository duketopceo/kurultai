---
index: kurultai/v1
folder: web/src/components
parent: web/src/INDEX.md
updated: 2026-10-20
version: 1
---

# `web/src/components`

**Does:** Shared dashboard components (client forms + server panels)
**Up:** [`web/src/INDEX.md`](../INDEX.md) · **Protocol:** [`docs/agent-index.md`](../../../docs/agent-index.md)

## Children

_None._

## Files

| File | Does | Needs | Touches | Stamp | Ver | Changelog |
|------|------|-------|---------|-------|-----|-----------|
| [`approve-flow-form.tsx`](approve-flow-form.tsx) | Client: approve device flow w/ team datalist | `approveDeviceAction` | approvals page | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`copy-button.tsx`](copy-button.tsx) | Client: clipboard copy feedback | — | key-issue-form | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`dash-nav.tsx`](dash-nav.tsx) | Client: console sub-nav w/ active link | — | dashboard layout | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`forbidden.tsx`](forbidden.tsx) | 403 panel for signed-in non-admins | — | dashboard layout | 2026-10-20 | 1 | 2026-10-20 U8 admin gate |
| [`hub-error.tsx`](hub-error.tsx) | Daemon-unreachable banner | `hubApiUrl()` | all console pages | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`key-issue-form.tsx`](key-issue-form.tsx) | Client: issue key + show-once plaintext | `issueKeyAction`, CopyButton | keys page | 2026-10-20 | 1 | 2026-10-20 U8 admin console |
| [`revoke-key-button.tsx`](revoke-key-button.tsx) | Client: confirm + revoke action | `revokeKeyAction` | keys page | 2026-10-20 | 1 | 2026-10-20 U8 admin console |

## Recent

- 2026-10-20 — U8 hub admin console components
