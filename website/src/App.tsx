import { useReducer, useEffect, useCallback, useRef, useState } from 'react';
import { reducer, initialState, AppContext } from './state';
import { fetchStatus, fetchGraph } from './api';
import { TopBar } from './components/TopBar';
import { BrainStage, BrainStageHandle } from './components/BrainStage';
import { CommandStrip } from './components/CommandStrip';
import { ActivityPanel } from './components/ActivityPanel';
import { InspectorPanel } from './components/InspectorPanel';
import { AskPanel } from './components/AskPanel';
import { StatsPanel } from './components/StatsPanel';
import type { Atom, LayoutMode, LoadTier } from './types';
import { LOAD_TIER_CAPS } from './types';

const dbg = (...args: unknown[]) => console.debug('[kurultai:app]', ...args);

function dateValue(atom: Atom): number {
  const d = Date.parse(atom.indexed_at || atom.last_accessed_at || '');
  return Number.isFinite(d) ? d : 0;
}

export function App() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const [selected, setSelected] = useState<Atom | null>(null);
  const [live, setLive] = useState(true);
  const [loadMsg, setLoadMsg] = useState('Loading memories…');
  const [loadTier, setLoadTier] = useState<LoadTier>('low');
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

  const loadAtoms = useCallback(async (tier: LoadTier = loadTier) => {
    abortRef.current?.abort();
    const ac = new AbortController();
    abortRef.current = ac;
    try {
      dbg('fetchGraph start, tier:', tier);
      setLoadMsg(`Loading ${tier}…`);
      const t0 = performance.now();
      const all = await fetchGraph(ac.signal);
      const cap = LOAD_TIER_CAPS[tier];
      const atoms = all.slice(0, cap);
      const elapsed = (performance.now() - t0).toFixed(0);
      dbg(`fetchGraph done: ${atoms.length}/${all.length} atoms in ${elapsed}ms (tier: ${tier})`);
      setLoadMsg(`${atoms.length} memories · ${tier}`);
      dispatch({ type: 'SET_ATOMS', atoms, total: all.length });
    } catch (e) {
      if ((e as Error)?.name !== 'AbortError') {
        dbg('fetchGraph error:', e);
        setLoadMsg('Failed to load memories — check daemon');
      }
    }
  }, [loadTier]);

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

  useEffect(() => { loadAtoms(loadTier); }, [loadTier]);

  const handleLayoutChange = (mode: LayoutMode) => {
    dispatch({ type: 'SET_LAYOUT', layout: mode });
    try {
      localStorage.setItem('kurultai-layout', mode);
    } catch { /* best-effort persistence */ }
  };

  const handleLoadTier = (tier: LoadTier) => {
    setLoadTier(tier);
    localStorage.setItem('kurultai-load-tier', tier);
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
  const renderCap = LOAD_TIER_CAPS[loadTier];

  const caption = (() => {
    const shown = Math.min(visible.length, renderCap);
    const total = Math.max(state.atomTotal, state.atoms.length);
    if (total > shown) return `${shown} of ${total} memories · ${loadTier} · hover to trace`;
    return `${shown} memories · ${loadTier} · hover to trace connections`;
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
          loadTier={loadTier}
          atomTotal={state.atomTotal}
          atomsLoaded={state.atoms.length}
          onLayoutChange={handleLayoutChange}
          onLoadTier={handleLoadTier}
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
