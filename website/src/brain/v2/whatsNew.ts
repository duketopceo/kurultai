/** "What's new since you last looked": last-visit marker + diff (pure, storage injected). */
export const LAST_VISIT_KEY = 'kurultai.brain.lastVisit';

export interface StorageLike {
  getItem(k: string): string | null;
  setItem(k: string, v: string): void;
}

export function readLastVisit(storage: StorageLike | null | undefined): number | null {
  try {
    const raw = storage?.getItem(LAST_VISIT_KEY);
    const n = raw ? Number(raw) : NaN;
    return Number.isFinite(n) && n > 0 ? n : null;
  } catch {
    return null;
  }
}

export function writeLastVisit(storage: StorageLike | null | undefined, now = Date.now()): void {
  try {
    storage?.setItem(LAST_VISIT_KEY, String(now));
  } catch { /* private mode / blocked storage: degrade silently */ }
}

export function safeLocalStorage(): StorageLike | null {
  try {
    return typeof window !== 'undefined' ? window.localStorage : null;
  } catch {
    return null;
  }
}

export interface NewItem { id: string; title: string; source: string; kind: 'new' | 'changed' }
export interface WhatsNew { items: NewItem[]; newSources: string[] }

interface A { id: string; title: string; source: string; indexed_at: string; last_accessed_at?: string }

/**
 * Atoms indexed after `since` are new. A first visit (no marker) reports
 * nothing so the whole brain does not light up. `newSources` = sources whose
 * every loaded atom is new.
 */
export function computeWhatsNew(atoms: A[], since: number | null): WhatsNew {
  if (since === null) return { items: [], newSources: [] };
  const items: NewItem[] = [];
  const bySource = new Map<string, { total: number; fresh: number }>();
  for (const a of atoms) {
    const t = Date.parse(a.indexed_at);
    const s = bySource.get(a.source) ?? { total: 0, fresh: 0 };
    s.total++;
    if (Number.isFinite(t) && t > since) {
      s.fresh++;
      items.push({ id: a.id, title: a.title, source: a.source, kind: 'new' });
    }
    bySource.set(a.source, s);
  }
  const newSources = [...bySource.entries()].filter(([, v]) => v.fresh > 0 && v.fresh === v.total).map(([k]) => k);
  return { items, newSources };
}
