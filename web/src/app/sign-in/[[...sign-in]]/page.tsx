import { SignIn } from "@clerk/nextjs";

export default function SignInPage() {
  return (
    <div className="auth-frame">
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
