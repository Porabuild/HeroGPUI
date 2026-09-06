import { Link } from "@heroui/react";
import { GitHubIcon } from "@/components/site/github-icon";
import { BrandLockup } from "@/components/site/brand-lockup";
import { SITE } from "@/lib/nav";

/**
 * Site footer in the poratake composition: one hairline mono row carrying the
 * brand lockup, the attribution line, the project links, and the copyright.
 */
export function SiteFooter() {
  return (
    <footer className="site-footer border-t border-separator">
      <div className="mx-auto grid w-full max-w-[1440px] grid-cols-1 gap-6 px-4 py-9 sm:px-6 md:grid-cols-[auto_minmax(0,1fr)_auto_auto] md:items-center md:gap-9">
        <Link
          aria-label="HeroGPUI home"
          className="w-fit text-foreground no-underline hover:no-underline"
          href="/"
        >
          <BrandLockup brand="Hero" className="text-[15px]" word="GPUI" />
        </Link>

        <p className="site-footer-copy max-w-xl text-muted">
          A Porabuild project. Based on HeroUI&apos;s design system; both licensed Apache-2.0.
          HeroUI is Copyright 2025 NextUI Inc —{" "}
          <Link
            className="text-accent transition-colors hover:text-[color:var(--pb-accent-soft)] no-underline"
            href={SITE.upstream}
            rel="noopener noreferrer"
            target="_blank"
          >
            heroui.com
          </Link>
          .
        </p>

        <nav aria-label="Footer navigation">
          {/* py extends the tap target to ~40px; the matching negative margin
              keeps the visual layout unchanged. */}
          <ul className="site-footer-nav flex flex-wrap items-center gap-x-5 gap-y-2">
            <li>
              <Link
                className="-my-2.5 flex items-center gap-1.5 py-2.5 text-muted transition-colors hover:text-foreground no-underline"
                href="https://porabuild.com/"
                rel="noopener noreferrer"
                target="_blank"
              >
                Porabuild
              </Link>
            </li>
            <li>
              <Link
                className="-my-2.5 flex items-center gap-1.5 py-2.5 text-muted transition-colors hover:text-foreground no-underline"
                href={SITE.github}
                rel="noopener noreferrer"
                target="_blank"
              >
                <GitHubIcon className="size-3.5" />
                GitHub
              </Link>
            </li>
            <li>
              <Link
                className="-my-2.5 py-2.5 text-muted transition-colors hover:text-foreground no-underline"
                href={SITE.llmsTxt}
              >
                llms.txt
              </Link>
            </li>
          </ul>
        </nav>

        <span className="text-muted">© {new Date().getFullYear()} Porabuild</span>
      </div>
    </footer>
  );
}
