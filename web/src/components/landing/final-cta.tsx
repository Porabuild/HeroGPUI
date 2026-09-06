import { GitHubIcon } from "@/components/site/github-icon";
import { SITE } from "@/lib/nav";
import { CtaLink } from "@/components/landing/cta-link";

export function FinalCta() {
  return (
    <section className="landing-closing landing-final-cta border-t border-separator">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-20 sm:px-6 md:py-28">
        <p className="pb-eyebrow justify-center text-center">
          <span aria-hidden="true" className="pb-live-dot" />
          Porabuild / open source
        </p>
        <h2 data-reveal>Build your desktop UI in Rust.</h2>
        <p className="mx-auto mt-6 max-w-xl text-base leading-relaxed text-muted">
          HeroGPUI ships with a desktop gallery and documentation for every component. Build from
          one codebase on Windows, macOS or Linux.
        </p>

        <div className="mt-10 flex flex-wrap items-center justify-center gap-3">
          <CtaLink href="/docs/getting-started/quick-start" variant="primary">
            Get started
          </CtaLink>
          <CtaLink href="/docs/components" variant="outline">
            Browse components
          </CtaLink>
          <CtaLink
            aria-label="HeroGPUI on GitHub"
            href={SITE.github}
            rel="noopener noreferrer"
            target="_blank"
            variant="ghost"
          >
            <GitHubIcon className="size-5" />
            GitHub
          </CtaLink>
        </div>
      </div>
    </section>
  );
}
