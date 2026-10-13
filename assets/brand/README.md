# Kurultai brand assets (PROPOSAL, awaiting owner sign-off)

Hand-written SVG. No AI image generation, no raster sources, no stock icons.

## Mark

A yurt crown ring (the *tono*): the smoke-ring at the top of a ger, seen from below, where the roof poles meet. It matches the name (an assembly gathered under one roof) and the product (many sources converging on one point).

Construction on a 32 x 32 grid:

- Ring: circle, centre (16,16), r 12.5, stroke 3.
- Poles: two diagonals from (7.2,7.2) to (24.8,24.8) and back, stroke 2.5, round caps. They meet at the centre.
- Hub: a background-coloured knockout circle r 5.6 cuts the poles, then an accent dot r 3.4 sits in it. The accent is the only colour in the mark.

At 16 px (`preview-mark-16.png`, `preview-mark-16-light.png`) the ring, the X and the dot all survive.

## Wordmark

Monoline capitals drawn as stroked paths (3.2 stroke, round caps and joins), 28 units tall, 10 units between letters. No font is needed, so it renders the same everywhere. The mark sits to the left at 1:1.

## Files

| File | Use |
|------|-----|
| `logo.svg` | mark for light backgrounds |
| `logo-dark.svg` | mark for dark backgrounds |
| `wordmark.svg`, `wordmark-dark.svg` | mark + name, light / dark |
| `social-card.svg` | 1200 x 630 social card (no text elements, so no font dependency) |
| `preview-*.png` | resvg renders, oxipng optimised, for review only |

Colours (from `DESIGN.md`): ink `#16171c` / `#ece8df`, accent ember `#9a5b00` (light) / `#e8a33d` (dark), canvas `#f6f4ee` / `#0c0d11`.

Regenerate previews: `resvg -w 810 --background '#0c0d11' wordmark-dark.svg preview-wordmark-dark.png && oxipng -o4 preview-*.png`.
