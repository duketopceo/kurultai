import Link from "next/link";
import { getAdminIdentity } from "@/lib/admin-gate";
import {
  listActivity,
  listDeviceFlows,
  listKeys,
  listScopes,
  type HubResult,
} from "@/lib/hub-admin";
import HubError from "@/components/hub-error";
import { fmtAge, fmtTime, isExpired } from "@/lib/format";

function errOf<T>(r: HubResult<T>, label: string): string | null {
  return r.ok ? null : `${label}: ${r.error}`;
}

/** Hub status — card overview + recent admin/write activity. */
export default async function DashboardPage() {
  const admin = await getAdminIdentity();
  if (!admin) return null; // layout renders the 403 panel

  const [keys, flows, scopes, activity] = await Promise.all([
    listKeys(),
    listDeviceFlows(),
    listScopes(),
    listActivity(10),
  ]);
  const errors = [
    errOf(keys, "keys"),
    errOf(flows, "device flows"),
    errOf(scopes, "scopes"),
    errOf(activity, "activity"),
  ].filter((e): e is string => Boolean(e));

  const keyRows = keys.ok ? keys.data.keys : [];
  const activeKeys = keyRows.filter((k) => !k.revoked_at);
  const adminKeys = activeKeys.filter((k) => k.is_admin);
  const flowRows = flows.ok ? flows.data.flows : [];
  const pending = flowRows.filter(
    (f) => f.status === "pending" && !isExpired(f.expires_at),
  );
  const scopeRows = scopes.ok ? scopes.data.scopes : [];
  const atoms = scopeRows.reduce((n, s) => n + s.atoms, 0);
  const teams = new Set(
    scopeRows
      .map((s) => s.team_id)
      .filter((t): t is string => Boolean(t))
      .concat(activeKeys.map((k) => k.team_id)),
  );
  const entries = activity.ok ? activity.data.entries : [];

  return (
    <div className="panel">
      <div className="panel-head">
        <div>
          <p className="eyebrow">Hub admin console</p>
          <h1>Hub status</h1>
        </div>
      </div>

      <HubError errors={errors} />

      <div className="card-grid">
        <Link href="/dashboard/keys" className="card">
          <span className="stat-num">{activeKeys.length}</span>
          <span className="stat-label">
            active keys · {adminKeys.length} admin
          </span>
        </Link>
        <Link href="/dashboard/approvals" className="card">
          <span className="stat-num">{pending.length}</span>
          <span className="stat-label">pending approvals</span>
        </Link>
        <Link href="/dashboard/scopes" className="card">
          <span className="stat-num">{teams.size}</span>
          <span className="stat-label">teams</span>
        </Link>
        <Link href="/dashboard/scopes" className="card">
          <span className="stat-num">{atoms}</span>
          <span className="stat-label">shared atoms</span>
        </Link>
      </div>

      <section className="settings-block">
        <h2>Recent hub activity</h2>
        {activity.ok && entries.length === 0 && (
          <p className="hint">No hub activity recorded yet.</p>
        )}
        {entries.length > 0 && (
          <table className="table">
            <thead>
              <tr>
                <th>When</th>
                <th>Agent</th>
                <th>Team</th>
                <th>Namespace</th>
                <th>Via</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => (
                <tr key={e.id}>
                  <td title={e.at}>{fmtAge(e.at)}</td>
                  <td>{e.agent_id}</td>
                  <td>{e.team_id}</td>
                  <td>{e.namespace}</td>
                  <td>{e.transport}</td>
                  <td className="cell-ellipsis">{e.reason ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {entries.length > 0 && (
          <p className="hint">
            Last activity {fmtTime(entries[0]?.at)} — full feed under{" "}
            <Link href="/dashboard/scopes">Scopes</Link>.
          </p>
        )}
      </section>
    </div>
  );
}
