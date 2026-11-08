/**
 * Minimal chrome for Brain v2 (`?brain=v2`): the brain is the page. Only a
 * search field, a status dot and one menu remain; Hey, Pulse, Ontology, Ask,
 * Store, Repos and Access sit behind a single drawer / command palette (Ctrl+K).
 */
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import type { Atom } from '../types';
import type { SearchResult, SearchState } from './gen/chrome/chrome';
import { PALETTE_COMMANDS, filterCommands, type PaletteCommand, type SurfaceKey } from '../brain/v2/palette';
import type { ChainStep } from '../brain/v2/why';
import './v2shell.css';

export interface V2ShellProps {
  brain: ReactNode;
  daemonStatus: 'online' | 'connecting' | 'offline';
  query: string;
  onQueryChange: (q: string) => void;
  results: SearchResult[] | null;
  searchState: SearchState;
  onSelectResult: (r: SearchResult) => void;
  onReset: () => void;
  renderSurface: (tab: SurfaceKey) => ReactNode;
  onOpenSettings: () => void;
  onOpenRepos?: () => void;
  selected: Atom | null;
  onCloseSelected: () => void;
  tracing: boolean;
  onStartTrace: () => void;
  onCancelTrace: () => void;
  chain: ChainStep[] | null;
  chainMissing: boolean;
  onAsk: (atom: Atom, signal: AbortSignal) => Promise<string>;
  newBadge?: ReactNode;
}

