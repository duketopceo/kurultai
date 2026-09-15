import type { Atom, OntologyEntity, OntologyLink } from '../../types';

export interface BoardNodeData extends Record<string, unknown> {
  entity: OntologyEntity;
  /** Instances hidden behind this class card until expanded. */
  hiddenCount: number;
  /** Entity is backed by a loaded atom (or shares an atom's id). */
  backed: boolean;
}

export interface BoardNode {
  id: string;
  position: { x: number; y: number };
  data: BoardNodeData;
}

export interface BoardEdge {
  id: string;
  source: string;
  target: string;
  rel: string;
}

/**
 * Layered board layout (#316): classes stack by `is_a` depth; an expanded
 * class's `instance_of` children fan out underneath; everything else goes in
 * a bottom row. Persisted drag positions win over the computed slot.
 */
export function layoutOntology(
  entities: OntologyEntity[],
  links: OntologyLink[],
  expanded: ReadonlySet<string>,
  saved: Record<string, { x: number; y: number }>,
  atoms: Atom[],
): BoardNode[] {
  const classes = entities.filter((e) => e.kind === 'class');
  const others = entities.filter((e) => e.kind !== 'class');
  const atomIds = new Set(atoms.map((a) => a.id));

  const parentOf = new Map<string, string>();
  for (const l of links) {
    if (l.rel === 'is_a' || l.rel === 'instance_of') parentOf.set(l.from_id, l.to_id);
  }
  const classIds = new Set(classes.map((c) => c.id));
  const depthCache = new Map<string, number>();
  const classDepth = (id: string, seen = new Set<string>()): number => {
    const hit = depthCache.get(id);
    if (hit !== undefined) return hit;
    if (seen.has(id)) return 0;
    seen.add(id);
    const parent = parentOf.get(id);
    const d = parent && classIds.has(parent) ? classDepth(parent, seen) + 1 : 0;
    depthCache.set(id, d);
    return d;
  };

  const instanceOf = new Map<string, OntologyEntity[]>();
  const orphanOthers: OntologyEntity[] = [];
  for (const e of others) {
    const parent = links.find((l) => l.rel === 'instance_of' && l.from_id === e.id)?.to_id;
    if (parent && classIds.has(parent)) {
      const list = instanceOf.get(parent) ?? [];
      list.push(e);
      instanceOf.set(parent, list);
    } else {
      orphanOthers.push(e);
    }
  }

  const backed = (e: OntologyEntity) =>
    !!(e.atom_id && atomIds.has(e.atom_id)) || atomIds.has(e.id);
  const nodes: BoardNode[] = [];
  const rows = new Map<number, number>();
  for (const c of classes) {
    const d = classDepth(c.id);
    const col = rows.get(d) ?? 0;
    rows.set(d, col + 1);
    const kids = instanceOf.get(c.id) ?? [];
    nodes.push({
      id: c.id,
      position: saved[c.id] ?? { x: col * 280, y: d * 180 },
      data: { entity: c, hiddenCount: expanded.has(c.id) ? 0 : kids.length, backed: backed(c) },
    });
    if (expanded.has(c.id)) {
      kids.forEach((k, i) => {
        nodes.push({
          id: k.id,
          position: saved[k.id] ?? { x: col * 280 + i * 240, y: d * 180 + 130 },
          data: { entity: k, hiddenCount: 0, backed: backed(k) },
        });
      });
    }
  }
  const bottom = (Math.max(0, ...depthCache.values()) + 1) * 180 + 80;
  orphanOthers.forEach((e, i) => {
    nodes.push({
      id: e.id,
      position: saved[e.id] ?? { x: i * 240, y: bottom },
      data: { entity: e, hiddenCount: 0, backed: backed(e) },
    });
  });
  return nodes;
}

/** Typed edges among currently-visible entities only. */
export function boardEdges(
  links: OntologyLink[],
  visible: ReadonlySet<string>,
): BoardEdge[] {
  return links
    .filter((l) => visible.has(l.from_id) && visible.has(l.to_id))
    .map((l) => ({ id: l.id, source: l.from_id, target: l.to_id, rel: l.rel }));
}

/** Map an entity to its backing atom, or synthesize a stub for the inspector. */
export function entityToAtom(e: OntologyEntity, atoms: Atom[]): Atom {
  const hit = atoms.find((a) => a.id === e.atom_id || a.id === e.id);
  if (hit) return hit;
  return {
    id: e.atom_id ?? e.id,
    title: e.name,
    summary: `Ontology ${e.kind} · ${e.id}`,
    source: 'ontology',
    source_id: e.id,
    tags: [e.kind],
    file: '',
    indexed_at: '',
    last_accessed_at: '',
    score: 0,
    tier: 'warm',
    lean: true,
  };
}
