# Prompt: Store / DB browser view

Design the Store surface (`#/db` route) — a dense data browser over the atom tables. React + CSS on the electric-purple token layer. Think VoltOps/pgAdmin-adjacent table UX, dark and compact — not a marketing page.

## Contract

- **Table tabs**: atoms / sources / threads / proposals — mono segmented control.
- **Filter bar**: search input (debounced), lane select (trusted/quarantine/all), tier select (hot/warm/cold), sort column + asc/desc toggle.
- **Data grid**: real table — columns `title · tier · lane · source · indexed_at`. Mono rows, zebra-free (hairline row separators instead), hover row highlight, click opens row detail (inspector-style side panel or expand-in-place — pick one).
- **Pagination**: `limit 50` + prev/next + total count readout (`50 / 12,629`).
- **States**: loading skeleton rows; empty → `no rows match — widen filters`; error → mono error + retry.
- **Density**: compact row height (~32px), this is a power surface — density over whitespace.

## Hard rules
- No card grid — this is a table, dense and fast.
- Column headers sticky on scroll; numeric columns right-aligned.
- Mobile (<768px): allow horizontal scroll of the table; filters collapse into a `filters` disclosure row.

## Output contract
`store-view.tsx` + `store-view.css`. Props: `{ table, rows, total, filters, onFilter, onSort, onPage, onOpenRow, loading, error }`.