export function V2Shell(p: V2ShellProps) {
  const [drawer, setDrawer] = useState<SurfaceKey | null>(null);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [pq, setPq] = useState('');
  const [answer, setAnswer] = useState<{ id: string; text: string } | { id: string; error: string } | null>(null);
  const [asking, setAsking] = useState(false);
  const paletteInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPaletteOpen((o) => !o);
      } else if (e.key === 'Escape') {
        setPaletteOpen(false);
        setMenuOpen(false);
        setDrawer(null);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  useEffect(() => { if (paletteOpen) { setPq(''); setTimeout(() => paletteInput.current?.focus(), 0); } }, [paletteOpen]);
  useEffect(() => { setAnswer(null); }, [p.selected?.id]);

  const matches = useMemo(() => filterCommands(PALETTE_COMMANDS, pq), [pq]);
  const run = (c: PaletteCommand) => {
    setPaletteOpen(false);
    setMenuOpen(false);
    if (c.surface) setDrawer(c.surface);
    else if (c.action === 'settings') p.onOpenSettings();
    else if (c.action === 'repos') p.onOpenRepos?.();
    else if (c.action === 'reset') p.onReset();
  };

  const ask = async (atom: Atom) => {
    setAsking(true);
    const ac = new AbortController();
    try {
      setAnswer({ id: atom.id, text: await p.onAsk(atom, ac.signal) });
    } catch (e) {
      setAnswer({ id: atom.id, error: e instanceof Error ? e.message : String(e) });
    } finally {
      setAsking(false);
    }
  };

  const statusLabel = p.daemonStatus === 'online' ? 'online' : p.daemonStatus === 'connecting' ? 'connecting' : 'offline';

  return (
    <div className="v2-app">
      <div className="v2-brain">{p.brain}</div>

      <div className="v2-top">
        <div className="v2-search">
          <input
            type="search"
            aria-label="Search memory"
            placeholder="search memory"
            value={p.query}
            onChange={(e) => p.onQueryChange(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Escape') p.onReset(); }}
          />
          {(p.query || p.results) && (
            <button type="button" className="v2-clear" aria-label="Clear search and reset view" onClick={p.onReset}>
              clear
            </button>
          )}
          {p.searchState === 'loading' && <span className="v2-meta">searching</span>}
          {p.results && p.results.length > 0 && (
            <ul className="v2-results" role="listbox">
              {p.results.slice(0, 8).map((r) => (
                <li key={r.id}>
                  <button type="button" role="option" aria-selected="false" onClick={() => p.onSelectResult(r)}>
                    {r.title}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="v2-right">
          {p.newBadge}
          <span className={`v2-dot v2-dot-${p.daemonStatus}`} role="status" aria-label={`daemon ${statusLabel}`} title={statusLabel} />
          <button type="button" className="v2-menu" aria-haspopup="menu" aria-expanded={menuOpen} onClick={() => setMenuOpen((o) => !o)}>
            menu
          </button>
          {menuOpen && (
            <div className="v2-menu-pop" role="menu">
              {PALETTE_COMMANDS.map((c) => (
                <button key={c.id} type="button" role="menuitem" onClick={() => run(c)}>{c.label}</button>
              ))}
              <span className="v2-meta">ctrl+k opens the palette</span>
            </div>
          )}
        </div>
      </div>

      {p.selected && (
        <aside className="v2-panel" aria-label="Selected atom">
          <header>
            <strong>{p.selected.title}</strong>
            <button type="button" aria-label="Close panel" onClick={p.onCloseSelected}>close</button>
          </header>
          <p className="v2-meta">{p.selected.source} · {p.selected.tier}{p.selected.tags.length ? ` · ${p.selected.tags.slice(0, 4).join(', ')}` : ''}</p>
          {p.selected.summary && <p className="v2-sum">{p.selected.summary}</p>}
          <div className="v2-actions">
            <button type="button" disabled={asking} onClick={() => void ask(p.selected!)}>{asking ? 'asking' : 'explain'}</button>
            {p.tracing
              ? <button type="button" onClick={p.onCancelTrace}>cancel trace</button>
              : <button type="button" onClick={p.onStartTrace}>trace to another node</button>}
          </div>
          {p.tracing && <p className="v2-meta">Pick a second node to see why they are connected.</p>}
          {answer && answer.id === p.selected.id && (
            <section className="v2-answer" aria-live="polite">
              {'text' in answer
                ? <><p>{answer.text}</p><p className="v2-meta">cited: {p.selected.file || p.selected.source_id || p.selected.id}</p></>
                : <p className="v2-err">{answer.error}</p>}
            </section>
          )}
          {p.chain && (
            <section className="v2-chain" aria-label="Why connected">
              <h3>why connected</h3>
              <ol>
                {p.chain.map((s) => (
                  <li key={s.id}>{s.via.length > 0 && <span className="v2-via">via {s.via.slice(0, 3).join(', ')}</span>}{s.title}</li>
                ))}
              </ol>
            </section>
          )}
          {p.chainMissing && <p className="v2-meta">No path over the loaded edges. Load a higher tier to widen the graph.</p>}
        </aside>
      )}

      {drawer && (
        <div className="v2-drawer" role="dialog" aria-label={drawer}>
          <header>
            <span className="v2-meta">{drawer}</span>
            <button type="button" aria-label="Close drawer" onClick={() => setDrawer(null)}>close</button>
          </header>
          <div className="v2-drawer-body" key={drawer}>{p.renderSurface(drawer)}</div>
        </div>
      )}

      {paletteOpen && (
        <div className="v2-scrim" onClick={(e) => { if (e.target === e.currentTarget) setPaletteOpen(false); }}>
          <div className="v2-palette" role="dialog" aria-label="Command palette">
            <input
              ref={paletteInput}
              aria-label="Command"
              placeholder="go to..."
              value={pq}
              onChange={(e) => setPq(e.target.value)}
              onKeyDown={(e) => { if (e.key === 'Enter' && matches[0]) run(matches[0]); }}
            />
            <ul>
              {matches.map((c) => (
                <li key={c.id}><button type="button" onClick={() => run(c)}>{c.label}<span className="v2-meta">{c.hint}</span></button></li>
              ))}
              {matches.length === 0 && <li className="v2-meta">no match</li>}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
