/**
 * Repo floorplan model (pure): folders are rooms, files are tiles sized by
 * weight, tinted by activity. Kept separate from the brain on purpose.
 */
export interface FloorEntry {
  /** repo-relative path with forward slashes */
  path: string;
  /** size proxy (bytes, chars or symbol count); must be >= 0 */
  size: number;
  /** 0..1 recency/activity */
  activity: number;
}

export interface Rect { x: number; y: number; w: number; h: number }
export interface Tile extends Rect { path: string; name: string; activity: number; size: number }
export interface Room extends Rect { path: string; name: string; weight: number }
export interface Floorplan { rooms: Room[]; tiles: Tile[] }

interface Node {
  name: string;
  path: string;
  children: Map<string, Node>;
  files: FloorEntry[];
  weight: number;
}

function newNode(name: string, path: string): Node {
  return { name, path, children: new Map(), files: [], weight: 0 };
}

export function buildTree(entries: FloorEntry[]): Node {
  const root = newNode('', '');
  for (const e of entries) {
    const segs = e.path.split('/').filter(Boolean);
    if (!segs.length) continue;
    let n = root;
    for (let i = 0; i < segs.length - 1; i++) {
      const p = segs.slice(0, i + 1).join('/');
      let c = n.children.get(segs[i]);
      if (!c) { c = newNode(segs[i], p); n.children.set(segs[i], c); }
      n = c;
    }
    n.files.push(e);
  }
  const sum = (n: Node): number => {
    n.weight = n.files.reduce((s, f) => s + Math.max(1, f.size), 0);
    for (const c of n.children.values()) n.weight += sum(c);
    return n.weight;
  };
  sum(root);
  return root;
}

type Item = { weight: number; node?: Node; file?: FloorEntry };

/** Slice-and-dice treemap with alternating axis; heaviest first. */
function split(items: Item[], r: Rect, vertical: boolean): Array<[Item, Rect]> {
  const total = items.reduce((s, i) => s + i.weight, 0) || 1;
  const out: Array<[Item, Rect]> = [];
  let off = 0;
  for (const it of items) {
    const f = it.weight / total;
    out.push([
      it,
      vertical
        ? { x: r.x, y: r.y + off * r.h, w: r.w, h: f * r.h }
        : { x: r.x + off * r.w, y: r.y, w: f * r.w, h: r.h },
    ]);
    off += f;
  }
  return out;
}

const GAP = 2;
const HEADER = 14;

export function layoutFloorplan(entries: FloorEntry[], width: number, height: number): Floorplan {
  const root = buildTree(entries);
  const plan: Floorplan = { rooms: [], tiles: [] };
  const place = (n: Node, r: Rect, depth: number) => {
    const items: Item[] = [
      ...[...n.children.values()].map((c) => ({ weight: c.weight, node: c })),
      ...n.files.map((f) => ({ weight: Math.max(1, f.size), file: f })),
    ].sort((a, b) => b.weight - a.weight);
    if (!items.length) return;
    for (const [it, cell] of split(items, r, r.h >= r.w)) {
      const inner: Rect = { x: cell.x + GAP / 2, y: cell.y + GAP / 2, w: Math.max(0, cell.w - GAP), h: Math.max(0, cell.h - GAP) };
      if (it.file) {
        const name = it.file.path.split('/').pop() ?? it.file.path;
        plan.tiles.push({ ...inner, path: it.file.path, name, activity: it.file.activity, size: it.file.size });
      } else if (it.node) {
        plan.rooms.push({ ...inner, path: it.node.path, name: it.node.name, weight: it.node.weight });
        const body: Rect = { x: inner.x, y: inner.y + Math.min(HEADER, inner.h / 3), w: inner.w, h: inner.h - Math.min(HEADER, inner.h / 3) };
        place(it.node, body, depth + 1);
      }
    }
  };
  place(root, { x: 0, y: 0, w: width, h: height }, 0);
  return plan;
}

/** Entries from indexed code atoms (real data path). */
export function entriesFromAtoms(
  atoms: Array<{ source_id: string; summary: string; tags: string[]; indexed_at: string; last_accessed_at: string }>,
  repo: string,
  now = Date.now(),
): FloorEntry[] {
  const seen = new Map<string, FloorEntry>();
  for (const a of atoms) {
    const sid = (a.source_id || '').replace(/\\/g, '/').split('#')[0];
    const idx = sid.indexOf(`/${repo.split('/').pop()}/`);
    const rel = (idx >= 0 ? sid.slice(idx + repo.split('/').pop()!.length + 2) : sid).replace(/^\/+/, '');
    if (!rel) continue;
    const ts = Date.parse(a.last_accessed_at || a.indexed_at || '') || 0;
    const ageDays = ts ? Math.max(0, (now - ts) / 86_400_000) : 365;
    const activity = Math.max(0, Math.min(1, 1 - ageDays / 90));
    const size = (a.summary?.length ?? 0) + a.tags.length * 40 + 60;
    const prev = seen.get(rel);
    if (prev) { prev.size += size; prev.activity = Math.max(prev.activity, activity); }
    else seen.set(rel, { path: rel, size, activity });
  }
  return [...seen.values()];
}

/**
 * Fallback/baked data: rows of an agent-index `INDEX.md` Files table,
 * `| [`name`](path) | Does | ...`. Size is the Does-text length as a weak proxy.
 * (codebase-memory-mcp architecture data can be dropped in as FloorEntry[]
 * JSON at `/ui/repo-manifests/<repo>.json`; the daemon cannot reach CBM itself.)
 */
export function entriesFromAgentIndex(markdown: string, folder = ''): FloorEntry[] {
  const out: FloorEntry[] = [];
  const re = /^\|\s*\[`([^`]+)`\]\(([^)]+)\)\s*\|\s*([^|]*)\|/gm;
  let m: RegExpExecArray | null;
  while ((m = re.exec(markdown))) {
    const rel = m[2].replace(/^\.\//, '');
    out.push({ path: [folder, rel].filter(Boolean).join('/'), size: 80 + m[3].trim().length * 4, activity: 0.3 });
  }
  return out;
}
