import { selectIntentionalLinks } from '../linkSelect.ts';
import type { Atom, Link } from '../../types.ts';

const MAX_LINKS_PER_TAG = 30;

/** Shared-tag derived synapses, sparsified to load-bearing links. Pure. */
export function buildTagLinks(atoms: Pick<Atom, 'id' | 'tags'>[]): Link[] {
  const tagIndex = new Map<string, string[]>();
  for (const a of atoms) {
    for (const t of a.tags) {
      const list = tagIndex.get(t) ?? [];
      list.push(a.id);
      tagIndex.set(t, list);
    }
  }
  const pairCount = new Map<string, number>();
  for (const ids of tagIndex.values()) {
    const limit = Math.min(ids.length, MAX_LINKS_PER_TAG);
    for (let i = 0; i < limit; i++) {
      for (let j = i + 1; j < limit; j++) {
        const key = ids[i] < ids[j] ? `${ids[i]}\u0000${ids[j]}` : `${ids[j]}\u0000${ids[i]}`;
        pairCount.set(key, (pairCount.get(key) ?? 0) + 1);
      }
    }
  }
  const links: Link[] = [];
  for (const [key, count] of pairCount) {
    const [a, b] = key.split('\u0000');
    links.push({ a, b, strength: count });
  }
  return selectIntentionalLinks(links);
}
