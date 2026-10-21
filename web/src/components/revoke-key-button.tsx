"use client";

import { useState, useTransition } from "react";
import { revokeKeyAction } from "@/app/dashboard/actions";

/** Revoke button with a confirm step + inline result. */
export default function RevokeKeyButton({
  id,
  agent,
}: {
  id: number;
  agent: string;
}) {
  const [pending, startTransition] = useTransition();
  const [error, setError] = useState<string | null>(null);

  return (
    <span className="revoke-cell">
      <button
        type="button"
        className="btn btn-small btn-danger"
        disabled={pending}
        onClick={() => {
          if (!window.confirm(`Revoke the key for ${agent} (#${id})?`)) {
            return;
          }
          startTransition(async () => {
            const res = await revokeKeyAction(id);
            if (res.status === "error") {
              setError(res.message ?? "revoke failed");
            }
          });
        }}
      >
        {pending ? "Revoking…" : "Revoke"}
      </button>
      {error && <span className="form-error">{error}</span>}
    </span>
  );
}
