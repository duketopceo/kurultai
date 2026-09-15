import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  ReactFlow,
  ReactFlowProvider,
  Background,
  Controls,
  Handle,
  Position,
  type Node,
  type Edge,
  type NodeProps,
  type NodeChange,
  type Connection,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import {
  createOntologyEntity,
  createOntologyLink,
  deleteOntologyEntity,
  deleteOntologyLink,
  fetchOntology,
} from '../api';
import { boardEdges, entityToAtom, layoutOntology, type BoardNodeData } from '../brain/layout/ontoBoard';
import type { Atom, OntologyEntity, OntologyResponse } from '../types';

const POS_KEY = 'kurultai-onto-pos';
const EXPAND_KEY = 'kurultai-onto-expanded';

const REL_OPTIONS = [
  { value: 'is_a', label: 'is a' },
  { value: 'instance_of', label: 'instance of' },
  { value: 'associates_with', label: 'associates with' },
  { value: 'triggered_by', label: 'triggered by' },
  { value: 'contradicts', label: 'contradicts' },
] as const;

type Menu =
  | { kind: 'pane'; x: number; y: number }
  | { kind: 'node'; x: number; y: number; entity: OntologyEntity }
  | { kind: 'edge'; x: number; y: number; edgeId: string };

type OntoNode = Node<BoardNodeData, 'onto'>;

function loadJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function saveJson(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch { /* ignore */ }
}

function OntoCard({ data }: NodeProps<OntoNode>) {
  const { entity } = data;
  return (
    <div className={`onto-card onto-card-${entity.kind}`}>
      <Handle type="target" position={Position.Top} className="onto-handle" />
      <span className="onto-kind">{entity.kind}</span>
      <span className="onto-name">{entity.name}</span>
      {data.backed && <span className="onto-backed" title="Backed by a memory">●</span>}
      {data.hiddenCount > 0 && (
        <span className="onto-expand" title="Right-click → Expand instances">
          +{data.hiddenCount}
        </span>
      )}
      <Handle type="source" position={Position.Bottom} className="onto-handle" />
    </div>
  );
}

const nodeTypes = { onto: OntoCard };

interface Props {
  ontology: OntologyResponse;
  atoms: Atom[];
  onSelect: (atom: Atom) => void;
  onOntologyChanged: (onto: OntologyResponse) => void;
}

