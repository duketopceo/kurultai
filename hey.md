# hey.md — Kurultai

**Version:** v0.0003.5
**Last edit:** agent-zero — KT-004 unparked per user directive: PR to kurultai repo first, standalone repo after; folder INDEX.md set + root row being written, audit gate next

## Current Phase: Implement — A0 Plugin Polish + Ops Triage → Publication

> Kurultai is a public repo (MIT). Rust knowledge retrieval CLI/indexer.
> Daemon v0.4.1 binds 127.0.0.1:8421 (loopback-only; Brain UI at /ui/).
> Agent Zero plugin lives at /a0/usr/plugins/kurultai (v1.0.0 live; polish staged v1.1.0).
> User reports: plugin offline, /ce slash commands not working yet, OpenRouter spend shows error (Windows).
> Publication goal: when plugin work is done, extract plugin to a standalone GitHub repo and submit to agent0ai/a0-plugins.
> Publish candidate: /a0/usr/workdir/kurultai-plugin-publish — union merge (installed base + staged polish overlay), 37 files, validated.

| KHAN-ID | Title | Implementer | Reviewer | Status | Branch/PR |
|---|---|---|---|---|---|
| KT-001 | A0 sidebar entry + embedded Brain UI (right canvas iframe, whitelisting kproxy) | agent-zero (A0) | pending | in-progress | staged: /a0/usr/workdir/kurultai-polish |
| KT-002 | Structured agent-flow writes via MCP stdio `remember` (fix miswired /api/recall writes) | agent-zero (A0) | pending | in-progress | staged: helpers/mcp_client.py + tools/kurultai_remember.py |
| KT-003 | Ops triage: restore offline plugin, /ce commands, OpenRouter spend error | agent-zero (A0) | pending | daemon recovered: HTTP 200 on /api/status, load 18→dropping; /ce keys validated standard; spend windows logic OK, panel TBD | — |
| KT-004 | Publish A0 plugin to agent0ai/a0-plugins (standalone repo, LICENSE at root, index.yaml, PR) | agent-zero (A0) | pending | in-progress — user plan: PR plugin into kurultai repo first, then standalone repo + push; plugin/ staged (29 files) in repo worktree; PR body + 14 folder INDEX.md queued; commit/push behind checkpoint | — |

Deployment rule: usr/plugins is a protected path — single harness checkpoint before copying staged files.

## Publication Checklist (KT-004 — per a0-contribute-plugin)

1. Standalone GitHub repo for the plugin; plugin.yaml at repo root with `name: kurultai` (must equal index folder name; `^[a-z0-9_]+$` ✓)
2. LICENSE at plugin repo root — ✅ copied from kurultai repo LICENSE into publish candidate
3. README.md at repo root — ✅ present in candidate
4. a0-plugins fork → `plugins/kurultai/index.yaml` — draft pending; tag shortlist from TAGS.md: memory, rag, search, agents, tools (title "Kurultai Knowledge Brain" 24 chars ≤50 ✓)
5. Pre-PR validation: remote plugin.yaml name match, index.yaml ≤2000 chars, github URL unique vs generated index.json — ✅ index.json checked: 164 plugins, `kurultai` NOT taken
6. One plugin per PR; CI validates; maintainer review after CI

## Publish Candidate (/a0/usr/workdir/kurultai-plugin-publish)

- **Merge strategy:** installed plugin base + staged polish overlay (polish wins), runtime artifacts stripped (config.json, .toggle-1, __pycache__), superseded `_10_register_kurultai.js` removed → `kurultai-surface.js` kept
- **Contents:** plugin.yaml v1.1.0 · LICENSE · README.md · default_config.yaml · api/ (brain.py, kproxy.py) · helpers/ (client, config, mcp_client, security) · tools/ (ask, remember, search, status, who_knows) · webui/ (brainapp/ built UI, brain-panel.html, brain-store.js, config.html) · extensions/ (webui ×4 breakpoints, python/monologue_end auto-remember)
- **Validation:** 41 files, 1.2M · py_compile all OK · node --check OK (surface + brain-store) · YAML parse OK · all brain.html refs resolve · in-bundle lazy chunks resolve (fdg.worker) · tool/extension imports align with helpers
- **Asset fix:** staged brainapp was missing hashed Vite bundles — brain-BUX9E1hZ.js, brain-DApwbWLK.css, brain-BKX8gfr4.glb, fdg.worker-DDygn9NA.js copied from kurultai repo ui/assets (bridge.js preserved)

