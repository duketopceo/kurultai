import { useCallback, useEffect, useState } from 'react';
import * as api from '../api';
import type { OntologyResponse } from '../types';

/**
 * O3 review queue (#118): agents propose ontology changes; humans approve or
 * reject here. Approvals mutate the ontology — the parent re-fetches it.
 */
export function ProposalsPanel({
  onOntologyChanged,
}: {
  onOntologyChanged?: (onto: OntologyResponse) => void;
}) {
  const [proposals, setProposals] = useState<api.OntologyProposal[]>([]);
  const [showDecided, setShowDecided] = useState(false);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshVersion, setRefreshVersion] = useState(0);
  const refresh = useCallback(() => {
    setRefreshVersion((v) => v + 1);
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    let running = false;
    const poll = async () => {
      if (running || controller.signal.aborted) return;
      running = true;
      try {
        const list = await api.fetchOntologyProposals(
          showDecided ? undefined : 'pending',
          showDecided ? 50 : 100,
          controller.signal,
        );
        if (!controller.signal.aborted) {
          setProposals(list);
          setError(null);
        }
      } catch {
        // Keep the last snapshot during transient failures.
      } finally {
        running = false;
      }
    };
    void poll();
    const timer = window.setInterval(() => void poll(), 15000);
    return () => {
      controller.abort();
      window.clearInterval(timer);
    };
  }, [showDecided, refreshVersion]);

  const onDecide = useCallback(
    async (id: string, action: 'approve' | 'reject') => {
      setBusyId(id);
      setError(null);
      try {
        await api.decideOntologyProposal(id, action);
        if (action === 'approve' && onOntologyChanged) {
          const onto = await api.fetchOntology();
          onOntologyChanged(onto);
        }
        refresh();
      } catch (e) {
        setError(e instanceof Error ? e.message : 'decide failed');
      } finally {
        setBusyId(null);
      }
    },
    [onOntologyChanged, refresh],
  );

  const pending = proposals.filter((p) => p.status === 'pending');

  return (
    <section className="panel chrome-panel proposals-panel" aria-label="Ontology review queue">
      <header className="panel-head">
        <h2>Review queue{pending.length > 0 ? ` (${pending.length})` : ''}</h2>
        <button
          type="button"
          className="ghost"
          onClick={() => setShowDecided((v) => !v)}
          aria-pressed={showDecided}
        >
          {showDecided ? 'Pending only' : 'History'}
        </button>
      </header>
      <p className="muted proposals-caption">
        Agent-proposed ontology changes — nothing mutates until you decide.
      </p>
      {error && <p className="proposals-error" role="alert">{error}</p>}
      {proposals.length === 0 && (
        <p className="muted">No {showDecided ? '' : 'pending '}proposals.</p>
      )}
      <ul className="proposals-list">
        {proposals.map((p) => (
          <li key={p.id} className={`proposal proposal-${p.status}`}>
            <div className="proposal-head">
              <span className="proposal-kind">{p.kind.replace('_', ' ')}</span>
              <span className="proposal-desc">{api.describeProposal(p)}</span>
            </div>
            <span className="proposal-meta">
              by {p.proposed_by}
              {p.reason ? ` · ${p.reason}` : ''}
              {p.status !== 'pending' && p.decided_by
                ? ` · ${p.status} by ${p.decided_by}`
                : ''}
            </span>
            {p.status === 'pending' && (
              <div className="proposal-actions">
                <button
                  type="button"
                  className="proposal-approve"
                  disabled={busyId === p.id}
                  onClick={() => void onDecide(p.id, 'approve')}
                >
                  Approve
                </button>
                <button
                  type="button"
                  className="proposal-reject"
                  disabled={busyId === p.id}
                  onClick={() => void onDecide(p.id, 'reject')}
                >
                  Reject
                </button>
              </div>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
