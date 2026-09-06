import type { Metadata } from "next";
import { readFileSync } from "node:fs";
import path from "node:path";
import { Chip, Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { PageHeader } from "@/components/ui/page-header";
import { ReleaseBody } from "./markdown";

export const metadata: Metadata = {
  title: "Releases",
  description: "Release notes for HeroGPUI, published on GitHub.",
};

const RELEASES_URL = "https://github.com/Porabuild/HeroGPUI/releases";

/*
 * `releases.json` is generated from the GitHub Releases API by
 * scripts/extract-releases.mjs. Bodies are release-note markdown, stored
 * verbatim; the page renders them with the local `./markdown` renderer.
 */

interface Release {
  tag: string;
  name: string;
  publishedAt: string;
  prerelease: boolean;
  url: string;
  body: string;
}

interface ReleasesFile {
  repository: string;
  releases: Release[];
}

const EMPTY: ReleasesFile = { repository: "Porabuild/HeroGPUI", releases: [] };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asString(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function parseRelease(value: unknown): Release | null {
  if (!isRecord(value)) return null;
  const tag = asString(value.tag);
  if (tag === "") return null;
  return {
    tag,
    name: asString(value.name, tag),
    publishedAt: asString(value.publishedAt),
    prerelease: value.prerelease === true,
    url: asString(value.url),
    body: asString(value.body),
  };
}

function getReleases(): ReleasesFile {
  try {
    const file = path.join(process.cwd(), "src", "data", "releases.json");
    const raw: unknown = JSON.parse(readFileSync(file, "utf8"));
    if (!isRecord(raw)) return EMPTY;
    const releases = Array.isArray(raw.releases)
      ? raw.releases.flatMap((entry) => {
          const parsed = parseRelease(entry);
          return parsed ? [parsed] : [];
        })
      : [];
    return { repository: asString(raw.repository, EMPTY.repository), releases };
  } catch {
    return EMPTY;
  }
}

const DAY_FORMAT = new Intl.DateTimeFormat("en-US", {
  year: "numeric",
  month: "long",
  day: "numeric",
  timeZone: "UTC",
});

function formatDay(publishedAt: string): string {
  const date = new Date(publishedAt);
  if (Number.isNaN(date.getTime())) return publishedAt;
  return DAY_FORMAT.format(date);
}

function ReleaseSection({ release }: { release: Release }) {
  return (
    <section aria-labelledby={release.tag} className="mt-12">
      <h2 id={release.tag}>{release.tag}</h2>
      <p className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-2 text-sm text-muted">
        {release.name !== release.tag && (
          <span className="font-medium text-foreground">{release.name}</span>
        )}
        {release.publishedAt !== "" && (
          <time dateTime={release.publishedAt}>{formatDay(release.publishedAt)}</time>
        )}
        {release.prerelease && (
          <Chip color="warning" size="sm" variant="soft">
            Pre-release
          </Chip>
        )}
      </p>
      <div className="mt-4">
        <ReleaseBody body={release.body} />
      </div>
      {release.url !== "" && (
        <p className="mt-4">
          <Link href={release.url} rel="noreferrer" target="_blank">
            View {release.tag} on GitHub
          </Link>
        </p>
      )}
    </section>
  );
}

export default function ReleasesPage() {
  const { releases } = getReleases();

  return (
    <>
      <PageHeader title="Releases" description="Release notes for HeroGPUI, published on GitHub." />

      <p>
        Releases are published on GitHub and listed here. Read them at{" "}
        <Link href={RELEASES_URL} rel="noreferrer" target="_blank">
          github.com/Porabuild/HeroGPUI/releases
        </Link>
        .
      </p>

      {releases.length === 0 ? (
        <Callout kind="note" title="No GitHub releases are listed here yet">
          This page is filled from the GitHub Releases API. Until a <code>vX.Y.Z</code> tag is
          published, read notes on{" "}
          <Link href={RELEASES_URL} rel="noreferrer" target="_blank">
            github.com/Porabuild/HeroGPUI/releases
          </Link>{" "}
          or start from the <Link href="/docs/getting-started/quick-start">Quick Start</Link>.
        </Callout>
      ) : (
        releases.map((release) => <ReleaseSection key={release.tag} release={release} />)
      )}

      <h2 id="how-releases-work" className="mt-12">
        How releases work
      </h2>
      <p>
        HeroGPUI uses one version across its crates and the <code>herogpui-gallery</code> CLI. Each
        release is cut from a <code>vX.Y.Z</code> Git tag that must match the workspace version in{" "}
        <code>Cargo.toml</code>. The release workflow runs the workspace test suite, builds the
        gallery for Windows, macOS and Linux, attests the binaries, and creates the GitHub Release
        that ships them. See the{" "}
        <Link href="/docs/getting-started/installation">installation guide</Link> to add HeroGPUI to
        a Rust project.
      </p>
    </>
  );
}
