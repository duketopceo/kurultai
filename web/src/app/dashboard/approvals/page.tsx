import { getAdminIdentity } from "@/lib/admin-gate";
import {
  collectTeams,
  listDeviceFlows,
  listKeys,
  listScopes,
  type HubDeviceFlow,
} from "@/lib/hub-admin";
import HubError from "@/components/hub-error";
import ApproveFlowForm from "@/components/approve-flow-form";
import { fmtAge, fmtExpiry, fmtTime, isExpired } from "@/lib/format";

function decidedFlows(flows: HubDeviceFlow[]): HubDeviceFlow[] {
  return flows.filter((f) => f.status !== "pending" || isExpired(f.expires_at));
}

/** Pending device-flow approvals — mint-at-approve with an admin team pick. */
export default async function ApprovalsPage() {
  const admin = await getAdminIdentity();
  if (!admin) return null; // layout renders the 403 panel

  const [flows, keys, scopes] = await Promise.all([
    listDeviceFlows(),
    listKeys(),
    listScopes(),
  ]);
  const errors = [
    ...(flows.ok ? [] : [`device flows: ${flows.error}`]),
    ...(keys.ok ? [] : [`teams: ${keys.error}`]),
    ...(scopes.ok ? [] : [`teams: ${scopes.error}`]),
  ];

  const rows = flows.ok ? flows.data.flows : [];
  const pending = rows.filter(
    (f) => f.status === "pending" && !isExpired(f.expires_at),
  );
  const decided = decidedFlows(rows).slice(0, 25);
  const teams = collectTeams(
    keys.ok ? keys.data.keys : undefined,
    scopes.ok ? scopes.data.scopes : undefined,
    rows,
  );

  return (
    <div className="panel">
      <div className="panel-head">
        <div>
          <p className="eyebrow">Device join requests</p>
          <h1>Approvals</h1>
        </div>
      </div>
      <p className="lede">
        Pending <code>kurultai connect</code> flows. Approving mints a
        team-bound member key for the device — the key goes to the CLI, never
        to this page.
      </p>

      <HubError errors={errors} />

      {/* Shared team suggestions for every approve form on this page —
          existing teams + any --team hints from the pending flows. */}
      <datalist id="team-options">
        {teams.map((t) => (
          <option key={t} value={t} />
        ))}
      </datalist>

      {flows.ok && pending.length === 0 && (
        <p className="hint">Nothing waiting for approval.</p>
      )}

      {pending.length > 0 && (
        <table className="table">
          <thead>
            <tr>
              <th>Code</th>
              <th>Agent</th>
              <th>Seat</th>
              <th>Requested team</th>
              <th>Requested</th>
              <th>Expires</th>
              <th>Approve as</th>
            </tr>
          </thead>
          <tbody>
            {pending.map((f) => (
              <tr key={f.id}>
                <td>
                  <code>{f.user_code}</code>
                </td>
                <td>{f.codename}</td>
                <td>{f.instance_id || "—"}</td>
                <td>{f.team_hint ?? "—"}</td>
                <td title={f.created_at}>{fmtAge(f.created_at)}</td>
                <td title={f.expires_at}>{fmtExpiry(f.expires_at)}</td>
                <td>
                  <ApproveFlowForm
                    code={f.user_code}
                    teamHint={f.team_hint ?? null}
                    teams={teams}
                    disabled={isExpired(f.expires_at)}
                  />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {decided.length > 0 && (
        <section className="settings-block">
          <h2>Recent decisions</h2>
          <table className="table">
            <thead>
              <tr>
                <th>Code</th>
                <th>Agent</th>
                <th>Status</th>
                <th>Team</th>
                <th>Decided</th>
                <th>By</th>
              </tr>
            </thead>
            <tbody>
              {decided.map((f) => (
                <tr key={f.id}>
                  <td>
                    <code>{f.user_code}</code>
                  </td>
                  <td>{f.codename}</td>
                  <td>
                    <span
                      className={
                        f.status === "approved" || f.status === "exchanged"
                          ? "badge badge-ok"
                          : "badge"
                      }
                    >
                      {f.status}
                      {f.status === "pending" && isExpired(f.expires_at)
                        ? " (expired)"
                        : ""}
                    </span>
                  </td>
                  <td>{f.team_id ?? "—"}</td>
                  <td title={f.approved_at ?? f.expires_at}>
                    {fmtTime(f.approved_at)}
                  </td>
                  <td>{f.approved_by ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}
    </div>
  );
}
