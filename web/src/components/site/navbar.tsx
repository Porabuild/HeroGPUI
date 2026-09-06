"use client";

import { Button, Link } from "@heroui/react";
import { Menu, X } from "lucide-react";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { CommandPalette } from "@/components/site/command-palette";
import { GitHubIcon } from "@/components/site/github-icon";
import { ThemeToggle } from "@/components/site/theme-toggle";
import { BrandLockup } from "@/components/site/brand-lockup";
import type { SearchItem } from "@/lib/docs-nav";
import { isNavLinkActive, NAV_LINKS, SITE } from "@/lib/nav";

const NAV_LINK_CLASS = "py-1.5 font-mono text-[11px] font-medium transition-colors no-underline";

/**
 * Sticky site header in the porabuild.com/poratake composition: a utility
 * project bar (umbrella breadcrumb left, version and license right) above a
 * three-column header — brand lockup, centred mono nav, actions on the right.
 * The project bar scrolls away; the header sticks to the top edge.
 */
export function Navbar({ searchItems = [] }: { searchItems?: SearchItem[] }) {
  const pathname = usePathname();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <>
      <div className="site-project-bar border-b border-separator bg-surface">
        <div className="mx-auto flex min-h-[34px] w-full max-w-[1440px] items-center justify-between gap-6 px-4 py-2 sm:px-6">
          <div className="flex min-w-0 items-center gap-[9px]">
            <Link
              className="text-foreground no-underline hover:no-underline"
              href="https://porabuild.com/"
              rel="noopener noreferrer"
              target="_blank"
            >
              Porabuild
            </Link>
            <span aria-hidden="true">/</span>
            <span className="truncate">HeroGPUI component library</span>
          </div>
          <span className="flex-none whitespace-nowrap">{SITE.version} · Apache-2.0</span>
        </div>
      </div>

      <header className="site-header sticky top-0 z-40 border-b border-separator bg-background/85 backdrop-blur-xl">
        <div className="mx-auto grid h-[68px] w-full max-w-[1440px] grid-cols-[1fr_auto] items-center gap-4 px-4 sm:px-6 md:grid-cols-[1fr_auto_1fr]">
          <Link
            aria-label="HeroGPUI home"
            className="site-brand justify-self-start py-2 text-foreground no-underline hover:no-underline"
            href="/"
          >
            <BrandLockup brand="Hero" className="text-[18px]" word="GPUI" />
          </Link>

          <nav
            aria-label="Primary navigation"
            className="site-nav hidden items-center gap-[30px] md:flex"
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

          <div className="site-header-actions flex items-center gap-1.5 justify-self-end sm:gap-3">
            {searchItems.length > 0 ? <CommandPalette items={searchItems} /> : null}
            <ThemeToggle />
            <Link
              aria-label="View source on GitHub"
              className="site-github-link hidden items-center gap-2 no-underline hover:no-underline sm:inline-flex"
              href={SITE.github}
              rel="noopener noreferrer"
              target="_blank"
            >
              <GitHubIcon className="size-[17px]" />
              <span>GitHub</span>
            </Link>
            <Button
              aria-expanded={menuOpen}
              aria-label={menuOpen ? "Close navigation" : "Open navigation"}
              className="size-9 text-muted md:hidden"
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
                      className={`block rounded-md px-3 py-2 font-mono text-xs font-medium no-underline ${
                        active ? "text-foreground" : "text-muted hover:text-foreground"
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
                  className="flex items-center gap-2 rounded-md px-3 py-2 font-mono text-xs font-medium text-muted hover:text-foreground no-underline"
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
    </>
  );
}
