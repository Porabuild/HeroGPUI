import { CodeBlock } from "@/components/ui/code-block";
import { SITE } from "@/lib/nav";
import { CtaLink } from "@/components/landing/cta-link";
import { Link } from "@heroui/react";

const INSTALL_TOML = `[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee", features = ["font-kit", "wayland", "x11", "runtime_shaders"] }
herogpui = { path = "../HeroGPUI/crates/herogpui" }`;

export function FinalCta() {
  return (
    <section className="landing-final-cta border-t border-separator">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-20 sm:px-6 md:py-28">
        <div className="mx-auto max-w-2xl text-center">
          <h2 className="text-3xl font-semibold tracking-tight text-balance text-foreground sm:text-4xl">
            Build your desktop UI in Rust.
          </h2>
          <p className="mt-4 text-base leading-relaxed text-muted">
            HeroGPUI ships with a desktop gallery and documentation for every component. Build from
            one codebase on Windows, macOS or Linux.
          </p>
        </div>

        <div className="mx-auto mt-10 max-w-3xl">
          <CodeBlock code={INSTALL_TOML} filename="Cargo.toml" lang="toml" />
          <p className="mt-3 text-xs leading-relaxed text-muted">
            The library is added as a git/path dependency. See the{" "}
            <Link
              className="text-accent transition-colors hover:text-accent-soft no-underline"
              href="/docs/getting-started/installation"
            >
              installation guide
            </Link>{" "}
            for the full steps.
          </p>
        </div>

        <div className="mt-8 flex flex-wrap items-center justify-center gap-3">
          <CtaLink href="/docs/getting-started" variant="primary">
            Read the docs
          </CtaLink>
          <CtaLink href="/docs/components" variant="outline">
            Browse components
          </CtaLink>
          <CtaLink href={SITE.github} rel="noopener noreferrer" target="_blank" variant="ghost">
            GitHub
          </CtaLink>
        </div>
      </div>
    </section>
  );
}
