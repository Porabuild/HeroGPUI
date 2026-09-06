import type { Metadata } from "next";
import { Callout } from "@/components/ui/callout";
import { PageHeader } from "@/components/ui/page-header";
import { ComponentCatalog, type CatalogGroup } from "@/components/catalog/component-catalog";
import { getCatalog } from "@/lib/catalog";

export const metadata: Metadata = {
  title: "Components",
  description:
    "Every HeroGPUI component, grouped by category, with a live WebAssembly preview and the Rust API.",
};

/**
 * The full component index: one card per catalog page, grouped by the
 * catalog's categories. All data comes from the generated catalog; counts are
 * computed from it, never hardcoded. The filter UI is the page's only client
 * component.
 */
export default function ComponentsPage() {
  const catalog = getCatalog();

  const groups: CatalogGroup[] = catalog.categories.map((category) => ({
    name: category.name,
    slug: category.slug,
    components: category.components.flatMap((slug) => {
      const component = catalog.components[slug];
      return component && component.slug && component.title ? [component] : [];
    }),
  }));

  const totalCount = groups.reduce((sum, group) => sum + group.components.length, 0);

  return (
    <>
      <PageHeader
        title="Components"
        description={`${totalCount} pages of typed Rust builders, grouped by what they help you build. Each page runs the component as WebAssembly next to its API.`}
      />

      {totalCount === 0 ? (
        <Callout kind="note" title="The catalog has not been generated yet">
          This index is built from <code>src/data/catalog.json</code>, which the data pipeline
          produces. It is currently empty — run the pipeline and reload.
        </Callout>
      ) : (
        <ComponentCatalog groups={groups} totalCount={totalCount} />
      )}
    </>
  );
}
