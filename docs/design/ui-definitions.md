# UI definitions — what we want

**Date:** 2026-09-26 · **Plan:** `docs/plans/2026-09-25-001-feat-ui-ux-teardown-rebuild-plan.md` (U1)
**Purpose:** this is the acceptance spec for the UI/UX rebuild. Batch-generated design variants are scored against it; the stitched `ui-next` app is verified against it. Nothing here is optional vocabulary — where a definition conflicts with generated output, this doc wins.

---

## 0. Teardown findings (what's actually wrong)

Three findings from auditing `website/src/` — more specific than "wrong direction":

1. **Three competing palettes, not one.** `styles.css` `:root` is dark purple (`--electric* #a855f7`) for the brain surface; a second `--chrome-*` family (`#5eb8a8` teal) styles all chrome; and `:root[data-theme="light"]` flips the whole chrome to lavender/white. `App.tsx` auto-selects `light` when `prefers-color-scheme: light` — so on a light-mode OS the user sees a **black brain canvas wrapped in pastel chrome**, and even in dark mode teal chrome fights the purple cortex. One product, three skins.
2. **Fake data shipping.** `LogsPanel.tsx` renders a hardcoded `SEED` array (invents log lines with timestamps); `SettingsPanel.tsx` "Agent sources" is a hardcoded `<ul>` of four agents shown as `connected`. Both violate the no-fake-claims rule.
3. **Chrome doesn't know what it is.** Mission Control tabs (Pulse/Focus/Synthesize/Ask) are low-signal panels below the hero; CommandRail crams Hey+Repos+Logs+Settings into a narrow rail; the top pill CommandStrip holds search, a timeline scrubber, layout and tier switchers, and a random button — five unrelated control groups in one strip.

## 1. Global design language

| Token | Definition |
|---|---|
| **Canvas** | Near-black everywhere. `#050508` family — the Brain's existing `--bg`. No light theme in v1 of the rebuild; the theme toggle dies (see §10). |
| **One accent** | Electric purple only — `--electric`/`--electric-strong`/`--electric-dim` family (`#a855f7`/`#c084fc`/`#7c3aed`). The teal `--chrome-*` family is deleted, not remapped. Functional states keep semantic colors (ok green, danger red, warn amber) at muted saturation. |
| **Typography** | Two faces: a display face for brand/hero numerals (Orbitron stays — it matches the neural-instrument feel) and JetBrains Mono for data/chrome text. Body UI text in mono is allowed at small sizes only; longer prose (Hey messages, answers) gets a readable sans fallback in the same stack. |
| **Surfaces** | Glass-over-black: `rgba(10,8,20,.6–.8)` panels, `1px` hairline borders at `rgba(168,85,247,.14)`, stronger on hover/focus (`.40`). Border radius: `6px` panels, `pill` for chips/toggles only. |
| **Motion** | Fast and short: `120–220ms`, `cubic-bezier(.22,1,.36,1)` (existing `--chrome-ease`, keep). ThreeUI-style play = hover lifts, focus glows, subtle shimmer — never layout-shifting, never animated for animation's sake. `prefers-reduced-motion` collapses all of it. |
| **HUD readouts** | Stats/status render as mono uppercase micro-labels + numeric readouts (FPS, atoms, tier, daemon version) — the instrument-cluster feel already present in the stats chip, extended consistently. |
| **Grid texture** | The existing faint radial/grid backdrop (`--bg-grid`) stays, subtle, behind the hero only — not tiled across every panel. |

## 2. Information architecture

The page is one instrument, top to bottom:

```
TopBar          brand · nav (Brain / Repos β / Store β) · daemon status · access
CommandStrip    search (⌘K) · tier (low/mid/high/max) · layout (brain/ontology)
                — and nothing else; timeline + random move out (see §4)
BrainStage      the cortex hero — untouched visually
Inspector       floating, on node select — kept, restyled
WorkSurface     the below-brain area: tabs → Hey / Pulse / Ontology / Ask / Store
RightRail       dies — its content redistributes (see §7)
Footer          single line
```

Two levels, not five: the Brain is the instrument; everything else is its console. Vertical scroll, no competing side rails.

## 3. Surface definitions

### 3.1 TopBar — keep + restyle
- **Purpose:** brand + primary nav + daemon liveness.
- **Must:** `KURULTAI` + version mark; nav links to Brain/Repos/Store; status dot + `online · api <ver>`; never "LOCAL BRAIN" copy on hosted builds.
- **Visual:** full-width hairline-underlined bar, mono nav, purple active-state underline.
- **Dies:** nothing structural; `BRAIN` subtitle becomes `—` nothing (brand stays `KURULTAI`, drop the second tag).

### 3.2 CommandStrip — rebuild (slim)
- **Purpose:** the three controls the Brain actually needs: search, memory tier, layout.
- **Must:** ⌘K search w/ dropdown + **clear control** (existing rule); tier segmented control showing `loaded/total` when partial; brain/ontology toggle.
- **Dies:** the timeline "memory horizon" scrubber (a live-play gimmick that re-filters the graph invisibly — move to a future "time" feature if ever wanted, not a top-strip control), the `random` button (move into an inspector/overflow affordance or cut).
- **Visual:** single slim bar attached under TopBar, not a floating pill.

