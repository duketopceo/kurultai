import { useEffect, useRef, useCallback, useState, forwardRef, useImperativeHandle } from 'react';
import { BrainView } from '../brain/BrainView';
import type { Atom, LayoutMode, OntologyResponse } from '../types';

const dbg = (...args: unknown[]) => console.debug('[kurultai:brain]', ...args);
const MAX_LINKS_PER_TAG = 30;

export interface BrainStageHandle {
  focusAtom: (atom: Atom) => void;
  randomConnection: () => void;
}

interface TooltipState {
  atom: Atom;
  x: number;
  y: number;
}

interface Props {
  atoms: Atom[];
  renderCap: number;
  layout: LayoutMode;
  ontology: OntologyResponse;
  atomTotal: number;
  onSelect: (atom: Atom) => void;
  onHover: (atom: Atom | null) => void;
  caption: string;
}

export const BrainStage = forwardRef<BrainStageHandle, Props>(function BrainStage({ atoms, renderCap, layout, ontology, atomTotal, onSelect, onHover, caption }, ref) {
  const hostRef = useRef<HTMLDivElement>(null);
  const brainRef = useRef<BrainView | null>(null);
  const [tooltip, setTooltip] = useState<TooltipState | null>(null);
  const [showFallback, setShowFallback] = useState(true);
  const [ready, setReady] = useState(false);

  useImperativeHandle(ref, () => ({
    focusAtom: (atom: Atom) => {
      if (!brainRef.current) return;
      brainRef.current.focusAtom(atom);
      brainRef.current.zoomToAtom(atom);
    },
    randomConnection: () => {
      if (!brainRef.current) return;
      brainRef.current.randomConnection();
    },
  }));

  const handleTooltip = useCallback((atom: Atom | null, x: number, y: number) => {
    if (!atom) { setTooltip(null); return; }
    setTooltip({ atom, x, y });
  }, []);

  useEffect(() => {
    if (!hostRef.current) return;
    dbg('BrainView init');
    const brain = new BrainView(hostRef.current, {
      theme: 'dark',
      reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
      onHoverAtom: (atom, x, y) => { onHover(atom); handleTooltip(atom, x, y); },
      onSelectAtom: (atom) => { onSelect(atom); },
      onClearHover: () => { onHover(null); setTooltip(null); },
      onZoomOut: () => {},
      onReady: () => { dbg('BrainView ready'); setReady(true); },
      onError: (msg) => { dbg('BrainView error:', msg); console.error('[BrainView]', msg); },
    });
    brainRef.current = brain;
    return () => { dbg('BrainView dispose'); brain.dispose(); brainRef.current = null; };
  }, []);

  useEffect(() => {
    if (!brainRef.current || !ready) return;
    const shown = atoms.slice(0, renderCap);
    dbg(`setData: ${shown.length} atoms (cap ${renderCap})`);
    const t0 = performance.now();
    const links = buildLinks(shown);
    dbg(`buildLinks: ${links.length} links in ${(performance.now() - t0).toFixed(0)}ms`);
    brainRef.current.setData(shown, links);
  }, [atoms, renderCap, ready]);

  useEffect(() => {
    const noGraph = atoms.slice(0, renderCap).length === 0;
    const ontoReady = layout === 'ontology' && ontology.entities.length > 0;
    setShowFallback(noGraph && !ontoReady);
  }, [atoms, renderCap, layout, ontology]);

  useEffect(() => {
    if (!brainRef.current || !ready) return;
    brainRef.current.setOntology(ontology);
  }, [ontology, ready]);

  useEffect(() => {
    if (!brainRef.current || !ready) return;
    dbg('setLayout:', layout);
    brainRef.current.setLayout(layout);
  }, [layout, ready]);

  return (
    <div
      id="brain-stage"
      className="brain-stage"
      tabIndex={0}
      role="application"
      aria-label="Interactive three-dimensional memory lattice."
    >
      {showFallback && (
        <div id="lattice-fallback" className="lattice-fallback" aria-hidden="true">
          <svg className="fallback-svg" viewBox="0 0 800 400" preserveAspectRatio="xMidYMid slice">
            <radialGradient id="bgGlow" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor="#a855f7" stopOpacity={0.12} />
              <stop offset="100%" stopColor="#050508" stopOpacity={0} />
            </radialGradient>
            <rect width="800" height="400" fill="url(#bgGlow)" />
            {[[400,200,200,100],[400,200,600,100],[400,200,600,300],[400,200,200,300],
              [200,100,600,100],[600,100,600,300],[600,300,200,300],[200,300,200,100],
              [400,200,400,60],[400,200,400,340],[400,200,100,200],[400,200,700,200]].map(([x1,y1,x2,y2],i) => (
              <line key={i} className="fallback-edge" x1={x1} y1={y1} x2={x2} y2={y2} />
            ))}
            {[[100,60],[700,60],[700,340],[100,340],[400,60],[400,340],[100,200],[700,200]].map(([cx,cy],i) => (
              <circle key={i} className="fallback-node" cx={cx} cy={cy} r="5" />
            ))}
            {[[200,100],[600,100],[600,300],[200,300]].map(([cx,cy],i) => (
              <circle key={i} className="fallback-node-white" cx={cx} cy={cy} r="7" />
            ))}
            <filter id="centreGlow" x="-80%" y="-80%" width="260%" height="260%">
              <feGaussianBlur stdDeviation="5" result="b" />
              <feMerge><feMergeNode in="b" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
            <circle cx="400" cy="200" r="12" fill="#a855f7" filter="url(#centreGlow)" />
          </svg>
        </div>
      )}
      <div id="brain-canvas" ref={hostRef} aria-label="3D memory graph" style={{ width: '100%', height: '100%' }} />
      <div className="brain-overlay" aria-hidden="true">
        <span>DRAG / ORBIT</span>
        <span>SCROLL / ZOOM</span>
      </div>
      {tooltip && (
        <div
          id="node-tooltip"
          className="node-tooltip"
          role="status"
          aria-live="polite"
          style={{ left: tooltip.x + 12, top: tooltip.y + 12 }}
        >
          <strong>{tooltip.atom.title}</strong>
          <span>{tooltip.atom.tags.length} tags · {tooltip.atom.tier}</span>
        </div>
      )}
      <div className="brain-caption">
        <span className="pulse-ring" aria-hidden="true" />
        <span id="brain-caption">{caption}</span>
      </div>
    </div>
  );
});

function buildLinks(atoms: Atom[]) {
  const tagIndex = new Map<string, string[]>();
  atoms.forEach((a) => {
    a.tags.forEach((t) => {
      const list = tagIndex.get(t) ?? [];
      list.push(a.id);
      tagIndex.set(t, list);
    });
  });
  const seen = new Set<string>();
  const links: { a: string; b: string; strength: number }[] = [];
  tagIndex.forEach((ids) => {
    // cap per-tag pairs to avoid O(n²) explosion on dense tags like "code" or "rs"
    const limit = Math.min(ids.length, MAX_LINKS_PER_TAG);
    for (let i = 0; i < limit; i++) {
      for (let j = i + 1; j < limit; j++) {
        const key = ids[i] < ids[j] ? `${ids[i]}:${ids[j]}` : `${ids[j]}:${ids[i]}`;
        if (!seen.has(key)) { seen.add(key); links.push({ a: ids[i], b: ids[j], strength: 1 }); }
      }
    }
  });
  return links;
}
