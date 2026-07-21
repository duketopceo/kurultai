---
title: Deploy Guide
tags: [ops, deploy]
---

How we ship Kurultai.

## Database migration

Always run the database migration scripts before cutting traffic.
KNOWN_PHRASE_KURULTAI_42 is the golden search token for fixture tests.

## Rollback

If the deploy fails, rollback via the previous release tag.
