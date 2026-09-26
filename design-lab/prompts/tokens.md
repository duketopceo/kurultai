# Prompt: design tokens

You are generating the design-token layer for a rebuilt dark-instrument dashboard ("Kurultai" — a knowledge-brain console). Produce `tokens.css` and a short `tokens.md` explaining the scale.

## Hard rules
- Single near-black canvas family (#050508-ish). No light theme, no teal second accent.
- ONE accent: electric purple — #a855f7 (base), #c084fc (strong), #7c3aed (dim), glows as rgba(168,85,247,.14–.42).
- Semantic state colors muted: ok ~#7fd4a8, danger ~#ff6b8a, warn ~#f0b45a. Never use accent purple for success/error.
- Surfaces: glass over black — rgba(10,8,20,.60–.92), 1px borders rgba(168,85,247,.14), hover/focus .40.
- Radius: 6px panels, pill for chips/segmented controls only.
- Type: display face Orbitron for hero numerals/brand; JetBrains Mono for chrome text, HUD micro-labels (uppercase, letter-spaced); readable sans fallback for prose blocks.
- Motion: 120–220ms cubic-bezier(.22,1,.36,1); include a `prefers-reduced-motion` block that zeroes transitions/animations.
- Include: focus-visible ring style; scrollbar styling for dark panels; selection color.

## Output contract
Two fenced code blocks: `tokens.css` (custom properties on :root only — no selectors beyond a few documented utilities) and `tokens.md` (one line per token group). Nothing else.
