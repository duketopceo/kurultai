import type { Atom } from '../../types.ts';

export interface ChainStep {
  id: string;
  title: string;
  /** Tags shared with the previous step (why this hop exists). */
  via: string[];
}

/** Turn a path of atom ids into a readable "why connected" chain. Pure. */
export function describeChain(path: string[], atoms: Map<string, Pick<Atom, 'id' | 'title' | 'tags'>>): ChainStep[] {
  const steps: ChainStep[] = [];
  for (let i = 0; i < path.length; i++) {
    const a = atoms.get(path[i]);
    const prev = i > 0 ? atoms.get(path[i - 1]) : undefined;
    const via = a && prev ? a.tags.filter((t) => prev.tags.includes(t)) : [];
    steps.push({ id: path[i], title: a?.title ?? path[i], via });
  }
  return steps;
}
