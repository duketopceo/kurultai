// ontology-board.tsx
//
// Layout decision: CLASSES AS COLUMNS (CSS grid), with a single SVG overlay for edges.
// Why not a positioned graph / @xyflow/react:
//   · The class set is fixed (six). A column per class makes type the primary axis, so
//     "what kind of thing is this" is answered by position before you read a chip.
//   · No layout algorithm and no physics. Positions are deterministic and stable
//     across renders, which keeps the Brain's organic cloud and this surface clearly distinct.
//   · Columns stack on mobile for free. A free-positioned canvas would need a second layout.
//   · No extra dependency. Edges are orthogonal elbow connectors measured from the DOM.
//     They run in the column gutters and pass *under* opaque cards.
// Zoom/pan: plain scroll. At ≥768px the grid shrinks its columns to fit the viewport.
//   Scrolling only kicks in when a column is taller than the viewport.
// Mobile (<768px): columns stack vertically. The edge overlay is not rendered, and link mode
//   is DISABLED. This is declared in the toolbar and on the menu item, not half-supported.

import {
  useCallback,
  useEffect,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { CSSProperties, FormEvent, KeyboardEvent, MouseEvent } from "react";
import "./ontology-board.css";

/* ───────────────────────── types ───────────────────────── */

export type OntologyClass = { id: string; name: string; color?: string };
export type OntologyInstance = { id: string; classId: string; title: string };
export type OntologyEdge = { id: string; source: string; target: string; type: string };

export interface OntologyBoardProps {
  classes: OntologyClass[];
  instances: OntologyInstance[];
  edges: OntologyEdge[];
  onOpen: (instanceId: string) => void;
  onLink: (sourceId: string, targetId: string, type: string) => Promise<void> | void;
  onDelete: (instanceId: string) => Promise<void> | void;
  onAdd: (input: { title: string; classId: string }) => Promise<void> | void;
  /** Selectable link types while linking. Defaults to the common set. */
  linkTypes?: string[];
}

type Rect = { x: number; y: number; w: number; h: number };
type Geo = { nodes: Record<string, Rect>; heads: Record<string, Rect>; w: number; h: number };
type BoardError = { id: number; message: string; retry?: () => void };
type MenuState = { id: string; confirmDelete: boolean } | null;
type LinkState = { sourceId: string; type: string } | null;
type AddState = { title: string; classId: string } | null;

const DEFAULT_LINK_TYPES = ["mentions", "belongs_to", "authored", "references", "blocks"];
/** Weak / referential links render dashed; structural links render solid. */
const DASHED_TYPES = new Set(["mentions", "references", "cites"]);
const FALLBACK_COLORS = ["#c084fc", "#60a5fa", "#34d399", "#fbbf24", "#f472b6", "#94a3b8"];
const NARROW_QUERY = "(max-width: 767px)";

/* ───────────────────────── helpers ───────────────────────── */

const errMsg = (e: unknown): string =>
  e instanceof Error ? e.message : typeof e === "string" ? e : "unknown error";

/** Orthogonal elbow connector. Vertical run sits in the gutter next to the source column. */
function route(a: Rect, b: Rect, lane: number) {
  const ac = a.x + a.w / 2;
  const bc = b.x + b.w / 2;
  const sy = a.y + a.h / 2;
  const ty = b.y + b.h / 2;

  if (Math.abs(ac - bc) < 8) {
    // same column → bracket out to the right gutter
    const sx = a.x + a.w;
    const tx = b.x + b.w;
    const vx = Math.max(sx, tx) + 8 + lane;
    return { d: `M${sx} ${sy}H${vx}V${ty}H${tx}`, lx: vx + 4, ly: (sy + ty) / 2 };
  }
  const ltr = ac < bc;
  const sx = ltr ? a.x + a.w : a.x;
  const tx = ltr ? b.x : b.x + b.w;
  const vx = ltr ? sx + 8 + lane : sx - 8 - lane;
  return { d: `M${sx} ${sy}H${vx}V${ty}H${tx}`, lx: vx + 4, ly: (sy + ty) / 2 };
}

function useIsNarrow(): boolean {
  const [narrow, setNarrow] = useState(
    () => typeof window !== "undefined" && window.matchMedia(NARROW_QUERY).matches,
  );
  useEffect(() => {
    const mq = window.matchMedia(NARROW_QUERY);
    const on = () => setNarrow(mq.matches);
    on();
    mq.addEventListener("change", on);
    return () => mq.removeEventListener("change", on);
  }, []);
  return narrow;
}

/* ───────────────────────── component ───────────────────────── */

export function OntologyBoard({
  classes,
  instances,
  edges,
  onOpen,
  onLink,
  onDelete,
  onAdd,
  linkTypes = DEFAULT_LINK_TYPES,
}: OntologyBoardProps) {
  const uid = useId().replace(/:/g, "");
  const isNarrow = useIsNarrow();

  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const [hovered, setHovered] = useState<string | null>(null);
  const [menu, setMenu] = useState<MenuState>(null);
  const [linking, setLinking] = useState<LinkState>(null);
  const [pendingLink, setPendingLink] = useState<{ source: string; target: string } | null>(null);
  const [cursor, setCursor] = useState<{ x: number; y: number } | null>(null);
  const [busy, setBusy] = useState<Record<string, string>>({});
  const [errors, setErrors] = useState<BoardError[]>([]);
  const [adding, setAdding] = useState<AddState>(null);
  const [addBusy, setAddBusy] = useState(false);
  const [addError, setAddError] = useState<string | null>(null);
  const [geo, setGeo] = useState<Geo>({ nodes: {}, heads: {}, w: 0, h: 0 });

  const contentRef = useRef<HTMLDivElement>(null);
  const nodeEls = useRef(new Map<string, HTMLElement>());
  const headEls = useRef(new Map<string, HTMLElement>());
  const errSeq = useRef(0);

  /* ── derived ── */

  const classById = useMemo(() => new Map(classes.map((c) => [c.id, c])), [classes]);
  const instById = useMemo(() => new Map(instances.map((i) => [i.id, i])), [instances]);

  const colorOf = useCallback(
    (classId: string) => {
      const idx = classes.findIndex((c) => c.id === classId);
      return classById.get(classId)?.color ?? FALLBACK_COLORS[Math.max(idx, 0) % FALLBACK_COLORS.length];
    },
    [classes, classById],
  );

  const byClass = useMemo(() => {
    const m = new Map<string, OntologyInstance[]>();
    classes.forEach((c) => m.set(c.id, []));
    instances.forEach((i) => m.get(i.classId)?.push(i));
    return m;
  }, [classes, instances]);

  const orphanCount = useMemo(
    () => instances.filter((i) => !classById.has(i.classId)).length,
    [instances, classById],
  );

  const validEdges = useMemo(
    () => edges.filter((e) => e.source !== e.target && instById.has(e.source) && instById.has(e.target)),
    [edges, instById],
  );
  const brokenEdgeCount = edges.length - validEdges.length;

  const linkCount = useMemo(() => {
    const m = new Map<string, number>();
    validEdges.forEach((e) => {
      m.set(e.source, (m.get(e.source) ?? 0) + 1);
      m.set(e.target, (m.get(e.target) ?? 0) + 1);
    });
    return m;
  }, [validEdges]);

  const focusId = linking?.sourceId ?? hovered;

  const related = useMemo(() => {
    if (!focusId) return null;
    const s = new Set<string>([focusId]);
    validEdges.forEach((e) => {
      if (e.source === focusId) s.add(e.target);
      if (e.target === focusId) s.add(e.source);
    });
    return s;
  }, [focusId, validEdges]);

  const isEligible = useCallback(
    (id: string) => {
      if (!linking || id === linking.sourceId || busy[id]) return false;
      return !validEdges.some(
        (e) =>
          e.type === linking.type &&
          ((e.source === linking.sourceId && e.target === id) ||
            (e.target === linking.sourceId && e.source === id)),
      );
    },
    [linking, busy, validEdges],
  );

  /* ── geometry ── */

  const measure = useCallback(() => {
    const root = contentRef.current;
    if (!root) return;
    const base = root.getBoundingClientRect();
    const read = (m: Map<string, HTMLElement>) => {
      const out: Record<string, Rect> = {};
      m.forEach((el, id) => {
        const r = el.getBoundingClientRect();
        out[id] = { x: r.left - base.left, y: r.top - base.top, w: r.width, h: r.height };
      });
      return out;
    };
    setGeo({ nodes: read(nodeEls.current), heads: read(headEls.current), w: root.scrollWidth, h: root.scrollHeight });
  }, []);

  useLayoutEffect(() => {
    measure();
  }, [measure, classes, instances, edges, collapsed, isNarrow]);

  useEffect(() => {
    const root = contentRef.current;
    if (!root || typeof ResizeObserver === "undefined") return;
    let raf = 0;
    const ro = new ResizeObserver(() => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(measure);
    });
    ro.observe(root);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  }, [measure]);

  const anchorOf = (id: string): Rect | null => {
    const inst = instById.get(id);
    if (!inst) return null;
    if (collapsed.has(inst.classId)) return geo.heads[inst.classId] ?? null;
    return geo.nodes[id] ?? null;
  };

  /* ── global listeners ── */

  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (e.key !== "Escape") return;
      setLinking(null);
      setCursor(null);
      setMenu(null);
    };
    const onDown = (e: globalThis.MouseEvent) => {
      const t = e.target as HTMLElement | null;
      if (t && !t.closest(".ob-menu") && !t.closest(".ob-more")) setMenu(null);
    };
    document.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onDown);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("mousedown", onDown);
    };
  }, []);

  // Crossing below 768px mid-link cancels link mode. It is unsupported there.
  useEffect(() => {
    if (isNarrow) {
      setLinking(null);
      setCursor(null);
    }
  }, [isNarrow]);

  /* ── mutations ── */

  const pushError = (message: string, retry?: () => void) =>
    setErrors((es) => [...es, { id: ++errSeq.current, message, retry }]);

  const run = async (key: string, label: string, describe: string, fn: () => Promise<void> | void) => {
    setBusy((b) => ({ ...b, [key]: label }));
    try {
      await fn();
    } catch (e) {
      pushError(`${describe}: ${errMsg(e)}`, () => void run(key, label, describe, fn));
    } finally {
      setBusy((b) => {
        const { [key]: _drop, ...rest } = b;
        return rest;
      });
    }
  };

  const doDelete = (inst: OntologyInstance) => {
    setMenu(null);
    void run(inst.id, "deleting", `delete "${inst.title}" failed`, () => onDelete(inst.id));
  };

  const startLink = (id: string) => {
    if (isNarrow) return;
    setMenu(null);
    setLinking({ sourceId: id, type: linkTypes[0] ?? "mentions" });
  };

  const pickTarget = (targetId: string) => {
    if (!linking || !isEligible(targetId)) return;
    const { sourceId, type } = linking;
    const src = instById.get(sourceId);
    const tgt = instById.get(targetId);
    setLinking(null);
    setCursor(null);
    setPendingLink({ source: sourceId, target: targetId });
    void run(
      sourceId,
      "linking",
      `link "${src?.title ?? sourceId}" —${type}→ "${tgt?.title ?? targetId}" failed`,
      async () => {
        try {
          await onLink(sourceId, targetId, type);
        } finally {
          setPendingLink(null);
        }
      },
    );
  };

  const submitAdd = async (e: FormEvent) => {
    e.preventDefault();
    if (!adding || addBusy) return;
    const title = adding.title.trim();
    if (!title || !classById.has(adding.classId)) return;
    setAddBusy(true);
    setAddError(null);
    try {
      await onAdd({ title, classId: adding.classId });
      setAdding(null);
    } catch (err) {
      setAddError(errMsg(err));
    } finally {
      setAddBusy(false);
    }
  };

  const openAdd = (classId?: string) => {
    setAddError(null);
    setAdding({ title: "", classId: classId ?? classes[0]?.id ?? "" });
  };

  const toggleClass = (id: string) =>
    setCollapsed((s) => {
      const n = new Set(s);
      n.has(id) ? n.delete(id) : n.add(id);
      return n;
    });

  const onContentMove = (e: MouseEvent<HTMLDivElement>) => {
    if (!linking || !contentRef.current) return;
    const base = contentRef.current.getBoundingClientRect();
    setCursor({ x: e.clientX - base.left, y: e.clientY - base.top });
  };

  /* ── render pieces ── */

  const renderMenu = (inst: OntologyInstance) => {
    if (menu?.id !== inst.id) return null;
    const links = linkCount.get(inst.id) ?? 0;
    return (
      <div className="ob-menu" role="menu" onClick={(e) => e.stopPropagation()}>
        {menu.confirmDelete ? (
          <div className="ob-confirm" role="alertdialog" aria-label={`Confirm delete ${inst.title}`}>
            <p className="ob-confirm-text">
              delete “{inst.title}”?
              <br />
              <span className="ob-muted">
                {links === 0 ? "no links affected" : `${links} link${links === 1 ? "" : "s"} will be removed`}
              </span>
            </p>
            <div className="ob-confirm-row">
              <button type="button" className="ob-btn ob-btn--danger" autoFocus onClick={() => doDelete(inst)}>
                confirm
              </button>
              <button type="button" className="ob-btn" onClick={() => setMenu({ id: inst.id, confirmDelete: false })}>
                cancel
              </button>
            </div>
          </div>
        ) : (
          <>
            <button
              type="button"
              role="menuitem"
              className="ob-menu-item"
              autoFocus
              onClick={() => {
                setMenu(null);
                onOpen(inst.id);
              }}
            >
              open
            </button>
            <button
              type="button"
              role="menuitem"
              className="ob-menu-item"
              disabled={isNarrow}
              title={isNarrow ? "link mode requires ≥768px" : undefined}
              onClick={() => startLink(inst.id)}
            >
              link{isNarrow && <span className="ob-muted"> · desktop only</span>}
            </button>
            <button
              type="button"
              role="menuitem"
              className="ob-menu-item ob-menu-item--danger"
              onClick={() => setMenu({ id: inst.id, confirmDelete: true })}
            >
              delete
            </button>
          </>
        )}
      </div>
    );
  };

  const renderCard = (inst: OntologyInstance) => {
    const cls = classById.get(inst.classId);
    const count = linkCount.get(inst.id) ?? 0;
    const busyLabel = busy[inst.id];
    const isSource = linking?.sourceId === inst.id;
    const eligible = linking ? isEligible(inst.id) : false;
    const dim = !linking && related ? !related.has(inst.id) : false;
    const hot = !linking && related?.has(inst.id);

    const cn = [
      "ob-card",
      hot && "is-hot",
      dim && "is-dim",
      isSource && "is-source",
      linking && !isSource && (eligible ? "is-eligible" : "is-ineligible"),
      busyLabel && "is-busy",
      menu?.id === inst.id && "has-menu",
    ]
      .filter(Boolean)
      .join(" ");

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ContextMenu" || (e.shiftKey && e.key === "F10")) {
        e.preventDefault();
        if (!busyLabel && !linking) setMenu({ id: inst.id, confirmDelete: false });
      }
    };

    return (
      <li key={inst.id} className="ob-card-slot">
        <div
          ref={(el) => {
            if (el) nodeEls.current.set(inst.id, el);
            else nodeEls.current.delete(inst.id);
          }}
          className={cn}
          style={{ "--chip": colorOf(inst.classId) } as CSSProperties}
          aria-busy={!!busyLabel}
          onMouseEnter={() => setHovered(inst.id)}
          onMouseLeave={() => setHovered((h) => (h === inst.id ? null : h))}
          onContextMenu={(e) => {
            e.preventDefault();
            if (!busyLabel && !linking) setMenu({ id: inst.id, confirmDelete: false });
          }}
          onClick={() => {
            if (linking) pickTarget(inst.id);
          }}
        >
          <div className="ob-card-row">
            <span className="ob-chip">{cls?.name ?? "?"}</span>
            <span className="ob-count" title={`${count} link${count === 1 ? "" : "s"}`}>
              {count}
              <span className="ob-count-unit">ln</span>
            </span>
            {busyLabel ? (
              <span className="ob-busy" role="status">
                <span className="ob-spin" aria-hidden />
                <span className="ob-sr">{busyLabel}</span>
              </span>
            ) : (
              <button
                type="button"
                className="ob-more"
                aria-label={`Actions for ${inst.title}`}
                aria-haspopup="menu"
                aria-expanded={menu?.id === inst.id}
                disabled={!!linking}
                onClick={(e) => {
                  e.stopPropagation();
                  setMenu((m) => (m?.id === inst.id ? null : { id: inst.id, confirmDelete: false }));
                }}
              >
                ⋯
              </button>
            )}
          </div>
          <button
            type="button"
            className="ob-title"
            title={inst.title}
            aria-label={
              linking
                ? eligible
                  ? `Link to ${inst.title}`
                  : `${inst.title} (not a valid target)`
                : `Open ${inst.title}`
            }
            onKeyDown={onKey}
            onFocus={() => setHovered(inst.id)}
            onBlur={() => setHovered((h) => (h === inst.id ? null : h))}
            onClick={() => {
              if (linking) return; // bubbles to the card, which handles target picking
              onOpen(inst.id);
            }}
          >
            {inst.title}
          </button>
          {renderMenu(inst)}
        </div>
      </li>
    );
  };

  /* ── edges ── */

  const edgeLayer = !isNarrow && (
    <svg className="ob-edges" width={geo.w} height={geo.h} aria-hidden>
      <defs>
        <marker id={`${uid}-m`} viewBox="0 0 6 6" refX="5" refY="3" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M0 0L6 3L0 6z" className="ob-marker" />
        </marker>
        <marker id={`${uid}-mh`} viewBox="0 0 6 6" refX="5" refY="3" markerWidth="6" markerHeight="6" orient="auto">
          <path d="M0 0L6 3L0 6z" className="ob-marker ob-marker--hot" />
        </marker>
      </defs>
      {validEdges.map((e, i) => {
        const a = anchorOf(e.source);
        const b = anchorOf(e.target);
        if (!a || !b) return null;
        const { d, lx, ly } = route(a, b, (i % 3) * 3);
        const hot = !!focusId && (e.source === focusId || e.target === focusId);
        const dim = !!focusId && !hot;
        return (
          <g key={e.id} className={`ob-edge-g${hot ? " is-hot" : ""}${dim ? " is-dim" : ""}`}>
            <path
              d={d}
              className={`ob-edge${DASHED_TYPES.has(e.type) ? " is-dashed" : ""}`}
              markerEnd={`url(#${uid}-${hot ? "mh" : "m"})`}
            />
            {hot && (
              <text x={lx} y={ly} className="ob-edge-label">
                {e.type}
              </text>
            )}
          </g>
        );
      })}
      {pendingLink &&
        (() => {
          const a = anchorOf(pendingLink.source);
          const b = anchorOf(pendingLink.target);
          if (!a || !b) return null;
          return <path d={route(a, b, 0).d} className="ob-edge is-pending" markerEnd={`url(#${uid}-mh)`} />;
        })()}
      {linking &&
        cursor &&
        (() => {
          const a = anchorOf(linking.sourceId);
          if (!a) return null;
          const sx = cursor.x > a.x + a.w / 2 ? a.x + a.w : a.x;
          return <line x1={sx} y1={a.y + a.h / 2} x2={cursor.x} y2={cursor.y} className="ob-edge is-rubber" />;
        })()}
    </svg>
  );

  /* ── main render ── */

  const linkSource = linking ? instById.get(linking.sourceId) : undefined;
  const busyCount = Object.keys(busy).length;

  return (
    <section className={`ob${linking ? " is-linking" : ""}`} aria-label="Ontology board">
      <header className="ob-toolbar">
        <div className="ob-stats">
          <span className="ob-label">ontology</span>
          <span>{classes.length} classes</span>
          <span className="ob-sep">·</span>
          <span>{instances.length} instances</span>
          <span className="ob-sep">·</span>
          <span>{validEdges.length} edges</span>
          {busyCount > 0 && (
            <span className="ob-busy-inline" role="status">
              <span className="ob-spin" aria-hidden /> {busyCount} pending
            </span>
          )}
        </div>

        <div className="ob-legend" aria-hidden={isNarrow}>
          {isNarrow ? (
            <span className="ob-muted">edges + link mode · desktop only (≥768px)</span>
          ) : (
            <>
              <span className="ob-legend-solid" /> structural
              <span className="ob-legend-dashed" /> reference
            </>
          )}
        </div>

        {!adding && (
          <button type="button" className="ob-btn ob-btn--accent" onClick={() => openAdd()} disabled={classes.length === 0}>
            + entity
          </button>
        )}
      </header>

      {adding && (
        <form className="ob-add" onSubmit={submitAdd} aria-label="Add entity">
          <input
            className="ob-input"
            autoFocus
            placeholder="entity name"
            value={adding.title}
            disabled={addBusy}
            onChange={(e) => setAdding({ ...adding, title: e.target.value })}
            onKeyDown={(e) => e.key === "Escape" && setAdding(null)}
            aria-label="Entity name"
          />
          <select
            className="ob-input ob-select"
            value={adding.classId}
            disabled={addBusy}
            onChange={(e) => setAdding({ ...adding, classId: e.target.value })}
            aria-label="Entity class"
          >
            {classes.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
          <button type="submit" className="ob-btn ob-btn--accent" disabled={addBusy || !adding.title.trim()}>
            {addBusy ? (
              <>
                <span className="ob-spin" aria-hidden /> adding
              </>
            ) : addError ? (
              "retry"
            ) : (
              "add"
            )}
          </button>
          <button type="button" className="ob-btn" disabled={addBusy} onClick={() => setAdding(null)}>
            cancel
          </button>
          {addError && (
            <p className="ob-err ob-add-err" role="alert">
              err · add failed: {addError}
            </p>
          )}
        </form>
      )}

      {linking && (
        <div className="ob-linkbar" role="status">
          <span className="ob-label">link</span>
          <span>
            from <strong>{linkSource?.title ?? linking.sourceId}</strong>
          </span>
          <label className="ob-linktype">
            type
            <select
              className="ob-input ob-select"
              value={linking.type}
              onChange={(e) => setLinking({ ...linking, type: e.target.value })}
            >
              {linkTypes.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </label>
          <span className="ob-muted">pick a pulsing target · esc cancels</span>
          <button
            type="button"
            className="ob-btn"
            onClick={() => {
              setLinking(null);
              setCursor(null);
            }}
          >
            cancel
          </button>
        </div>
      )}

      {(orphanCount > 0 || brokenEdgeCount > 0) && (
        <p className="ob-warn">
          {orphanCount > 0 && `${orphanCount} instance${orphanCount === 1 ? "" : "s"} reference unknown classes (not shown). `}
          {brokenEdgeCount > 0 &&
            `${brokenEdgeCount} edge${brokenEdgeCount === 1 ? "" : "s"} with missing or self endpoints hidden.`}
        </p>
      )}

      {errors.length > 0 && (
        <ul className="ob-errors" role="alert">
          {errors.map((er) => (
            <li key={er.id} className="ob-err-row">
              <span className="ob-err">err · {er.message}</span>
              {er.retry && (
                <button
                  type="button"
                  className="ob-btn"
                  onClick={() => {
                    setErrors((es) => es.filter((x) => x.id !== er.id));
                    er.retry?.();
                  }}
                >
                  retry
                </button>
              )}
              <button
                type="button"
                className="ob-btn"
                onClick={() => setErrors((es) => es.filter((x) => x.id !== er.id))}
              >
                dismiss
              </button>
            </li>
          ))}
        </ul>
      )}

      <div className="ob-viewport">
        {classes.length === 0 ? (
          <p className="ob-empty-board">no classes defined. The ontology has no schema to render.</p>
        ) : (
          <div
            ref={contentRef}
            className="ob-content"
            onMouseMove={onContentMove}
            onMouseLeave={() => setCursor(null)}
          >
            {edgeLayer}
            <div className="ob-grid" style={{ "--cols": classes.length } as CSSProperties}>
              {classes.map((cls) => {
                const list = byClass.get(cls.id) ?? [];
                const isCollapsed = collapsed.has(cls.id);
                const bodyId = `${uid}-col-${cls.id}`;
                return (
                  <div key={cls.id} className="ob-col" style={{ "--chip": colorOf(cls.id) } as CSSProperties}>
                    <button
                      type="button"
                      ref={(el) => {
                        if (el) headEls.current.set(cls.id, el);
                        else headEls.current.delete(cls.id);
                      }}
                      className="ob-col-head"
                      aria-expanded={!isCollapsed}
                      aria-controls={bodyId}
                      onClick={() => toggleClass(cls.id)}
                    >
                      <span className="ob-caret" aria-hidden>
                        {isCollapsed ? "▸" : "▾"}
                      </span>
                      <span className="ob-col-swatch" aria-hidden />
                      <span className="ob-col-name">{cls.name}</span>
                      <span className="ob-col-count">{list.length}</span>
                    </button>

                    <div id={bodyId} className="ob-col-body" hidden={isCollapsed}>
                      {list.length === 0 ? (
                        <p className="ob-empty">— no {cls.name.toLowerCase()} instances</p>
                      ) : (
                        <ul className="ob-list">{list.map(renderCard)}</ul>
                      )}
                      {!linking && (
                        <button type="button" className="ob-col-add" onClick={() => openAdd(cls.id)}>
                          + entity
                        </button>
                      )}
                    </div>
                    {isCollapsed && list.length > 0 && (
                      <p className="ob-empty">{list.length} hidden · edges anchor to header</p>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </section>
  );
}

export default OntologyBoard;
