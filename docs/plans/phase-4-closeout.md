# Phase 4 closeout — tracker hygiene

**Product Phase 4 solo expansion is shipped** (#62 Dayflow+Pond, #63 GitHub FS).  
This file is **issue/milestone hygiene** so Milestone 4 can close.

Agent tokens **cannot** call `closeIssue` (403). A maintainer runs the commands below.

---

## Close (shipped / umbrella done)

After the closeout PR is on `main`:

```bash
./scripts/phase-4-closeout.sh
```

The script preflights that #62/#63 are merged into `main`, that wrap docs exist on `origin/main`, then closes [#8](https://github.com/duketopceo/kurultai/issues/8) with the full deferred list from [phase-4-complete.md](phase-4-complete.md).

---

## Already closed

- [#21](https://github.com/duketopceo/kurultai/issues/21) Dayflow — closed via #62

---

## Leave open (deferred / other milestones)

- [#4](https://github.com/duketopceo/kurultai/issues/4) AppFlowy — deferred Expansion leftover (still on Milestone 1 historically; remilestone optional)
- [#14](https://github.com/duketopceo/kurultai/issues/14) Plugin ecosystem — later
- [#23](https://github.com/duketopceo/kurultai/issues/23) Testing gates — cross-cutting; tick Phase 4 items when coverage/deny land

---

## Milestone 4

When Milestone 4 has no blocking open issues (#8 closed):

```bash
gh api -X PATCH repos/duketopceo/kurultai/milestones/4 -f state=closed
```

---

## Done when

1. Phase 4 connectors on `main` (#62, #63)  
2. Verification on the closeout landing commit (record results):  
   - `cargo test --locked`  
   - `cargo clippy --all-targets -- -D warnings`  
   - required files exist: `docs/plans/phase-4-complete.md`, `docs/plans/phase-4-closeout.md`, `scripts/phase-4-closeout.sh`  
   - README contains a Markdown link to `phase-4-complete.md`  
3. `./scripts/phase-4-closeout.sh` succeeds (#8 closed)  
4. Milestone 4 closable  
