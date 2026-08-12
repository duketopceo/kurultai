# Gate 0 — query rewrite on live AI Search

Status: **not run**. Decision required before Nimrod hub cutover (plan 003, U2 / KTD6).

## Purpose

Prove whether “Ask Bartlett feels dumb” dies with query rewriting on the **existing** Cloudflare AI Search pipeline — before betting on a full Kurultai hub migration.

## Procedure

1. Freeze 20–50 real questions with expected doc ids or keywords.
2. Confirm whether AI Search query rewriting is already enabled.
3. Score baseline vs rewrite (hit@k and/or human 1–5).
4. Write the go/no-go line below.

## Scores

| Condition | n | hit@5 | notes |
|-----------|---|-------|-------|
| Baseline (rewrite off / current) | — | — | |
| Rewrite on | — | — | |

## Decision

- [ ] Pause hub migration — rewrite closed the quality gap
- [ ] Continue hub for isolation + ops (quality already OK)
- [ ] Continue hub for quality — rewrite insufficient

Decision (one line): _pending_

Identity: run brain-worker changes under Bartlett work logins; keep this file in `duketopceo/kurultai`.