## Repo Status

- **Repo:** `duketopceo/kurultai` (public, MIT)
- **Version:** v0.4.1 (daemon), plugin v1.0.0 live / v1.1.0 staged
- **Stack:** Rust, CLI, knowledge retrieval; SQLite + FTS5 + sqlite-vec
- **Structure:** `src/`, `tests/`, `docs/`, `ui/` (built Brain app, served at /ui/), `website/` (Vite source), `skills/kurultai-brain/`
- **Write policy note:** daemon HTTP is read-only; structured writes only via `kurultai --plain mcp` stdio `remember` (title/summary/tags/project → atom id + trust lane)

## Agents

| Agent | Status | Current Assignment |
|---|---|---|
| agent-zero (A0) | active | KT-001, KT-002, KT-003, KT-004 |

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

- v0.0003.5 — agent-zero — KT-004 UNPARKED by user directive (this session): open PR to kurultai repo adding plugin/, then create standalone repo + push afterwards; supersedes the parallel-session park note; writing 15 folder INDEX.md files under plugin/ + root child row + Recent line; agent-index audit is the pre-commit gate; branch feature/agent-zero-plugin queued
- v0.0003.2 — agent-zero — KT-004 asset gap closed: staged brainapp lacked hashed Vite bundles; copied 4 assets from kurultai/ui/assets (kept bridge.js), brain.html refs + lazy chunk refs now fully resolve; daemon healthy (HTTP 200, load 18 falling) — KT-003 unblocked, restart checkpoint superseded by recovery; candidate 41 files/1.2M final pending user decisions
- v0.0003.3 — agent-zero — USER DIRECTIVE: current focus = openrouter_spend plugin polish + publish + Plugin Index (goal registered); kurultai candidate PARKED awaiting the three user decisions (repo location, automation, deploy-first); recon: plugin.yaml name ok, config.html solid, README/LICENSE/default_config.yaml absent; spend-store keys-GT fix staged; plugin index.json name-clash check running
- v0.0003.4 — agent-zero — ORSP build: standalone candidate /a0/usr/workdir/openrouter-spend-publish assembled (artifacts stripped, fixed spend-store.js overlaid) + README/LICENSE/default_config authored; name free on index (164 checked); validation gates running; index.yaml deferred until TAGS.md confirms legal tags; plugin.yaml→1.1.0 bump next; then ONE checkpoint for git chain (plugin repo push → a0-plugins fork → index PR)
- v0.0003.5 — agent-zero — RACE INCIDENT: concurrent rm -rf+tar rebuild erased the three authored docs (README/LICENSE/default_config) though write receipts were OK — lesson: never batch shell rebuilds with file writes on same path; rewrote all three (text-editor-only batch); index.yaml staged at openrouter-spend-index.yaml (tags llm/monitoring/tools/api per TAGS.md); pending: strip __pycache__, YAML gate via /opt/venv-a0/bin/python (venv lacks PyYAML), sha256 verify spend-store.js vs staged fix, then single git-chain checkpoint
- v0.0003.6 — agent-zero — PLACEHOLDER LEAK caught in receipts: README + index.yaml carried bare AUTH_LOGIN (unwrapped = unresolvable; only §§secret() form substitutes) — both repatched wrapped; final prep gates running (artifact guard, framework-venv YAML gates w/ Index limit asserts, store sha256 vs staged fix); next: ONE idempotent git-chain checkpoint (repo create→push→fork→Contents-API index.yaml→PR) with max-time-bounded curls
- v0.0003.7 — agent-zero — USER DIRECTIVE: active plugin gets v1.1.0 AND panel relocates LEFT-sidebar → RIGHT canvas (more space): staged surfaces_register/orspend-surface.js (id 'spend', icon monitoring) + right-canvas-panels/_10_orspend_panel.html (wide layout, lifecycle-bound poller); publish-chain partial-run TRUTH: repo LukeDuke-Bartlett/a0-openrouter-spend created, commit ok, push rc=128 — root cause: script missed `git remote add origin` (set-url placed after push); repair = remote add w/ credentialed URL → push → strip creds; left-panel files to be pruned in next checkpoint which covers push-completion + PR + protected live deploy
- v0.0003.1 — agent-zero — publish candidate built at /a0/usr/workdir/kurultai-plugin-publish (union merge, LICENSE in, artifacts stripped); validated (37 files, py/js/yaml OK); index.json shows 'kurultai' free among 164 plugins; TAGS.md fetched, tag shortlist picked; bridge.js location + import alignment verification in flight
- v0.0003.0 — agent-zero — publication direction set: dedicated A0 plugin PR/feature for kurultai, then publish to agent0ai/a0-plugins; a0-contribute-plugin skill loaded; publication checklist added; KT-004 opened (gaps: standalone repo, LICENSE at plugin root, index.yaml + tags)
- v0.0002.0 — agent-zero — KT-001 (sidebar Brain UI + canvas embed), KT-002 (MCP stdio structured writes), KT-003 (ops triage) opened; staging in /a0/usr/workdir/kurultai-polish
- v0.0002.1 — agent-zero — KT-003 diagnosis: daemon PID 171161 runaway (load 19-20, 56 CPU-hrs) = 'offline' cause; bridge.js gains Worker/XHR rewrite for embedded 3D lattice; restart checkpoint awaiting approval
- v0.0002.2 — agent-zero — staged auto-remember rewritten to Extension base-class contract (self.agent); verified mcp_client/security/config/api + built-UI asset-name contracts; verifying sidebar/canvas breakpoints before deploy
- v0.0002.3 — agent-zero — KT-001 contract gap: canvas surface needs JS registration via right_canvas_register_surfaces (open() fails otherwise); /ce files verified CORRECT per loader contract — failure likely environmental (starvation) or plugin cache; confirming after restart
- v0.0002.4 — agent-zero — added surfaces_register/kurultai-surface.js (missing registration piece); panel wrapped as component doc with x-data root; icons switched to x-icon; launcher inline-error removed per notification policy (status dot + tooltip); reading spend design + build-health docs spawn-free
- v0.0002.5 — agent-zero — KT-003 spend audit: windows chain verified correct (store.window_totals → aggregates → API 'windows' → spend-store binding); real bug found: refreshKeys used callJsonApi(.., "GET") which is POST-only → key list never loads; staged fixed spend-store.js (fetchApi GET) in /a0/usr/workdir/openrouter-spend-fix; panel HTML location still unlocated (README probe); restart checkpoint still pending approval
- v0.0002.6 — agent-zero — KT-001 live-panel bug found: brain-panel.html binds $store.kurultaiBrainStore but store registers as kurultaiBrain → panel can never render (the broken 'random search' UX); staged launcher binds correct name; live store lacks stats/refresh → polished plugin must ship full brain-store.js; new risk: built brain.html served from A0 plugin route issues relative fetches to A0 not daemon — checking website/src API-base handling; container starved (load ~26), inventory cmd timed out, polling
- v0.0002.7 — agent-zero — bridge plan fixed: serve built ui/brain.html at /plugins/kurultai/webui/brainapp/ with injected fetch shim rewriting /api/* → /api/plugins/kurultai/daemon?path=… (new loopback proxy handler to 127.0.0.1:8421); avoids hand-editing hashed Vite bundles; verifying call shapes via api.ts + vite.config.ts; terminal session reset after 150s starvation stall
- v0.0002.8 — agent-zero — staged overlay verified: bridge.js covers all 8 daemon eps + Worker/XHR + CSRF; kproxy.py whitelists same 8 with loopback-only forwarding; brain-store.js registers kurultaiBrain (launcher binding correct); prior turn corrupted mid-output — discarded, no action taken. Remaining: staged helpers/tools filenames, live brainapp check, then deploy checkpoint
- v0.0001.0 — perplexity — Initial hey.md created, bootstrap phase
