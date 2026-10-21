"use client";

import { useActionState } from "react";
import {
  issueKeyAction,
  type IssueKeyState,
} from "@/app/dashboard/actions";
import CopyButton from "@/components/copy-button";

const initial: IssueKeyState = { status: "idle" };

/**
 * Issue-key form + show-once plaintext panel. The minted member key is
 * rendered once here for the admin to hand off; it is never listed again.
 */
export default function KeyIssueForm({ teams }: { teams: string[] }) {
  const [state, formAction, pending] = useActionState(issueKeyAction, initial);

  return (
    <section className="settings-block">
      <h2>Issue a key</h2>
      <form action={formAction} className="form-grid">
        <label>
          Agent id
          <input
            name="agent_id"
            required
            placeholder="cursor · codex · devin-2"
            autoComplete="off"
          />
        </label>
        <label>
          Team
          <input
            name="team_id"
            required
            list="team-options"
            placeholder="eng"
            pattern="[a-z0-9._\-]{1,64}"
            title="1-64 chars from [a-z0-9._-]"
            autoComplete="off"
          />
        </label>
        <label className="form-check">
          <input type="checkbox" name="is_admin" /> Admin key (full /api/hub/*
          access)
        </label>
        <button type="submit" className="btn btn-primary" disabled={pending}>
          {pending ? "Issuing…" : "Issue key"}
        </button>
        <datalist id="team-options">
          {teams.map((t) => (
            <option key={t} value={t} />
          ))}
        </datalist>
      </form>

      {state.status === "error" && (
        <p className="form-error" role="alert">
          {state.error}
        </p>
      )}

      {state.status === "ok" && state.minted && (
        <div className="minted" role="status">
          <p>
            <strong>{state.minted.agent_id}</strong> → team{" "}
            <strong>{state.minted.team_id}</strong> — key #{state.minted.id}
          </p>
          <p className="minted-key">
            <code>{state.minted.key}</code> <CopyButton text={state.minted.key} />
          </p>
          <p className="hint">
            Shown once — the daemon stores only the hash. Copy it now.
          </p>
        </div>
      )}
    </section>
  );
}
