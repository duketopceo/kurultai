# Prompt: floating Inspector panel

Design the node Inspector for the Kurultai Brain — a floating glass panel that appears when a memory node is selected/hovered on the cortex canvas. React + CSS on the electric-purple token layer.

## Content (all real fields — show each with a mono micro-label)
- Title, summary (prose, readable sans)
- Tier (hot/warm/cold — styled tier pill, purple family differentiation)
- Score, source, tags, indexed-at
- Ontology links: linked classes/instances as chips
- Actions: `Promote to ontology` (opens a small class-picker inline, busy spinner while posting, success/error line), `Open source` (link button)
- Close: `×` top-right AND Esc support

## Requirements
- Positioned right-of-canvas, overlapping the hero — glass, hairline border, ~340px wide, max-height with internal scroll.
- Enrich state: shows a `loading detail…` state while full atom detail fetches.
- Honest errors: promote failure shows the error inline, recoverable.
- ≤768px: slides up as a bottom sheet instead of floating right.
- No fake content — every field comes from props; absent fields render `—`.

## Output contract
`inspector.tsx` + `inspector.css` — typed `Atom`/`OntologyLink` props, callbacks for promote/open/close. Self-contained, no deps beyond React.
