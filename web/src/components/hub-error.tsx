import { hubApiUrl } from "@/lib/hub-admin";

/**
 * Banner shown when the hub daemon is unreachable or a call failed.
 * Server component — safe to render anywhere under the admin gate.
 */
export default function HubError({ errors }: { errors: string[] }) {
  if (errors.length === 0) return null;
  const url = hubApiUrl();
  return (
    <div className="banner banner-error" role="alert">
      <p className="banner-title">
        Hub daemon {url ? `at ${url} ` : ""}not answering
      </p>
      <ul>
        {errors.map((e) => (
          <li key={e}>{e}</li>
        ))}
      </ul>
      <p className="hint">
        Check <code>KURULTAI_HUB_API_URL</code> /{" "}
        <code>KURULTAI_ADMIN_KEY</code> and that the hub daemon is running
        with <code>KURULTAI_FEATURE_HUB=1</code> + Postgres.
      </p>
    </div>
  );
}
