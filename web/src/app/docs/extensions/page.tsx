import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { PageHeader } from "@/components/ui/page-header";
import { ReleaseBody } from "@/app/docs/releases/markdown";
import { getExtensions } from "@/lib/extensions";
import { pageMetadata } from "@/lib/seo";

export const metadata: Metadata = pageMetadata({
  path: "/docs/extensions",
  title: "HeroGPUI extensions",
  description:
    "The APIs HeroGPUI ships beyond the HeroUI component set: shared builder traits, i18n, theme files and hot reload, a test kit, and components like TreeView, Sidebar and CommandPalette.",
});

/*
 * The HeroGPUI extensions — the APIs with no HeroUI counterpart, described by
 * the `## HeroGPUI extensions` section of the repository's llms.txt and
 * extracted into `src/data/extensions.json` by scripts/extract-extensions.mjs
 * (`pnpm run extract:check` gates it). Bodies stay markdown and render through
 * the release notes renderer; `live` lists the component-page examples that
 * demonstrate each extension, pre-anchored to the example heading.
 */

/// The qualifier as the page shows it: no markdown backticks and no
/// surrounding parentheses ("(`serde` feature)" -> "serde feature").
function plainQualifier(qualifier: string): string {
  const stripped = qualifier.replaceAll("`", "").trim();
  return stripped.startsWith("(") && stripped.endsWith(")")
    ? stripped.slice(1, -1).trim()
    : stripped;
}

export default function ExtensionsPage() {
  const { entries, componentExtensions } = getExtensions();

  return (
    <>
      <PageHeader
        title="HeroGPUI extensions"
        description="The APIs HeroGPUI ships beyond the HeroUI component set: shared builder traits, i18n, theme files and hot reload, a test kit, and components like TreeView, Sidebar and CommandPalette."
      />

      {entries.length === 0 ? (
        <Callout kind="note" title="The extension list has not been generated yet">
          This page is built from <code>src/data/extensions.json</code>, which the data pipeline
          extracts from the repository&rsquo;s <code>llms.txt</code>. It is currently empty — run
          the pipeline and reload.
        </Callout>
      ) : (
        <>
          <p>
            These APIs have no HeroUI counterpart. They ship in the same <code>herogpui</code> crate
            as the components; the ones marked with a Cargo feature need that feature enabled.
          </p>
          {entries.map((entry) => (
            <section key={entry.slug} aria-labelledby={entry.slug}>
              <h2 id={entry.slug}>{entry.name}</h2>
              {entry.qualifier !== "" && (
                // The qualifier carries the cargo-feature gate, e.g.
                // ("serde" feature). It stays out of the heading so the
                // on-page TOC and the heading's accessible name are the name
                // alone; a mono pill keeps it subordinate.
                <p className="mt-2 text-sm text-muted">
                  <code className="font-normal">{plainQualifier(entry.qualifier)}</code>
                </p>
              )}
              <ReleaseBody body={entry.body} />
              {entry.live.length > 0 && (
                <>
                  <p className="text-xs font-semibold uppercase tracking-wider text-muted">
                    See it live
                  </p>
                  <ul>
                    {entry.live.map((link) => (
                      <li key={`${link.component}-${link.anchor}`}>
                        <Link href={`/docs/components/${link.component}#${link.anchor}`}>
                          {link.title} — {link.heading}
                        </Link>
                      </li>
                    ))}
                  </ul>
                </>
              )}
            </section>
          ))}
          {componentExtensions.length > 0 && (
            <section aria-labelledby="in-component-pages">
              <h2 id="in-component-pages">In component pages</h2>
              <p>
                Each link below lands on a component-page example that is marked as a HeroGPUI
                extension and that no section above already links.
              </p>
              <ul>
                {componentExtensions.map((link) => (
                  <li key={`${link.component}-${link.anchor}`}>
                    <Link href={`/docs/components/${link.component}#${link.anchor}`}>
                      {link.title} — {link.heading}
                    </Link>
                    {" — "}
                    {/*
                     * The summary is one paragraph of markdown, so it renders
                     * through the same release-notes renderer as the bodies
                     * above; the wrappers put its single paragraph on the
                     * link's line instead of a block of its own.
                     */}
                    <div className="inline [&_p]:inline [&_p]:mt-0">
                      <ReleaseBody body={link.summary} />
                    </div>
                  </li>
                ))}
              </ul>
            </section>
          )}
        </>
      )}
    </>
  );
}
