import { Show, SignInButton } from "@clerk/nextjs";
import Link from "next/link";

export default function HomePage() {
  return (
    <div className="hero">
      <p className="eyebrow">Kurultai · shared tier</p>
      <h1>Kurultai</h1>
      <p className="lede">
        Human sign-in for the team brain. Use GitHub via Clerk below — this is
        not the agent API-key path.
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
            Open account
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
      <section className="auth-split" aria-label="Human vs agent access">
        <div>
          <h2>Humans</h2>
          <p>
            Sign in with GitHub. Manage org membership and account from the
            dashboard. Personal knowledge stays on your machine via the CLI.
          </p>
        </div>
        <div>
          <h2>Agents</h2>
          <p>
            Keep using API-key auth —{" "}
            <code>kurultai agent add &lt;codename&gt;</code> then MCP tools.
            Agent keys are separate from human Clerk login and already work.
          </p>
        </div>
      </section>
      <p className="hint">
        Enable the <strong>GitHub</strong> social connection in your Clerk
        dashboard. See <code>web/README.md</code>.
      </p>
    </div>
  );
}
