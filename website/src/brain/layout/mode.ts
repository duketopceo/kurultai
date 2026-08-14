import type { LayoutMode } from '../../types.ts';

/** Maps persisted / unknown layout strings onto the two supported modes. */
export function normalizeLayout(raw: string | null | undefined): LayoutMode {
  return raw === 'ontology' ? 'ontology' : 'brain';
}
