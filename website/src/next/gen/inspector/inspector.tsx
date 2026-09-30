// inspector.tsx
// Floating node Inspector for the Kurultai Brain cortex canvas.
// Mount inside the canvas/hero wrapper (which must be `position: relative`).
// Desktop: floats top-right over the hero. ≤768px: bottom sheet.
// Every value comes from props; absent fields render "—".

import React, {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
import "./inspector.css";

/* ───────────────────────── Types ───────────────────────── */

export type Tier = "hot" | "warm" | "cold";

export interface OntologyLink {
  id: string;
  label: string;
  kind: "class" | "instance";
}

export interface Atom {
  id: string;
  title?: string | null;
  summary?: string | null;
  tier?: Tier | null;
  score?: number | null;
  source?: string | null;
  sourceUrl?: string | null;
  tags?: string[] | null;
  /** ISO string or epoch ms */
  indexedAt?: string | number | null;
  links?: OntologyLink[] | null;
}

export interface OntologyClass {
  id: string;
  label: string;
}

export interface InspectorProps {
  /** Selected or hovered atom. `null` renders nothing. */
  atom: Atom | null;
  /** "preview" = hover (no focus steal, no actions). "pinned" = selected. */
  mode?: "preview" | "pinned";
  /** True while full atom detail is being fetched. */
  detailLoading?: boolean;
  /** Error from the detail fetch, if any. */
  detailError?: string | null;
  onRetryDetail?: () => void;

  /** Ontology classes available to promote into. */
  classes: OntologyClass[];
  classesLoading?: boolean;
  classesError?: string | null;

  /** Resolve on success; throw/reject with an Error on failure. */
  onPromote: (atomId: string, classId: string) => Promise<void>;
  onOpenSource: (atom: Atom) => void;
  onClose: () => void;
}

/* ───────────────────────── Helpers ───────────────────────── */

const DASH = "—";

type PromoteState =
  | { phase: "idle" }
  | { phase: "picking" }
  | { phase: "posting"; classId: string }
  | { phase: "success"; classId: string; label: string }
  | { phase: "error"; classId: string; message: string };

function formatScore(score: number | null | undefined): string | null {
  if (typeof score !== "number" || !Number.isFinite(score)) return null;
  return Math.abs(score) < 1000 ? score.toFixed(3) : score.toLocaleString();
}

function formatIndexedAt(
  v: string | number | null | undefined
): { text: string; iso?: string } | null {
  if (v === null || v === undefined || v === "") return null;
  const d = new Date(v);
  if (Number.isNaN(d.getTime())) return { text: String(v) };
  return {
    text: new Intl.DateTimeFormat(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(d),
    iso: d.toISOString(),
  };
}

function errorMessage(e: unknown): string {
  if (e instanceof Error && e.message) return e.message;
  if (typeof e === "string" && e) return e;
  return "Promote failed (no error detail returned).";
}

function isPresent<T>(v: T | null | undefined): v is T {
  if (v === null || v === undefined) return false;
  if (typeof v === "string") return v.trim().length > 0;
  if (Array.isArray(v)) return v.length > 0;
  return true;
}

/* ───────────────────────── Sub-components ───────────────────────── */

function Spinner({ label }: { label?: string }) {
  return (
    <span className="insp-spinner" role={label ? "status" : undefined}>
      <span className="insp-spinner__ring" aria-hidden="true" />
      {label ? <span className="insp-sr">{label}</span> : null}
    </span>
  );
}

/** Renders value, a pending marker while detail loads, or "—". */
function Value({
  present,
  pending,
  children,
}: {
  present: boolean;
  pending: boolean;
  children?: React.ReactNode;
}) {
  if (present) return <>{children}</>;
  if (pending)
    return (
      <span className="insp-pending" aria-label="loading">
        …
      </span>
    );
  return <span className="insp-dash">{DASH}</span>;
}

function Field({
  label,
  children,
  wide,
}: {
  label: string;
  children: React.ReactNode;
  wide?: boolean;
}) {
  return (
    <div className={`insp-field${wide ? " insp-field--wide" : ""}`}>
      <dt className="insp-label">{label}</dt>
      <dd className="insp-value">{children}</dd>
    </div>
  );
}

function TierPill({ tier }: { tier: Tier }) {
  return (
    <span className={`insp-tier insp-tier--${tier}`}>
      <span className="insp-tier__dot" aria-hidden="true" />
      {tier}
    </span>
  );
}

/* ───────────────────────── Inspector ───────────────────────── */

export function Inspector({
  atom,
  mode = "pinned",
  detailLoading = false,
  detailError = null,
  onRetryDetail,
  classes,
  classesLoading = false,
  classesError = null,
  onPromote,
  onOpenSource,
  onClose,
}: InspectorProps) {
  const titleId = useId();
  const pickerId = useId();
  const filterId = useId();

  const closeRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLElement>(null);
  const reqRef = useRef(0);
  const mountedRef = useRef(true);

  const [promote, setPromote] = useState<PromoteState>({ phase: "idle" });
  const [selectedClass, setSelectedClass] = useState<string>("");
  const [filter, setFilter] = useState("");

  const pinned = mode === "pinned";
  const atomId = atom?.id ?? null;

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  // Reset transient state when the inspected atom changes; drop stale requests.
  useEffect(() => {
    reqRef.current += 1;
    setPromote({ phase: "idle" });
    setSelectedClass("");
    setFilter("");
  }, [atomId]);

  // Focus management for pinned mode: focus close on open, restore after.
  useEffect(() => {
    if (!pinned || !atomId) return;
    const prev = document.activeElement as HTMLElement | null;
    closeRef.current?.focus({ preventScroll: true });
    return () => {
      if (prev && document.contains(prev)) prev.focus({ preventScroll: true });
    };
  }, [pinned, atomId]);

  const pickerOpen =
    promote.phase === "picking" ||
    promote.phase === "posting" ||
    promote.phase === "error";

  // Esc: collapse picker first (unless posting), otherwise close inspector.
  useEffect(() => {
    if (!atomId) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || e.defaultPrevented) return;
      if (pickerOpen) {
        if (promote.phase !== "posting") {
          e.preventDefault();
          setPromote({ phase: "idle" });
        }
        return;
      }
      e.preventDefault();
      onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [atomId, pickerOpen, promote.phase, onClose]);

  const linkedIds = useMemo(
    () => new Set((atom?.links ?? []).map((l) => l.id)),
    [atom?.links]
  );

  const filteredClasses = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return classes;
    return classes.filter(
      (c) => c.label.toLowerCase().includes(q) || c.id.toLowerCase().includes(q)
    );
  }, [classes, filter]);

  const submitPromote = useCallback(
    async (classId: string) => {
      if (!atom || !classId) return;
      const req = ++reqRef.current;
      setPromote({ phase: "posting", classId });
      try {
        await onPromote(atom.id, classId);
        if (!mountedRef.current || req !== reqRef.current) return;
        const label = classes.find((c) => c.id === classId)?.label ?? classId;
        setPromote({ phase: "success", classId, label });
      } catch (e) {
        if (!mountedRef.current || req !== reqRef.current) return;
        setPromote({ phase: "error", classId, message: errorMessage(e) });
      }
    },
    [atom, classes, onPromote]
  );

  if (!atom) return null;

  const pending = detailLoading;
  const score = formatScore(atom.score);
  const indexed = formatIndexedAt(atom.indexedAt);
  const tags = (atom.tags ?? []).filter((t) => t && t.trim());
  const links = atom.links ?? [];
  const canOpenSource = isPresent(atom.sourceUrl);
  const posting = promote.phase === "posting";

  return (
    <aside
      ref={panelRef}
      className={`insp insp--${mode}`}
      role={pinned ? "dialog" : "region"}
      aria-modal={pinned ? false : undefined}
      aria-labelledby={titleId}
      aria-busy={pending || posting || undefined}
      data-atom-id={atom.id}
    >
      <div className="insp__grip" aria-hidden="true" />

      {/* Header */}
      <header className="insp__head">
        <div className="insp__head-meta">
          <span className="insp-label">
            {pinned ? "inspector" : "preview"} · atom
          </span>
          <code className="insp__id" title={atom.id}>
            {atom.id}
          </code>
        </div>
        <button
          ref={closeRef}
          type="button"
          className="insp__close"
          onClick={onClose}
          aria-label="Close inspector (Esc)"
          title="Close (Esc)"
        >
          ×
        </button>
      </header>

      <div className="insp__body">
        {/* Detail fetch state */}
        {detailLoading ? (
          <div className="insp-banner insp-banner--loading">
            <Spinner />
            <span>loading detail…</span>
          </div>
        ) : detailError ? (
          <div className="insp-banner insp-banner--error" role="alert">
            <span>detail failed: {detailError}</span>
            {onRetryDetail ? (
              <button
                type="button"
                className="insp-textbtn"
                onClick={onRetryDetail}
              >
                retry
              </button>
            ) : null}
          </div>
        ) : null}

        {/* Title + summary */}
        <section className="insp__lede">
          <span className="insp-label">title</span>
          <h2 id={titleId} className="insp__title">
            <Value present={isPresent(atom.title)} pending={pending}>
              {atom.title}
            </Value>
          </h2>

          <span className="insp-label">summary</span>
          <p className="insp__summary">
            <Value present={isPresent(atom.summary)} pending={pending}>
              {atom.summary}
            </Value>
          </p>
        </section>

        {/* Facts */}
        <dl className="insp__grid">
          <Field label="tier">
            <Value present={isPresent(atom.tier)} pending={pending}>
              {atom.tier ? <TierPill tier={atom.tier} /> : null}
            </Value>
          </Field>

          <Field label="score">
            <Value present={score !== null} pending={pending}>
              <span className="insp-num">{score}</span>
            </Value>
          </Field>

          <Field label="source" wide>
            <Value present={isPresent(atom.source)} pending={pending}>
              <span className="insp-mono insp-trunc" title={atom.source ?? ""}>
                {atom.source}
              </span>
            </Value>
          </Field>

          <Field label="indexed-at" wide>
            <Value present={indexed !== null} pending={pending}>
              {indexed?.iso ? (
                <time className="insp-mono" dateTime={indexed.iso} title={indexed.iso}>
                  {indexed.text}
                </time>
              ) : (
                <span className="insp-mono">{indexed?.text}</span>
              )}
            </Value>
          </Field>

          <Field label="tags" wide>
            <Value present={tags.length > 0} pending={pending}>
              <ul className="insp-chips" aria-label="tags">
                {tags.map((t) => (
                  <li key={t} className="insp-chip insp-chip--tag">
                    #{t}
                  </li>
                ))}
              </ul>
            </Value>
          </Field>

          <Field label="ontology links" wide>
            <Value present={links.length > 0} pending={pending}>
              <ul className="insp-chips" aria-label="ontology links">
                {links.map((l) => (
                  <li
                    key={`${l.kind}:${l.id}`}
                    className={`insp-chip insp-chip--${l.kind}`}
                    title={`${l.kind}: ${l.id}`}
                  >
                    <span className="insp-chip__kind">
                      {l.kind === "class" ? "cls" : "ins"}
                    </span>
                    {l.label}
                  </li>
                ))}
              </ul>
            </Value>
          </Field>
        </dl>

        {/* Promote picker (inline) */}
        {pinned && pickerOpen ? (
          <form
            className="insp-picker"
            aria-labelledby={pickerId}
            onSubmit={(e) => {
              e.preventDefault();
              if (!posting && selectedClass) void submitPromote(selectedClass);
            }}
          >
            <div className="insp-picker__head">
              <span id={pickerId} className="insp-label">
                promote to class
              </span>
              <span className="insp-picker__count insp-mono">
                {classesLoading ? "…" : `${classes.length}`}
              </span>
            </div>

            {classesLoading ? (
              <div className="insp-picker__state">
                <Spinner /> <span>loading classes…</span>
              </div>
            ) : classesError ? (
              <div className="insp-picker__state insp-picker__state--error" role="alert">
                classes failed: {classesError}
              </div>
            ) : classes.length === 0 ? (
              <div className="insp-picker__state">
                no ontology classes defined
              </div>
            ) : (
              <>
                {classes.length > 6 ? (
                  <>
                    <label htmlFor={filterId} className="insp-sr">
                      Filter classes
                    </label>
                    <input
                      id={filterId}
                      className="insp-picker__filter"
                      type="search"
                      placeholder="filter classes…"
                      value={filter}
                      onChange={(e) => setFilter(e.target.value)}
                      disabled={posting}
                      autoComplete="off"
                      spellCheck={false}
                    />
                  </>
                ) : null}

                <ul className="insp-picker__list" role="radiogroup">
                  {filteredClasses.length === 0 ? (
                    <li className="insp-picker__state">no match</li>
                  ) : (
                    filteredClasses.map((c) => {
                      const linked = linkedIds.has(c.id);
                      return (
                        <li key={c.id}>
                          <label
                            className={`insp-opt${
                              selectedClass === c.id ? " is-selected" : ""
                            }${linked ? " is-linked" : ""}`}
                          >
                            <input
                              type="radio"
                              name={`${pickerId}-class`}
                              value={c.id}
                              checked={selectedClass === c.id}
                              disabled={posting || linked}
                              onChange={() => setSelectedClass(c.id)}
                            />
                            <span className="insp-opt__label">{c.label}</span>
                            {linked ? (
                              <span className="insp-opt__tag">linked</span>
                            ) : null}
                          </label>
                        </li>
                      );
                    })
                  )}
                </ul>
              </>
            )}

            {promote.phase === "error" ? (
              <div className="insp-line insp-line--error" role="alert">
                <span>✕ {promote.message}</span>
              </div>
            ) : null}

            <div className="insp-picker__actions">
              <button
                type="button"
                className="insp-btn insp-btn--ghost"
                onClick={() => setPromote({ phase: "idle" })}
                disabled={posting}
              >
                cancel
              </button>
              <button
                type="submit"
                className="insp-btn insp-btn--primary"
                disabled={posting || !selectedClass}
              >
                {posting ? (
                  <>
                    <Spinner label="Promoting" /> posting…
                  </>
                ) : promote.phase === "error" ? (
                  "retry"
                ) : (
                  "promote"
                )}
              </button>
            </div>
          </form>
        ) : null}

        {pinned && promote.phase === "success" ? (
          <div className="insp-line insp-line--ok" role="status">
            <span>
              ✓ promoted to <strong>{promote.label}</strong>
            </span>
            <button
              type="button"
              className="insp-textbtn"
              onClick={() => setPromote({ phase: "idle" })}
            >
              dismiss
            </button>
          </div>
        ) : null}
      </div>

      {/* Actions */}
      {pinned ? (
        <footer className="insp__foot">
          <button
            type="button"
            className="insp-btn insp-btn--primary"
            onClick={() => {
              setSelectedClass("");
              setPromote({ phase: "picking" });
            }}
            disabled={pickerOpen || detailLoading}
            aria-expanded={pickerOpen}
            aria-controls={pickerOpen ? pickerId : undefined}
          >
            Promote to ontology
          </button>
          <button
            type="button"
            className="insp-btn insp-btn--link"
            onClick={() => onOpenSource(atom)}
            disabled={!canOpenSource}
            title={canOpenSource ? atom.sourceUrl ?? "" : "No source URL"}
          >
            Open source <span aria-hidden="true">↗</span>
          </button>
        </footer>
      ) : (
        <footer className="insp__foot insp__foot--hint">
          <span className="insp-label">click node to pin · esc to dismiss</span>
        </footer>
      )}
    </aside>
  );
}

export default Inspector;
