import { useReducer, useEffect, useCallback, useRef, useState } from 'react';
import { reducer, initialState, AppContext } from './state';
import { fetchStatus, fetchAtoms, fetchGraph } from './api';
import { TopBar } from './components/TopBar';
import { BrainStage, BrainStageHandle } from './components/BrainStage';
import { CommandStrip } from './components/CommandStrip';
import { ActivityPanel } from './components/ActivityPanel';
import { InspectorPanel } from './components/InspectorPanel';
import { AskPanel } from './components/AskPanel';
import { StatsPanel } from './components/StatsPanel';
import type { Atom, LayoutMode } from './types';

const DEFAULT_ATOM_LIMIT = 450;
const MAX_GRAPH_LIMIT = 10000;

function dateValue(atom: Atom): number {
  const d = Date.parse(atom.indexed_at || atom.last_accessed_at || '');
  return Number.isFinite(d) ? d : 0;
}

export function App() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const [selected, setSelected] = useState<Atom | null>(null);
  const [live, setLive] = useState(true);
  const abortRef = useRef<AbortController | null>(null);
  const brainRef = useRef<BrainStageHandle | null>(null);

  const filteredAtoms = useCallback(() => {
    if (state.since <= 0 || !state.atoms.length) return state.atoms;
    const horizon = state.since;
    const dates = state.atoms.map(dateValue).filter(Boolean);
    const min = Math.min(...dates, Date.now());
    const cutoff = min + (Date.now() - min) * (1 - horizon);
    return state.atoms.filter((a) => !dateValue(a) || dateValue(a) >= cutoff);
  }, [state.atoms, state.since]);

  const loadAtoms = useCallback(async () => {
    abortRef.current?.abort();
    const ac = new AbortController();
    abortRef.current = ac;
    try {
      let atoms: Atom[];
      if (state.maxMode) {
        atoms = await fetchGraph(ac.signal);
      } else {
        atoms = await fetchAtoms(DEFAULT_ATOM_LIMIT, ac.signal);
      }
      dispatch({ type: 'SET_ATOMS', atoms, total: atoms.length });
    } catch { /* ignore abort */ }
  }, [state.maxMode]);

  useEffect(() => {
    let alive = true;
    const poll = async () => {
      while (alive) {
        try {
          abortRef.current?.abort();
          const ac = new AbortController();
          abortRef.current = ac;
          const status = await fetchStatus(ac.signal);
          dispatch({ type: 'SET_DAEMON', ok: status.ok, version: status.version });
        } catch { /* ignore */ }
        await new Promise((r) => setTimeout(r, 8000));
      }
    };
    poll();
    return () => { alive = false; abortRef.current?.abort(); };
  }, []);

  useEffect(() => {
    loadAtoms();
  }, [state.maxMode]);

  const handleLayoutChange = (mode: LayoutMode) => {
    dispatch({ type: 'SET_LAYOUT', layout: mode });
    try {
      localStorage.setItem('kurultai-layout', mode);
    } catch { /* storage unavailable — layout persistence is best-effort */ }
  };

  const handleMaxMode = async (enabled: boolean) => {
    dispatch({ type: 'SET_MAX_MODE', maxMode: enabled });
    localStorage.setItem('kurultai-max-mode', enabled ? '1' : '0');
  };

  const handleSince = (since: number) => {
    dispatch({ type: 'SET_SINCE', since });
  };

  const handleSelectAndFocus = (atom: Atom) => {
    setSelected(atom);
    brainRef.current?.focusAtom(atom);
  };

  const handleRandom = () => {
    brainRef.current?.randomConnection();
  };

  const visible = filteredAtoms();
  const renderCap = state.maxMode ? 2500 : DEFAULT_ATOM_LIMIT;

  const caption = (() => {
    const shown = Math.min(visible.length, renderCap);
    const total = Math.max(state.atomTotal, state.atoms.length);
    const mode = state.maxMode ? 'max' : 'standard';
    if (state.maxMode && visible.length > shown) return `${shown} shown · ${visible.length} loaded (cap ${renderCap}) · ${mode}`;
    if (total > visible.length && !state.maxMode) return `${visible.length} of ${total} memories · ${mode} · hover max for ghost preview`;
    return `${shown} memories · ${mode} · hover to trace connections`;
  })();

  return (
    <AppContext.Provider value={{ state, dispatch }}>
      <a className="skip-link" href="#workspace">Skip to workspace</a>
      <TopBar daemonOk={state.daemonOk} daemonVersion={state.daemonVersion} />
      <main id="workspace">
        <section id="brain" className="brain-hero" aria-label="Brain visualization">
          <BrainStage
            ref={brainRef}
            atoms={visible}
            renderCap={renderCap}
            layout={state.layout}
            atomTotal={state.atomTotal}
            onSelect={setSelected}
            onHover={(atom) => { if (atom) setSelected(atom); }}
            caption={caption}
          />
        </section>
        <CommandStrip
          layout={state.layout}
          maxMode={state.maxMode}
          atomTotal={state.atomTotal}
          atomsLoaded={state.atoms.length}
          onLayoutChange={handleLayoutChange}
          onMaxMode={handleMaxMode}
          onSince={handleSince}
          onSelectAtom={handleSelectAndFocus}
          onRandom={handleRandom}
        />
        <section className="dashboard-grid" aria-label="Brain dashboard">
          <ActivityPanel live={live} onLiveToggle={setLive} />
          <InspectorPanel atom={selected} allAtoms={visible} />
          <AskPanel />
          <StatsPanel atoms={visible} atomTotal={state.atomTotal} />
        </section>
      </main>
      <footer>Kurultai runs locally. Your knowledge remains yours.</footer>
    </AppContext.Provider>
  );
}
