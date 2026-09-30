# Prompt: Brain hero frame (chrome only — NOT the 3D)

Design the frame that wraps the Kurultai cortex canvas. React + CSS on the electric-purple token layer. **The 3D canvas itself is out of scope** — you are designing the chrome around it: the hero container, caption line, and the stats chip.

## Contract

- **Hero container**: full-width section, the canvas fills it (a `<canvas>` placeholder div is fine — style the wrapper, its hairline border treatment or lack thereof, and how it sits between CommandStrip and the console). Deep black bg — the brain IS the visual.
- **Caption line**: bottom-left overlay on the hero — mono micro-copy like `3,516 neurons · max · hover to trace connections`. Dynamic count prop.
- **Stats chip**: top-right overlay — a compact HUD readout: `fps 42 · nodes 3,516 · syn 8.2k`. Mono, letter-spaced, glass chip, hairline border. Values come as props.
- **Layout mode badge**: when layout === 'ontology', a tiny `ONTOLOGY` badge appears near the caption.
- **Empty brain state**: when atoms = 0, the hero shows a centered mono empty-state (`no memories yet — kurultai init --docs`), NOT a broken canvas.

## Hard rules
- Zero buttons on the canvas area itself — no zoom controls, no settings gear, no tooltips chrome. The brain gets exactly: caption + stats chip + ontology badge.
- Overlays must not intercept pointer events over the canvas except their own tiny hit areas (`pointer-events: none` on wrappers, `auto` on the chip).
- Hairline at most: the hero reads as infinite black space, not a card with a thick frame.

## Output contract
`brain-hero.tsx` + `brain-hero.css`. Props: `{ atomCount, atomTotal, tier, layout, fps, synapses, empty }`. No three.js imports — the canvas is a styled mount point.
