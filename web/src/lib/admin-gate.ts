import "server-only";

import { cache } from "react";
import { auth, currentUser } from "@clerk/nextjs/server";

/**
 * Admin authorization for the hub console (U8/R14).
 *
 * A signed-in Clerk user is an admin when EITHER:
 *   - their primary email is in `WEB_ADMIN_EMAILS` (comma-separated,
 *     case-insensitive), OR
 *   - their Clerk `publicMetadata.role` is `"admin"`.
 *
 * Fail closed: no allowlist configured and no admin role → nobody is admin.
 * React `cache` dedupes the Clerk lookup within a single request (layout +
 * page + action each call this without extra fetches).
 */

export interface AdminIdentity {
  userId: string;
  email: string;
  name: string | null;
}

function adminEmails(): string[] {
  return (process.env.WEB_ADMIN_EMAILS ?? "")
    .split(",")
    .map((s) => s.trim().toLowerCase())
    .filter(Boolean);
}

/** Resolved admin identity, or null when the caller is not an admin. */
export const getAdminIdentity = cache(
  async (): Promise<AdminIdentity | null> => {
    const { userId } = await auth();
    if (!userId) return null;

    const user = await currentUser();
    const email = user?.primaryEmailAddress?.emailAddress?.toLowerCase();
    if (!email) return null;

    const role = (user?.publicMetadata as { role?: unknown } | undefined)?.role;
    if (role === "admin" || adminEmails().includes(email)) {
      return {
        userId,
        email,
        name: user?.fullName ?? user?.username ?? null,
      };
    }
    return null;
  },
);

/**
 * Gate for pages/layouts: returns the identity to render, or null → the
 * caller renders the 403 panel. Unauthenticated users never reach this —
 * `proxy.ts` already redirects them to Clerk sign-in.
 */
export async function requireAdmin(): Promise<AdminIdentity | null> {
  return getAdminIdentity();
}
