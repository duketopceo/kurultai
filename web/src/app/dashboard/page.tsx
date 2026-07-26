import { auth, currentUser } from "@clerk/nextjs/server";
import { redirect } from "next/navigation";

export default async function DashboardPage() {
  const { userId, orgId, orgSlug } = await auth();
  if (!userId) {
    redirect("/sign-in");
  }

  const user = await currentUser();
  const github =
    user?.externalAccounts?.find((a) => a.provider === "oauth_github") ??
    user?.externalAccounts?.find((a) => a.provider.includes("github"));

  return (
    <div className="panel">
      <h1>Dashboard</h1>
      <p>Signed in to this Kurultai instance.</p>
      <dl className="meta">
        <dt>User</dt>
        <dd>
          {user?.primaryEmailAddress?.emailAddress ??
            user?.username ??
            userId}
        </dd>
        <dt>GitHub</dt>
        <dd>{github?.username ?? github?.emailAddress ?? "— (link in Clerk)"}</dd>
        <dt>Organization</dt>
        <dd>
          {orgId
            ? `${orgSlug ?? orgId} — shared team brain`
            : "Personal workspace (create a Clerk Organization for team aggregate)"}
        </dd>
      </dl>
      <p className="hint">
        Multi-user model:{" "}
        <a href="https://github.com/duketopceo/kurultai/blob/main/docs/multi-user-kurultai.md">
          docs/multi-user-kurultai.md
        </a>
      </p>
    </div>
  );
}
