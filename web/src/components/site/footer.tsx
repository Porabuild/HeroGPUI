import { Link } from "@heroui/react";
import { GitHubIcon } from "@/components/site/github-icon";
import { SITE } from "@/lib/nav";

/** Site footer: project links plus the required HeroUI attribution. */
export function SiteFooter() {
  return (
    <footer className="site-footer border-t border-separator">
      <div className="mx-auto flex w-full max-w-[1440px] flex-col gap-6 px-4 py-10 sm:px-6 md:flex-row md:items-start md:justify-between">
        <div className="max-w-sm">
          <p className="site-footer-brand flex items-center gap-2 text-sm font-semibold text-foreground">
            <span className="pb-brand-lockup flex items-center text-[15px] tracking-[-0.04em]">
              <strong className="font-semibold text-foreground">Hero</strong>
              <span
                aria-hidden="true"
                className="pb-brand-dot mx-1 inline-block size-1.5 rounded-full bg-accent shadow-[0_0_8px_var(--pb-accent-glow)]"
              />
              <span className="pb-brand-lockup-word font-semibold text-accent">GPUI</span>
            </span>
            <span className="font-mono text-[10px] text-muted/60">· by Porabuild</span>
          </p>
          <p className="site-footer-copy docs-measure mt-2 text-sm text-muted">
            HeroGPUI is a Rust/GPUI UI library based on HeroUI&apos;s design system. Both projects
            are licensed under the Apache License 2.0. HeroUI is Copyright 2025 NextUI Inc; see{" "}
            <Link
              className="py-1 text-accent transition-colors hover:text-accent-soft no-underline"
              href={SITE.upstream}
              rel="noopener noreferrer"
              target="_blank"
            >
              heroui.com
            </Link>
            .
          </p>
        </div>

        <nav aria-label="Footer navigation">
          {/* py extends the tap target to ~40px; the matching negative margin
              keeps the visual layout unchanged. */}
          <ul className="site-footer-nav flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
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
                <GitHubIcon className="size-4" />
                GitHub
              </Link>
            </li>
            <li>
              <Link
                className="-my-2.5 py-2.5 text-muted transition-colors hover:text-foreground no-underline"
                href={SITE.cratesio}
                rel="noopener noreferrer"
                target="_blank"
              >
                crates.io
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
            <li>
              <Link
                className="-my-2.5 py-2.5 text-muted transition-colors hover:text-foreground no-underline"
                href={SITE.upstream}
                rel="noopener noreferrer"
                target="_blank"
              >
                HeroUI
              </Link>
            </li>
          </ul>
        </nav>
      </div>
    </footer>
  );
}
