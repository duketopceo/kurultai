import { Show, SignInButton } from "@clerk/nextjs";
import Link from "next/link";

export default function HomePage() {
  return (
    <div className="hero">
      <p className="eyebrow">Kurultai · shared tier</p>
      <h1>Kurultai</h1>
      <p className="lede">
        Sign in to the team brain. Personal knowledge stays on your machine via
        the CLI — this app is the shared org surface only.
      </p>
      <div className="actions">
        <Show when="signed-out">
          <SignInButton mode="modal">
            <button type="button" className="btn btn-primary">
              Continue with GitHub
            </button>
          </SignInButton>
        </Show>
        <Show when="signed-in">
          <Link href="/dashboard" className="btn btn-primary">
            Open dashboard
          </Link>
        </Show>
        <a
          className="btn"
          href="https://github.com/duketopceo/kurultai"
          target="_blank"
          rel="noreferrer"
        >
          CLI / docs
        </a>
      </div>
      <p className="hint">
        Enable the <strong>GitHub</strong> social connection in your Clerk
        dashboard. See <code>web/README.md</code>.
      </p>
    </div>
  );
}
