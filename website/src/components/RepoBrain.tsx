import { useEffect, useRef, useState } from 'react';
import { BrainView } from '../brain/BrainView';
import { fetchGraph } from '../api';
import type { Atom } from '../types';

const dbg = (...args: unknown[]) => console.debug('[kurultai:repo-brain]', ...args);

function repoOf(atom: Atom): string {
  return atom.source_id.split('/')[0] ?? '';
}

export function RepoBrainPage() {
  const hash = window.location.hash;
  const repoName = hash.startsWith('#/repo/') ? decodeURIComponent(hash.slice(7)) : null;

  if (repoName) return <RepoBrainView repo={repoName} />;
  return <RepoListPage />;
}

function RepoListPage() {
  const [repos, setRepos] = useState<{ name: string; count: number }[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchGraph().then((atoms) => {
      const counts = new Map<string, number>();
      atoms.forEach((a) => {
        const r = repoOf(a);
        if (r) counts.set(r, (counts.get(r) ?? 0) + 1);
      });
      const sorted = Array.from(counts.entries())
        .map(([name, count]) => ({ name, count }))
        .sort((a, b) => b.count - a.count);
      setRepos(sorted);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  return (
    <div className="repo-list-page">
      <div className="repo-list-header">
        <a href="#/" className="back-link">← brain</a>
        <h2>Repo Brains <span className="beta-badge">beta</span></h2>
        <p className="repo-list-sub">Each repository mapped as its own memory lattice.</p>
      </div>
      {loading && <p className="repo-list-loading">Loading repositories…</p>}
      <div className="repo-grid">
        {repos.map(({ name, count }) => (
          <a
            key={name}
            href={`#/repo/${encodeURIComponent(name)}`}
            className="repo-card"
          >
            <strong>{name}</strong>
            <span>{count} memories</span>
          </a>
        ))}
      </div>
    </div>
  );
}

function RepoBrainView({ repo }: { repo: string }) {
  const hostRef = useRef<HTMLDivElement>(null);
  const brainRef = useRef<BrainView | null>(null);
  const [status, setStatus] = useState(`Loading ${repo}…`);
  const [selected, setSelected] = useState<Atom | null>(null);

  useEffect(() => {
    if (!hostRef.current) return;
    dbg('RepoBrainView init for', repo);
    const brain = new BrainView(hostRef.current, {
      theme: 'dark',
      reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
      onHoverAtom: (atom) => setSelected(atom),
      onSelectAtom: (atom) => setSelected(atom),
      onClearHover: () => {},
      onZoomOut: () => {},
      onReady: () => {
        dbg('ready');
        fetchGraph().then((all) => {
          const atoms = all.filter((a) => repoOf(a) === repo);
          dbg(`${atoms.length} atoms for repo ${repo}`);
          setStatus(`${atoms.length} memories`);
          const links = buildLinks(atoms);
          brain.setData(atoms, links);
        }).catch((e) => { setStatus('Failed to load'); dbg('error', e); });
      },
      onError: (msg) => { setStatus('WebGL error'); console.error('[RepoBrain]', msg); },
    });
    brainRef.current = brain;
    return () => { brain.dispose(); brainRef.current = null; };
  }, [repo]);

  return (
    <div className="repo-brain-page">
      <div className="repo-brain-topbar">
        <a href="#/repos" className="back-link">← repos</a>
        <span className="repo-brain-title">{repo} <span className="beta-badge">beta</span></span>
        <span className="repo-brain-count">{status}</span>
      </div>
      <div className="repo-brain-canvas" ref={hostRef} />
      {selected && (
        <div className="repo-brain-inspector">
          <strong>{selected.title}</strong>
          <span>{selected.source_id}</span>
          <span>{selected.tags.join(', ')}</span>
        </div>
      )}
    </div>
  );
}

function buildLinks(atoms: Atom[]) {
  const tagIndex = new Map<string, string[]>();
  atoms.forEach((a) => a.tags.forEach((t) => {
    const list = tagIndex.get(t) ?? [];
    list.push(a.id);
    tagIndex.set(t, list);
  }));
  const seen = new Set<string>();
  const links: { a: string; b: string; strength: number }[] = [];
  tagIndex.forEach((ids) => {
    const limit = Math.min(ids.length, 30);
    for (let i = 0; i < limit; i++) {
      for (let j = i + 1; j < limit; j++) {
        const key = ids[i] < ids[j] ? `${ids[i]}:${ids[j]}` : `${ids[j]}:${ids[i]}`;
        if (!seen.has(key)) { seen.add(key); links.push({ a: ids[i], b: ids[j], strength: 1 }); }
      }
    }
  });
  return links;
}
