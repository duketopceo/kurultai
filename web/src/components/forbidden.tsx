/** 403 panel for signed-in non-admins (WEB_ADMIN_EMAILS / role miss). */
export default function Forbidden({ email }: { email: string | null }) {
  return (
    <div className="panel forbidden">
      <p className="eyebrow">403 — admin only</p>
      <h1>Not a hub admin</h1>
      <p className="lede">
        {email ? (
          <>
            <code>{email}</code> is signed in but is not on the admin
            allowlist.
          </>
        ) : (
          "This account is not on the admin allowlist."
        )}{" "}
        Access is granted via <code>WEB_ADMIN_EMAILS</code> or a Clerk{" "}
        <code>publicMetadata.role = &quot;admin&quot;</code>.
      </p>
      <p className="hint">
        If you should be an admin, ask the operator to add your Clerk account
        email to the deployment env.
      </p>
    </div>
  );
}
