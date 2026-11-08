import type { Link } from '../../types.ts';

export type Adjacency = Map<string, string[]>;

export function buildAdjacency(links: Pick<Link, 'a' | 'b'>[]): Adjacency {
  const adj: Adjacency = new Map();
  const add = (x: string, y: string) => {
    const l = adj.get(x);
    if (l) l.push(y);
    else adj.set(x, [y]);
  };
  for (const l of links) {
    add(l.a, l.b);
    add(l.b, l.a);
  }
  return adj;
}

/**
 * Client-side "why connected": BFS shortest path over the loaded edges.
 * (No server traverse API exists yet; this is the minimal stand-in.)
 * Returns node ids from `from` to `to` inclusive, or null.
 */
export function shortestPath(adj: Adjacency, from: string, to: string, maxVisited = 200_000): string[] | null {
  if (from === to) return [from];
  if (!adj.has(from) || !adj.has(to)) return null;
  const prev = new Map<string, string>([[from, from]]);
  let queue = [from];
  let visited = 0;
  while (queue.length) {
    const next: string[] = [];
    for (const cur of queue) {
      for (const n of adj.get(cur) ?? []) {
        if (prev.has(n)) continue;
        prev.set(n, cur);
        if (n === to) {
          const out = [to];
          let p = to;
          while (p !== from) {
            p = prev.get(p)!;
            out.push(p);
          }
          return out.reverse();
        }
        if (++visited > maxVisited) return null;
        next.push(n);
      }
    }
    queue = next;
  }
  return null;
}
