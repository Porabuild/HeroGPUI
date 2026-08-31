// Extract the component catalog into web/src/data/catalog.json.
//
// Source: gallery/src/pages/mod.rs — the `Page` enum (variants grouped by
// `// Category` comments) plus `Page::title()` / `Page::description()`.
// Overview, Releases and "Getting started" pages are hand-written elsewhere
// and skipped; only component categories are emitted.
//
// importLine comes from reference.json (run scripts/extract-reference.mjs
// first); shots are verified against the real files in .shots/. The `demos`
// field remains in the catalog shape as an empty compatibility field.

import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  readSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseGalleryPages } from "./lib/gallery-pages.mjs";
import { slugify } from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const MOD_SOURCE = resolve(repoRoot, "gallery", "src", "pages", "mod.rs");
const SHOTS_DIR = resolve(repoRoot, ".shots");

/// Dark captures rejected because they disagree with their light pair.
const staleDarkShots = [];
const REFERENCE = resolve(webRoot, "src", "data", "reference.json");
const OUT = resolve(webRoot, "src", "data", "catalog.json");

const VERSION = "0.1.0";
const SKIPPED_CATEGORIES = new Set(["Overview", "Releases", "Getting started"]);

/// `Button Group` -> `buttongroup-v3.png`; dark variant appends `-dark`.
function shotFile(title, dark) {
  const base = title.toLowerCase().replaceAll(" ", "");
  return `${base}${dark ? "-dark" : ""}-v3.png`;
}

/// PNG dimensions, read straight from the IHDR chunk (bytes 16..24).
function pngSize(path) {
  const head = Buffer.alloc(24);
  const fd = openSync(path, "r");
  try {
    readSync(fd, head, 0, 24, 0);
  } finally {
    closeSync(fd);
  }
  return { width: head.readUInt32BE(16), height: head.readUInt32BE(20) };
}

function shotField(title, dark) {
  const file = shotFile(title, dark);
  const path = join(SHOTS_DIR, file);
  if (!existsSync(path)) return null;
  if (!dark) return `/shots/${file}`;

  // A dark capture is only trustworthy when its geometry matches the light one
  // taken from the same gallery build. Several `-dark-v3.png` files are older
  // captures of a gallery that predates the current component set — they still
  // show components and prop vocabularies that no longer exist, so publishing
  // them on a current page documents the wrong API. Rather than hard-coding a
  // known-good size, which would rot, require the pair to agree.
  const lightPath = join(SHOTS_DIR, shotFile(title, false));
  if (!existsSync(lightPath)) return null;
  const dim = pngSize(path);
  const lightDim = pngSize(lightPath);
  if (dim.width !== lightDim.width || dim.height !== lightDim.height) {
    staleDarkShots.push(
      `${file} (${dim.width}x${dim.height} vs light ${lightDim.width}x${lightDim.height})`,
    );
    return null;
  }
  return `/shots/${file}`;
}

export function run() {
  const pages = parseGalleryPages(readFileSync(MOD_SOURCE, "utf8"));

  let reference = null;
  if (existsSync(REFERENCE)) {
    reference = JSON.parse(readFileSync(REFERENCE, "utf8"));
  } else {
    console.error(
      "WARNING: src/data/reference.json not found — run scripts/extract-reference.mjs first; importLine will be null",
    );
  }

  /** @type {Map<string, { name: string, slug: string, components: string[] }>} */
  const categories = new Map();
  const components = {};

  for (const page of pages) {
    if (SKIPPED_CATEGORIES.has(page.category)) continue;
    if (!categories.has(page.category)) {
      categories.set(page.category, {
        name: page.category,
        slug: slugify(page.category),
        components: [],
      });
    }
    const slug = page.titleSlug;
    const refEntry = reference?.[slug];
    components[slug] = {
      slug,
      title: page.title,
      description: page.description,
      category: page.category,
      importLine: refEntry ? refEntry.importLine : null,
      shot: shotField(page.title, false),
      shotDark: shotField(page.title, true),
      demos: [],
      hasReference: Boolean(refEntry),
    };
    categories.get(page.category).components.push(slug);
  }

  const catalog = {
    version: VERSION,
    categories: [...categories.values()],
    components,
  };

  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, JSON.stringify(catalog, null, 2) + "\n");

  if (staleDarkShots.length > 0) {
    console.warn(
      `  ${staleDarkShots.length} dark screenshot(s) ignored — geometry disagrees with the light capture, so they predate the current gallery:`,
    );
    for (const s of staleDarkShots) console.warn(`    ${s}`);
  }

  const missingShots = Object.values(components).filter((c) => !c.shot);
  console.log(
    `catalog.json: ${Object.keys(components).length} components in ${categories.size} categories ` +
      `(${missingShots.length} without a light screenshot)`,
  );
  for (const category of catalog.categories) {
    console.log(`  ${category.name}: ${category.components.length}`);
  }
  return catalog;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run();
}
