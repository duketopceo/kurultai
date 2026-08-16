---
index: kurultai/v1
folder: docs
title: Agent index protocol
updated: 2026-08-16
version: 1
---

# Agent index protocol

**Up:** [INDEX.md](../INDEX.md) · **This folder:** [docs/INDEX.md](INDEX.md)

A brand-new agent should open **[INDEX.md](../INDEX.md)** first, then the folder index for the surface they need. Do not glob the whole repo to learn where a file lives.

## Schema

Every in-scope folder has `INDEX.md` with frontmatter:

```yaml
index: kurultai/v1
folder: src/store          # repo-relative folder, or `.` at root
parent: src/INDEX.md       # omit on root
updated: 2026-08-16
version: 1                 # folder map version (integer)
```

### File row

| Field | Meaning |
|-------|---------|
| Does | One sentence. What the file is for. |
| Needs | 2–5 repo-relative paths this file imports or reads. `—` if none. |
| Touches | 2–5 paths that import this file or that it writes/embeds. `—` if none. |
| Stamp | Last git commit date `YYYY-MM-DD` when the row was last refreshed. |
| Ver | Per-file integer in this index, start at `1`. |
| Changelog | At most **3** lines, newest first. Seed is `YYYY-MM-DD indexed (v1 seed)`. |

### Folder Recent

At most **8** lines at the folder index; root Recent at most **15**. When a file row changes, prepend one line here **and** on every ancestor up to root.

## Update ritual (from now on)

1. Edit the source file.
2. In that folder's `INDEX.md`: bump the file **Ver**, set **Stamp** to today, prepend a changelog line, drop the oldest if over 3.
3. Prepend one line to that folder's **Recent**.
4. Prepend the same line (or a shorter rollup) on each parent `INDEX.md` through root.
5. Run `python3 scripts/audit-agent-index.py`.

## Skip interiors (no `INDEX.md` inside)

| Path | Why | Where it is catalogued |
|------|-----|------------------------|
| `target/` | Rust build | not tracked |
| `node_modules/` | npm vendor | not tracked |
| `.git/` | vcs | not tracked |
| `ui/assets/` | hashed Vite output | `ui/INDEX.md` as `assets/*` |
| `tests/fixtures/vault/` | ingest corpus | `tests/fixtures/INDEX.md` |
| `tests/fixtures/code_repo/` | ingest corpus | `tests/fixtures/INDEX.md` |

Do not add `INDEX.md` under those interiors — connector tests would ingest markdown fixtures.

## Search recipe

1. `INDEX.md` — pick a surface (`src/`, `website/`, `docs/`, …).
2. Folder `INDEX.md` — pick the file row.
3. Open the file. Use `CONCEPTS.md` only for domain words.

## Audit

```bash
python3 scripts/audit-agent-index.py
```

CI job **Agent index** runs the same command. A new tracked file that is not named in its folder index fails the job.
