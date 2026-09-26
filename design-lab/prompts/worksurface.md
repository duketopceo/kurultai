# Prompt: WorkSurface — the below-brain console

Design the tabbed console that sits below the Brain hero in the Kurultai dashboard. React + CSS on the dark glass/electric-purple token layer. This replaces a sidebar rail + bottom tab cluster with ONE full-width instrument console.

## Tabs

| Tab | Contents |
|---|---|
| **Hey** | Agent coordination board. Left: thread list (mono thread names, unread dot, last-active time). Right: conversation pane — message rows styled like an ops transcript: `codename@instance_id` mono identity, timestamp, body (readable sans), hover affordances for reply/react/edit/delete. A segmented toggle switches conversation ↔ kanban (columns: queued/doing/done cards dragged between columns; a compact, credible kanban — not a toy). Presence strip on top: `devin@omarchy-max online · kurultai`. |
| **Pulse** | Top row: HUD stat readouts — memories (loaded/total), hot/warm/cold, trusted — mono label + numeral. Below: live memory stream — rows `event · title · source`, auto-updating, with a live/paused toggle button. |
| **Ontology** | Left: class tree (6 classes, expandable rows, instance counts). Right: proposals queue — cards showing proposal description + approve/reject buttons + decided-state styling. |
| **Ask** | Question textarea + `Ask ↗` button; answer renders below in a prose block; `Thinking…` state; error line on failure. |
| **Store** | Compact teaser: table of recent atoms (title · tier · indexed) + `Open Store →` link to the full `#/db` route. |

## Requirements
- Tab bar: segmented control, mono uppercase labels, purple active pill, `role=tablist`/`aria-selected`.
- Every tab defines honest loading / empty / error states — NEVER fake seed data. Empty = one actionable hint line.
- Mobile: tab bar scrolls horizontally; Hey collapses to one pane with a back affordance.
- Kanban drag can be HTML5 native DnD — no library.
- Panel chrome: glass cards, hairline purple borders, 6px radius; hover lifts ≤2px; transitions 120–220ms.

## Output contract
`worksurface.tsx` + `worksurface.css` — typed props for each tab's data; internals may be stubbed where a child (chat rows, kanban card) is its own component — mark them clearly. No external deps beyond React.
