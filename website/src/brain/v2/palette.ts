/** Command palette model (pure). Secondary surfaces live behind one drawer/palette. */
export type SurfaceKey = 'hey' | 'pulse' | 'ontology' | 'ask' | 'store';

export type ViewKey = 'brain' | 'repos' | 'ontology';

export interface PaletteCommand {
  id: string;
  label: string;
  hint?: string;
  /** Opens the side panel on this tab. */
  surface?: SurfaceKey;
  /** Switches the canvas mode. */
  view?: ViewKey;
  action?: 'settings' | 'reset' | 'mode32';
}

export const PALETTE_COMMANDS: PaletteCommand[] = [
  { id: 'brain', label: 'Brain view', hint: 'whole graph', view: 'brain' },
  { id: 'repos', label: 'Repos view', hint: 'floorplan view', view: 'repos' },
  { id: 'ontology', label: 'Ontology view', hint: 'classes and proposals', view: 'ontology' },
  { id: 'hey', label: 'Hey', hint: 'agent board', surface: 'hey' },
  { id: 'pulse', label: 'Pulse', hint: 'activity and stats', surface: 'pulse' },
  { id: 'ask', label: 'Ask', hint: 'question the brain', surface: 'ask' },
  { id: 'store', label: 'Store', hint: 'raw atoms', surface: 'store' },
  { id: 'reset', label: 'Reset view', hint: 'whole graph', action: 'reset' },
  { id: 'mode32', label: '32-bit mode', hint: 'pixel look, for fun', action: 'mode32' },
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