export function OntologyBoard({ ontology, atoms, onSelect, onOntologyChanged }: Props) {
  const [expanded, setExpanded] = useState<Set<string>>(
    () => new Set(loadJson<string[]>(EXPAND_KEY, [])),
  );
  const [positions, setPositions] = useState<Record<string, { x: number; y: number }>>(
    () => loadJson(POS_KEY, {}),
  );
  const [menu, setMenu] = useState<Menu | null>(null);
  const [connect, setConnect] = useState<Connection | null>(null);
  const [addOpen, setAddOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  const nodes: OntoNode[] = useMemo(
    () =>
      layoutOntology(ontology.entities, ontology.links, expanded, positions, atoms).map((n) => ({
        ...n,
        type: 'onto' as const,
      })),
    [ontology, expanded, positions, atoms],
  );
  const edges: Edge[] = useMemo(
    () =>
      boardEdges(ontology.links, new Set(nodes.map((n) => n.id))).map((e) => ({
        ...e,
        label: e.rel,
        className: `onto-edge onto-edge-${e.rel}`,
      })),
    [ontology, nodes],
  );

  const refresh = useCallback(async () => {
    try {
      onOntologyChanged(await fetchOntology());
    } catch { /* ignore */ }
  }, [onOntologyChanged]);

  const onNodesChange = useCallback((changes: NodeChange<OntoNode>[]) => {
    for (const c of changes) {
      if (c.type === 'position' && c.position) {
        setPositions((prev) => {
          const next = { ...prev, [c.id]: c.position! };
          if (!c.dragging) saveJson(POS_KEY, next);
          return next;
        });
      }
    }
  }, []);

  const onConnect = useCallback((conn: Connection) => {
    setMenu(null);
    setConnect(conn);
  }, []);

  const submitLink = useCallback(
    async (rel: string, target: string | null) => {
      const to = connect?.target || target;
      if (!connect?.source || !to) return;
      setBusy(true);
      setErr(null);
      try {
        await createOntologyLink(connect.source, to, rel);
        setConnect(null);
        await refresh();
      } catch (e) {
        setErr((e as Error).message);
      } finally {
        setBusy(false);
      }
    },
    [connect, refresh],
  );

  const removeEntity = useCallback(
    async (id: string) => {
      setBusy(true);
      setErr(null);
      try {
        await deleteOntologyEntity(id);
        setMenu(null);
        setConfirmDelete(null);
        // Drop stale board-local state for the removed entity.
        setPositions((prev) => {
          const next = { ...prev };
          delete next[id];
          saveJson(POS_KEY, next);
          return next;
        });
        setExpanded((prev) => {
          const next = new Set(prev);
          next.delete(id);
          saveJson(EXPAND_KEY, [...next]);
          return next;
        });
        await refresh();
      } catch (e) {
        setErr((e as Error).message);
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  const removeLink = useCallback(
    async (id: string) => {
      setBusy(true);
      setErr(null);
      try {
        await deleteOntologyLink(id);
        setMenu(null);
        await refresh();
      } catch (e) {
        setErr((e as Error).message);
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setMenu(null);
        setConnect(null);
        setAddOpen(false);
        setConfirmDelete(null);
        setErr(null);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const toggleExpanded = useCallback((id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      saveJson(EXPAND_KEY, [...next]);
      return next;
    });
  }, []);

  return (
    <ReactFlowProvider>
      <div className="onto-board" onClick={() => setMenu(null)}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodesChange={onNodesChange}
          onConnect={onConnect}
          onNodeClick={(_, n) => {
            setMenu(null);
            onSelect(entityToAtom(n.data.entity, atoms));
          }}
          onNodeContextMenu={(e, n) => {
            e.preventDefault();
            setMenu({ kind: 'node', x: e.clientX, y: e.clientY, entity: n.data.entity });
          }}
          onEdgeContextMenu={(e, edge) => {
            e.preventDefault();
            setMenu({ kind: 'edge', x: e.clientX, y: e.clientY, edgeId: edge.id });
          }}
          onPaneContextMenu={(e) => {
            e.preventDefault();
            const ev = e as MouseEvent;
            setMenu({ kind: 'pane', x: ev.clientX, y: ev.clientY });
          }}
          fitView
          minZoom={0.2}
          proOptions={{ hideAttribution: true }}
          colorMode="dark"
        >
          <Background gap={24} size={1} color="rgba(168,85,247,0.12)" />
          <Controls showInteractive={false} position="bottom-right" />
        </ReactFlow>

        {menu?.kind === 'pane' && (
          <div className="onto-menu" style={{ left: menu.x, top: menu.y }}>
            <button
              type="button"
              className="onto-menu-item"
              onClick={() => {
                setMenu(null);
                setAddOpen(true);
              }}
            >
              Add entity…
            </button>
          </div>
        )}
        {menu?.kind === 'node' && (
          <div className="onto-menu" style={{ left: menu.x, top: menu.y }}>
            <button
              type="button"
              className="onto-menu-item"
              onClick={() => {
                onSelect(entityToAtom(menu.entity, atoms));
                setMenu(null);
              }}
            >
              Inspect
            </button>
            {menu.entity.kind === 'class' && (
              <button
                type="button"
                className="onto-menu-item"
                onClick={() => {
                  toggleExpanded(menu.entity.id);
                  setMenu(null);
                }}
              >
                {expanded.has(menu.entity.id) ? 'Collapse instances' : 'Expand instances'}
              </button>
            )}
            <button
              type="button"
              className="onto-menu-item"
              onClick={() => {
                setConnect({ source: menu.entity.id, target: '', sourceHandle: null, targetHandle: null });
                setMenu(null);
              }}
            >
              Relate to…
            </button>
            <button
              type="button"
              className="onto-menu-item onto-menu-danger"
              disabled={busy}
              onClick={() => {
                if (confirmDelete === menu.entity.id) {
                  removeEntity(menu.entity.id);
                } else {
                  setConfirmDelete(menu.entity.id);
                }
              }}
            >
              {confirmDelete === menu.entity.id
                ? `Confirm delete ${menu.entity.name}?`
                : 'Delete entity'}
            </button>
          </div>
        )}
        {menu?.kind === 'edge' && (
          <div className="onto-menu" style={{ left: menu.x, top: menu.y }}>
            <button
              type="button"
              className="onto-menu-item onto-menu-danger"
              disabled={busy}
              onClick={() => removeLink(menu.edgeId)}
            >
              Delete link
            </button>
          </div>
        )}
        {err && !connect && !addOpen && (
          <div className="onto-error onto-error-float" role="alert">{err}</div>
        )}

        {connect && (
          <RelateDialog
            entities={ontology.entities}
            sourceId={connect.source}
            targetId={connect.target || null}
            busy={busy}
            error={err}
            onSubmit={submitLink}
            onClose={() => {
              setConnect(null);
              setErr(null);
            }}
          />
        )}
        {addOpen && (
          <AddEntityDialog
            busy={busy}
            error={err}
            onBusy={setBusy}
            onError={setErr}
            onCreated={async () => {
              setAddOpen(false);
              await refresh();
            }}
            onClose={() => {
              setAddOpen(false);
              setErr(null);
            }}
          />
        )}
      </div>
    </ReactFlowProvider>
  );
}

function RelateDialog({
  entities,
  sourceId,
  targetId,
  busy,
  error,
  onSubmit,
  onClose,
}: {
  entities: OntologyEntity[];
  sourceId: string | null;
  targetId: string | null;
  busy: boolean;
  error: string | null;
  onSubmit: (rel: string, target: string | null) => void;
  onClose: () => void;
}) {
  const [target, setTarget] = useState(targetId ?? '');
  const [rel, setRel] = useState<string>(REL_OPTIONS[0].value);
  const nameOf = (id: string | null) => entities.find((e) => e.id === id)?.name ?? id ?? '?';
  return (
    <div className="onto-dialog" role="dialog" aria-label="Create relation">
      <h3>Create relation</h3>
      <p className="onto-dialog-row">
        <strong>{nameOf(sourceId)}</strong>
        <select value={rel} onChange={(e) => setRel(e.target.value)} aria-label="Relation type">
          {REL_OPTIONS.map((r) => (
            <option key={r.value} value={r.value}>
              {r.label}
            </option>
          ))}
        </select>
        {targetId ? (
          <strong>{nameOf(targetId)}</strong>
        ) : (
          <select value={target} onChange={(e) => setTarget(e.target.value)} aria-label="Target entity">
            <option value="">choose target…</option>
            {entities
              .filter((e) => e.id !== sourceId)
              .map((e) => (
                <option key={e.id} value={e.id}>
                  {e.name}
                </option>
              ))}
          </select>
        )}
      </p>
      {error && <p className="onto-error">{error}</p>}
      <div className="onto-dialog-actions">
        <button type="button" className="quiet-button" onClick={onClose}>
          Cancel
        </button>
        <button
          type="button"
          className="quiet-button is-active"
          disabled={busy || (!targetId && !target)}
          onClick={() => onSubmit(rel, targetId ? null : target || null)}
        >
          Create
        </button>
      </div>
    </div>
  );
}

function AddEntityDialog({
  busy,
  error,
  onBusy,
  onError,
  onCreated,
  onClose,
}: {
  busy: boolean;
  error: string | null;
  onBusy: (v: boolean) => void;
  onError: (v: string | null) => void;
  onCreated: () => void;
  onClose: () => void;
}) {
  const [kind, setKind] = useState<'class' | 'instance' | 'metric'>('class');
  const [name, setName] = useState('');
  const [atomId, setAtomId] = useState('');
  return (
    <div className="onto-dialog" role="dialog" aria-label="Add entity">
      <h3>Add entity</h3>
      <label className="onto-field">
        <span>kind</span>
        <select value={kind} onChange={(e) => setKind(e.target.value as typeof kind)}>
          <option value="class">class</option>
          <option value="instance">instance</option>
          <option value="metric">metric</option>
        </select>
      </label>
      <label className="onto-field">
        <span>name</span>
        <input value={name} onChange={(e) => setName(e.target.value)} autoFocus />
      </label>
      <label className="onto-field">
        <span>backing atom (optional)</span>
        <input
          value={atomId}
          onChange={(e) => setAtomId(e.target.value)}
          placeholder="atom id"
        />
      </label>
      {error && <p className="onto-error">{error}</p>}
      <div className="onto-dialog-actions">
        <button type="button" className="quiet-button" onClick={onClose}>
          Cancel
        </button>
        <button
          type="button"
          className="quiet-button is-active"
          disabled={busy || !name.trim()}
          onClick={async () => {
            onBusy(true);
            onError(null);
            try {
              await createOntologyEntity({
                kind,
                name: name.trim(),
                ...(atomId.trim() ? { atom_id: atomId.trim() } : {}),
              });
              onCreated();
            } catch (e) {
              onError((e as Error).message);
            } finally {
              onBusy(false);
            }
          }}
        >
          Add
        </button>
      </div>
    </div>
  );
}