### 3.3 BrainStage — keep visuals, replace chrome
- The canvas, camera-open behavior, sprite/mesh paths, hover-trace, hull constraint: **untouched** (protected by AGENTS.md).
- The floating caption line ("3,516 neurons · max · hover to trace") stays but styled to the HUD language.
- No new buttons or overlays on the canvas.

### 3.4 Inspector — keep + restyle
- **Purpose:** node detail + promote-to-ontology actions.
- **Must:** floating panel, `Esc` + `×` close, atom fields, ontology links, promote flow, open-source action.
- **Visual:** right-anchored glass card over the hero; mono field labels; busy/error states stay explicit.

### 3.5 WorkSurface (below-brain) — rebuild
Replaces both MissionControl and CommandRail. One tabbed console:

| Tab | Contains | From |
|---|---|---|
| **Hey** | threads list + chat + kanban toggle + presence | HeyPanel/Chatboard/HeyKanban |
| **Pulse** | live memory stream + brain-state stats | ActivityPanel + StatsPanel merged — they answer the same question ("what's happening") |
| **Ontology** | class tree summary + proposals queue | OntologyBoard overview + ProposalsPanel |
| **Ask** | question → answer | AskPanel (rename tab `Synthesize`→`Ask`, drop the unused `ask` proposals slot confusion — today 'ask' tab renders ProposalsPanel, mislabeled) |
| **Store** | link/mini-view into `#/db` | DbView stays its own route |

- **Full-width**, not a rail; tabs are segmented-control style, mono labels, purple active pill.
- Empty states are honest and actionable (existing "kurultai init --docs" pattern is the model).

### 3.6 Hey surface — keep behavior, restyle
- Threads list + Chatboard (post/reply/react/edit/delete/whole-thread overlay) + kanban toggle + presence strip — all current behavior preserved (R4).
- **Visual:** two-pane thread list / conversation inside the Hey tab; monospace agent names `codename@instance_id`; purple unread dots; kanban columns on dark glass.
- **Mobile:** collapses to single pane with back-navigation (current rail behavior is already broken-ish; fix in rebuild).

### 3.7 Pulse surface — merge
- Activity stream (live toggle) + Brain state stats (memories, tiers, trusted) as one panel: stats row on top (HUD readouts), stream below.
- Kill the duplicated daemon-status (lives in TopBar once).

### 3.8 Logs / Settings — redistribute
- **Logs:** the fake `SEED` panel dies entirely. A real logs surface returns only when backed by `/api` data — **do not ship a stub**. (Settings' "Show structured logs" checkbox dies with it.)
- **Settings:** theme toggle dies (single dark theme). "Agent sources" hardcoded list dies — replaced by real presence data or removed. Layout/tier controls already live in CommandStrip — Settings becomes: access/human-login management + daemon info + (real) preferences as they exist.

### 3.9 Secondary routes — keep, restyle
- `#/repos` (RepoBrain), `#/db` (DbView), HumanAccess gate — all survive with the new tokens; no behavior change.

## 4. States (every surface must define)

| State | Requirement |
|---|---|
| Loading | Skeleton or mono "connecting…", never blank panels |
| Empty | Honest + one actionable hint (no fake seed content — hard rule) |
| Error | Mono error line + retry affordance where retryable |
| Offline | TopBar status flips red; surfaces keep last-good data (existing ProposalsPanel pattern) |

## 5. Responsive

- ≥1280px: full layout as §2.
- 768–1280px: CommandStrip wraps to two rows; WorkSurface tabs scroll horizontally.
- <768px: top-to-bottom single column; Hey two-pane collapses; `scrollWidth === viewport` (the 390px probe from #357 stays the check).

## 6. Accessibility floor

- Tab roles/`aria-selected` on all tab systems (existing pattern holds).
- Focus-visible rings in purple, never removed.
- `prefers-reduced-motion` → no shimmer/lift.
- Contrast: text ≥4.5:1 on panels; mono micro-labels ≥3:1.

## 7. What dies (explicit)

| Component/element | Fate |
|---|---|
| `--chrome-*` teal token family | deleted |
| `[data-theme="light"]` + theme toggle + auto light pick | deleted — single dark theme |
| `LogsPanel` fake SEED data | deleted (returns only if API-backed) |
| `SettingsPanel` hardcoded agent list | deleted (real presence or nothing) |
| Timeline "memory horizon" scrubber + play | removed from strip |
| `random` button | removed from strip (optional inspector overflow) |
| `CommandRail` as a layout element | dissolved into WorkSurface |
| Mission Control 'ask' tab mislabel | fixed (ProposalsPanel moves to Ontology tab) |
| `BRAIN` subtitle in brand | dropped |

## 8. Non-negotiables (carried from AGENTS.md)

- Brain: deep black, white/purple only, shimmer not orbs, hover-trace, whole-graph open, no extra chrome, nodes inside hull.
- Ontology: typed 2D board only — never a 3D organic cloud.
- "UI fun, not literal": ThreeUI-style CSS/motion in chrome; literal ontology only in cortex + inspector.
- Hosted: no "LOCAL BRAIN" branding; Cloudflare-Access-friendly, password-manager-friendly.
- Tiered loading + `perf.ts` telemetry preserved.
