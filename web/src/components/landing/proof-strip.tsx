import { getCatalog } from "@/lib/catalog";

/** Statement band: what the library is, in one sentence. */
export function ProofStrip() {
  const catalog = getCatalog();
  const componentCount = Object.keys(catalog.components).length;
  const categoryCount = catalog.categories.length;

  return (
    <section className="landing-statement">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-16 sm:px-6 md:py-24">
        <div className="landing-statement-grid">
          <div className="landing-section-index">HeroGPUI</div>
          <div data-reveal>
            <h2>HeroGPUI is a UI library for Rust desktop applications, built on GPUI.</h2>
            <p className="landing-statement-body">
              {componentCount} components in {categoryCount} categories, OKLCH semantic tokens,
              typed Rust builders, and a desktop gallery — for Windows, macOS and Linux from one
              codebase.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
