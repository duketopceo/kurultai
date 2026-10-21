import { getAdminIdentity } from "@/lib/admin-gate";
import { listActivity, listScopes } from "@/lib/hub-admin";
import HubError from "@/components/hub-error";
import { fmtAge } from "@/lib/format";

/** Scope census — atoms by visibility × team — plus the hub activity feed. */
export default async function ScopesPage() {
  const admin = await getAdminIdentity();
  if (!admin) return null; // layout renders the 403 panel

  const [scopes, activity] = await Promise.all([listScopes(), listActivity(50)]);
  const errors = [
    ...(scopes.ok ? [] : [`scopes: ${scopes.error}`]),
    ...(activity.ok ? [] : [`activity: ${activity.error}`]),
  ];

  const rows = scopes.ok ? scopes.data.scopes : [];
  const total = rows.reduce((n, s) => n + s.atoms, 0);
  const entries = activity.ok ? activity.data.entries : [];

  return (
    <div className="panel">
      <div className="panel-head">
        <div>
          <p className="eyebrow">Visibility census</p>
          <h1>Scopes</h1>
        </div>
      </div>
      <p className="lede">
        Shared atoms grouped by <code>visibility</code> × <code>team_id</code> —
        what each team boundary actually contains (RLS-enforced underneath).
      </p>

      <HubError errors={errors} />

      {scopes.ok && rows.length === 0 && (
        <p className="hint">No shared atoms yet.</p>
      )}
      {rows.length > 0 && (
        <table className="table">
          <thead>
            <tr>
              <th>Visibility</th>
              <th>Team</th>
              <th className="num">Atoms</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((s, i) => (
              <tr key={`${s.visibility}-${s.team_id ?? ""}-${i}`}>
                <td>
                  <span
                    className={
                      s.visibility === "company" ? "badge badge-ok" : "badge"
                    }
                  >
                    {s.visibility}
                  </span>
                </td>
                <td>{s.team_id ?? "—"}</td>
                <td className="num">{s.atoms}</td>
              </tr>
            ))}
            <tr className="row-total">
              <td colSpan={2}>Total</td>
              <td className="num">{total}</td>
            </tr>
          </tbody>
        </table>
      )}

      <section className="settings-block">
        <h2>Hub activity</h2>
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
      </section>
    </div>
  );
}
