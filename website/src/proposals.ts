/**
 * O3 proposal-queue shared shape + presentation helpers (#118).
 * Pure module — no fetch/auth imports so node --test can exercise it.
 */

export type OntologyProposal = {
  id: string;
  /** `promote_atom` | `new_link` | `new_entity`. */
  kind: string;
  /** Kind-specific params (atom_id/class_id, from/to/rel, entity fields). */
  payload: Record<string, unknown>;
  status: 'pending' | 'approved' | 'rejected' | string;
  proposed_by: string;
  reason?: string | null;
  created_at: string;
  decided_by?: string | null;
  decided_at?: string | null;
};

/** One-line human summary of a proposal payload for the review queue. */
export function describeProposal(p: OntologyProposal): string {
  const pl = p.payload ?? {};
  switch (p.kind) {
    case 'promote_atom':
      return `${pl.atom_id ?? '?'} → ${pl.class_id ?? '?'}`;
    case 'new_link':
      return `${pl.from_id ?? '?'} —${pl.rel ?? '?'}→ ${pl.to_id ?? '?'}`;
    case 'new_entity':
      return `${pl.entity_kind ?? 'entity'} “${pl.name ?? pl.id ?? '?'}”`;
    default:
      return JSON.stringify(pl).slice(0, 80);
  }
}
