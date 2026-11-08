/** Optional Repos view: one repo as a floorplan (folders = rooms, files = tiles). */
import { useEffect, useMemo, useState } from 'react';
import { fetchGraph } from '../api';
import { codeLatticeOf } from '../repoLattice';
import { countCodeRepos } from '../components/RepoBrain';
import type { Atom } from '../types';
import { entriesFromAtoms, layoutFloorplan, type FloorEntry } from './floorplan';

const W = 1000;
const H = 620;

async function manifestFor(repo: string): Promise<FloorEntry[] | null> {
  // Optional baked data (e.g. codebase-memory-mcp architecture export). 404 is normal.
  try {
    const r = await fetch(`/ui/repo-manifests/${encodeURIComponent(repo.replace('/', '__'))}.json`);
    if (!r.ok) return null;
    const j = (await r.json()) as FloorEntry[];
    return Array.isArray(j) ? j : null;
  } catch {
    return null;
  }
}

export function RepoFloorplan({ onClose }: { onClose: () => void }) {
  const [atoms, setAtoms] = useState<Atom[] | null>(null);
  const [repo, setRepo] = useState<string | null>(null);
  const [baked, setBaked] = useState<FloorEntry[] | null>(null);
  const [hover, setHover] = useState<string | null>(null);

  useEffect(() => {
    fetchGraph({ limit: 20000 }).then(setAtoms).catch(() => setAtoms([]));
  }, []);
  const repos = useMemo(() => (atoms ? countCodeRepos(atoms) : []), [atoms]);
  useEffect(() => { if (!repo && repos[0]) setRepo(repos[0].name); }, [repos, repo]);
  useEffect(() => { setBaked(null); if (repo) void manifestFor(repo).then(setBaked); }, [repo]);

  const plan = useMemo(() => {
    if (!atoms || !repo) return null;
    const entries = baked ?? entriesFromAtoms(atoms.filter((a) => codeLatticeOf(a) === repo), repo);
    return layoutFloorplan(entries, W, H);
  }, [atoms, repo, baked]);

  return (
    <div className="v2-repos" role="dialog" aria-label="Repos floorplan">
      <header>
        <strong>Repos</strong>
        <select aria-label="Repository" value={repo ?? ''} onChange={(e) => setRepo(e.target.value)}>
          {repos.map((r) => <option key={r.name} value={r.name}>{r.name} ({r.count})</option>)}
        </select>
        <span className="v2-meta">{baked ? 'baked manifest' : 'from indexed code atoms'}{hover ? ` · ${hover}` : ''}</span>
        <button type="button" onClick={onClose}>close</button>
      </header>
      {atoms && repos.length === 0 && <p className="v2-meta">No code repositories indexed. Enable a github/code source.</p>}
      {plan && plan.tiles.length > 0 && (
        <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Repository floorplan">
          {plan.rooms.map((r) => (
            <g key={`r:${r.path}`}>
              <rect x={r.x} y={r.y} width={r.w} height={r.h} className="fp-room" />
              {r.w > 60 && r.h > 20 && <text x={r.x + 4} y={r.y + 11} className="fp-label">{r.name}</text>}
            </g>
          ))}
          {plan.tiles.map((t) => (
            <rect
              key={`t:${t.path}`} x={t.x} y={t.y} width={t.w} height={t.h} className="fp-tile"
              style={{ fillOpacity: 0.12 + 0.6 * t.activity }}
              onMouseEnter={() => setHover(t.path)} onMouseLeave={() => setHover(null)}
            ><title>{`${t.path} · ${t.size}`}</title></rect>
          ))}
        </svg>
      )}
    </div>
  );
}
