import { useRef, useState, useEffect } from 'react';
import type { Atom, LayoutMode } from '../types';
import { searchAtoms } from '../api';

interface Props {
  layout: LayoutMode;
  maxMode: boolean;
  atomTotal: number;
  atomsLoaded: number;
  onLayoutChange: (mode: LayoutMode) => void;
  onMaxMode: (enabled: boolean) => void;
  onSince: (since: number) => void;
  onSelectAtom: (atom: Atom) => void;
  onRandom: () => void;
}

export function CommandStrip({
  layout, maxMode, atomTotal, atomsLoaded,
  onLayoutChange, onMaxMode, onSince, onSelectAtom, onRandom
}: Props) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Atom[]>([]);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [timelineValue, setTimelineValue] = useState(100);
  const [playing, setPlaying] = useState(false);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  useEffect(() => {
    if (!query.trim()) { setResults([]); setDropdownOpen(false); return; }
    abortRef.current?.abort();
    const ac = new AbortController();
    abortRef.current = ac;
    searchAtoms(query, ac.signal).then((r) => {
      setResults(r);
      setDropdownOpen(r.length > 0);
    }).catch(() => {});
    return () => ac.abort();
  }, [query]);

  useEffect(() => {
    if (timerRef.current) clearInterval(timerRef.current);
    if (!playing) return;
    timerRef.current = setInterval(() => {
      setTimelineValue((v) => {
        const next = v >= 100 ? 0 : v + 1;
        onSince(next / 100);
        return next;
      });
    }, 80);
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [playing, onSince]);

  const handleTimelineChange = (v: number) => {
    setTimelineValue(v);
    onSince(v / 100);
  };

  const timelineLabel = timelineValue >= 100 ? 'all time' : `${timelineValue}%`;

  return (
    <section className="command-strip" aria-label="Brain controls">
      <label className="search-control" htmlFor="brain-search">
        <span aria-hidden="true">⌕</span>
        <input
          id="brain-search"
          type="search"
          autoComplete="off"
          placeholder="Search your memory"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onBlur={() => setTimeout(() => setDropdownOpen(false), 150)}
        />
        <kbd>⌘ K</kbd>
      </label>
      {dropdownOpen && (
        <div id="search-dropdown" className="search-dropdown" role="listbox" aria-label="Search results">
          {results.slice(0, 6).map((atom) => (
            <button
              key={atom.id}
              type="button"
              className="search-result"
              role="option"
              onMouseDown={() => { onSelectAtom(atom); setDropdownOpen(false); setQuery(''); }}
            >
              <strong>{atom.title}</strong>
              <span>{atom.summary}</span>
            </button>
          ))}
        </div>
      )}
      <div className="timeline-control">
        <button
          id="timeline-play"
          className="icon-button"
          type="button"
          aria-label={playing ? 'Pause timeline' : 'Play timeline'}
          aria-pressed={playing}
          onClick={() => setPlaying((p) => !p)}
        >
          {playing ? '⏸' : '▶'}
        </button>
        <label htmlFor="timeline-range">Memory horizon</label>
        <input
          id="timeline-range"
          type="range"
          min="0"
          max="100"
          value={timelineValue}
          onChange={(e) => handleTimelineChange(Number(e.target.value))}
        />
        <output id="timeline-output">{timelineLabel}</output>
      </div>

      <div className="layout-switcher" role="group" aria-label="Brain layout">
        {([
          ['lattice', 'regions'],
          ['force', 'force'],
          ['galaxy', 'solar'],
        ] as const).map(([label, mode]) => (
          <button
            key={mode}
            className={`quiet-button layout-toggle${layout === mode ? ' is-active' : ''}`}
            type="button"
            aria-pressed={layout === mode}
            aria-label={`Switch to ${label} layout`}
            onClick={() => onLayoutChange(mode)}
          >
            {label}
          </button>
        ))}
      </div>

      <button
        id="max-toggle"
        className={`quiet-button max-toggle${maxMode ? ' is-active' : ''}`}
        type="button"
        aria-pressed={maxMode}
        aria-label={maxMode
          ? 'Exit max mode and reload the standard memory subset'
          : 'Load up to 10,000 memories via tiered graph'}
        title="Hover: ghost preview · Click: load up to 10,000 memories (hot → warm → cold)"
        onClick={() => onMaxMode(!maxMode)}
        onMouseEnter={() => {
          if (!maxMode) {
            // Ghost preview triggered externally via onMaxMode flow
          }
        }}
      >
        max
        {maxMode && atomTotal > atomsLoaded && (
          <small style={{ marginLeft: 4, opacity: 0.7 }}>{atomsLoaded}/{atomTotal}</small>
        )}
      </button>

      <button
        id="random-pick"
        className="quiet-button"
        type="button"
        aria-label="Jump to a random memory"
        title="Random: zoom to a random memory and highlight its connections"
        onClick={onRandom}
      >
        random
      </button>
    </section>
  );
}
