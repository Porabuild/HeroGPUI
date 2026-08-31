import { Chip } from "@heroui/react";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import type { ReactNode } from "react";
import { GalleryFrame } from "@/components/preview/gallery-frame";
import { NativeShot } from "@/components/preview/native-shot";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { PropsTable } from "@/components/ui/props-table";
import { getCatalog } from "@/lib/catalog";
import { getComponentReference, getRustExamples, type RustExample } from "./data";
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

interface ExampleCardProps {
  id: string;
  heading: string;
  code: string;
  preview?: ReactNode;
}

function ExampleCard({ id, heading, code, preview }: ExampleCardProps) {
  return (
    <article
      aria-labelledby={id}
      className="mt-6 overflow-hidden rounded-xl border border-separator bg-surface"
    >
      {preview ? (
        <div className="border-b border-separator bg-surface-secondary p-4">{preview}</div>
      ) : null}
      <div className="px-4 pt-4">
        <h3 className="text-base font-medium text-foreground" id={id}>
          {heading}
        </h3>
      </div>
      <CodeBlock
        className="mt-3 rounded-none border-0 bg-transparent"
        code={code}
        id={`${id}-code`}
        lang="rust"
      />
    </article>
  );
}

function exampleCode(example: RustExample): string {
  const imports = example.imports?.trim();
  return imports ? `${imports}\n\n${example.code}` : example.code;
}

export default async function ComponentPage({ params }: ComponentPageProps) {
  const { slug } = await params;
  const component = getCatalog().components[slug];
  // Every real slug is generated above; anything else is not a component.
  if (!component) notFound();

  const reference = getComponentReference(slug);
  const rustExamples = getRustExamples(slug);
  const sections = buildExampleSections(rustExamples);
  const importLine = component.importLine || reference?.importLine || "";
  // Both variables are NEXT_PUBLIC_, build-time-inlined, so this check runs
  // identically on the server and in the client bundle. Unset is the
  // shipped default until the wasm artifact is hosted (see .env.example).
  const galleryConfigured = Boolean(process.env.NEXT_PUBLIC_GALLERY_URL);

  return (
    <>
      <PageHeader
        description={component.description}
        importLine={importLine || undefined}
        title={component.title}
      />

      {galleryConfigured ? (
        <section aria-labelledby="live-preview" className="mb-12">
          <h2 id="live-preview">Live preview</h2>
          <p className="mt-2 text-sm text-muted">
            Every example for {component.title} below, rendered live in the same frame.
          </p>
          <div className="mt-4">
            <GalleryFrame slug={component.slug} title={component.title} />
          </div>
        </section>
      ) : component.shot ? (
        <section aria-labelledby="native-preview" className="mb-12">
          <h2 id="native-preview">Native preview</h2>
          <div className="mt-4">
            <NativeShot
              alt={`${component.title} rendered natively by GPUI`}
              shot={component.shot}
              shotDark={component.shotDark}
            />
          </div>
        </section>
      ) : null}

      <h2 id="examples">Examples</h2>
      <p className="mt-2 text-sm text-muted">
        These examples are the Rust builders used by the HeroGPUI desktop gallery.
      </p>

      {sections.map((section) => (
        <ExampleCard
          code={exampleCode(section.rust)}
          heading={section.heading}
          id={section.id}
          key={section.id}
        />
      ))}

      {reference ? (
        <>
          <h2 id="api-reference">API reference</h2>
          <PropsTable label={`${component.title} props`} rows={reference.api} />

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
          <div className="mt-4">
            <PartsTable rows={reference.parts} title={component.title} />
          </div>

          <h2 id="states">States</h2>
          <StatesTable rows={reference.states} title={component.title} />

          <h2 id="styling-tokens">Styling tokens</h2>
          <StylingTable rows={reference.styling} title={component.title} />
        </>
      ) : (
        <Callout kind="note" title="Detailed prop documentation is not available yet">
          Detailed prop documentation is not available for {component.title} yet. The examples above
          show the available HeroGPUI usage.
        </Callout>
      )}
    </>
  );
}
