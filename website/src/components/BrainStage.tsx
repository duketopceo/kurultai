import { useEffect, useRef, useCallback, useState, forwardRef, useImperativeHandle } from 'react';
import { BrainView } from '../brain/BrainView';
import { buildTagLinks } from '../brain/v2/links';
import { brainVersion } from '../brain/v2/flag';
import { OntologyBoard } from './OntologyBoard';
import { reportFps } from '../perf';
import type { Atom, LayoutMode, OntologyResponse } from '../types';

const dbg = (...args: unknown[]) => console.debug('[kurultai:brain]', ...args);

/** Surface shared by BrainView (v1) and BrainV2. */
export interface BrainLike {
  setData(atoms: Atom[], links: { a: string; b: string; strength: number }[]): void;
  setOntology(doc: OntologyResponse): void;
  setLayout(mode: LayoutMode): void;
  focusAtom(atom: Atom): void;
  zoomToAtom(atom: Atom): void;
  randomConnection(): void;
  /** v2 only */
  findPath?(from: string, to: string): string[] | null;
  setPath?(ids: string[]): void;
  resetView?(): void;
  setFresh?(ids: Iterable<string>): void;
  setPixelMode?(on: boolean): void;
  metrics(): { fps: number; nodes: number; renderedEdges: number };
  dispose(): void;
}

export interface BrainStageHandle {
  focusAtom: (atom: Atom) => void;
  randomConnection: () => void;
  /** v2: client-side shortest path over loaded edges (no server traverse API yet). */
  findPath: (from: string, to: string) => string[] | null;
  setPath: (ids: string[]) => void;
  resetView: () => void;
  setFresh: (ids: Iterable<string>) => void;
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
  onOntologyChanged: (onto: OntologyResponse) => void;
  caption: string;
  /** v2: 32-bit render mode. */
  pixel?: boolean;
}

export const BrainStage = forwardRef<BrainStageHandle, Props>(function BrainStage({ atoms, renderCap, layout, ontology, atomTotal, onSelect, onHover, onOntologyChanged, caption, pixel }, ref) {
  const hostRef = useRef<HTMLDivElement>(null);
  const brainRef = useRef<BrainLike | null>(null);
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
    findPath: (from, to) => brainRef.current?.findPath?.(from, to) ?? null,
    setPath: (ids) => brainRef.current?.setPath?.(ids),
    resetView: () => brainRef.current?.resetView?.(),
    setFresh: (ids) => brainRef.current?.setFresh?.(ids),
  }));

  const handleTooltip = useCallback((atom: Atom | null, x: number, y: number) => {
    if (!atom) { setTooltip(null); return; }
    setTooltip({ atom, x, y });
  }, []);

  useEffect(() => {
    if (!hostRef.current) return;
    let cancelled = false;
    let brain: BrainLike | null = null;
    const host = hostRef.current;
    const common = {
      reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
      onHoverAtom: (atom: Atom, x: number, y: number) => { onHover(atom); handleTooltip(atom, x, y); },
      onSelectAtom: (atom: Atom) => { onSelect(atom); },
      onClearHover: () => { onHover(null); setTooltip(null); },
      onReady: () => { dbg('BrainView ready'); setReady(true); },
      onError: (msg: string) => { dbg('BrainView error:', msg); console.error('[BrainView]', msg); },
    };
    const attach = (b: BrainLike) => {
      brain = b;
      brainRef.current = b;
      // Console handle for scene measurement: `__kurultaiBrain.metrics()`.
      (window as unknown as { __kurultaiBrain?: BrainLike }).__kurultaiBrain = b;
    };
    if (brainVersion() === 'v2') {
      dbg('BrainV2 init (code-split)');
      // Dynamic import keeps v2 (and its OrbitControls) out of the v1 bundle.
      import('../brain/v2/BrainV2').then(({ BrainV2: V2 }) => {
        if (cancelled) return;
        attach(new V2(host, common));
      }).catch((e) => common.onError(String(e)));
    } else {
      dbg('BrainView init');
      attach(new BrainView(host, { theme: 'dark', onZoomOut: () => {}, ...common }));
    }
    return () => {
      cancelled = true;
      dbg('BrainView dispose');
      delete (window as unknown as { __kurultaiBrain?: BrainLike }).__kurultaiBrain;
      brain?.dispose();
      brainRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (ready) brainRef.current?.setPixelMode?.(!!pixel);
  }, [pixel, ready]);

  useEffect(() => {
    if (!brainRef.current || !ready) return;
    const shown = atoms.slice(0, renderCap);
    dbg(`setData: ${shown.length} atoms (cap ${renderCap})`);
    const t0 = performance.now();
    const links = buildTagLinks(shown);
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
    // The 2D board owns ontology mode; BrainView keeps running brain layout underneath.
    dbg('setLayout:', layout);
    brainRef.current.setLayout(layout === 'ontology' ? 'brain' : layout);
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
              <stop offset="0%" stopColor="#e8a33d" stopOpacity={0.12} />
              <stop offset="100%" stopColor="#0c0d11" stopOpacity={0} />
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
            <circle cx="400" cy="200" r="12" fill="#e8a33d" filter="url(#centreGlow)" />
          </svg>
        </div>
      )}
      <div id="brain-canvas" ref={hostRef} aria-label="3D memory graph" style={{ width: '100%', height: '100%' }} />
      {layout === 'ontology' ? (
        <div className="onto-board-host">
          <OntologyBoard
            ontology={ontology}
            atoms={atoms}
            onSelect={onSelect}
            onOntologyChanged={onOntologyChanged}
          />
        </div>
      ) : (
        <>
          <div className="brain-overlay" aria-hidden="true">
            <span>DRAG / ORBIT</span>
            <span>SCROLL / ZOOM</span>
          </div>
          <BrainHud getBrain={() => brainRef.current} />
        </>
      )}
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

/** Live scene readout — FPS · synapses · nodes, polled from BrainView.metrics(). */
function BrainHud({ getBrain }: { getBrain: () => BrainLike | null }) {
  const [stats, setStats] = useState<{ fps: number; nodes: number; synapses: number } | null>(null);
  useEffect(() => {
    const id = window.setInterval(() => {
      const m = getBrain()?.metrics();
      if (m && m.nodes > 0) {
        setStats({ fps: m.fps, nodes: m.nodes, synapses: m.renderedEdges });
        reportFps(m.fps);
      }
    }, 1000);
    return () => window.clearInterval(id);
  }, [getBrain]);
  if (!stats) return null;
  return (
    <div className="brain-hud" aria-hidden="true">
      <span>{Math.round(stats.fps)} FPS</span>
      <span>{stats.nodes.toLocaleString()} NEURONS</span>
      <span>{stats.synapses.toLocaleString()} SYNAPSES</span>
    </div>
  );
}
