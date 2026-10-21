import { getAdminIdentity } from "@/lib/admin-gate";
import {
  collectTeams,
  listKeys,
  listScopes,
  type HubKeyRecord,
} from "@/lib/hub-admin";
import HubError from "@/components/hub-error";
import KeyIssueForm from "@/components/key-issue-form";
import RevokeKeyButton from "@/components/revoke-key-button";
import { fmtTime } from "@/lib/format";

/** Group active keys by team — the "members" view (keys are member seats). */
function membersByTeam(keys: HubKeyRecord[]): Map<string, HubKeyRecord[]> {
  const map = new Map<string, HubKeyRecord[]>();
  for (const k of keys.filter((k) => !k.revoked_at)) {
    const list = map.get(k.team_id) ?? [];
    list.push(k);
    map.set(k.team_id, list);
  }
  return new Map([...map.entries()].sort(([a], [b]) => a.localeCompare(b)));
}

/** Hub key issue / list / revoke. */
export default async function KeysPage() {
  const admin = await getAdminIdentity();
  if (!admin) return null; // layout renders the 403 panel

  const [keys, scopes] = await Promise.all([listKeys(), listScopes()]);
  const errors = [
    ...(keys.ok ? [] : [`keys: ${keys.error}`]),
    ...(scopes.ok ? [] : [`teams: ${scopes.error}`]),
  ];

  const rows = (keys.ok ? keys.data.keys : [])
    .slice()
    .sort((a, b) => b.created_at.localeCompare(a.created_at));
  const teams = collectTeams(
    keys.ok ? keys.data.keys : undefined,
    scopes.ok ? scopes.data.scopes : undefined,
    undefined,
  );
  const byTeam = membersByTeam(rows);

  return (
    <div className="panel">
      <div className="panel-head">
        <div>
          <p className="eyebrow">Member credentials</p>
          <h1>Hub keys</h1>
        </div>
      </div>
      <p className="lede">
        Keys authenticate agents to the hub. Only the prefix and metadata are
        listed — plaintext shows once at issue.
      </p>

      <HubError errors={errors} />

      <KeyIssueForm teams={teams} />

      {byTeam.size > 0 && (
        <section className="settings-block">
          <h2>Members by team</h2>
          <div className="chip-groups">
            {[...byTeam.entries()].map(([team, members]) => (
              <div key={team} className="chip-group">
                <span className="chip-team">{team}</span>
                {members.map((m) => (
                  <span key={m.id} className="chip">
                    {m.agent_id}
                    {m.is_admin ? " · admin" : ""}
                  </span>
                ))}
              </div>
            ))}
          </div>
        </section>
      )}

      <section className="settings-block">
        <h2>All keys</h2>
        {keys.ok && rows.length === 0 && (
          <p className="hint">No keys issued yet — mint the first one above.</p>
        )}
        {rows.length > 0 && (
          <table className="table">
            <thead>
              <tr>
                <th>Prefix</th>
                <th>Agent</th>
                <th>Team</th>
                <th>Role</th>
                <th>Created</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {rows.map((k) => (
                <tr key={k.id} className={k.revoked_at ? "row-muted" : ""}>
                  <td>
                    <code>{k.key_prefix}…</code>
                  </td>
                  <td>{k.agent_id}</td>
                  <td>{k.team_id}</td>
                  <td>
                    {k.is_admin ? (
                      <span className="badge badge-admin">admin</span>
                    ) : (
                      <span className="badge">member</span>
                    )}
                  </td>
                  <td title={k.created_at}>{fmtTime(k.created_at)}</td>
                  <td>
                    {k.revoked_at ? (
                      <span className="badge">revoked {fmtTime(k.revoked_at)}</span>
                    ) : (
                      <span className="badge badge-ok">active</span>
                    )}
                  </td>
                  <td>
                    {!k.revoked_at && (
                      <RevokeKeyButton id={k.id} agent={k.agent_id} />
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}
