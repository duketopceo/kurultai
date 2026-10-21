import type { Metadata } from "next";
import Link from "next/link";
import {
  ClerkProvider,
  Show,
  SignInButton,
  UserButton,
} from "@clerk/nextjs";
import "./globals.css";

export const metadata: Metadata = {
  title: "Kurultai — App",
  description: "Team Kurultai: Sign in with GitHub, then manage your shared brain.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <ClerkProvider>
      <html lang="en">
        <body>
          <header className="site-header">
            <Link href="/" className="brand">
              kurultai
            </Link>
            <nav className="auth-nav">
              <Show when="signed-out">
                {/* Admin console is invite-only — sign-up stays disabled in
                    the Clerk dashboard; admins are allowlisted via
                    WEB_ADMIN_EMAILS. */}
                <SignInButton mode="modal">
                  <button type="button" className="btn">
                    Sign in
                  </button>
                </SignInButton>
              </Show>
              <Show when="signed-in">
                <Link href="/dashboard" className="nav-link">
                  Dashboard
                </Link>
                <UserButton />
              </Show>
            </nav>
          </header>
          <main>{children}</main>
        </body>
      </html>
    </ClerkProvider>
  );
}
