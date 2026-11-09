/**
 * ONE shell for Brain v2 (`?brain=v2`): the brain canvas is the whole page; Brain | Repos | Ontology are
 * modes of that canvas (segmented control in the single top bar and in Ctrl+K); Inspect, Hey, Pulse, Ask
 * and Store live in ONE resizable right-hand slide-over panel (width remembered). Selecting a node opens
 * the panel on Inspect (cited answer + why-connected). A menu holds settings and the 32-bit toggle.
 */
import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import type { Atom } from '../types';
import type { SearchResult, SearchState } from './gen/chrome/chrome';
import { PALETTE_COMMANDS, filterCommands, type PaletteCommand, type SurfaceKey, type ViewKey } from '../brain/v2/palette';
import type { ChainStep } from '../brain/v2/why';
import type { ThemePref } from '../brain/v2/theme';
import './v2shell.css';

export type PanelTab = 'inspect' | Exclude<SurfaceKey, 'ontology'>;
const TABS: PanelTab[] = ['inspect', 'hey', 'pulse', 'ask', 'store'];
const PANEL_W_KEY = 'kurultai.v2.panelW';
const VIEWS: { key: ViewKey; label: string }[] = [
  { key: 'brain', label: 'Brain' },
  { key: 'repos', label: 'Repos' },
  { key: 'ontology', label: 'Ontology' },
];

function readPanelW(): number {
  try {
    const n = Number(window.localStorage.getItem(PANEL_W_KEY));
    if (n >= 300 && n <= 1400) return n;
  } catch { /* storage blocked */ }
  return 400;
}

export interface V2ShellProps {
  brain: ReactNode;
  daemonStatus: 'online' | 'connecting' | 'offline';
  query: string;
  onQueryChange: (q: string) => void;
  results: SearchResult[] | null;
  searchState: SearchState;
  onSelectResult: (r: SearchResult) => void;
  onReset: () => void;
  /** Panel tab body (hey/pulse/ask/store). */
  renderSurface: (tab: SurfaceKey) => ReactNode;
  /** Views offered in the top-bar switch besides Brain, and their full-canvas bodies. */
  availableViews?: ViewKey[];
  renderView?: (view: Exclude<ViewKey, 'brain'>) => ReactNode;
  onOpenSettings: () => void;
  mode32: boolean;
  onMode32: (on: boolean) => void;
  themePref: ThemePref;
  onThemePref: (p: ThemePref) => void;
  reduceEffects: boolean;
  onReduceEffects: (on: boolean) => void;
  /** Reports how many px the open side panel covers (0 when closed / on narrow screens). */
  onPanelInset?: (px: number) => void;
  selected: Atom | null;
  onCloseSelected: () => void;
  tracing: boolean;
  onStartTrace: () => void;
  onCancelTrace: () => void;
  chain: ChainStep[] | null;
  chainMissing: boolean;
  onAsk: (atom: Atom, signal: AbortSignal) => Promise<string>;
  newBadge?: ReactNode;
  orConnected?: boolean;
  orMessage?: { ok: boolean; text: string } | null;
  onConnectOpenRouter?: () => void;
  mcpInfo?: { enabled: boolean; path: string } | null;
}

