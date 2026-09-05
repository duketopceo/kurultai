import { SignIn } from "@clerk/nextjs";

export default function SignInPage() {
  return (
    <div className="auth-frame">
      <div className="auth-copy">
        <p className="eyebrow">Human sign-in</p>
        <h1>Continue with GitHub</h1>
        <p className="lede">
          Sign in to the shared Kurultai web app. Agents should keep using
          API-key MCP auth — this form is for people only.
        </p>
      </div>
      <SignIn
        appearance={{ elements: { rootBox: "clerk-box" } }}
        routing="path"
        path="/sign-in"
        signUpUrl="/sign-up"
        forceRedirectUrl="/dashboard"
      />
    </div>
  );
}
