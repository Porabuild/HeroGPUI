import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { notFound } from "next/navigation";
import { MigrationBody } from "@/app/docs/releases/markdown";
import { PageHeader } from "@/components/ui/page-header";
import { getGuide, getMigrations } from "@/lib/migrations";
import { pageMetadata } from "@/lib/seo";

interface MigrationPageProps {
  params: Promise<{ version: string }>;
}

/*
 * The upgrade guides are generated from the repository's `docs/migration-*.md`
 * by scripts/extract-migrations.mjs (`pnpm run extract:check` gates it). The
 * bodies stay markdown and render through the release notes renderer, whose
 * `MigrationBody` gives each heading the GitHub-style id the guide's own
 * in-page links target.
 */

/** One static page per guide; every other version 404s. */
export const dynamicParams = false;

export function generateStaticParams(): { version: string }[] {
  return getMigrations().guides.map((guide) => ({ version: guide.slug }));
}

export async function generateMetadata({ params }: MigrationPageProps): Promise<Metadata> {
  const { version } = await params;
  const guide = getGuide(version);
  if (!guide) return {};
  return pageMetadata({
    path: `/docs/migration/${guide.slug}`,
    title: guide.title,
    description: `The breaking API and behaviour changes to check when a HeroGPUI build moves to ${guide.version}.`,
  });
}

export default async function MigrationGuidePage({ params }: MigrationPageProps) {
  const { version } = await params;
  const guide = getGuide(version);
  if (!guide) notFound();
  const others = getMigrations().guides.filter((entry) => entry.slug !== guide.slug);

  return (
    <>
      <PageHeader
        title={guide.title}
        description={`The changes in ${guide.version} that can break a build or change behaviour an application relied on.`}
      />
      <MigrationBody body={guide.body} />
      {others.length > 0 && (
        <p className="mt-12">
          Other guides:{" "}
          {others.map((entry, index) => (
            <span key={entry.slug}>
              {index > 0 && ", "}
              <Link href={`/docs/migration/${entry.slug}`}>{entry.title}</Link>
            </span>
          ))}
          .
        </p>
      )}
    </>
  );
}
