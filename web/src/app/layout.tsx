import type { Metadata } from "next";
import {
  ClerkProvider,
  Show,
  SignInButton,
  SignUpButton,
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
            <a href="/" className="brand">
              kurultai
            </a>
            <nav className="auth-nav">
              <Show when="signed-out">
                <SignInButton mode="modal">
                  <button type="button" className="btn">
                    Sign in
                  </button>
                </SignInButton>
                <SignUpButton mode="modal">
                  <button type="button" className="btn btn-primary">
                    Sign up
                  </button>
                </SignUpButton>
              </Show>
              <Show when="signed-in">
                <a href="/dashboard" className="nav-link">
                  Dashboard
                </a>
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
