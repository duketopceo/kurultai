# Kurultai design direction: "Council Ring"

Status: proposal (2026-10-10). Owner signs off the direction, palette and logo. Supersedes nothing yet: the shipped UI still follows the dark-purple language in `docs/design/ui-definitions.md`. See "Decision for the owner" below.

## Surfaces this repo has

| Surface | Where | Notes |
|---|---|---|
| CLI and `kurultai status` output | `src/main.rs` | plain text, no colour requirement |
| MCP tool descriptions | `src/mcp/` | text only; the "UI" agents see |
| Brain UI (embedded) | `website/` built to `ui/`, served at `/ui/` | Vite, React, Three.js; `ui-next` is the default |
| Team web app | `web/` | Next.js + Clerk scaffold |
| Docs and README | repo root, `docs/` | GitHub renders light and dark |
| Launch video, social card | `launch/`, `assets/brand/social-card.svg` | |

There is no bar widget, TUI or plugin panel in this repo.

## Concept

A kurultai is a council: many voices, one floor. The visual language is a night council fire seen from inside a ger: near-black canvas, warm paper-white ink, and one ember accent used the way a fire is used in a dark room, rarely and for the thing you should look at. The identity mark is the yurt crown ring (see `assets/brand/README.md`).

## Research and reference lock (Refero MCP was not available; method followed with in-repo and public references)

Reviewed: `docs/design/bases/linear.DESIGN.md` (vendored; near-black canvas, hairline panels, one chromatic accent), `docs/design/research-bases.md` (ThreeUI, VoltOps, bolt.new tokens), the current `ui/favicon.svg` and `website/src/next/` palette (dark purple `#0d0b18` + `#a855f7`), and the visual treatment of Letta's, Mem0's and basic-memory's public sites as category peers (all use light canvases with blue or purple accents, so a warm-ember dark identity is unclaimed in this category).

- Primary direction: Linear-style single-accent dark product canvas (already the repo's design law).
- Preserve: near-black canvas; hairline 1 px borders instead of shadows; one accent used only for focus, primary action and the live/active state; mono micro-labels; honest empty and error states (rubric from `design-lab/README.md`).
- Borrow only: warm off-white ink (not pure white) from editorial reading UIs, so long citations read comfortably; ring-and-spoke geometry for glyphs.
- Role rules: accent is never a background fill for panels, never decorative gradients; status colours are separate from the accent.
- Reject: purple/indigo as the brand colour (it is the category default and the owner has not tied the brand to it), glassmorphism blur, cream-and-terracotta "calm editorial", gradient hero.

Decision for the owner: moving the accent from purple `#a855f7` to ember `#e8a33d` touches `website/src/next/` tokens and the 3D brain node colours. It is a token swap, not a redesign. Keep purple if you prefer; the rest of this document still holds with the accent value replaced.

## Decision ledger

| Decision | Source | Role rule | Why |
|---|---|---|---|
| Near-black canvas `#0c0d11` | linear.DESIGN.md, ui-definitions.md | surfaces only | already the repo's design law; suits a long-running dashboard |
| One accent, ember | reference lock; category gap | focus, primary CTA, live state only | name and mark (fire under the crown ring) |
| Warm ink `#ece8df` | craft rule (long-form reading) | text only | citations and excerpts are the product |
| Hairline borders, no shadow | linear base | panels, inputs | matches ui-definitions "hairline glass" without blur |
| Mono micro-labels | ui-definitions.md rubric | metadata, ids, timestamps | atoms have ids, lanes, scopes |
| Stroked geometric glyphs | mark construction | icons | one stroke weight, ring and spoke motifs |

## Tokens

Contrast ratios computed with the WCAG 2.x relative-luminance formula.

| Token | Dark | Light | Notes |
|---|---|---|---|
| `--bg` | `#0c0d11` | `#f6f4ee` | |
| `--panel` | `#15171d` | `#ffffff` | |
| `--border` | `#262932` | `#dcd8cc` | decorative, not relied on for meaning |
| `--ink` | `#ece8df` | `#16171c` | 15.88:1 / 16.27:1 on bg |
| `--ink-muted` | `#9a968c` | `#5c594f` | 6.58:1 / 6.37:1 on bg; 6.07:1 on dark panel |
| `--accent` | `#e8a33d` | `#9a5b00` | 9.01:1 on dark bg, 8.31:1 on dark panel; 4.93:1 on light bg, 5.43:1 on white |
| `--on-accent` | `#0c0d11` | `#ffffff` | button label on accent: 9.01:1 dark |
| `--ok` | `#6fcf97` | `#1f7a4a` | 10.22:1 / 4.84:1 |
| `--err` | `#f2786b` | `#b3382c` | 7.10:1 / 5.43:1 |

All text pairs above meet WCAG AA (4.5:1). Still to verify on the real UI: `--on-accent` on the light accent (white on `#9a5b00`), and focus rings against `--panel`.

Type: UI sans = system stack (`ui-sans-serif, Inter, system-ui`), 14/20 body, 12/16 labels; mono = `ui-monospace, "JetBrains Mono"` for ids, scopes, lanes, timestamps and all citations. Display only in the website hero, 600 weight, -0.02em tracking. No webfont is required to ship.

Space and shape: 4 px grid, panel radius 8, control radius 6, 1 px borders.

## Glyph style

Stroked, 1.5 px at 16 px, round caps, built from the mark's ring and spokes: ring = a source or scope, X-spokes = convergence (ingest), dot = the answer or active atom. Trust lanes: `trusted` = filled dot, `quarantine` = dashed ring. Scopes: `personal` one ring, `team` two linked rings, `company` three. No stock icon packs mixed in.

## Motion

- Purpose only: motion shows state change (atom ingested, answer streaming, node focus). No idle animation except the live dot.
- 120 to 200 ms, ease-out; Brain camera moves up to 600 ms.
- The live indicator pulses opacity 1 to 0.5 over 2 s.
- `prefers-reduced-motion`: disable pulse and camera easing, jump to the end state (already a rubric item in `design-lab/README.md`).

## Voice

Plain and exact: say what the brain does not know (the gap report is a product feature). Lanes and scopes are shown, never hidden. Empty states say what to run (`kurultai index --full`), never "Nothing here yet!".

## Not yet done

- No UI code changes in this pass; token swap is a follow-up roadmap unit (U22 in the overlay plan).
- Rendered UI has not been compared against these tokens.
