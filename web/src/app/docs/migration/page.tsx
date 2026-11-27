import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { PageHeader } from "@/components/ui/page-header";
import { getMigrations } from "@/lib/migrations";
import { pageMetadata } from "@/lib/seo";

export const metadata: Metadata = pageMetadata({
  path: "/docs/migration",
  title: "Upgrade guides",
  description:
    "What to change when a HeroGPUI upgrade breaks a build: one guide per minor version, with before-and-after code.",
});

/*
 * Generated from the repository's `docs/migration-*.md` by
 * scripts/extract-migrations.mjs, the way /docs/releases is generated from
 * `CHANGELOG.md`. Each guide is one static page at /docs/migration/<slug>.
 */

export default function MigrationIndexPage() {
  const { guides } = getMigrations();

  return (
    <>
      <PageHeader
        title="Upgrade guides"
        description="What to change when a HeroGPUI upgrade breaks a build: one guide per minor version, with before-and-after code."
      />

      {guides.length === 0 ? (
        <Callout kind="note" title="No upgrade guides are listed here yet">
          This page is built from the repository&rsquo;s <code>docs/migration-*.md</code>. Until the
          data is generated, read the release notes on <Link href="/docs/releases">Releases</Link>.
        </Callout>
      ) : (
        <>
          <p>
            Each guide covers one minor version: the changes that break a build or alter behaviour,
            and the code to move from and to.
          </p>
          <ul>
            {guides.map((guide) => (
              <li key={guide.slug}>
                <Link href={`/docs/migration/${guide.slug}`}>{guide.title}</Link>
              </li>
            ))}
          </ul>
        </>
      )}
    </>
  );
}
