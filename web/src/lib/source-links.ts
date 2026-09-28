/**
 * docs.rs and GitHub source links for a component page, derived from the
 * generated reference data: the page's import line names the
 * `herogpui::components::<module>` path and the primary type, and the
 * catalog's workspace version picks the release tag.
 *
 * Kept free of `@/` imports so `scripts/source-links.test.mjs` can load it
 * with Node's type stripping and check every derived path against
 * `crates/herogpui-components/src`.
 */

export const REPOSITORY = "https://github.com/Porabuild/HeroGPUI";
const DOCS_RS = "https://docs.rs/herogpui-components/latest/herogpui_components";
const CRATE_SRC = "crates/herogpui-components/src";

/**
 * Component modules that are directories (`<module>/mod.rs` re-exporting
 * private submodules). Every other module is a single `<module>.rs`. The
 * source-links test fails when this set and the crate disagree.
 */
export const DIRECTORY_MODULES: ReadonlySet<string> = new Set(["color_picker", "date_picker"]);

export interface ComponentSourceLinks {
  /** `herogpui_components` module, e.g. `color_picker`. */
  module: string;
  /** The component's primary public type, e.g. `ColorSlider`. */
  type: string;
  /** Repository-relative source path: a `.rs` file or a module directory. */
  sourcePath: string;
  docsRs: string;
  github: string;
}

interface ComponentSourceInput {
  slug: string;
  title: string;
  importLine: string;
  /** Workspace version from catalog.json, e.g. `0.12.0`. */
  version: string;
}

function importedNames(importLine: string): string[] {
  const list = importLine.match(/::\{([^}]*)\}/)?.[1] ?? importLine.match(/::(\w+);?\s*$/)?.[1];
  return (list ?? "")
    .split(",")
    .map((name) => name.trim())
    .filter((name) => /^[A-Z][A-Za-z0-9]*$/.test(name));
}

export function componentSourceLinks({
  slug,
  title,
  importLine,
  version,
}: ComponentSourceInput): ComponentSourceLinks | null {
  if (!version) return null;
  const names = importedNames(importLine);
  if (names.length === 0) return null;
  // `herogpui::components::<module>::…`; the prelude form (`Button`) has no
  // module in the path, and its module is the page slug.
  const module = importLine.match(/herogpui::components::(\w+)::/)?.[1] ?? slug.replace(/-/g, "_");
  if (!/^[a-z][a-z0-9_]*$/.test(module)) return null;
  // The type named like the page ("Color Slider" → ColorSlider), else the
  // first one imported.
  const wanted = title.replace(/[^A-Za-z0-9]/g, "").toLowerCase();
  const type = names.find((name) => name.toLowerCase() === wanted) ?? names[0];
  const directory = DIRECTORY_MODULES.has(module);
  const sourcePath = directory ? `${CRATE_SRC}/${module}` : `${CRATE_SRC}/${module}.rs`;
  return {
    module,
    type,
    sourcePath,
    docsRs: `${DOCS_RS}/${module}/struct.${type}.html`,
    github: `${REPOSITORY}/${directory ? "tree" : "blob"}/v${version}/${sourcePath}`,
  };
}
