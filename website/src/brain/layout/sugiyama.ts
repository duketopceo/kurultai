/** Sugiyama-style layered layout for a directed `is_a` DAG (O2 / #117). */

export interface HierNode {
  id: string;
  layer?: number;
}

/** `from` is the subclass, `to` is the superclass. */
export interface HierEdge {
  from: string;
  to: string;
}

export interface HierPos {
  x: number;
  y: number;
  z: number;
}

/**
 * Longest-path layers: sinks (no superclass after back-edge reversal) sit at
 * layer 0. Cycles reverse one back-edge for layering only — callers must not
 * write that reversal to the store.
 */
export function assignLayers(nodes: HierNode[], edges: HierEdge[]): Map<string, number> {
  const ids = nodes.map((n) => n.id);
  const idSet = new Set(ids);
  const succs = new Map<string, string[]>();
  for (const id of ids) succs.set(id, []);
  for (const e of edges) {
    if (!idSet.has(e.from) || !idSet.has(e.to) || e.from === e.to) continue;
    succs.get(e.from)!.push(e.to);
  }
  reverseBackEdges(ids, succs);
  for (const id of ids) {
    succs.set(id, [...new Set(succs.get(id))]);
  }

  const memo = new Map<string, number>();
  const visiting = new Set<string>();
  const layerOf = (id: string): number => {
    const hit = memo.get(id);
    if (hit !== undefined) return hit;
    if (visiting.has(id)) return 0;
    visiting.add(id);
    const outs = succs.get(id) ?? [];
    const value = outs.length === 0 ? 0 : 1 + Math.max(...outs.map(layerOf));
    visiting.delete(id);
    memo.set(id, value);
    return value;
  };

  const out = new Map<string, number>();
  for (const id of ids) out.set(id, layerOf(id));
  return out;
}

/** Bucket `assignLayers` into `layers[layer] = ids` (layer 0 = superclasses). */
export function bucketsFromAssign(assign: Map<string, number>): string[][] {
  let max = 0;
  for (const layer of assign.values()) if (layer > max) max = layer;
  const buckets: string[][] = Array.from({ length: max + 1 }, () => []);
  for (const [id, layer] of assign) buckets[layer].push(id);
  return buckets;
}

/**
 * Barycenter sweeps against adjacent layers. Neighborless nodes keep relative
 * order. Edges are treated as undirected for crossing reduction.
 */
export function orderLayers(layers: string[][], edges: HierEdge[]): string[][] {
  const ordered = layers.map((row) => [...row]);
  const undirected = undirectedIndex(edges);
  for (let sweep = 0; sweep < 4; sweep++) {
    for (let i = 1; i < ordered.length; i++) {
      ordered[i] = sortByBarycenter(ordered[i], ordered[i - 1], undirected);
    }
    for (let i = ordered.length - 2; i >= 0; i--) {
      ordered[i] = sortByBarycenter(ordered[i], ordered[i + 1], undirected);
    }
  }
  return ordered;
}

/** Y = layer * yStep (superclass at y=0). X spaced by xzStep; slight Z for depth. */
export function hierPositions(
  ordered: string[][],
  yStep: number,
  xzStep: number,
): Map<string, HierPos> {
  const out = new Map<string, HierPos>();
  for (let layer = 0; layer < ordered.length; layer++) {
    const ids = ordered[layer];
    const n = ids.length;
    const mid = (n - 1) / 2;
    for (let i = 0; i < n; i++) {
      out.set(ids[i], {
        x: (i - mid) * xzStep,
        y: layer * yStep,
        z: (i - mid) * xzStep * 0.18,
      });
    }
  }
  return out;
}

function reverseBackEdges(ids: string[], succs: Map<string, string[]>): void {
  const WHITE = 0;
  const GRAY = 1;
  const BLACK = 2;
  const color = new Map<string, number>();
  for (const id of ids) color.set(id, WHITE);

  const visit = (u: string) => {
    color.set(u, GRAY);
    const outs = succs.get(u) ?? [];
    for (let i = outs.length - 1; i >= 0; i--) {
      const v = outs[i];
      const c = color.get(v) ?? WHITE;
      if (c === GRAY) {
        outs.splice(i, 1);
        succs.get(v)!.push(u);
      } else if (c === WHITE) {
        visit(v);
      }
    }
    color.set(u, BLACK);
  };

  for (const id of ids) {
    if (color.get(id) === WHITE) visit(id);
  }
}

function undirectedIndex(edges: HierEdge[]): Map<string, Set<string>> {
  const idx = new Map<string, Set<string>>();
  const add = (a: string, b: string) => {
    let set = idx.get(a);
    if (!set) {
      set = new Set();
      idx.set(a, set);
    }
    set.add(b);
  };
  for (const e of edges) {
    add(e.from, e.to);
    add(e.to, e.from);
  }
  return idx;
}

function sortByBarycenter(
  row: string[],
  neighborRow: string[],
  undirected: Map<string, Set<string>>,
): string[] {
  const neighborIndex = new Map(neighborRow.map((id, i) => [id, i]));
  const original = new Map(row.map((id, i) => [id, i]));
  const score = (id: string): number => {
    const nbrs = undirected.get(id);
    if (!nbrs) return original.get(id) ?? 0;
    let sum = 0;
    let n = 0;
    for (const nbr of nbrs) {
      const i = neighborIndex.get(nbr);
      if (i === undefined) continue;
      sum += i;
      n++;
    }
    return n === 0 ? (original.get(id) ?? 0) : sum / n;
  };
  return [...row].sort((a, b) => {
    const d = score(a) - score(b);
    if (d !== 0) return d;
    return (original.get(a) ?? 0) - (original.get(b) ?? 0);
  });
}
