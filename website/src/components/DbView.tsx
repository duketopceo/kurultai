import { useCallback, useEffect, useState } from 'react';
import { fetchDbTable, fetchStatus, type DbRow } from '../api';
import { TopBar } from './TopBar';

type Table = 'atoms' | 'links';
const PAGE = 100;

const SORTABLE: Record<Table, string[]> = {
  atoms: ['id', 'source', 'title', 'trust_lane', 'corpus_tier', 'indexed_at', 'last_accessed_at'],
  links: ['a', 'b', 'strength'],
};

function cell(v: unknown): string {
  if (v === null || v === undefined) return '';
  if (Array.isArray(v)) return v.join(', ');
  return String(v);
}

/** Read-only raw store browser: /ui/#/db — view, sort, filter. No writes. */
export function DbPage() {
  const [table, setTable] = useState<Table>('atoms');
  const [q, setQ] = useState('');
  const [qLive, setQLive] = useState('');
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
      const res = await fetchDbTable(table, { sort, dir, q: qLive || undefined, limit: PAGE, offset });
      setRows(res.rows);
      setCount(res.count);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'fetch failed');
      setRows([]);
    } finally {
      setLoading(false);
    }
  }, [table, sort, dir, qLive, offset]);

  useEffect(() => { void load(); }, [load]);

  const switchTable = (t: Table) => {
    setTable(t);
    setOffset(0);
    setSort(t === 'atoms' ? 'indexed_at' : 'strength');
    setDir('desc');
  };

  const clickSort = (col: string) => {
    if (!SORTABLE[table].includes(col)) return;
    if (sort === col) setDir(dir === 'desc' ? 'asc' : 'desc');
    else { setSort(col); setDir('desc'); }
    setOffset(0);
  };

  const cols: string[] = rows[0] ? Object.keys(rows[0]) :
    (table === 'atoms'
      ? ['id', 'source', 'title', 'tags_json', 'trust_lane', 'corpus_tier', 'indexed_at', 'last_accessed_at', 'quarantine_reason']
      : ['a_title', 'b_title', 'shared_tags', 'strength']);

  return (
    <div className="db-page">
      <TopBar daemonOk={daemonVersion !== ''} daemonVersion={daemonVersion} />
      <header className="db-header">
        <h1>Store browser <small className="db-ro">read-only</small></h1>
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
                {t}
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
              placeholder={table === 'atoms' ? 'filter title / source / tags…' : 'filter node or tag…'}
              aria-label="Filter rows"
            />
          </form>
          <span className="db-count" aria-live="polite">
            {loading ? '…' : `${count} row${count === 1 ? '' : 's'} @ ${offset}`}
          </span>
          <div className="db-pager">
            <button disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>‹ prev</button>
            <button disabled={count < PAGE} onClick={() => setOffset(offset + PAGE)}>next ›</button>
          </div>
        </div>
      </header>
      {error && <p className="db-error" role="alert">{error}</p>}
      <div className="db-table-wrap">
        <table className="db-table">
          <thead>
            <tr>
              {cols.map((c) => (
                <th
                  key={c}
                  onClick={() => clickSort(c)}
                  className={SORTABLE[table].includes(c) ? 'sortable' : ''}
                >
                  {c}{sort === c ? (dir === 'desc' ? ' ▾' : ' ▴') : ''}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr key={i}>
                {cols.map((c) => <td key={c}>{cell(row[c])}</td>)}
              </tr>
            ))}
            {!loading && rows.length === 0 && (
              <tr><td colSpan={cols.length} className="db-empty">no rows</td></tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
