// store-view.tsx
import {
  useEffect,
  useId,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import "./store-view.css";

export type StoreTable = "atoms" | "sources" | "threads" | "proposals";
export type StoreTier = "hot" | "warm" | "cold";
export type StoreLane = "trusted" | "quarantine";
export type StoreColumn = "title" | "tier" | "lane" | "source" | "indexed_at";
export type SortDirection = "asc" | "desc";

export interface StoreRow {
  id: string | number;
  title: string | null;
  tier: StoreTier | null;
  lane: StoreLane | null;
  source: string | null;
  indexed_at: string | null;
  [field: string]: unknown;
}

export interface StoreFilters {
  search: string;
  lane: StoreLane | "all";
  tier: StoreTier | "all";
  sortColumn: StoreColumn;
  sortDirection: SortDirection;
  /** Zero-based page. The data provider must use a fixed limit of 50. */
  page: number;
}

export interface StoreViewProps {
  table: StoreTable;
  rows: StoreRow[];
  total: number;
  filters: StoreFilters;
  /** Apply the patch and fetch matching data; table changes use this callback too. */
  onFilter: (patch: Partial<StoreFilters> & { table?: StoreTable }) => void;
  /** Apply sorting and reset the page to zero. */
  onSort: (column: StoreColumn, direction: SortDirection) => void;
  /** Fetch this zero-based page with limit 50. */
  onPage: (page: number) => void;
  onOpenRow: (row: StoreRow) => void;
  loading: boolean;
  error: string | Error | null;
}

const TABLES: StoreTable[] = ["atoms", "sources", "threads", "proposals"];
const COLUMNS: StoreColumn[] = ["title", "tier", "lane", "source", "indexed_at"];
const LIMIT = 50;
const number = new Intl.NumberFormat("en-US");

function dateLabel(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : date.toISOString().slice(0, 19).replace("T", " ");
}

function fieldLabel(value: unknown): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(
      value,
      (_, item) => (typeof item === "bigint" ? String(item) : item),
      2,
    ) ?? String(value);
  } catch {
    return String(value);
  }
}

