import { getComponentReference } from "@/app/docs/components/[slug]/data";
import { getCatalog } from "@/lib/catalog";
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

/** The site search index: pages, components, and each component's Rust API. */
export function siteSearchItems(): SearchItem[] {
  if (cached && process.env.NODE_ENV === "production") return cached;
  cached = buildSearchItems(getCatalog(), componentSearchApi);
  return cached;
}
