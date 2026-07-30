import type { Atom } from '../types';

interface Props {
  atoms: Atom[];
  atomTotal: number;
}

export function StatsPanel({ atoms, atomTotal }: Props) {
  const hot = atoms.filter((a) => a.tier === 'hot').length;
  const warm = atoms.filter((a) => a.tier === 'warm').length;
  const cold = atoms.filter((a) => a.tier === 'cold').length;
  const trusted = atoms.filter((a) => a.score >= 0.8).length;

  return (
    <section className="panel signal-panel" aria-labelledby="signal-heading">
      <div className="panel-heading">
        <p className="eyebrow">Signal</p>
        <h2 id="signal-heading">Brain state</h2>
      </div>
      <dl className="stats">
        <div><dt>memories</dt><dd id="stat-atoms">{atomTotal > atoms.length ? `${atoms.length} / ${atomTotal}` : atoms.length || '—'}</dd></div>
        <div><dt>hot / warm / cold</dt><dd id="stat-tiers">{atoms.length ? `${hot} / ${warm} / ${cold}` : '—'}</dd></div>
        <div><dt>trusted</dt><dd id="stat-trusted">{atoms.length ? trusted : '—'}</dd></div>
      </dl>
    </section>
  );
}
