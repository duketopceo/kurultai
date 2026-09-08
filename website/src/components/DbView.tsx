import { useCallback, useEffect, useState } from 'react';
import { fetchDbTable, fetchStatus, type DbRow } from '../api';
import { TopBar } from './TopBar';

type Table = 'atoms' | 'links';
type Lane = '' | 'trusted' | 'quarantine';
type Tier = '' | 'hot' | 'warm' | 'cold';
const PAGE = 100;

/** Friendly column presentation — raw keys stay in the row for sorting. */
const COLS: Record<Table, { key: string; label: string; sortable: boolean; hide?: boolean }[]> = {
  atoms: [
    { key: 'title', label: 'Title', sortable: true },
    { key: 'source', label: 'Source', sortable: true },
    { key: 'tags_json', label: 'Tags', sortable: false },
    { key: 'trust_lane', label: 'Status', sortable: true },
    { key: 'corpus_tier', label: 'Tier', sortable: true },
    { key: 'indexed_at', label: 'Added', sortable: true },
    { key: 'last_accessed_at', label: 'Last seen', sortable: true },
    { key: 'quarantine_reason', label: 'Why held', sortable: false, hide: true },
    { key: 'id', label: 'ID', sortable: true, hide: true },
  ],
  links: [
    { key: 'a_title', label: 'From', sortable: true },
    { key: 'b_title', label: 'To', sortable: true },
    { key: 'shared_tags', label: 'Shared tags', sortable: false },
    { key: 'strength', label: 'Strength', sortable: true },
  ],
};

/** Aliases so machiney values read like English. */
const ALIAS: Record<string, Record<string, string>> = {
  trust_lane: { trusted: 'Live', quarantine: 'Held' },
  corpus_tier: { hot: 'Hot', warm: 'Warm', cold: 'Cold' },
  source: {
    notes: 'Notes', code: 'Code', repos: 'Repos', pond: 'Pond',
    hey: 'Hey board', mcp: 'MCP', github: 'GitHub',
  },
};

function fmtTime(iso: unknown): string {
  if (!iso || typeof iso !== 'string') return '—';
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso.slice(0, 16);
  return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}

function fmtTags(v: unknown): string {
  if (typeof v !== 'string') return Array.isArray(v) ? v.join(', ') : '';
  try {
    const arr = JSON.parse(v);
    return Array.isArray(arr) ? arr.join(', ') : v;
  } catch { return v; }
}

function cell(key: string, v: unknown): string {
  if (v === null || v === undefined || v === '') return '—';
  const s = String(v);
  if (key === 'id' || key === 'a' || key === 'b') return s.length > 12 ? `${s.slice(0, 8)}…` : s;
  if (key === 'tags_json' || key === 'shared_tags') return fmtTags(v);
  if (key.endsWith('_at')) return fmtTime(v);
  const alias = ALIAS[key]?.[s];
  return alias ?? s;
}

