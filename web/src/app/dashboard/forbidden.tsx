import { currentUser } from "@clerk/nextjs/server";
import Forbidden from "@/components/forbidden";

/**
 * HTTP 403 boundary — rendered when `forbidden()` fires in the dashboard
 * layout gate (signed-in user who isn't on the admin allowlist).
 */
export default async function ForbiddenBoundary() {
  const user = await currentUser().catch(() => null);
  return (
    <Forbidden email={user?.primaryEmailAddress?.emailAddress ?? null} />
  );
}
