# feat: Brain memories solar-system layout toggle

**Date:** 2026-07-29  
**Branch:** `cursor/memories-solar-system-7a74`  
**Origin:** User — toggle memories into a solar system; same hover/click/labels; click solar and watch it flow.  
**Scope note:** This is an **explicit user-requested Brain UI feature**, not a Phase 1 / phase-closeout change. Product behavior in `ui/` is intentional and approved for this plan.

## Goal

Add a thin **Lattice ↔ Solar** toggle on the Brain UI. Default stays the Fibonacci lattice. Choosing **Solar** animates nodes into a solar-system arrangement (sun + orbital rings). Hover, click, tooltip, and inspector behavior stay identical.

## Settled decisions

| Decision | Choice |
|----------|--------|
| Layout modes | `lattice` (default) · `solar` |
| Sun | Highest-score visible atom (stable tie → first) |
| Orbits | One ring per `source`, radius capped so whole system stays in the existing camera frustum |
| Motion | Lerp flow on toggle (~0.85s); gentle Y-spin only in solar (respect `prefers-reduced-motion`) |
| Chrome | One `quiet-button` in the command strip (same family as stream live/paused) |
| Persist | `localStorage` key `kurultai-layout` |
| AGENTS.md | Explicit user request overrides “no circle/brain-shape layout modes” for this solar mode only |

## Non-goals

- Force-directed physics / `3d-force-graph`
- New dashboard chrome clusters, MCP callouts, or parallel brain under `website/`/`web/`
- Changing color language (still black / white / purple)
- Phase-closeout / agent-initialization-only work (out of scope for this plan)

## Files

- `ui/brain.html` — layout toggle control
- `ui/brain.js` — positions, transition, solar spin, wiring
- `ui/index.css` — command-strip column for the toggle
- `docs/plans/2026-07-29-002-feat-brain-solar-layout-plan.md` — this plan

## Verification

### Manual UI

- Toggle lattice → solar: nodes animate into orbits; sun centered
- Toggle back: animate to lattice
- Hover / click / tooltip / inspector unchanged in both modes
- Timeline / search still refresh current layout
- `prefers-reduced-motion`: skip spin; keep short or instant reposition
- Manual: Vite `ui` preview or daemon `/ui/`
- Persisted `kurultai-layout=solar`: toggle shows destination label `lattice` and `aria-pressed=true` after init

### Repository gates

- README links and ✅ status checks still valid for touched docs
- `cargo test --locked`
- `cargo clippy --all-targets -- -D warnings`
- Shell scripts (if any touched): `bash -n` syntax check
