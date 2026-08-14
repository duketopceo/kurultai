import { createContext, useContext } from 'react';
import { normalizeLayout } from './brain/layout/mode';
import type { Atom, LayoutMode } from './types';

export interface AppState {
  atoms: Atom[];
  selected: Atom | null;
  query: string;
  since: number;
  live: boolean;
  maxMode: boolean;
  atomTotal: number;
  layout: LayoutMode;
  daemonOk: boolean;
  daemonVersion: string;
}

export type AppAction =
  | { type: 'SET_ATOMS'; atoms: Atom[]; total?: number }
  | { type: 'SET_SELECTED'; atom: Atom | null }
  | { type: 'SET_QUERY'; query: string }
  | { type: 'SET_SINCE'; since: number }
  | { type: 'SET_LIVE'; live: boolean }
  | { type: 'SET_MAX_MODE'; maxMode: boolean }
  | { type: 'SET_LAYOUT'; layout: LayoutMode }
  | { type: 'SET_DAEMON'; ok: boolean; version?: string };

export function reducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'SET_ATOMS':
      return { ...state, atoms: action.atoms, atomTotal: action.total ?? action.atoms.length };
    case 'SET_SELECTED':
      return { ...state, selected: action.atom };
    case 'SET_QUERY':
      return { ...state, query: action.query };
    case 'SET_SINCE':
      return { ...state, since: action.since };
    case 'SET_LIVE':
      return { ...state, live: action.live };
    case 'SET_MAX_MODE':
      return { ...state, maxMode: action.maxMode };
    case 'SET_LAYOUT':
      return { ...state, layout: action.layout };
    case 'SET_DAEMON':
      return { ...state, daemonOk: action.ok, daemonVersion: action.version ?? state.daemonVersion };
    default:
      return state;
  }
}

/** Restores the persisted layout, falling back to 'brain' on missing/corrupt values or storage errors. */
function initialLayout(): LayoutMode {
  try {
    return normalizeLayout(localStorage.getItem('kurultai-layout'));
  } catch {
    return 'brain';
  }
}

export const initialState: AppState = {
  atoms: [],
  selected: null,
  query: '',
  since: 0,
  live: true,
  maxMode: localStorage.getItem('kurultai-max-mode') === '1',
  atomTotal: 0,
  layout: initialLayout(),
  daemonOk: false,
  daemonVersion: '',
};

export const AppContext = createContext<{
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
} | null>(null);

/** FNV-1a 32-bit hash — used by BrainView to deterministically pin atoms to cortex vertices. */
export function hashId(id: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < id.length; i++) {
    h ^= id.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h;
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used inside AppProvider');
  return ctx;
}
