import type { ReactNode } from "react";
import { auth } from "@clerk/nextjs/server";
import { forbidden, redirect } from "next/navigation";
import DashNav from "@/components/dash-nav";
import { getAdminIdentity } from "@/lib/admin-gate";

/**
 * Admin gate for every /dashboard/* surface (U8/R14):
 * signed-out → Clerk sign-in (proxy.ts protects first; this is the
 * belt-and-braces redirect), signed-in non-admin → real HTTP 403 via
 * `forbidden()` → `dashboard/forbidden.tsx` boundary.
 */
export default async function DashboardLayout({
  children,
}: {
  children: ReactNode;
}) {
  const { userId } = await auth();
  if (!userId) {
    redirect("/sign-in");
  }

  if (!(await getAdminIdentity())) {
    forbidden();
  }

  return (
    <div className="dash">
      <DashNav />
      {children}
    </div>
  );
}
