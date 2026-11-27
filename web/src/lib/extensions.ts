import { readFileSync } from "node:fs";
import path from "node:path";

/**
 * Typed reader for the data pipeline's `src/data/extensions.json` — the
 * HeroGPUI extensions section of the repository's `llms.txt`, extracted by
 * `scripts/extract-extensions.mjs`. Like `@/lib/catalog`, the loader parses
 * defensively and falls back to empty data so the site builds even while the
 * pipeline is regenerating. Extensions are deliberately absent from
 * `reference.json`; this file is their only site-side source.
 */

export interface ExtensionLiveLink {
  /** The component page slug the example lives on. */
  component: string;
  /** The component's catalog title. */
  title: string;
  /** The example heading on that page. */
  heading: string;
  /** The page anchor `buildExampleSections` assigns the example. */
  anchor: string;
}

export interface ExtensionEntry {
  name: string;
  /** Text between the name and the body em dash; empty when absent. */
  qualifier: string;
  /** The section heading id on `/docs/extensions`. */
  slug: string;
  /** The extension's markdown description. */
  body: string;
  /** Component-page examples that demonstrate the extension, capped at 6. */
  live: ExtensionLiveLink[];
}

/**
 * A component-page example that demonstrates an extension no entry's `live`
 * list claims — the "In component pages" section of `/docs/extensions`.
 */
export interface ComponentExtension {
  /** The component page slug the example lives on. */
  component: string;
  /** The component's catalog title. */
  title: string;
  /** The example heading on that page. */
  heading: string;
  /** The page anchor `buildExampleSections` assigns the example. */
  anchor: string;
  /** The example description's first sentence; markdown backticks kept. */
  summary: string;
}

export interface ExtensionsFile {
  source: string;
  intro: string;
  entries: ExtensionEntry[];
  /** Extension-marked component-page examples no entry links. */
  componentExtensions: ComponentExtension[];
}

const EMPTY: ExtensionsFile = {
  source: "llms.txt",
  intro: "",
  entries: [],
  componentExtensions: [],
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function parseLiveLink(value: unknown): ExtensionLiveLink | null {
  if (!isRecord(value)) return null;
  const component = asString(value.component);
  if (component === "") return null;
  return {
    component,
    title: asString(value.title),
    heading: asString(value.heading),
    anchor: asString(value.anchor),
  };
}

function parseEntry(value: unknown): ExtensionEntry | null {
  if (!isRecord(value)) return null;
  const name = asString(value.name);
  if (name === "") return null;
  return {
    name,
    qualifier: asString(value.qualifier),
    slug: asString(value.slug),
    body: asString(value.body),
    live: Array.isArray(value.live)
      ? value.live.flatMap((link) => {
          const parsed = parseLiveLink(link);
          return parsed ? [parsed] : [];
        })
      : [],
  };
}

function parseComponentExtension(value: unknown): ComponentExtension | null {
  if (!isRecord(value)) return null;
  const component = asString(value.component);
  if (component === "") return null;
  return {
    component,
    title: asString(value.title),
    heading: asString(value.heading),
    anchor: asString(value.anchor),
    summary: asString(value.summary),
  };
}

function parseExtensions(raw: unknown): ExtensionsFile {
  if (!isRecord(raw)) return EMPTY;
  const entries = Array.isArray(raw.entries)
    ? raw.entries.flatMap((entry) => {
        const parsed = parseEntry(entry);
        return parsed ? [parsed] : [];
      })
    : [];
  const componentExtensions = Array.isArray(raw.componentExtensions)
    ? raw.componentExtensions.flatMap((item) => {
        const parsed = parseComponentExtension(item);
        return parsed ? [parsed] : [];
      })
    : [];
  return {
    source: asString(raw.source) || EMPTY.source,
    intro: asString(raw.intro),
    entries,
    componentExtensions,
  };
}

let cached: ExtensionsFile | null = null;

export function getExtensions(): ExtensionsFile {
  if (cached) return cached;
  try {
    const file = path.join(process.cwd(), "src", "data", "extensions.json");
    cached = parseExtensions(JSON.parse(readFileSync(file, "utf8")) as unknown);
  } catch {
    cached = EMPTY;
  }
  return cached;
}

/** Sidebar links for the extensions section, in llms.txt order. */
export function getExtensionSidebarLinks(): { href: string; label: string }[] {
  return getExtensions().entries.flatMap((entry) =>
    entry.slug === "" ? [] : [{ href: `/docs/extensions#${entry.slug}`, label: entry.name }],
  );
}
