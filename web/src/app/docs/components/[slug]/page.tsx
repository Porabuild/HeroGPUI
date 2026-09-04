import { Chip, Link } from "@heroui/react";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import type { ReactNode } from "react";
import { GalleryFrame } from "@/components/preview/gallery-frame";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { PropsTable } from "@/components/ui/props-table";
import { getCatalog } from "@/lib/catalog";
import { getComponentReference, getRustExamples, type RustExample } from "./data";
import { buildExampleSections, type ExampleSection } from "./examples";
import { PartsTable, StatesTable, StylingTable } from "./reference-tables";

interface ComponentPageProps {
  params: Promise<{ slug: string }>;
}

export function generateStaticParams(): { slug: string }[] {
  return Object.keys(getCatalog().components).map((slug) => ({ slug }));
}

export async function generateMetadata({ params }: ComponentPageProps): Promise<Metadata> {
  const { slug } = await params;
  const component = getCatalog().components[slug];
  if (!component) return {};
  return { title: component.title, description: component.description };
}

interface ExampleCardProps {
  id: string;
  heading: string;
  description?: string;
  code: string;
  preview?: ReactNode;
}

function ExampleCard({ id, heading, description, code, preview }: ExampleCardProps) {
  return (
    <section aria-labelledby={id} className="mt-10">
      <h3 className="text-xl font-semibold text-foreground" id={id}>
        {heading}
      </h3>
      {description ? <p className="mt-2 text-sm leading-6 text-muted">{description}</p> : null}
      <div className="mt-4 overflow-hidden rounded-xl border border-separator bg-surface">
        {preview ? (
          <div className="border-b border-separator bg-surface-secondary p-4">{preview}</div>
        ) : null}
        <CodeBlock
          className="rounded-none border-0 bg-transparent"
          code={code}
          id={`${id}-code`}
          lang="rust"
        />
      </div>
    </section>
  );
}

function exampleCode(example: RustExample): string {
  const imports = example.imports?.trim();
  return imports ? `${imports}\n\n${example.code}` : example.code;
}

/** The Usage card shows this example's code; prefer the section the gallery
    calls "Usage", otherwise the first section. */
function pickUsageSection(sections: ExampleSection[]): ExampleSection | undefined {
  return sections.find((section) => section.heading.toLowerCase() === "usage") ?? sections[0];
}

export default async function ComponentPage({ params }: ComponentPageProps) {
  const { slug } = await params;
  const catalog = getCatalog();
  const component = catalog.components[slug];
  // Every real slug is generated above; anything else is not a component.
  if (!component) notFound();

  const reference = getComponentReference(slug);
  const rustExamples = getRustExamples(slug);
  const sections = buildExampleSections(rustExamples);
  const usage = pickUsageSection(sections);
  const rest = usage ? sections.filter((section) => section !== usage) : [];
  const importLine = component.importLine || reference?.importLine || "";
  // Siblings in the same catalog category, the way getComponentSidebarGroups
  // groups components: one group per category, in catalog order.
  const category = catalog.categories.find((entry) => entry.components.includes(slug));
  const related = (category?.components ?? []).filter(
    (sibling) => sibling !== slug && catalog.components[sibling]?.title,
  );

  return (
    <>
      <PageHeader
        description={component.description}
        importLine={importLine || undefined}
        title={component.title}
      />

      {usage ? (
        <section aria-labelledby="usage">
          <h2 id="usage">Usage</h2>
          {usage?.rust.description ? (
            <p className="mt-2 text-sm leading-6 text-muted">{usage.rust.description}</p>
          ) : null}
          <div className="mt-4 overflow-hidden rounded-xl border border-separator bg-surface">
            <GalleryFrame
              bare
              section={usage.heading}
              slug={component.slug}
              title={component.title}
            />
            <div className="border-t border-separator">
              <CodeBlock
                className="rounded-none border-0 bg-transparent"
                code={exampleCode(usage.rust)}
                id={`${usage.id}-code`}
                lang="rust"
              />
            </div>
          </div>
          <p className="mt-3 flex items-center gap-2 text-xs text-muted">
            <span
              aria-hidden="true"
              className="shot-window-dot size-1.5 shrink-0 rounded-full bg-accent"
            />
            Live HeroGPUI compiled to WebAssembly. Interact with this component directly.
          </p>
        </section>
      ) : null}

      {reference ? (
        <section aria-labelledby="anatomy">
          <h2 id="anatomy">Anatomy</h2>
          {reference.requiredParts.length > 0 ? (
            <div className="mt-4">
              <p className="mb-2 text-sm font-medium text-foreground">Required parts</p>
              <div className="flex flex-wrap gap-2">
                {reference.requiredParts.map((part) => (
                  <Chip key={part} size="sm" variant="soft">
                    {part}
                  </Chip>
                ))}
              </div>
            </div>
          ) : null}
          <p className="mt-4 text-sm leading-6 text-muted">
            {component.title} composes these parts into one native GPUI control. Detailed slot
            support is listed in the API reference below.
          </p>
        </section>
      ) : null}

      {rest.length > 0 ? (
        <>
          <h2 id="examples">Examples</h2>
          <p className="mt-2 text-sm text-muted">
            These examples are the Rust builders used by the HeroGPUI desktop gallery.
          </p>

          {rest.map((section) => (
            <ExampleCard
              code={exampleCode(section.rust)}
              description={section.rust.description}
              heading={section.heading}
              id={section.id}
              key={section.id}
            />
          ))}
        </>
      ) : null}

      {reference ? (
        <section aria-labelledby="customization">
          <h2 id="customization">Customization</h2>
          <p className="mt-2 text-sm text-muted">
            Theme tokens for {component.title} and their HeroGPUI equivalents.
          </p>
          <h3 id="styling-reference">Styling reference</h3>
          <div className="mt-4">
            <StylingTable rows={reference.styling} title={component.title} />
          </div>
        </section>
      ) : null}

      {reference ? (
        <section aria-labelledby="api-reference">
          <h2 id="api-reference">API reference</h2>
          <h3 id="props">Props</h3>
          <div className="mt-4">
            <PropsTable label={`${component.title} props`} rows={reference.api} />
          </div>

          <h3 id="parts">Parts and slots</h3>
          <div className="mt-4">
            <PartsTable rows={reference.parts} title={component.title} />
          </div>

          <h3 id="states">States</h3>
          <div className="mt-4">
            <StatesTable rows={reference.states} title={component.title} />
          </div>
        </section>
      ) : (
        <Callout kind="note" title="Detailed prop documentation is not available yet">
          Detailed prop documentation is not available for {component.title} yet. The examples above
          show the available HeroGPUI usage.
        </Callout>
      )}

      {related.length > 0 ? (
        <section aria-labelledby="related">
          <h2 id="related">Related components</h2>
          <div className="mt-4 flex flex-wrap gap-x-5 gap-y-2">
            {related.map((sibling) => (
              <Link
                className="text-sm text-muted transition-colors hover:text-foreground"
                href={`/docs/components/${sibling}`}
                key={sibling}
              >
                {catalog.components[sibling].title}
              </Link>
            ))}
          </div>
        </section>
      ) : null}
    </>
  );
}
