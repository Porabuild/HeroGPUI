"use client";

import { Button, Chip, Link } from "@heroui/react";
import { Menu, X } from "lucide-react";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { GitHubIcon } from "@/components/site/github-icon";
import { ThemeToggle } from "@/components/site/theme-toggle";
import { isNavLinkActive, NAV_LINKS, SITE } from "@/lib/nav";

const NAV_LINK_CLASS = "relative px-3 py-1.5 transition-colors";

/**
 * Sticky site navbar. The mobile menu is inline (no overlay) so it works
 * without waiting on a portal — it toggles below the bar under `md`.
 */
export function Navbar() {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <header className="site-header sticky top-0 z-40 border-b border-separator bg-background/85 backdrop-blur">
      <div className="mx-auto flex h-16 w-full max-w-[1440px] items-center gap-3 px-4 sm:px-6">
        <Link
          aria-label="HeroGPUI, a Porabuild project"
          className="site-brand flex shrink-0 items-center gap-2.5 py-2 text-foreground"
          href="/"
        >
          <span aria-hidden="true" className="site-brand-dot" />
          <span className="site-wordmark">
            Hero<span className="text-accent">GPUI</span>
          </span>
          <Chip className="site-version-chip hidden sm:inline-flex" size="sm" variant="soft">
            {SITE.version}
          </Chip>
        </Link>

        <nav
          aria-label="Primary navigation"
          className="site-nav ml-4 hidden items-center gap-1 md:flex"
        >
          {NAV_LINKS.map((link) => {
            const active = isNavLinkActive(pathname, link);
            return (
              <Link
                aria-current={active ? "page" : undefined}
                className={
                  active
                    ? `${NAV_LINK_CLASS} text-foreground`
                    : `${NAV_LINK_CLASS} text-muted hover:text-foreground`
                }
                href={link.href}
                key={link.href}
              >
                {link.label}
              </Link>
            );
          })}
        </nav>

        <div className="site-header-actions ml-auto flex items-center gap-1.5">
          <Link
            aria-label="View source on GitHub"
            className="hidden size-10 items-center justify-center rounded-md text-muted transition-colors hover:text-foreground sm:inline-flex"
            href={SITE.github}
            rel="noopener noreferrer"
            target="_blank"
          >
            <GitHubIcon className="size-[18px]" />
          </Link>
          <ThemeToggle />
          <Button
            aria-expanded={menuOpen}
            aria-label={menuOpen ? "Close navigation" : "Open navigation"}
            className="size-10 md:hidden"
            isIconOnly
            onPress={() => setMenuOpen((open) => !open)}
            size="sm"
            variant="ghost"
          >
            {menuOpen ? <X className="size-4" /> : <Menu className="size-4" />}
          </Button>
        </div>
      </div>

      {menuOpen && (
        <nav
          aria-label="Primary navigation on mobile"
          className="site-nav-mobile border-t border-separator md:hidden"
        >
          <ul className="mx-auto w-full max-w-[1440px] px-4 py-2 sm:px-6">
            {NAV_LINKS.map((link) => {
              const active = isNavLinkActive(pathname, link);
              return (
                <li key={link.href}>
                  <Link
                    className={`block rounded-md px-3 py-2 text-sm font-medium ${
                      active
                        ? "bg-default-soft text-foreground"
                        : "text-muted hover:text-foreground"
                    }`}
                    href={link.href}
                    onPress={() => setMenuOpen(false)}
                  >
                    {link.label}
                  </Link>
                </li>
              );
            })}
            <li>
              <Link
                className="flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium text-muted hover:text-foreground"
                href={SITE.github}
                rel="noopener noreferrer"
                target="_blank"
              >
                <GitHubIcon className="size-4" />
                GitHub
              </Link>
            </li>
          </ul>
        </nav>
      )}
    </header>
  );
}
