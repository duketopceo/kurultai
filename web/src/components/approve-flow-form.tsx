"use client";

import { useActionState } from "react";
import {
  approveDeviceAction,
  type SimpleState,
} from "@/app/dashboard/actions";

const initial: SimpleState = { status: "idle" };

/**
 * Per-flow approve control — free-text team with a datalist seeded from the
 * scopes census (existing teams + "new team" free text, mirroring /connect).
 */
export default function ApproveFlowForm({
  code,
  teamHint,
  teams,
  disabled,
}: {
  code: string;
  teamHint: string | null;
  teams: string[];
  disabled: boolean;
}) {
  const [state, formAction, pending] = useActionState(
    approveDeviceAction,
    initial,
  );

  if (state.status === "ok") {
    return <span className="badge badge-ok">{state.message}</span>;
  }

  return (
    <form action={formAction} className="approve-form">
      <input type="hidden" name="code" value={code} />
      <input
        name="team_id"
        required
        list="team-options"
        defaultValue={teamHint ?? ""}
        placeholder="team (e.g. eng)"
        pattern="[a-z0-9._\-]{1,64}"
        title="1-64 chars from [a-z0-9._-]"
        autoComplete="off"
        disabled={disabled || pending}
      />
      <button
        type="submit"
        className="btn btn-small btn-primary"
        disabled={disabled || pending}
      >
        {pending ? "Approving…" : "Approve"}
      </button>
      {state.status === "error" && (
        <span className="form-error" role="alert">
          {state.message}
        </span>
      )}
    </form>
  );
}
