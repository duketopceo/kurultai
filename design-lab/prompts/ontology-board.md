# Prompt: Ontology board (typed 2D board)

Design the Ontology surface for the Kurultai dashboard — a **typed 2D board**, NOT an organic 3D cloud. React + CSS on the electric-purple token layer.

## Mental model
Six fixed classes (e.g. Note, Person, Project, Task, Entity, Source). Each class is a column or expandable row showing its instances. Edges between instances are typed links (`mentions`, `belongs_to`, `authored`…). The board is Flowsint-style: nodes are small cards with a colored class chip + title + link count; edges are thin lines/dashed connectors, NOT physics blobs.

## Contract

- **Board view**: classes as columns (or a compact graph of positioned cards) — pick ONE and justify in a comment. Cards: `class chip · title · link-count`. Hover: card lifts ≤2px, its edges highlight purple, others dim.
- **Class header**: class name (mono) + instance count + expand/collapse.
- **Node menu**: right-click or `⋯` — `open`, `link`, `delete` (delete gets a confirm state inline, not a browser confirm()).
- **Link mode**: click `link` → pick target card → line draws; while linking, eligible targets get a purple outline pulse.
- **Add entity**: `+ entity` affordance — inline name+class picker.
- **States**: empty class → honest empty line; busy mutations → inline spinner; error → mono error text, recoverable.

## Hard rules
- Orthogonal to the Brain: squared cards, hairline borders, thin edges. No glow blobs, no force-graph physics aesthetic.
- Zoom/pan container allowed (CSS transform or plain scroll) but default view fits the board.
- Mobile: columns stack vertically; link mode disabled under 768px (declare it, don't half-support it).

## Output contract
`ontology-board.tsx` + `ontology-board.css`. Typed props: `{ classes, instances, edges, onOpen, onLink, onDelete, onAdd }`. Self-contained React+CSS only — no external deps.