export default function StoreView({
  table,
  rows,
  total,
  filters,
  onFilter,
  onSort,
  onPage,
  onOpenRow,
  loading,
  error,
}: StoreViewProps) {
  const id = useId();
  const [search, setSearch] = useState(filters.search);
  const [filtersExpanded, setFiltersExpanded] = useState(false);
  const [selected, setSelected] = useState<StoreRow | null>(null);
  const searchTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const filterCallback = useRef(onFilter);
  const closeButton = useRef<HTMLButtonElement>(null);
  const rowTrigger = useRef<HTMLButtonElement | null>(null);
  const gridRegion = useRef<HTMLDivElement>(null);

  useEffect(() => {
    filterCallback.current = onFilter;
  }, [onFilter]);

  useEffect(() => {
    setSearch(filters.search);
    if (searchTimer.current) clearTimeout(searchTimer.current);
  }, [filters.search, table]);

  useEffect(
    () => () => {
      if (searchTimer.current) clearTimeout(searchTimer.current);
    },
    [],
  );

  useEffect(() => {
    setSelected(null);
    gridRegion.current?.scrollTo({ top: 0 });
  }, [
    table,
    filters.page,
    filters.search,
    filters.lane,
    filters.tier,
    filters.sortColumn,
    filters.sortDirection,
  ]);

  useEffect(() => {
    if (selected) closeButton.current?.focus();
  }, [selected?.id]);

  const errorMessage = error instanceof Error ? error.message : error;
  const hasError = error !== null;
  const busy = loading || hasError;
  const start = filters.page * LIMIT;
  const activeFilters =
    Number(filters.lane !== "all") + Number(filters.tier !== "all");
  const selectedCurrent =
    selected && (rows.find((row) => row.id === selected.id) ?? selected);

  function updateSearch(value: string) {
    setSearch(value);
    if (searchTimer.current) clearTimeout(searchTimer.current);
    searchTimer.current = setTimeout(() => {
      filterCallback.current({ search: value, page: 0 });
    }, 300);
  }

  function changeFilter(patch: Partial<StoreFilters> & { table?: StoreTable }) {
    if (searchTimer.current) clearTimeout(searchTimer.current);
    onFilter({ search, ...patch, page: 0 });
  }

  function sort(column: StoreColumn) {
    onSort(
      column,
      filters.sortColumn === column && filters.sortDirection === "asc"
        ? "desc"
        : "asc",
    );
  }

  function openRow(row: StoreRow, trigger: HTMLButtonElement | null) {
    rowTrigger.current = trigger;
    setSelected(row);
    onOpenRow(row);
  }

  function closeInspector() {
    setSelected(null);
    if (rowTrigger.current?.isConnected) rowTrigger.current.focus();
    else gridRegion.current?.focus();
  }

  return (
    <section className="store-view" aria-labelledby={`${id}-heading`}>
      <header className="store-header">
        <div className="store-identity">
          <span className="store-mark" aria-hidden="true">▤</span>
          <h1 id={`${id}-heading`}>Store</h1>
          <span className="store-route">#/db</span>
        </div>

        <div className="store-tabs" role="group" aria-label="Database table">
          {TABLES.map((name) => (
            <button
              key={name}
              type="button"
              aria-pressed={table === name}
              onClick={() => {
                if (table !== name) changeFilter({ table: name });
              }}
            >
              {name}
            </button>
          ))}
        </div>
      </header>

      <div className="store-toolbar">
        <label className="store-search">
          <span className="store-sr-only">Search {table}</span>
          <svg
            width="14"
            height="14"
            viewBox="0 0 16 16"
            fill="none"
            aria-hidden="true"
          >
            <circle cx="6.75" cy="6.75" r="4.75" />
            <path d="m10.25 10.25 3.75 3.75" />
          </svg>
          <input
            type="search"
            value={search}
            placeholder={`search ${table}…`}
            onChange={(event) => updateSearch(event.target.value)}
            autoComplete="off"
            spellCheck={false}
          />
        </label>

        <button
          className="store-filter-disclosure"
          type="button"
          aria-expanded={filtersExpanded}
          aria-controls={`${id}-filters`}
          onClick={() => setFiltersExpanded((value) => !value)}
        >
          <span>filters{activeFilters > 0 ? ` · ${activeFilters}` : ""}</span>
          <span aria-hidden="true">{filtersExpanded ? "−" : "+"}</span>
        </button>

        <div
          id={`${id}-filters`}
          className={`store-filters${filtersExpanded ? " is-expanded" : ""}`}
        >
          <label className="store-control">
            <span>lane</span>
            <select
              value={filters.lane}
              onChange={(event) =>
                changeFilter({ lane: event.target.value as StoreFilters["lane"] })
              }
            >
              <option value="all">all</option>
              <option value="trusted">trusted</option>
              <option value="quarantine">quarantine</option>
            </select>
          </label>

          <label className="store-control">
            <span>tier</span>
            <select
              value={filters.tier}
              onChange={(event) =>
                changeFilter({ tier: event.target.value as StoreFilters["tier"] })
              }
            >
              <option value="all">all</option>
              <option value="hot">hot</option>
              <option value="warm">warm</option>
              <option value="cold">cold</option>
            </select>
          </label>

          <div className="store-sort">
            <label className="store-control">
              <span>sort</span>
              <select
                value={filters.sortColumn}
                onChange={(event) =>
                  onSort(event.target.value as StoreColumn, filters.sortDirection)
                }
              >
                {COLUMNS.map((column) => (
                  <option key={column} value={column}>{column}</option>
                ))}
              </select>
            </label>
            <button
              type="button"
              className="store-direction"
              aria-label={`Sort ${filters.sortDirection === "asc" ? "descending" : "ascending"}`}
              onClick={() =>
                onSort(
                  filters.sortColumn,
                  filters.sortDirection === "asc" ? "desc" : "asc",
                )
              }
            >
              <span aria-hidden="true">
                {filters.sortDirection === "asc" ? "↑" : "↓"}
              </span>
              {filters.sortDirection}
            </button>
          </div>
        </div>
      </div>

      <div className="store-workspace">
        <div className="store-browser">
          <div
            ref={gridRegion}
            className="store-grid-scroll"
            role="region"
            aria-label={`${table} data, horizontally scrollable`}
            aria-busy={loading}
            tabIndex={0}
          >
            <table className="store-grid">
              <caption className="store-sr-only">
                {table} records. Select a title to inspect a row. Indexed timestamps
                are displayed in UTC.
              </caption>
              <colgroup>
                <col className="store-col-title" />
                <col className="store-col-tier" />
                <col className="store-col-lane" />
                <col className="store-col-source" />
                <col className="store-col-date" />
              </colgroup>
              <thead>
                <tr>
                  {COLUMNS.map((column) => (
                    <th
                      key={column}
                      scope="col"
                      className={column === "indexed_at" ? "store-numeric" : ""}
                      aria-sort={
                        filters.sortColumn === column
                          ? filters.sortDirection === "asc"
                            ? "ascending"
                            : "descending"
                          : "none"
                      }
                    >
                      <button type="button" onClick={() => sort(column)}>
                        {column}
                        <span
                          className={
                            filters.sortColumn === column
                              ? "store-sort-active"
                              : "store-sort-idle"
                          }
                          aria-hidden="true"
                        >
                          {filters.sortColumn === column
                            ? filters.sortDirection === "asc" ? "↑" : "↓"
                            : "↕"}
                        </span>
                      </button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {loading ? (
                  Array.from({ length: 14 }, (_, index) => (
                    <tr key={index} aria-hidden="true" className="store-skeleton-row">
                      {COLUMNS.map((column, cell) => (
                        <td key={column}>
                          <span
                            className="store-skeleton"
                            style={{
                              "--skeleton-width": `${55 + ((index + cell) % 4) * 10}%`,
                            } as CSSProperties}
                          />
                        </td>
                      ))}
                    </tr>
                  ))
                ) : hasError ? (
                  <tr>
                    <td colSpan={5} className="store-state-cell">
                      <div className="store-state store-error" role="alert">
                        <strong>query failed</strong>
                        <pre>{errorMessage || "Unable to load rows."}</pre>
                        <button type="button" onClick={() => onPage(filters.page)}>
                          retry ↻
                        </button>
                      </div>
                    </td>
                  </tr>
                ) : rows.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="store-state-cell">
                      <div className="store-state">
                        no rows match — widen filters
                      </div>
                    </td>
                  </tr>
                ) : (
                  rows.map((row) => (
                    <tr
                      key={row.id}
                      className={selected?.id === row.id ? "is-selected" : ""}
                      onClick={(event) => {
                        // Preserve normal text selection and avoid duplicate button events.
                        if ((event.target as HTMLElement).closest("button")) return;
                        if (window.getSelection()?.toString()) return;
                        openRow(row, event.currentTarget.querySelector("button"));
                      }}
                    >
                      <td>
                        <button
                          type="button"
                          className="store-row-title"
                          aria-label={`Inspect ${row.title || `row ${row.id}`}`}
                          aria-controls={selected?.id === row.id ? `${id}-inspector` : undefined}
                          aria-expanded={selected?.id === row.id}
                          onClick={(event) => openRow(row, event.currentTarget)}
                        >
                          <span>{row.title ?? "—"}</span>
                          <span className="store-open-arrow" aria-hidden="true">↗</span>
                        </button>
                      </td>
                      <td>
                        <span className={`store-tier ${row.tier ? `is-${row.tier}` : ""}`}>
                          {row.tier ?? "—"}
                        </span>
                      </td>
                      <td>
                        <span className={`store-lane ${row.lane === "quarantine" ? "is-quarantine" : ""}`}>
                          {row.lane ?? "—"}
                        </span>
                      </td>
                      <td className="store-source" title={row.source ?? undefined}>
                        {row.source ?? "—"}
                      </td>
                      <td className="store-numeric store-timestamp" title={row.indexed_at ?? undefined}>
                        {dateLabel(row.indexed_at)}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          <footer className="store-pagination">
            <span className="store-limit">limit <b>50</b></span>
            <span className="store-count" role="status" aria-live="polite" aria-atomic="true">
              {loading ? "loading…" : hasError ? "query error" : (
                <>
                  <b>{number.format(rows.length)}</b>
                  <span> / {number.format(total)}</span>
                  <span className="store-range">
                    {rows.length > 0
                      ? ` · ${number.format(start + 1)}–${number.format(start + rows.length)}`
                      : ""}
                  </span>
                </>
              )}
            </span>
            <span className="store-timezone">UTC</span>
            <div className="store-page-buttons" role="group" aria-label="Pagination">
              <button
                type="button"
                disabled={busy || filters.page === 0}
                onClick={() => onPage(Math.max(0, filters.page - 1))}
                aria-label="Previous page"
              >
                ← prev
              </button>
              <button
                type="button"
                disabled={busy || start + LIMIT >= total}
                onClick={() => onPage(filters.page + 1)}
                aria-label="Next page"
              >
                next →
              </button>
            </div>
          </footer>
        </div>

        {selectedCurrent && (
          <aside
            id={`${id}-inspector`}
            className="store-inspector"
            aria-labelledby={`${id}-inspector-title`}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.stopPropagation();
                closeInspector();
              }
            }}
          >
            <header className="store-inspector-header">
              <h2 id={`${id}-inspector-title`}>row inspector</h2>
              <button
                ref={closeButton}
                type="button"
                onClick={closeInspector}
                aria-label="Close row inspector"
                title="Close (Escape)"
              >
                ×
              </button>
            </header>
            <div className="store-inspector-body">
              <p className="store-inspector-table">{table} / {selectedCurrent.id}</p>
              <h3>{selectedCurrent.title ?? "Untitled row"}</h3>
              <dl>
                {Object.entries(selectedCurrent).map(([field, value]) => (
                  <div key={field}>
                    <dt>{field}</dt>
                    <dd>
                      <pre className={value == null ? "store-null" : undefined}>
                        {fieldLabel(value)}
                      </pre>
                    </dd>
                  </div>
                ))}
              </dl>
            </div>
          </aside>
        )}
      </div>
    </section>
  );
}
