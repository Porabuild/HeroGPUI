import { Chip, Link } from "@heroui/react";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { ComponentExampleBrowser } from "@/components/preview/gallery-frame";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { PropsTable } from "@/components/ui/props-table";
import { getCatalog } from "@/lib/catalog";
import {
  gpuiPartRows,
  gpuiPropRows,
  gpuiStateRows,
  gpuiStyleRows,
  rustRequiredParts,
  scrubDescription,
} from "@/lib/gpui-docs";
import {
  getComponentReference,
  getRustExamples,
  getWasmArtifactVersion,
  getWasmSections,
  type RustExample,
} from "./data";
import { buildExampleSections } from "./examples";
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

function exampleCode(example: RustExample): string {
  const imports = example.imports?.trim();
  return imports ? `${imports}\n\n${example.code}` : example.code;
}

export default async function ComponentPage({ params }: ComponentPageProps) {
  const { slug } = await params;
  const catalog = getCatalog();
  const component = catalog.components[slug];
  if (!component) notFound();

  const reference = getComponentReference(slug);
  const rustExamples = getRustExamples(slug);
  const sections = buildExampleSections(rustExamples);
  const wasmSections = new Set(getWasmSections(slug));
  const liveSections = sections.filter((section) => wasmSections.has(section.heading));
  const importLine = component.importLine || reference?.importLine || "";
  const category = catalog.categories.find((entry) => entry.components.includes(slug));
  const related = (category?.components ?? []).filter(
    (sibling) => sibling !== slug && catalog.components[sibling]?.title,
  );

  const rustParts = reference ? rustRequiredParts(reference.requiredParts, reference.parts) : [];
  const hasProps = reference ? gpuiPropRows(reference.api).length > 0 : false;
  const hasParts = reference ? gpuiPartRows(reference.parts).length > 0 : false;
  const hasStates = reference ? gpuiStateRows(reference.states).length > 0 : false;
  const hasStyles = reference ? gpuiStyleRows(reference.api).length > 0 : false;
  const hasApi = hasProps || hasParts || hasStates;

  return (
    <>
      {category ? (
        <p className="mb-3">
          <Link className="no-underline" href={`/docs/components#category-${category.slug}`}>
            <Chip size="sm" variant="soft">
              {category.name}
            </Chip>
          </Link>
        </p>
      ) : null}

      <PageHeader
        description={component.description}
        importLine={importLine || undefined}
        title={component.title}
      />

      {liveSections.length > 0 ? (
        <ComponentExampleBrowser
          examples={liveSections.map((section) => ({
            code: (
              <CodeBlock
                className="rounded-none border-0 bg-transparent"
                code={exampleCode(section.rust)}
                id={`${section.id}-live-code`}
                lang="rust"
              />
            ),
            description: section.rust.description
              ? scrubDescription(section.rust.description)
              : section.rust.description,
            heading: section.heading,
            id: section.id,
          }))}
          key={component.slug}
          slug={component.slug}
          title={component.title}
          wasmVersion={getWasmArtifactVersion()}
        />
      ) : null}

      {reference && (rustParts.length > 0 || hasParts) ? (
        <section aria-labelledby="anatomy">
          <h2 id="anatomy">Anatomy</h2>
          {rustParts.length > 0 ? (
            <div className="mt-4">
              <p className="mb-2 text-sm font-medium text-foreground">Rust types</p>
              <div className="flex flex-wrap gap-2">
                {rustParts.map((part) => (
                  <Chip key={part} size="sm" variant="soft">
                    {part}
                  </Chip>
                ))}
              </div>
            </div>
          ) : null}
          <p className="mt-4 text-sm leading-6 text-muted">
            {component.title} is assembled from these builders. The API reference lists each one.
          </p>
        </section>
      ) : null}

      {reference && hasStyles ? (
        <section aria-labelledby="customization">
          <h2 id="customization">Customization</h2>
          <p className="mt-2 text-sm text-muted">
            Shared appearance belongs on <code>ThemeBuilder::components</code> and{" "}
            <code>.recipe(&quot;name&quot;)</code>. Typed builders change how {component.title} looks
            at one call site. Every builder also carries <code>sx</code> for placement, width and
            flex: it takes GPUI&apos;s own styling methods and refines them over the component&apos;s
            root after the theme, so an instance override wins. Behaviour builders live in the API
            reference below.
          </p>
          <h3 id="styling-reference">Styling</h3>
          <div className="mt-4">
            <StylingTable api={reference.api} title={component.title} />
          </div>
          <div className="mt-4">
            <CodeBlock
              code={`// The one slot for caller-owned low-level styling: GPUI's
// styling methods, refined over the root element last.
.sx(|el| {
    el.bg(gpui::rgba(0xffa500ff))             // background
        .text_color(gpui::rgba(0x000000ff))   // text
        .w(gpui::px(13.))                     // size
        .h(gpui::px(12.))
})`}
              lang="rust"
            />
          </div>
        </section>
      ) : null}

      {reference && hasApi ? (
        <section aria-labelledby="api-reference">
          <h2 id="api-reference">API reference</h2>
          {hasProps ? (
            <>
              <h3 id="props">Builders</h3>
              <div className="mt-4">
                <PropsTable label={`${component.title} builders`} rows={reference.api} />
              </div>
            </>
          ) : null}

          {hasParts ? (
            <>
              <h3 id="parts">Parts</h3>
              <div className="mt-4">
                <PartsTable rows={reference.parts} title={component.title} />
              </div>
            </>
          ) : null}

          {hasStates ? (
            <>
              <h3 id="states">States</h3>
              <div className="mt-4">
                <StatesTable rows={reference.states} title={component.title} />
              </div>
            </>
          ) : null}
        </section>
      ) : (
        <Callout kind="note" title={`No API reference for ${component.title}`}>
          {liveSections.length > 0 ? (
            <>The examples above show {component.title} in use.</>
          ) : (
            <>
              There is no live preview or builder table for {component.title} yet. Run the{" "}
              <Link href="/docs/getting-started/gallery">desktop gallery</Link> to see it locally.
            </>
          )}
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