/** Read-only raw store browser: /ui/#/db — view, sort, filter. No writes. */
export function DbPage() {
  const [table, setTable] = useState<Table>('atoms');
  const [q, setQ] = useState('');
  const [qLive, setQLive] = useState('');
  const [lane, setLane] = useState<Lane>('');
  const [tier, setTier] = useState<Tier>('');
  const [sort, setSort] = useState('indexed_at');
  const [dir, setDir] = useState<'asc' | 'desc'>('desc');
  const [offset, setOffset] = useState(0);
  const [rows, setRows] = useState<DbRow[]>([]);
  const [count, setCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [daemonVersion, setDaemonVersion] = useState('');

  useEffect(() => {
    fetchStatus()
      .then((s) => setDaemonVersion(s.version ?? ''))
      .catch(() => setDaemonVersion(''));
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetchDbTable(table, {
        sort, dir, q: qLive || undefined,
        lane: lane || undefined, tier: tier || undefined,
        limit: PAGE, offset,
      });
      setRows(res.rows);
      setCount(res.count);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'fetch failed');
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [table, sort, dir, qLive, lane, tier, offset]);

  useEffect(() => { void load(); }, [load]);

  const switchTable = (t: Table) => {
    setTable(t);
    setOffset(0);
    setLane('');
    setTier('');
    setSort(t === 'atoms' ? 'indexed_at' : 'strength');
    setDir('desc');
  };

  const sortKey = (key: string) => (key === 'a_title' ? 'a' : key === 'b_title' ? 'b' : key);

  const clickSort = (col: (typeof COLS.atoms)[number]) => {
    if (!col.sortable) return;
    const key = sortKey(col.key);
    if (sort === key) setDir(dir === 'desc' ? 'asc' : 'desc');
    else { setSort(key); setDir('desc'); }
    setOffset(0);
  };

  const hasFilters = qLive !== '' || lane !== '' || tier !== '';
  const clearAll = () => { setQ(''); setQLive(''); setLane(''); setTier(''); setOffset(0); };
  const visibleCols = COLS[table].filter((c) => !c.hide);

  const chip = (label: string, active: boolean, onClick: () => void) => (
    <button
      key={label}
      className={`db-chip${active ? ' active' : ''}`}
      aria-pressed={active}
      onClick={() => { onClick(); setOffset(0); }}
    >
      {label}
    </button>
  );

  return (
    <div className="db-page">
      <TopBar daemonOk={daemonVersion !== ''} daemonVersion={daemonVersion} />
      <header className="db-header">
        <h1>Store <small className="db-ro">read-only</small></h1>
        <div className="db-controls">
          <div className="db-tabs" role="tablist">
            {(['atoms', 'links'] as Table[]).map((t) => (
              <button
                key={t}
                role="tab"
                aria-selected={table === t}
                className={table === t ? 'active' : ''}
                onClick={() => switchTable(t)}
              >
                {t === 'atoms' ? 'Memories' : 'Connections'}
              </button>
            ))}
          </div>
          <form
            className="db-filter"
            onSubmit={(e) => { e.preventDefault(); setQLive(q); setOffset(0); }}
          >
            <input
              type="search"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder={table === 'atoms' ? 'Search title, source, or tags…' : 'Search a node or tag…'}
              aria-label="Filter rows"
            />
          </form>
          <button className="db-chip" onClick={() => void load()} title="Refresh">↻</button>
          {hasFilters && (
            <button className="db-chip" onClick={clearAll}>Clear</button>
          )}
          <span className="db-count" aria-live="polite">
            {loading ? '…' : `${count} shown${offset ? ` · ${offset} skipped` : ''}`}
          </span>
          <div className="db-pager">
            <button disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>‹ prev</button>
            <button disabled={count < PAGE} onClick={() => setOffset(offset + PAGE)}>next ›</button>
          </div>
        </div>
        {table === 'atoms' && (
          <div className="db-chips">
            <span className="db-chip-label">Status:</span>
            {chip('All', lane === '', () => setLane(''))}
            {chip('Live', lane === 'trusted', () => setLane('trusted'))}
            {chip('Held', lane === 'quarantine', () => setLane('quarantine'))}
            <span className="db-chip-label">Tier:</span>
            {chip('Any', tier === '', () => setTier(''))}
            {chip('Hot', tier === 'hot', () => setTier('hot'))}
            {chip('Warm', tier === 'warm', () => setTier('warm'))}
            {chip('Cold', tier === 'cold', () => setTier('cold'))}
          </div>
        )}
      </header>
      {error && <p className="db-error" role="alert">{error}</p>}
      <div className="db-table-wrap">
        <table className="db-table">
          <thead>
            <tr>
              {visibleCols.map((c) => (
                <th
                  key={c.key}
                  onClick={() => clickSort(c)}
                  className={c.sortable ? 'sortable' : ''}
                >
                  {c.label}{sort === sortKey(c.key) ? (dir === 'desc' ? ' ▾' : ' ▴') : ''}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr key={i}>
                {visibleCols.map((c) => (
                  <td key={c.key} title={String(row[c.key] ?? '')}>{cell(c.key, row[c.key])}</td>
                ))}
              </tr>
            ))}
            {!loading && rows.length === 0 && (
              <tr><td colSpan={visibleCols.length} className="db-empty">
                {hasFilters ? 'nothing matches — try clearing filters' : 'no rows'}
              </td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
