import { readFileSync } from "node:fs";
import path from "node:path";

/**
 * Typed reader for the data pipeline's `src/data/migrations.json` — the
 * repository's upgrade guides (`docs/migration-*.md`), extracted by
 * `scripts/extract-migrations.mjs`. Like `@/lib/catalog`, the loader parses
 * defensively and falls back to empty data so the site builds even while the
 * pipeline is regenerating.
 */

export interface MigrationGuide {
  /** The minor version the guide upgrades to, e.g. "0.15". */
  version: string;
  /** The route segment, the version with the dot replaced ("0-15"). */
  slug: string;
  /** The guide's first heading, e.g. "Migrating from 0.14 to 0.15". */
  title: string;
  /** The markdown after the title, links rewritten for the site. */
  body: string;
}

export interface MigrationsFile {
  source: string;
  repository: string;
  /** Newest version first. */
  guides: MigrationGuide[];
}

const EMPTY: MigrationsFile = { source: "docs/migration-*.md", repository: "", guides: [] };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function parseGuide(value: unknown): MigrationGuide | null {
  if (!isRecord(value)) return null;
  const slug = asString(value.slug);
  if (slug === "") return null;
  return {
    version: asString(value.version),
    slug,
    title: asString(value.title),
    body: asString(value.body),
  };
}

function parseMigrations(raw: unknown): MigrationsFile {
  if (!isRecord(raw)) return EMPTY;
  const guides = Array.isArray(raw.guides)
    ? raw.guides.flatMap((entry) => {
        const parsed = parseGuide(entry);
        return parsed ? [parsed] : [];
      })
    : [];
  return {
    source: asString(raw.source) || EMPTY.source,
    repository: asString(raw.repository),
    guides,
  };
}

let cached: MigrationsFile | null = null;

export function getMigrations(): MigrationsFile {
  if (cached) return cached;
  try {
    const file = path.join(process.cwd(), "src", "data", "migrations.json");
    cached = parseMigrations(JSON.parse(readFileSync(file, "utf8")) as unknown);
  } catch {
    cached = EMPTY;
  }
  return cached;
}

/** One guide by its route slug ("0-15"), or null. */
export function getGuide(slug: string): MigrationGuide | null {
  return getMigrations().guides.find((guide) => guide.slug === slug) ?? null;
}

/** Sidebar links for the upgrade-guides section, newest first. */
export function getMigrationSidebarLinks(): { href: string; label: string }[] {
  return getMigrations().guides.map((guide) => ({
    href: `/docs/migration/${guide.slug}`,
    label: `Upgrade to ${guide.version}`,
  }));
}
