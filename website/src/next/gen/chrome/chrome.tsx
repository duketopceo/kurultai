// chrome.tsx
import React, {
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";
import "./chrome.css";

/* ───────────────────────── types ───────────────────────── */

export type NavKey = "brain" | "repos" | "store";
export type DaemonStatus = "online" | "connecting" | "offline";
export type MemoryTier = "low" | "mid" | "high" | "max";
export type LayoutMode = "brain" | "ontology";
export type SearchState = "idle" | "loading" | "error";

export interface DaemonInfo {
  status: DaemonStatus;
  /** Reported by the daemon; omitted until known. Never hardcoded. */
  apiVersion?: string | null;
}

export interface SearchResult {
  id: string;
  title: string;
  summary?: string;
}

export interface TierLoad {
  loaded: number;
  total: number;
}

/* ───────────────────────── TopBar ───────────────────────── */

const NAV_ITEMS: ReadonlyArray<{ key: NavKey; label: string; beta?: boolean }> = [
  { key: "brain", label: "Brain" },
  { key: "repos", label: "Repos", beta: true },
  { key: "store", label: "Store", beta: true },
];

export interface TopBarProps {
  /** App build version, e.g. "v0.6.0". Omit to hide the mark. */
  version?: string;
  active: NavKey;
  onNavigate: (key: NavKey) => void;
  daemon: DaemonInfo;
  onOpenAccess?: () => void;
}

function statusText(d: DaemonInfo): string {
  switch (d.status) {
    case "online":
      return d.apiVersion ? `online · api ${d.apiVersion}` : "online";
    case "connecting":
      return "connecting";
    case "offline":
      return "offline";
  }
}

export function TopBar({ version, active, onNavigate, daemon, onOpenAccess }: TopBarProps) {
  return (
    <header className="k-topbar k-chrome">
      <div className="k-brand">
        <span className="k-brand__name">KURULTAI</span>
        {version && <span className="k-brand__ver">{version}</span>}
      </div>

      <nav className="k-nav" aria-label="Primary">
        {NAV_ITEMS.map((item) => {
          const isActive = item.key === active;
          return (
            <button
              key={item.key}
              type="button"
              className="k-nav__item"
              data-active={isActive || undefined}
              aria-current={isActive ? "page" : undefined}
              onClick={() => onNavigate(item.key)}
            >
              {item.label}
              {item.beta && (
                <span className="k-nav__beta" aria-label="beta">
                  β
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <div className="k-status">
        <div className="k-status__daemon" role="status" aria-live="polite">
          <span className="k-dot" data-state={daemon.status} aria-hidden="true" />
          <span className="k-status__text">{statusText(daemon)}</span>
        </div>
        {onOpenAccess && (
          <button
            type="button"
            className="k-iconbtn"
            onClick={onOpenAccess}
            aria-label="Access and settings"
            title="Access & settings"
          >
            <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
              <path
                fill="none"
                stroke="currentColor"
                strokeWidth="1.3"
                strokeLinecap="round"
                d="M8 10.2a2.2 2.2 0 1 0 0-4.4 2.2 2.2 0 0 0 0 4.4ZM8 1.5v1.6M8 12.9v1.6M1.5 8h1.6M12.9 8h1.6M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1"
              />
            </svg>
          </button>
        )}
      </div>
    </header>
  );
}

/* ───────────────────────── Segmented ───────────────────────── */

interface SegOption<T extends string> {
  value: T;
  label: string;
  meta?: string;
  ariaLabel?: string;
}

interface SegmentedProps<T extends string> {
  /** radiogroup for settings, tablist for view switching. */
  kind: "radiogroup" | "tablist";
  caption: string;
  options: ReadonlyArray<SegOption<T>>;
  value: T;
  onChange: (v: T) => void;
  disabled?: boolean;
}

function Segmented<T extends string>({
  kind,
  caption,
  options,
  value,
  onChange,
  disabled,
}: SegmentedProps<T>) {
  const captionId = useId();
  const refs = useRef<Array<HTMLButtonElement | null>>([]);
  const itemRole = kind === "radiogroup" ? "radio" : "tab";

  // Roving tabindex: arrows move + select (selection follows focus).
  const onKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    const idx = options.findIndex((o) => o.value === value);
    let next = -1;
    if (e.key === "ArrowRight" || e.key === "ArrowDown") next = (idx + 1) % options.length;
    else if (e.key === "ArrowLeft" || e.key === "ArrowUp")
      next = (idx - 1 + options.length) % options.length;
    else if (e.key === "Home") next = 0;
    else if (e.key === "End") next = options.length - 1;
    if (next < 0) return;
    e.preventDefault();
    onChange(options[next].value);
    refs.current[next]?.focus();
  };

  return (
    <div className="k-group">
      <span id={captionId} className="k-caption">
        {caption}
      </span>
      <div
        className="k-seg"
        role={kind}
        aria-labelledby={captionId}
        aria-disabled={disabled || undefined}
        onKeyDown={disabled ? undefined : onKeyDown}
      >
        {options.map((o, i) => {
          const selected = o.value === value;
          return (
            <button
              key={o.value}
              ref={(el) => {
                refs.current[i] = el;
              }}
              type="button"
              role={itemRole}
              className="k-seg__opt"
              data-active={selected || undefined}
              aria-checked={kind === "radiogroup" ? selected : undefined}
              aria-selected={kind === "tablist" ? selected : undefined}
              aria-label={o.ariaLabel}
              tabIndex={selected ? 0 : -1}
              disabled={disabled}
              onClick={() => onChange(o.value)}
            >
              <span>{o.label}</span>
              {o.meta && <span className="k-seg__meta">{o.meta}</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}

/* ───────────────────────── Search ───────────────────────── */

const MAX_RESULTS = 6;

interface SearchProps {
  query: string;
  onQueryChange: (q: string) => void;
  /** null = no search has run for the current query yet. */
  results: SearchResult[] | null;
  state: SearchState;
  onSelect: (r: SearchResult) => void;
  onReset: () => void;
  disabled: boolean;
  disabledReason?: string;
}

function Search({
  query,
  onQueryChange,
  results,
  state,
  onSelect,
  onReset,
  disabled,
  disabledReason,
}: SearchProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const listId = useId();
  const optPrefix = useId();
  const [focused, setFocused] = useState(false);
  const [activeIdx, setActiveIdx] = useState(-1);
  const [modLabel, setModLabel] = useState("⌘K");

  const shown = (results ?? []).slice(0, MAX_RESULTS);
  const hasQuery = query.trim().length > 0;
  const open = focused && hasQuery && !disabled;

  useEffect(() => {
    // Resolved client-side to avoid SSR mismatch.
    const mac = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
    setModLabel(mac ? "⌘K" : "Ctrl K");
  }, []);

  useEffect(() => setActiveIdx(-1), [results]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        if (disabled) return;
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [disabled]);

  const choose = useCallback(
    (r: SearchResult) => {
      onSelect(r);
      setFocused(false);
      inputRef.current?.blur();
    },
    [onSelect],
  );

  const reset = () => {
    onReset();
    setActiveIdx(-1);
    inputRef.current?.focus();
  };

  const onKeyDown = (e: ReactKeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown" && shown.length) {
      e.preventDefault();
      setActiveIdx((i) => (i + 1) % shown.length);
    } else if (e.key === "ArrowUp" && shown.length) {
      e.preventDefault();
      setActiveIdx((i) => (i <= 0 ? shown.length - 1 : i - 1));
    } else if (e.key === "Enter" && activeIdx >= 0 && shown[activeIdx]) {
      e.preventDefault();
      choose(shown[activeIdx]);
    } else if (e.key === "Escape") {
      // First Esc closes the list, second clears.
      if (open) {
        e.preventDefault();
        setFocused(false);
      } else if (query) {
        e.preventDefault();
        reset();
      }
    }
  };

  const placeholder = disabled ? disabledReason ?? "search unavailable" : "search memory";

  return (
    <div
      ref={wrapRef}
      className="k-search"
      data-disabled={disabled || undefined}
      onBlur={(e) => {
        if (!wrapRef.current?.contains(e.relatedTarget as Node | null)) setFocused(false);
      }}
    >
      <div className="k-search__field">
        <svg className="k-search__icon" viewBox="0 0 16 16" width="13" height="13" aria-hidden="true">
          <circle cx="7" cy="7" r="4.5" fill="none" stroke="currentColor" strokeWidth="1.4" />
          <path d="m10.5 10.5 3 3" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
        </svg>
        <input
          ref={inputRef}
          className="k-search__input"
          type="text"
          role="combobox"
          aria-label="Search memory"
          aria-expanded={open}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-activedescendant={open && activeIdx >= 0 ? `${optPrefix}-${activeIdx}` : undefined}
          aria-busy={state === "loading" || undefined}
          autoComplete="off"
          spellCheck={false}
          placeholder={placeholder}
          value={query}
          disabled={disabled}
          onChange={(e) => onQueryChange(e.target.value)}
          onFocus={() => setFocused(true)}
          onKeyDown={onKeyDown}
        />
        {query ? (
          <button
            type="button"
            className="k-search__clear"
            onClick={reset}
            aria-label="Clear search"
            title="Clear (Esc)"
            disabled={disabled}
          >
            <svg viewBox="0 0 12 12" width="10" height="10" aria-hidden="true">
              <path d="M2.5 2.5l7 7M9.5 2.5l-7 7" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
            </svg>
          </button>
        ) : (
          !disabled && (
            <kbd className="k-chip" aria-hidden="true">
              {modLabel}
            </kbd>
          )
        )}
      </div>

      {open && (
        <div className="k-search__panel">
          {state === "loading" && shown.length === 0 && (
            <div className="k-search__note">searching…</div>
          )}
          {state === "error" && <div className="k-search__note" data-tone="danger">search failed</div>}
          {state === "idle" && results !== null && shown.length === 0 && (
            <div className="k-search__note">no matches</div>
          )}
          <ul id={listId} role="listbox" aria-label="Search results" className="k-search__list">
            {shown.map((r, i) => (
              <li
                key={r.id}
                id={`${optPrefix}-${i}`}
                role="option"
                aria-selected={i === activeIdx}
                className="k-search__opt"
                data-active={i === activeIdx || undefined}
                onMouseDown={(e) => e.preventDefault()} // keep input focus
                onMouseEnter={() => setActiveIdx(i)}
                onClick={() => choose(r)}
              >
                <span className="k-search__title">{r.title}</span>
                {r.summary && <span className="k-search__summary">{r.summary}</span>}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

/* ───────────────────────── CommandStrip ───────────────────────── */

const TIERS: ReadonlyArray<MemoryTier> = ["low", "mid", "high", "max"];

export interface CommandStripProps {
  daemonStatus: DaemonStatus;

  query: string;
  onQueryChange: (q: string) => void;
  results: SearchResult[] | null;
  searchState: SearchState;
  onSelectResult: (r: SearchResult) => void;
  onResetSearch: () => void;

  tier: MemoryTier;
  onTierChange: (t: MemoryTier) => void;
  /** Only tiers with a known load state; count renders when loaded < total. */
  tierLoad?: Partial<Record<MemoryTier, TierLoad>>;

  layout: LayoutMode;
  onLayoutChange: (l: LayoutMode) => void;
}

export function CommandStrip(props: CommandStripProps) {
  const { daemonStatus, tier, onTierChange, tierLoad, layout, onLayoutChange } = props;

  const searchDisabled = daemonStatus !== "online";
  const disabledReason =
    daemonStatus === "connecting" ? "connecting to daemon…" : "search unavailable · daemon offline";

  const tierOptions = TIERS.map((t) => {
    const load = tierLoad?.[t];
    const partial = load && load.total > 0 && load.loaded < load.total;
    return {
      value: t,
      label: t,
      meta: partial ? `${load.loaded}/${load.total}` : undefined,
      ariaLabel: partial ? `${t}, ${load.loaded} of ${load.total} loaded` : undefined,
    };
  });

  return (
    <div className="k-strip k-chrome" role="toolbar" aria-label="Command strip">
      <Search
        query={props.query}
        onQueryChange={props.onQueryChange}
        results={props.results}
        state={props.searchState}
        onSelect={props.onSelectResult}
        onReset={props.onResetSearch}
        disabled={searchDisabled}
        disabledReason={disabledReason}
      />
      <div className="k-strip__controls">
        <Segmented
          kind="radiogroup"
          caption="Tier"
          options={tierOptions}
          value={tier}
          onChange={onTierChange}
        />
        <Segmented
          kind="tablist"
          caption="View"
          options={[
            { value: "brain", label: "brain" },
            { value: "ontology", label: "ontology" },
          ]}
          value={layout}
          onChange={onLayoutChange}
        />
      </div>
    </div>
  );
}
