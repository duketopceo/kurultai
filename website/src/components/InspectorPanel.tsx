import { useEffect, useState } from 'react';
import type { Atom } from '../types';
import { touchAtom, openFile } from '../api';

interface Props {
  atom: Atom | null;
  allAtoms: Atom[];
}

function buildRelationshipCount(atom: Atom, allAtoms: Atom[]): number {
  return allAtoms.filter((b) => {
    if (b.id === atom.id) return false;
    const sharedTags = atom.tags.filter((t) => b.tags.includes(t));
    return sharedTags.length > 0 || (atom.source && atom.source === b.source);
  }).length;
}

function hexToCss(hex: number): string {
  return `#${(hex >>> 0).toString(16).padStart(6, '0')}`;
}

function lerpHex(a: number, b: number, t: number): number {
  const ar = (a >> 16) & 0xff, ag = (a >> 8) & 0xff, ab = a & 0xff;
  const br = (b >> 16) & 0xff, bg = (b >> 8) & 0xff, bb = b & 0xff;
  return (
    (Math.round(ar + (br - ar) * t) << 16) |
    (Math.round(ag + (bg - ag) * t) << 8) |
    Math.round(ab + (bb - ab) * t)
  );
}

function hashLabel(label: string): number {
  let h = 0;
  for (let i = 0; i < label.length; i++) h = ((h << 5) - h + label.charCodeAt(i)) | 0;
  return (h >>> 0) / 0xffffffff;
}

const PURPLE_STOPS = [0x4c1d95, 0x5b21b6, 0x6d28d9, 0x7c3aed, 0x8b5cf6, 0xa78bfa, 0xc4b5fd];

function labelGradientCss(label: string): string {
  const t = hashLabel(label);
  const scaled = t * (PURPLE_STOPS.length - 1);
  const i = Math.floor(scaled);
  const f = scaled - i;
  const a = PURPLE_STOPS[i];
  const b = PURPLE_STOPS[Math.min(i + 1, PURPLE_STOPS.length - 1)];
  const mid = lerpHex(a, b, f);
  const deep = lerpHex(mid, 0x2e1065, 0.45);
  const light = lerpHex(mid, 0xffffff, 0.28);
  return `linear-gradient(135deg, ${hexToCss(deep)}, ${hexToCss(mid)} 55%, ${hexToCss(light)})`;
}

export function InspectorPanel({ atom, allAtoms }: Props) {
  const [enriched, setEnriched] = useState<Atom | null>(null);

  useEffect(() => {
    if (!atom) { setEnriched(null); return; }
    setEnriched(atom);
    if (!atom.lean) return;
    touchAtom(atom.id).then((full) => {
      if (full) setEnriched(full);
    });
  }, [atom]);

  const display = enriched ?? atom;

  return (
    <section className="panel inspector-panel" aria-labelledby="inspector-heading">
      <div className="panel-heading">
        <p className="eyebrow">Focus</p>
        <h2 id="inspector-heading">Node inspector</h2>
      </div>
      <div id="node-inspector" className="node-inspector" aria-live="polite">
        {!display ? (
          <p className="empty-state">Hover or select a memory node to reveal its place in the lattice.</p>
        ) : (
          <>
            <h3 className="inspector-title">{display.title}</h3>
            <p className="inspector-summary">{display.summary}</p>
            <div className="node-metrics">
              {([
                ['weight', display.score.toFixed(2)],
                ['recency', display.last_accessed_at || display.indexed_at || 'unknown'],
                ['relations', buildRelationshipCount(display, allAtoms)],
                ['tier', display.tier],
              ] as [string, string | number][]).map(([name, value]) => (
                <span key={name} className="metric">
                  {name} <b>{value}</b>
                </span>
              ))}
            </div>
            {display.tags.length > 0 && (
              <div className="inspector-tags">
                {display.tags.map((tag) => (
                  <span key={tag} className="tag-chip" style={{ background: labelGradientCss(tag) }}>
                    #{tag}
                  </span>
                ))}
              </div>
            )}
            {display.file && (
              <button
                className="open-button"
                type="button"
                onClick={() => openFile(display.file)}
              >
                Open source file ↗
              </button>
            )}
          </>
        )}
      </div>
    </section>
  );
}
