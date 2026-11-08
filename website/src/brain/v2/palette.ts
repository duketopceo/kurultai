/** Command palette model (pure). Secondary surfaces live behind one drawer/palette. */
export type SurfaceKey = 'hey' | 'pulse' | 'ontology' | 'ask' | 'store';

export interface PaletteCommand {
  id: string;
  label: string;
  hint?: string;
  surface?: SurfaceKey;
  action?: 'repos' | 'settings' | 'reset';
}

export const PALETTE_COMMANDS: PaletteCommand[] = [
  { id: 'hey', label: 'Hey', hint: 'agent board', surface: 'hey' },
  { id: 'pulse', label: 'Pulse', hint: 'activity and stats', surface: 'pulse' },
  { id: 'ontology', label: 'Ontology', hint: 'classes and proposals', surface: 'ontology' },
  { id: 'ask', label: 'Ask', hint: 'question the brain', surface: 'ask' },
  { id: 'store', label: 'Store', hint: 'raw atoms', surface: 'store' },
  { id: 'repos', label: 'Repos', hint: 'floorplan view', action: 'repos' },
  { id: 'reset', label: 'Reset view', hint: 'whole graph', action: 'reset' },
  { id: 'settings', label: 'Access settings', action: 'settings' },
];

/** Prefix matches first, then substring matches; empty query returns everything. */
export function filterCommands(commands: PaletteCommand[], query: string): PaletteCommand[] {
  const q = query.trim().toLowerCase();
  if (!q) return commands;
  const starts: PaletteCommand[] = [];
  const contains: PaletteCommand[] = [];
  for (const c of commands) {
    const l = c.label.toLowerCase();
    if (l.startsWith(q)) starts.push(c);
    else if (l.includes(q) || (c.hint ?? '').toLowerCase().includes(q)) contains.push(c);
  }
  return [...starts, ...contains];
}
