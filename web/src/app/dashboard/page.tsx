import { auth, currentUser } from "@clerk/nextjs/server";
import { UserButton } from "@clerk/nextjs";
import Link from "next/link";
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
      <div className="panel-head">
        <div>
          <p className="eyebrow">Account settings</p>
          <h1>Human access</h1>
        </div>
        <UserButton
          appearance={{ elements: { avatarBox: "clerk-avatar" } }}
          showName
        />
      </div>
      <p className="lede">
        You are signed in as a human via Clerk. Agent MCP keys are managed
        separately with <code>kurultai agent add</code> — do not paste those
        here.
      </p>
      <dl className="meta">
        <dt>User</dt>
        <dd>
          {user?.primaryEmailAddress?.emailAddress ??
            user?.username ??
            userId}
        </dd>
        <dt>GitHub</dt>
        <dd>{github?.username ?? github?.emailAddress ?? "— (link in Clerk profile)"}</dd>
        <dt>Organization</dt>
        <dd>
          {orgId
            ? `${orgSlug ?? orgId} — shared team brain`
            : "Personal workspace (create a Clerk Organization for team aggregate)"}
        </dd>
      </dl>
      <section className="settings-block">
        <h2>What this controls</h2>
        <ul>
          <li>Team web app identity (this site)</li>
          <li>Org membership for the shared tier</li>
        </ul>
        <h2>What stays elsewhere</h2>
        <ul>
          <li>
            Brain UI on a locked daemon — paste a human owner/hub key from{" "}
            <code>kurultai admin key issue</code>
          </li>
          <li>
            Agents — API keys via <code>kurultai agent add</code> + MCP (unchanged)
          </li>
        </ul>
      </section>
      <p className="hint">
        Multi-user model:{" "}
        <Link href="https://github.com/duketopceo/kurultai/blob/main/docs/multi-user-kurultai.md">
          docs/multi-user-kurultai.md
        </Link>
      </p>
    </div>
  );
}
