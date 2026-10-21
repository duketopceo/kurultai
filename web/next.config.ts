import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // The repo root also has a package-lock.json (tooling) — pin the
  // workspace root to web/ so Turbopack doesn't warn about lockfile
  // inference.
  turbopack: {
    root: dirname(fileURLToPath(import.meta.url)),
  },
  experimental: {
    // Real HTTP 403 via forbidden() for the signed-in non-admin path
    // (dashboard/forbidden.tsx boundary).
    authInterrupts: true,
  },
};

export default nextConfig;
