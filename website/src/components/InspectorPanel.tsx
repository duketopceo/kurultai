import { useEffect, useState } from 'react';
import type { Atom, OntologyResponse } from '../types';
import { touchAtom, openFile, fetchOntology, promoteAtomToOntology } from '../api';

interface OntologyEdgeView {
  key: string;
  rel: string;
  other: string;
}

const CLASS_OPTIONS: { id: string; label: string }[] = [
  { id: 'class:note', label: 'Note' },
  { id: 'class:code', label: 'Code' },
  { id: 'class:decision', label: 'Decision' },
  { id: 'class:person', label: 'Person' },
  { id: 'class:system', label: 'System' },
  { id: 'class:memory', label: 'Memory' },
];

function approvedLinksForAtom(atomId: string, onto: OntologyResponse): OntologyEdgeView[] {
  const byId = new Map(onto.entities.map((e) => [e.id, e]));
  const related = new Set<string>([atomId]);
  for (const e of onto.entities) {
    if (e.id === atomId || e.atom_id === atomId) related.add(e.id);
  }
  const out: OntologyEdgeView[] = [];
  for (const link of onto.links) {
    if (link.status && link.status !== 'approved') continue;
    const fromHit = related.has(link.from_id);
    const toHit = related.has(link.to_id);
    if (!fromHit && !toHit) continue;
    const otherId = fromHit ? link.to_id : link.from_id;
    out.push({
      key: link.id || `${link.from_id}:${link.rel}:${link.to_id}`,
      rel: link.rel,
      other: byId.get(otherId)?.name ?? otherId,
    });
  }
  return out;
}

function suggestClassId(atom: Atom): string {
  const tags = atom.tags.map((t) => t.toLowerCase());
  const src = (atom.source || '').toLowerCase();
  if (tags.some((t) => t.includes('decision') || t === 'adr') || src.includes('decision')) {
    return 'class:decision';
  }
  if (tags.some((t) => t.includes('person') || t.includes('people')) || src.includes('person')) {
    return 'class:person';
  }
  if (tags.some((t) => t.includes('system') || t.includes('service')) || src.includes('system')) {
    return 'class:system';
  }
  if (
    tags.some((t) => t.includes('code') || t.includes('repo')) ||
    /^(code|github|git|repos?)\b/i.test(src)
  ) {
    return 'class:code';
  }
  return 'class:note';
}

interface Props {
  atom: Atom | null;
  allAtoms: Atom[];
  onOntologyChanged?: (onto: OntologyResponse) => void;
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

export function InspectorPanel({ atom, allAtoms, onOntologyChanged }: Props) {
  const [enriched, setEnriched] = useState<Atom | null>(null);
  const [ontoLinks, setOntoLinks] = useState<OntologyEdgeView[]>([]);
  const [linkedClassIds, setLinkedClassIds] = useState<string[]>([]);
  const [classId, setClassId] = useState('class:note');
  const [busy, setBusy] = useState(false);
  const [promoMsg, setPromoMsg] = useState<string | null>(null);
  const [promoErr, setPromoErr] = useState<string | null>(null);

  useEffect(() => {
    if (!atom) { setEnriched(null); return; }
    setEnriched(atom);
    setClassId(suggestClassId(atom));
    setPromoMsg(null);
    setPromoErr(null);
    if (!atom.lean) return;
    touchAtom(atom.id).then((full) => {
      if (full) setEnriched(full);
    });
  }, [atom]);

  useEffect(() => {
    if (!atom) {
      setOntoLinks([]);
      setLinkedClassIds([]);
      return;
    }
    const ac = new AbortController();
    fetchOntology(ac.signal)
      .then((onto) => {
        if (ac.signal.aborted) return;
        setOntoLinks(approvedLinksForAtom(atom.id, onto));
        const entId = `ent:${atom.id}`;
        const classes = onto.links
          .filter(
            (l) =>
              (!l.status || l.status === 'approved') &&
              l.rel === 'instance_of' &&
              (l.from_id === entId || l.from_id === atom.id),
          )
          .map((l) => l.to_id);
        setLinkedClassIds(classes);
      })
      .catch(() => {
        if (!ac.signal.aborted) {
          setOntoLinks([]);
          setLinkedClassIds([]);
        }
      });
    return () => ac.abort();
  }, [atom]);

  const display = enriched ?? atom;
  const linkedToSelected = linkedClassIds.includes(classId);

  const handlePromote = async () => {
    if (!display || linkedToSelected) return;
    setBusy(true);
    setPromoErr(null);
    setPromoMsg(null);
    try {
      const res = await promoteAtomToOntology(display.id, classId);
      const onto = await fetchOntology();
      setOntoLinks(approvedLinksForAtom(display.id, onto));
      const entId = `ent:${display.id}`;
      setLinkedClassIds(
        onto.links
          .filter(
            (l) =>
              (!l.status || l.status === 'approved') &&
              l.rel === 'instance_of' &&
              (l.from_id === entId || l.from_id === display.id),
          )
          .map((l) => l.to_id),
      );
      onOntologyChanged?.(onto);
      setPromoMsg(`Linked as ${CLASS_OPTIONS.find((c) => c.id === classId)?.label ?? classId} (${res.entity_id})`);
    } catch (e) {
      setPromoErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel chrome-panel inspector-panel" aria-labelledby="inspector-heading">
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
            {ontoLinks.length > 0 && (
              <ul className="inspector-links">
                {ontoLinks.map((edge) => (
                  <li key={edge.key}>
                    <span className="inspector-rel">{edge.rel}</span> {edge.other}
                  </li>
                ))}
              </ul>
            )}
            <div className="inspector-promote chrome-promote">
              <p className="chrome-promote-label">Map into ontology</p>
              <div className="chrome-promote-row">
                <label className="sr-only" htmlFor="ontology-class">Class</label>
                <select
                  id="ontology-class"
                  className="chrome-select"
                  value={classId}
                  onChange={(e) => setClassId(e.target.value)}
                  disabled={busy}
                >
                  {CLASS_OPTIONS.map((c) => (
                    <option key={c.id} value={c.id}>{c.label}</option>
                  ))}
                </select>
                <button
                  type="button"
                  className="chrome-btn chrome-btn-primary"
                  onClick={handlePromote}
                  disabled={busy || linkedToSelected}
                >
                  {busy ? 'Linking…' : linkedToSelected ? 'Already linked' : 'Link to class'}
                </button>
              </div>
              {promoMsg && <p className="chrome-promote-ok">{promoMsg}</p>}
              {promoErr && <p className="chrome-promote-err" role="alert">{promoErr}</p>}
            </div>
            {display.file && (
              <button
                className="open-button chrome-btn"
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
