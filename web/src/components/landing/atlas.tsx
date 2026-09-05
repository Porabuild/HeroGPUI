import { Link } from "@heroui/react";
import { ArrowRight } from "lucide-react";
import { getCatalog } from "@/lib/catalog";
import { SectionHeading } from "@/components/landing/shared";

/**
 * The component catalog teaser: category names, counts and sample components
 * come straight from the generated catalog. If the catalog has not been
 * generated yet the section collapses to the heading and the index link.
 */
export function Atlas() {
  const catalog = getCatalog();

  return (
    <section className="landing-atlas border-y border-separator bg-surface-secondary/50">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-16 sm:px-6 md:py-24">
        <SectionHeading
          eyebrow="The index"
          sub="Browse the library by category. Each category links to the component index and its examples."
          title="Find the right component"
        />

        {catalog.categories.length > 0 && (
          <ul className="mt-12 grid gap-x-10 sm:grid-cols-2 lg:grid-cols-3">
            {catalog.categories.map((category) => (
              <li key={category.slug}>
                <Link
                  className="group block w-full border-t border-separator py-4 transition-colors no-underline hover:no-underline"
                  href="/docs/components"
                >
                  <span className="flex items-baseline justify-between gap-4">
                    <span className="font-medium text-foreground transition-colors group-hover:text-accent">
                      {category.name}
                    </span>
                    <span className="font-mono text-xs text-muted tabular-nums">
                      {category.components.length}
                    </span>
                  </span>
                  <span className="mt-1 block truncate text-xs text-muted">
                    {previewTitles(catalog.components, category.components)}
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        )}

        <Link
          className="group mt-8 inline-flex items-center gap-2 py-2.5 text-sm font-medium text-accent transition-colors hover:text-accent-soft no-underline hover:no-underline"
          href="/docs/components"
        >
          Open the component index
          <ArrowRight
            aria-hidden="true"
            className="size-4 transition-transform group-hover:translate-x-1"
          />
        </Link>
      </div>
    </section>
  );
}

/** The first component titles of a category, plus how many follow. */
function previewTitles(components: Record<string, { title: string }>, slugs: string[]): string {
  const titles = slugs.flatMap((slug) => {
    const component = components[slug];
    return component?.title ? [component.title] : [];
  });
  if (titles.length === 0) return "";
  const head = titles.slice(0, 3).join(" · ");
  const remainder = titles.length - 3;
  return remainder > 0 ? `${head} +${remainder}` : head;
}
