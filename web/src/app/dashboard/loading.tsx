/** Route-level loading state for the admin console. */
export default function Loading() {
  return (
    <div className="panel">
      <p className="eyebrow">Loading</p>
      <p className="lede">Talking to the hub daemon…</p>
    </div>
  );
}
