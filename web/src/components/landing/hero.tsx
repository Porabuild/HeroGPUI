import { CodeBlock } from "@/components/ui/code-block";
import { GitHubIcon } from "@/components/site/github-icon";
import { SITE } from "@/lib/nav";
import { CtaLink } from "@/components/landing/cta-link";
import { HeroWasmShowcase } from "@/components/landing/hero-wasm-showcase";

const INSTALL_TOML = `[dependencies]
gpui = "0.2"
herogpui = "0.1"`;

/**
 * Above the fold: Porabuild positioning, the two CTAs, Cargo.toml snippet,
 * and a live WebAssembly specimen gallery rendered directly by GPUI.
 */
export function Hero() {
  return (
    <section className="landing-hero relative overflow-hidden">
      {/* Blueprint hairline grid, faded out before the proof strip. */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10"
        style={{
          backgroundImage:
            "repeating-linear-gradient(to right, color-mix(in oklab, var(--separator) 60%, transparent) 0 1px, transparent 1px 56px), repeating-linear-gradient(to bottom, color-mix(in oklab, var(--separator) 60%, transparent) 0 1px, transparent 1px 56px)",
          maskImage: "radial-gradient(120% 90% at 60% 0%, black 0%, transparent 72%)",
          WebkitMaskImage: "radial-gradient(120% 90% at 60% 0%, black 0%, transparent 72%)",
        }}
      />

      <div className="mx-auto grid w-full max-w-[1440px] items-center gap-12 px-4 pt-14 pb-16 sm:px-6 md:pt-20 lg:grid-cols-[minmax(0,7fr)_minmax(0,5fr)] lg:gap-16 lg:pb-24">
        <div>
          <div className="landing-hero-meta flex flex-wrap items-center gap-3">
            <p className="pb-eyebrow flex items-center gap-2 font-mono text-[11px] font-medium tracking-[0.06em] text-muted uppercase">
              <span className="pb-live-dot inline-block size-1.5 rounded-full bg-accent shadow-[0_0_10px_var(--pb-accent-glow)] animate-pulse" />
              PORABUILD / OPEN SOURCE
            </p>
            <span className="pb-chip font-mono text-[11px] text-muted">HeroUI for Rust</span>
            <span className="font-mono text-xs text-muted/70">Windows · macOS · Linux</span>
          </div>

          <h1 className="mt-6 text-4xl leading-[1.05] font-semibold tracking-[-0.052em] text-balance text-foreground sm:text-5xl lg:text-6xl">
            A Rust UI library for <span className="text-accent">desktop apps.</span>
          </h1>

          <p className="mt-6 max-w-xl text-lg leading-relaxed text-muted">
            HeroGPUI brings HeroUI&apos;s design system to Rust desktop applications. Built on GPUI,
            the GPU-accelerated framework behind Zed, it gives you typed builders, OKLCH semantic
            tokens, and light and dark themes.
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-3">
            <CtaLink href="/docs/getting-started" variant="primary">
              Read the docs
            </CtaLink>
            <CtaLink href="/docs/components" variant="outline">
              Browse components
            </CtaLink>
            <CtaLink
              aria-label="HeroGPUI on GitHub"
              className="px-3"
              href={SITE.github}
              rel="noopener noreferrer"
              target="_blank"
              variant="ghost"
            >
              <GitHubIcon className="size-5" />
            </CtaLink>
          </div>

          <div className="mt-10 max-w-md">
            <CodeBlock code={INSTALL_TOML} filename="Cargo.toml" lang="toml" />
            <p className="mt-3 text-xs leading-relaxed text-muted">
              Add HeroGPUI to <span className="font-mono">Cargo.toml</span> and follow the
              installation guide.
            </p>
          </div>
        </div>

        {/* Live WebAssembly specimen gallery instead of static white screenshot */}
        <HeroWasmShowcase />
      </div>
    </section>
  );
}
