import { getComponentReference } from "@/app/docs/components/[slug]/data";
import { getCatalog } from "@/lib/catalog";
import { getExtensions } from "@/lib/extensions";
import { getMigrations } from "@/lib/migrations";
import { buildSearchItems, type SearchApi, type SearchItem } from "@/lib/docs-nav";
import { builderNames, isCallableRust } from "@/lib/gpui-docs";

/**
 * Server-side half of the site search: the public Rust surface of each
 * component, read from the generated `reference.json`, so the command palette
 * finds a component by builder (`selection_mode`) or type (`ListBoxItem`).
 */

const RUST_TYPE = /^[A-Z][A-Za-z0-9]*$/;

function importedTypes(importLine: string): string[] {
  const list = importLine.match(/::\{([^}]*)\}/)?.[1] ?? importLine.match(/::(\w+);?\s*$/)?.[1];
  return (list ?? "")
    .split(",")
    .map((name) => name.trim())
    .filter((name) => RUST_TYPE.test(name));
}

export function componentSearchApi(slug: string): SearchApi {
  const reference = getComponentReference(slug);
  if (!reference) return { builders: [], types: [] };
  const builders = new Set<string>();
  for (const row of reference.api) {
    if (!isCallableRust(row.rust, row.status)) continue;
    for (const name of builderNames(row.rust!.trim())) builders.add(name);
  }
  const types = new Set(importedTypes(reference.importLine));
  for (const part of reference.parts) {
    const owner = part.rustOwner?.trim() ?? "";
    if (part.status !== "unavailable" && RUST_TYPE.test(owner)) types.add(owner);
  }
  return { builders: [...builders].sort(), types: [...types] };
}

let cached: SearchItem[] | null = null;

/** The site search index: pages, components, each component's Rust API, one entry per HeroGPUI extension, and one per upgrade guide. */
export function siteSearchItems(): SearchItem[] {
  if (cached && process.env.NODE_ENV === "production") return cached;
  const items = buildSearchItems(getCatalog(), componentSearchApi);
  // One entry per HeroGPUI extension, so "HoverCard", "i18n" or "test kit"
  // lands on the extensions page section for it.
  for (const entry of getExtensions().entries) {
    items.push({
      href: `/docs/extensions#${entry.slug}`,
      title: entry.name,
      group: "Extensions",
      keywords: entry.name,
    });
  }
  // One entry per extension example on a component page, keyed by the Rust
  // names its summary quotes, so "fit_panel_width" or "TooltipPresenter" finds
  // the live example.
  for (const example of getExtensions().componentExtensions) {
    const names = [...example.summary.matchAll(/`([^`]+)`/g)].flatMap(
      ([, code]) => code.match(/[A-Za-z_][A-Za-z0-9_]{2,}/g) ?? [],
    );
    items.push({
      href: `/docs/components/${example.component}#${example.anchor}`,
      title: `${example.title} — ${example.heading}`,
      group: "Extensions",
      keywords: [example.heading, ...new Set(names)].join(" "),
    });
  }
  // One entry per upgrade guide, so "migration" or "upgrade 0.15" finds the
  // guide for that version.
  for (const guide of getMigrations().guides) {
    items.push({
      href: `/docs/migration/${guide.slug}`,
      title: guide.title,
      group: "Upgrade guides",
      keywords: `migration upgrade ${guide.version} breaking changes`,
    });
  }
  cached = items;
  return items;
}
