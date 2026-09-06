import { CodeBlock } from "@/components/ui/code-block";
import { GitHubIcon } from "@/components/site/github-icon";
import { SITE } from "@/lib/nav";
import { getCatalog } from "@/lib/catalog";
import { CtaLink } from "@/components/landing/cta-link";
import { HeroWasmShowcase } from "@/components/landing/hero-wasm-showcase";
import { Link } from "@heroui/react";

const INSTALL_TOML = `[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee", features = ["font-kit", "wayland", "x11", "runtime_shaders"] }
herogpui = { path = "../HeroGPUI/crates/herogpui" }`;

/**
 * Above the fold: Porabuild positioning, the two CTAs, Cargo.toml snippet,
 * and a live WebAssembly specimen gallery rendered directly by GPUI.
 */
export function Hero() {
  const catalog = getCatalog();
  const componentCount = Object.keys(catalog.components).length;
  const categoryCount = catalog.categories.length;

  return (
    <section className="landing-hero relative overflow-hidden">
      <div aria-hidden="true" className="landing-hero-noise" />

      <div className="relative z-[1] mx-auto grid w-full max-w-[1440px] grid-cols-1 items-center gap-12 px-4 pt-14 pb-16 sm:px-6 md:pt-20 lg:grid-cols-[minmax(0,7fr)_minmax(0,5fr)] lg:gap-16 lg:pb-24">
        <div className="min-w-0">
          <p className="landing-hero-fade landing-hero-delay-1 pb-eyebrow">
            <span aria-hidden="true" className="pb-live-dot" />
            Porabuild / open source / built on GPUI
          </p>

          <h1 className="landing-hero-fade landing-hero-delay-2 mt-8">
            A Rust UI library for desktop apps.
          </h1>

          <p className="landing-hero-fade landing-hero-delay-3 mt-6 max-w-xl text-lg leading-relaxed text-muted">
            HeroGPUI is a component library for Rust desktop applications. Built on GPUI, the
            GPU-accelerated framework behind Zed, it gives you typed builders, OKLCH semantic
            tokens, and light and dark themes.
          </p>

          <div className="landing-hero-fade landing-hero-delay-3 mt-8 flex flex-wrap items-center gap-3">
            <CtaLink href="/docs/getting-started/quick-start" variant="primary">
              Get started
            </CtaLink>
            <CtaLink href="/docs/components" variant="outline">
              Browse components
            </CtaLink>
            <CtaLink
              aria-label="HeroGPUI on GitHub"
              className="site-cta--icon"
              href={SITE.github}
              rel="noopener noreferrer"
              target="_blank"
              variant="ghost"
            >
              <GitHubIcon className="size-5" />
            </CtaLink>
          </div>

          <p className="landing-hero-footnote mt-5">Apache-2.0 · macOS · Windows · Linux</p>

          <div className="mt-10 w-full">
            <CodeBlock code={INSTALL_TOML} filename="Cargo.toml" lang="toml" wrap />
            <p className="mt-3 text-xs leading-relaxed text-muted">
              The library is added as a git/path dependency. See the{" "}
              <Link
                className="text-accent transition-colors hover:text-[color:var(--pb-accent-soft)] no-underline"
                href="/docs/getting-started/quick-start"
              >
                Quick Start
              </Link>{" "}
              for a window you can run.
            </p>
          </div>
        </div>

        {/* Live WebAssembly specimen gallery instead of static white screenshot */}
        <HeroWasmShowcase />
      </div>

      {/* Poratake spec rail: hairline rows of the library's hard numbers. */}
      <dl className="landing-specs mx-auto w-full max-w-[1440px] px-4 sm:px-6">
        <div>
          <dt>Components</dt>
          <dd>
            {componentCount} in {categoryCount} categories
          </dd>
        </div>
        <div>
          <dt>Platforms</dt>
          <dd>macOS · Windows · Linux</dd>
        </div>
        <div>
          <dt>Runtime</dt>
          <dd>GPUI · WebAssembly</dd>
        </div>
        <div>
          <dt>License</dt>
          <dd>Apache-2.0</dd>
        </div>
      </dl>
    </section>
  );
}