export function V2Shell(p: V2ShellProps) {
  const [view, setView] = useState<ViewKey>('brain');
  const [panel, setPanel] = useState<PanelTab | null>(null);
  const [width, setWidth] = useState<number>(() => readPanelW());
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [agentOpen, setAgentOpen] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [pq, setPq] = useState('');
  const [answer, setAnswer] = useState<{ id: string; text: string } | { id: string; error: string } | null>(null);
  const [asking, setAsking] = useState(false);
  const paletteInput = useRef<HTMLInputElement>(null);
  const views = VIEWS.filter((v) => v.key === 'brain' || p.availableViews?.includes(v.key));

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPaletteOpen((o) => !o);
      } else if (e.key === 'Escape') {
        setPaletteOpen(false);
        setMenuOpen(false);
        setPanel(null);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);
  useEffect(() => { if (paletteOpen) { setPq(''); setTimeout(() => paletteInput.current?.focus(), 0); } }, [paletteOpen]);
  useEffect(() => { setAnswer(null); if (p.selected) setPanel('inspect'); }, [p.selected?.id]); // selecting a node opens Inspect

  const matches = useMemo(() => filterCommands(PALETTE_COMMANDS.filter((c) => !c.view || c.view === 'brain' || views.some((v) => v.key === c.view)), pq), [pq, views.length]);
  const run = (c: PaletteCommand) => {
    setPaletteOpen(false);
    setMenuOpen(false);
    if (c.surface) setPanel(c.surface as PanelTab);
    else if (c.view) setView(c.view);
    else if (c.action === 'settings') p.onOpenSettings();
    else if (c.action === 'mode32') p.onMode32(!p.mode32);
    else if (c.action === 'reset') { setView('brain'); p.onReset(); }
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

  const startResize = useCallback((e: React.PointerEvent) => {
    e.preventDefault();
    const move = (ev: PointerEvent) => setWidth(Math.max(300, Math.min(window.innerWidth - 80, window.innerWidth - ev.clientX)));
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      setWidth((w) => { try { window.localStorage.setItem(PANEL_W_KEY, String(Math.round(w))); } catch { /* ignore */ } return w; });
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  }, []);

  const { onPanelInset } = p;
  useEffect(() => {
    const narrow = typeof window !== 'undefined' && window.innerWidth <= 640;
    onPanelInset?.(panel && !narrow ? width : 0);
  }, [panel, width, onPanelInset]);

  const statusLabel = p.daemonStatus === 'online' ? 'online' : p.daemonStatus === 'connecting' ? 'connecting' : 'offline';
  const sel = p.selected;
  const overlayView = view !== 'brain' ? p.renderView?.(view) : null;

  return (
    <div className="v2-app" data-view={view}>
      <div className="v2-brain" style={{ visibility: overlayView ? 'hidden' : 'visible' }}>{p.brain}</div>
      {overlayView && <div className="v2-view" key={view}>{overlayView}</div>}

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
        <div className="v2-seg" role="tablist" aria-label="View">
          {views.map((v) => (
            <button key={v.key} type="button" role="tab" aria-selected={view === v.key} className={view === v.key ? 'on' : ''} onClick={() => setView(v.key)}>
              {v.label}
            </button>
          ))}
        </div>
        <div className="v2-right">
          {p.newBadge}
          <button type="button" className="v2-panel-toggle" aria-pressed={panel !== null} onClick={() => setPanel((t) => (t ? null : sel ? 'inspect' : 'hey'))}>panel</button>
          <span className={`v2-dot v2-dot-${p.daemonStatus}`} role="status" aria-label={`daemon ${statusLabel}`} title={statusLabel} />
          <button type="button" className="v2-menu" aria-haspopup="menu" aria-expanded={menuOpen} onClick={() => setMenuOpen((o) => !o)}>
            menu
          </button>
          {menuOpen && (
            <div className="v2-menu-pop" role="menu">
              <div className="v2-themes" role="group" aria-label="Theme">
                {(['light', 'dark', 'system'] as ThemePref[]).map((t) => (
                  <button key={t} type="button" aria-pressed={p.themePref === t} className={p.themePref === t ? 'on' : ''} onClick={() => p.onThemePref(t)}>{t}</button>
                ))}
              </div>
              <label className="v2-switch" role="menuitemcheckbox" aria-checked={p.reduceEffects}>
                <input type="checkbox" checked={p.reduceEffects} onChange={(e) => p.onReduceEffects(e.target.checked)} />
                <span>reduce effects</span>
              </label>
              <label className="v2-switch" role="menuitemcheckbox" aria-checked={p.mode32}>
                <input type="checkbox" checked={p.mode32} onChange={(e) => p.onMode32(e.target.checked)} />
                <span>32-bit mode</span>
              </label>
              {p.onConnectOpenRouter && (
                <button type="button" role="menuitem" disabled={p.orConnected} onClick={() => { setMenuOpen(false); p.onConnectOpenRouter?.(); }}>
                  {p.orConnected ? 'OpenRouter connected' : 'Connect OpenRouter'}
                </button>
              )}
              {p.mcpInfo && (
                <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); setAgentOpen(true); }}>Connect an agent</button>
              )}
              {p.orMessage && <span className={p.orMessage.ok ? 'v2-meta' : 'v2-meta v2-err'} role="status">{p.orMessage.text}</span>}
              <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); p.onOpenSettings(); }}>Access settings</button>
              <button type="button" role="menuitem" onClick={() => { setMenuOpen(false); setPaletteOpen(true); }}>Command palette</button>
              <span className="v2-meta">ctrl+k opens the palette</span>
            </div>
          )}
        </div>
      </div>

      {panel && (
        <aside className="v2-panel" style={{ width }} aria-label="Side panel">
          <div className="v2-grip" role="separator" aria-orientation="vertical" aria-label="Resize panel" onPointerDown={startResize} />
          <header className="v2-panel-head">
            <div className="v2-tabs" role="tablist" aria-label="Panel">
              {TABS.filter((t) => t !== 'inspect' || sel).map((t) => (
                <button key={t} type="button" role="tab" aria-selected={panel === t} className={panel === t ? 'on' : ''} onClick={() => setPanel(t)}>{t}</button>
              ))}
            </div>
            <button type="button" aria-label="Close panel" onClick={() => setPanel(null)}>close</button>
          </header>
          <div className="v2-panel-body" key={panel}>
            {panel === 'inspect' && sel ? (
              <div className="v2-inspect">
                <strong>{sel.title}</strong>
                <p className="v2-meta">{sel.source} · {sel.tier}{sel.tags.length ? ` · ${sel.tags.slice(0, 4).join(', ')}` : ''}</p>
                {sel.summary && <p className="v2-sum">{sel.summary}</p>}
                <div className="v2-actions">
                  <button type="button" disabled={asking} onClick={() => void ask(sel)}>{asking ? 'asking' : 'explain'}</button>
                  {p.tracing
                    ? <button type="button" onClick={p.onCancelTrace}>cancel trace</button>
                    : <button type="button" onClick={p.onStartTrace}>trace to another node</button>}
                  <button type="button" onClick={p.onCloseSelected}>deselect</button>
                </div>
                {p.tracing && <p className="v2-meta">Pick a second node to see why they are connected.</p>}
                {answer && answer.id === sel.id && (
                  <section className="v2-answer" aria-live="polite">
                    {'text' in answer
                      ? <><p>{answer.text}</p><p className="v2-meta">cited: {sel.file || sel.source_id || sel.id}</p></>
                      : <p className="v2-err">{answer.error}</p>}
                  </section>
                )}
                {p.chain && (
                  <section className="v2-chain" aria-label="Why connected">
                    <h3>why connected</h3>
                    <ol>
                      {p.chain.map((st) => (
                        <li key={st.id}>{st.via.length > 0 && <span className="v2-via">via {st.via.slice(0, 3).join(', ')}</span>}{st.title}</li>
                      ))}
                    </ol>
                  </section>
                )}
                {p.chainMissing && <p className="v2-meta">No path over the loaded edges. Load a higher tier to widen the graph.</p>}
              </div>
            ) : panel !== 'inspect' ? p.renderSurface(panel) : null}
          </div>
        </aside>
      )}

      {agentOpen && p.mcpInfo && (
        <div className="v2-scrim" onClick={(e) => { if (e.target === e.currentTarget) setAgentOpen(false); }}>
          <div className="v2-palette" role="dialog" aria-label="Connect an agent">
            {p.mcpInfo.enabled ? (() => {
              const url = `${window.location.origin}${p.mcpInfo.path}`;
              return (
                <>
                  <span className="v2-meta">MCP endpoint</span>
                  <pre className="v2-meta">{url}</pre>
                  <span className="v2-meta">Claude Code</span>
                  <pre className="v2-meta">{`claude mcp add --transport http kurultai ${url} --header "Authorization: Bearer <your token>"`}</pre>
                  <span className="v2-meta">Codex / generic JSON</span>
                  <pre className="v2-meta">{JSON.stringify({ url, headers: { Authorization: 'Bearer <your token>' } }, null, 2)}</pre>
                  <span className="v2-meta">Replace {'<your token>'} with your own token. It is never shown here.</span>
                  <a className="v2-meta" href="https://github.com/duketopceo/kurultai/blob/main/docs/AGENT_CONNECTION_PROMPT.md" target="_blank" rel="noreferrer">Agent connection guide</a>
                </>
              );
            })() : (
              <span className="v2-meta">MCP over HTTP is off. Set KURULTAI_MCP_HTTP_SECRET on the server to enable it.</span>
            )}
            <button type="button" onClick={() => setAgentOpen(false)}>Close</button>
          </div>
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
