"use client";

/** Last-resort boundary — hub calls return error states, never throw. */
export default function DashboardError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <div className="panel">
      <p className="eyebrow">Console error</p>
      <h1>Something broke</h1>
      <p className="lede">
        {error.message || "An unexpected error rendered this page."}
      </p>
      <div className="actions">
        <button type="button" className="btn" onClick={reset}>
          Retry
        </button>
      </div>
    </div>
  );
}
