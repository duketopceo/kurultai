"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const LINKS = [
  { href: "/dashboard", label: "Hub status", exact: true },
  { href: "/dashboard/approvals", label: "Approvals", exact: false },
  { href: "/dashboard/keys", label: "Keys", exact: false },
  { href: "/dashboard/scopes", label: "Scopes", exact: false },
  { href: "/dashboard/account", label: "Account", exact: false },
];

/** Sub-nav for the admin console — active link follows the pathname. */
export default function DashNav() {
  const pathname = usePathname();
  return (
    <nav className="dash-nav" aria-label="Hub admin">
      {LINKS.map((l) => {
        const active = l.exact
          ? pathname === l.href
          : pathname.startsWith(l.href);
        return (
          <Link
            key={l.href}
            href={l.href}
            className={active ? "dash-link dash-link-active" : "dash-link"}
            aria-current={active ? "page" : undefined}
          >
            {l.label}
          </Link>
        );
      })}
    </nav>
  );
}
