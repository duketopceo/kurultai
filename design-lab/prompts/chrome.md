# Prompt: top chrome (TopBar + CommandStrip)

Design the top chrome of the Kurultai dark-instrument dashboard as React + CSS using the token layer (electric-purple glass-over-black; see tokens prompt for the palette contract — import it, don't restate it).

## Surface contract

**TopBar** — full-width, hairline bottom border, height ~48px:
- Left: brand `KURULTAI` (mono, letter-spaced) + tiny version mark.
- Center: nav — Brain · Repos β · Store β (mono labels, purple underline on active).
- Right: daemon status — dot (ok-green / danger-red) + `online · api 0.6.0` mono text; an access/settings affordance.
- Never says "LOCAL BRAIN".

**CommandStrip** — slim bar directly under TopBar (~44px), three control groups only:
- Search input with ⌘K hint chip, dropdown results (title + one-line summary, ≤6), and an explicit clear/reset control.
- Memory tier segmented control: low / mid / high / max — mono, purple active pill, tiny `loaded/total` count when partial.
- Layout segmented control: brain / ontology.
- Nothing else. No timeline scrubber, no random button.

## Requirements
- Loading/offline states: status dot flips + `connecting` text; search disabled gracefully.
- ≤768px: strip wraps to two rows; search takes full row.
- Focus-visible purple rings; aria roles for tablist/listbox/segmented groups.
- Mono micro-labels (uppercase, letter-spaced) for group captions where needed — sparingly.

## Output contract
`chrome.tsx` + `chrome.css` — self-contained, props-typed, no external deps beyond React. Comments only where behavior isn't obvious.
