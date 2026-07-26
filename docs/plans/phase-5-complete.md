# Phase 5 complete — wrap-up

**Status:** ✅ Product production-readiness on `main` (2026-07-25–26)  
**Plans:** [daemon poll](2026-07-25-003-feat-phase-5-daemon-poll-plan.md) · [notify watch](2026-07-25-004-feat-phase-5-notify-watch-plan.md) · [local embeddings](2026-07-25-009-feat-phase-5-local-embeddings-plan.md) · [multi-agent MCP init](2026-07-25-010-feat-phase5-multi-agent-mcp-init-plan.md) · [LFG closeout](2026-07-26-002-chore-phase5-lfg-closeout-plan.md)  
**Tracking:** [#9](https://github.com/duketopceo/kurultai/issues/9) (closed early; product exit via PRs below)  
**Exit path:**

```text
configured sources → daemon HTTP + poll (+ notify watch)
  → IndexPipeline incremental index → SQLite brain
  → FTS / optional local ONNX or OpenRouter vectors → search / ask / MCP
```

---

## What shipped

| Work order | PR | Notes |
|------------|-----|--------|
| Daemon background poll | [#65](https://github.com/duketopceo/kurultai/pull/65) | Immediate first `index_all(false)`; `--no-poll`; soft-fail |
| Notify filesystem watch | [#66](https://github.com/duketopceo/kurultai/pull/66) | Debounced re-index; `--no-watch`; markdown/github roots |
| Local ONNX embeddings | [#84](https://github.com/duketopceo/kurultai/pull/84) | Opt-in `fastembed` / `LocalEmbedder`; FTS-first still default |
| Multi-agent MCP init | [#83](https://github.com/duketopceo/kurultai/pull/83) | `init --agent` wires Claude Code + Codex |

**Product audience exit:** shared localhost daemon keeps the brain fresh (poll + watch); optional offline vectors without cloud keys; agents connect via MCP init.

---

## Deferred (not Phase 5 product exit)

| Item | Tracker | Notes |
|------|---------|--------|
| Self-hosted CI (ARC + ephemeral runners) | [#20](https://github.com/duketopceo/kurultai/issues/20) | Ops/infra; remilestone to Milestone 6 |
| Environments deploy hardening | [#29](https://github.com/duketopceo/kurultai/issues/29) | Foundation shipped [#30](https://github.com/duketopceo/kurultai/pull/30); gates/secrets/RBAC remain |
| GlitchTip `kurultai-{dev,staging,prod}` | [#35](https://github.com/duketopceo/kurultai/issues/35) | Error tracking wiring; remilestone to Milestone 6 |

---

## Exit criteria — verified

1. `kurultai daemon` serves HTTP and polls incrementally (default on; `--no-poll` disables)  
2. Notify watch debounces FS changes for markdown/github roots (`--no-watch` disables)  
3. Local embeddings selectable via config; NullEmbedder / FTS-only remains the no-key default  
4. `kurultai init --agent` can wire Claude Code and Codex MCP  
5. README roadmap marks Phase 5 product exit ✅  

---

## Hardened invariants (do not regress)

| Invariant | Where |
|-----------|--------|
| Poll soft-fails; HTTP stays up | `daemon` poll loop |
| Watch soft-fails; debounce + single-flight with poll | `daemon` watch |
| FTS-first without API key (NullEmbedder) | `embed` / bootstrap |
| Local embed opt-in; dim mismatch fails fast | `LocalEmbedder` / `ensure_vec_table` |
| MCP never dumps full `content` by default | `AgentAtomView` |

---

## Tracker closeout

Maintainer: [phase-5-closeout.md](phase-5-closeout.md) / `scripts/phase-5-closeout.sh`  
(Agent token often lacks issue/milestone write.)

---

## Next: Phase 6

1. [#10](https://github.com/duketopceo/kurultai/issues/10) open source launch & community  
2. Deferred ops from this phase: [#20](https://github.com/duketopceo/kurultai/issues/20) ARC · [#29](https://github.com/duketopceo/kurultai/issues/29) env hardening · [#35](https://github.com/duketopceo/kurultai/issues/35) GlitchTip  
3. Optional: Expansion leftovers (Composio / plugins) already named in [phase-4-complete.md](phase-4-complete.md)  
