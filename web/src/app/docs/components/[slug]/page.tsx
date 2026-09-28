import { Chip, Link } from "@heroui/react";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { LiveExamplePreview, ShowLiveButton } from "@/components/preview/gallery-frame";
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
  notPortedRows,
  portSummary,
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
import { NotPortedTable, PartsTable, StatesTable, StylingTable } from "./reference-tables";

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

/** One gallery helper definition: its name and verbatim source. */
interface HelperItem {
  name: string;
  source: string;
}

/** Split a snippet's `helpers` block into its top-level items. */
function helperItems(helpers: string | undefined): HelperItem[] {
  if (!helpers) return [];
  return helpers
    .split(/\n\n(?=(?:\/\/\/|#\[|fn |const |static |struct |thread_local))/)
    .map((source) => ({
      name:
        source.match(/^(?:fn|const|static|struct)\s+([A-Za-z_]\w*)/m)?.[1] ??
        source.match(/\bstatic\s+([A-Za-z_]\w*)/)?.[1] ??
        "helper",
      source,
    }));
}

/** Anchor of one helper's block; the source hash keeps same-named helpers apart. */
function helperBlockId(item: HelperItem): string {
  let hash = 2166136261;
  for (let index = 0; index < item.source.length; index += 1) {
    hash = Math.imul(hash ^ item.source.charCodeAt(index), 16777619);
  }
  return `helper-${item.name.toLowerCase().replace(/_/g, "-")}-${(hash >>> 0).toString(36)}`;
}

/**
 * Every distinct helper the page's examples call, in first-use order. Each
 * example's Copy button carries its own helpers, so a copied example stays
 * complete; the page shows each definition once instead of under every
 * example that calls it.
 */
function pageHelpers(examples: RustExample[]): HelperItem[] {
  const seen = new Map<string, HelperItem>();
  for (const example of examples) {
    for (const item of helperItems(example.helpers)) {
      if (!seen.has(item.source)) seen.set(item.source, item);
    }
  }
  return [...seen.values()];
}

/**
 * Where the shown code runs. Every snippet is compiled against the public
 * `herogpui` API in that same context (gallery/build.rs), so this is exactly
 * what a reader needs around it.
 */
function contextNote(context: RustExample["context"]): string {
  switch (context) {
    case "view":
      return "Runs in a view's render: self is the view holding this example's state and cx is its Context<Self>.";
    case "statements":
      return "Statements with window: &mut Window and cx: &mut App in scope.";
    default:
      return "An element expression with window: &mut Window and cx: &mut App in scope.";
  }
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
  const liveIds = new Set(liveSections.map((section) => section.id));
  const helpers = pageHelpers(rustExamples);
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
  const notPorted = reference
    ? {
        api: notPortedRows("api", reference.api),
        parts: notPortedRows("parts", reference.parts),
        states: notPortedRows("states", reference.states),
      }
    : { api: [], parts: [], states: [] };
  const summary = reference
    ? portSummary(reference.api, reference.parts, reference.states)
    : { implemented: 0, partial: 0, notPorted: 0 };
  const hasApi =
    hasProps ||
    hasParts ||
    hasStates ||
    notPorted.api.length + notPorted.parts.length + notPorted.states.length > 0;

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

      {sections.length > 0 ? (
        <section aria-labelledby="usage">
          <h2 id="usage">Usage</h2>
          {liveSections.length > 0 ? (
            <LiveExamplePreview
              examples={liveSections.map(({ id, heading }) => ({ id, heading }))}
              key={component.slug}
              slug={component.slug}
              title={component.title}
              wasmVersion={getWasmArtifactVersion()}
            />
          ) : null}
          {sections.map((section) => (
            <section aria-labelledby={section.id} className="scroll-mt-24" key={section.id}>
              <div className="mt-10 flex flex-wrap items-center justify-between gap-3">
                <h3 className="m-0" id={section.id}>
                  <a className="no-underline" href={`#${section.id}`}>
                    {section.heading}
                  </a>
                </h3>
                {liveIds.has(section.id) ? (
                  <ShowLiveButton heading={section.heading} id={section.id} />
                ) : null}
              </div>
              <p className="mt-2 text-sm leading-6 text-muted" id={`${section.id}-description`}>
                {section.rust.description ? scrubDescription(section.rust.description) : null}
                {section.rust.description ? " " : null}
                <span className="text-xs">{contextNote(section.rust.context)}</span>
              </p>
              <div className="mt-3">
                <CodeBlock
                  code={exampleCode(section.rust)}
                  copyAlso={helperItems(section.rust.helpers).map(helperBlockId)}
                  id={`${section.id}-code`}
                  lang="rust"
                />
              </div>
              {section.rust.helpers ? (
                <p className="mt-2 text-xs text-muted">
                  Also calls{" "}
                  {helperItems(section.rust.helpers).map((item, index) => (
                    <span key={helperBlockId(item)}>
                      {index > 0 ? ", " : null}
                      <a href={`#${helperBlockId(item)}`}>
                        <code>{item.name}</code>
                      </a>
                    </span>
                  ))}{" "}
                  (included when you copy).
                </p>
              ) : null}
            </section>
          ))}
          {helpers.length > 0 ? (
            <section aria-labelledby="example-helpers" className="scroll-mt-24">
              <h3 className="mt-10" id="example-helpers">
                Helpers used by these examples
              </h3>
              <p className="mt-2 text-sm text-muted">
                Plain functions over the public API, defined once here and copied with each example
                that calls them.
              </p>
              {helpers.map((item) => (
                <div className="mt-3" key={helperBlockId(item)}>
                  <CodeBlock
                    code={item.source}
                    filename={item.name}
                    id={helperBlockId(item)}
                    lang="rust"
                  />
                </div>
              ))}
            </section>
          ) : null}
        </section>
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
            <code>.recipe(&quot;name&quot;)</code>. Typed builders change how {component.title}{" "}
            looks at one call site. Every builder also carries <code>sx</code> for placement, width
            and flex: it takes GPUI&apos;s own styling methods and refines them over the
            component&apos;s root after the theme, so an instance override wins. Behaviour builders
            live in the API reference below.
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
          <p className="mt-2 text-sm text-muted">
            Against HeroUI v{reference.version}: {summary.implemented} entries ported,{" "}
            {summary.partial} partial, {summary.notPorted} not ported. Partial rows are marked in
            the tables; entries not ported are listed under each table with the reason.
          </p>
          {hasProps || notPorted.api.length > 0 ? (
            <>
              <h3 id="props">Builders</h3>
              {hasProps ? (
                <div className="mt-4">
                  <PropsTable label={`${component.title} builders`} rows={reference.api} />
                </div>
              ) : null}
              <NotPortedTable
                label={`${component.title} builders not ported`}
                rows={notPorted.api}
              />
            </>
          ) : null}

          {hasParts || notPorted.parts.length > 0 ? (
            <>
              <h3 id="parts">Parts</h3>
              {hasParts ? (
                <div className="mt-4">
                  <PartsTable rows={reference.parts} title={component.title} />
                </div>
              ) : null}
              <NotPortedTable
                label={`${component.title} parts not ported`}
                rows={notPorted.parts}
              />
            </>
          ) : null}

          {hasStates || notPorted.states.length > 0 ? (
            <>
              <h3 id="states">States</h3>
              {hasStates ? (
                <div className="mt-4">
                  <StatesTable rows={reference.states} title={component.title} />
                </div>
              ) : null}
              <NotPortedTable
                label={`${component.title} states not ported`}
                rows={notPorted.states}
              />
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
