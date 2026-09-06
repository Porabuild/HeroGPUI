import type { Metadata } from "next";
import { Callout } from "@/components/ui/callout";
import { PageHeader } from "@/components/ui/page-header";
import { ComponentCatalog, type CatalogGroup } from "@/components/catalog/component-catalog";
import { getCatalog } from "@/lib/catalog";

export const metadata: Metadata = {
  title: "Components",
  description:
    "The HeroGPUI component pages, grouped by category, each with the running component, its Rust code, and API reference data.",
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
      // A category entry without a matching component record is a pipeline
      // inconsistency; skip the card rather than render a broken one.
      return component && component.slug && component.title ? [component] : [];
    }),
  }));

  const totalCount = groups.reduce((sum, group) => sum + group.components.length, 0);
  const referenceCount = groups.reduce(
    (sum, group) => sum + group.components.filter((component) => component.hasReference).length,
    0,
  );

  return (
    <>
      <PageHeader
        title="Components"
        description={`${totalCount} pages cover the 71 components HeroUI documents. Each page runs the component as WebAssembly with its Rust code, and ${referenceCount} pages include API reference data.`}
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
