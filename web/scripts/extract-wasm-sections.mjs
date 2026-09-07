// Pin the checked-in web-gallery artifact to the gallery source it was built
// from, and record which example headings the embed can render.
//
//   node scripts/extract-wasm-sections.mjs
//
// Writes `src/data/wasm-sections.json` (consumed by the component page to
// decide which headings get a live embed) and `src/data/wasm-parity.json`
// (whose `artifactSha256` doubles as the embed's cache-busting version).
//
// There is one gallery source. `crates/herogpui-web` is a workspace member
// that links the `herogpui-gallery` library and compiles for wasm32, so the
// browser runs the same `gallery/src/pages/components/` the native binary
// does. This script used to compare that file against a second, separately
// checked-out copy and report native-vs-WASM "drift"; there is no second copy
// to drift from now, so it reads one source and the drift fields are gone.
//
// What it still guards is the committed binary. `web/public/gallery/` holds a
// ~19 MB artifact that no compiler checks against the sources in this
// repository. Hashing every example body and the artifact together means that
// editing a gallery example without rebuilding fails `pnpm run extract:check`
// (see the manifest tests in extract-rust-examples.test.mjs) instead of
// shipping a page whose code block and live embed disagree.

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { parseInvocation } from "./extract-rust-examples.mjs";
import { galleryComponentsDir, readGallerySource } from "./lib/gallery-source.mjs";
import { skipTrivia, slugify, stepOver } from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const sectionsOut = resolve(webRoot, "src", "data", "wasm-sections.json");
const parityOut = resolve(webRoot, "src", "data", "wasm-parity.json");

export const MANIFEST_VERSION = 2;

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index === -1) return fallback;
  if (!process.argv[index + 1]) throw new Error(`${name} requires a path`);
  return resolve(process.argv[index + 1]);
}

function normalizeDescription(value) {
  return value.replace(/\s+/g, " ").trim();
}

function descriptionHash(value) {
  return createHash("sha256").update(normalizeDescription(value)).digest("hex");
}

// Hash the example body with whitespace, comments and trailing commas removed,
// so reformatting the gallery does not demand a 19 MB artifact rebuild while a
// real edit to the code still does.
function codeHash(value) {
  let normalized = "";
  let index = 0;
  while (index < value.length) {
    if (/\s/.test(value[index])) {
      index += 1;
      continue;
    }
    const stepped = stepOver(value, index);
    if (stepped !== null) {
      if (!value.startsWith("//", index) && !value.startsWith("/*", index)) {
        normalized += value.slice(index, stepped);
      }
      index = stepped;
      continue;
    }
    if (value[index] === ",") {
      const next = skipTrivia(value, index + 1);
      if (")]}`".includes(value[next])) {
        index += 1;
        continue;
      }
    }
    normalized += value[index];
    index += 1;
  }
  return createHash("sha256").update(normalized).digest("hex");
}

export function parseExampleSource(source) {
  const pages = new Map();
  const examples = new Map();
  let index = 0;

  while ((index = source.indexOf("component_doc_page!", index)) !== -1) {
    const page = parseInvocation(source, index);
    if (!page || page.end === -1) {
      throw new Error(
        `could not parse component page at ${index}: ${page?.error ?? "unknown error"}`,
      );
    }
    const slug = slugify(page.title);
    const headings = page.sections.map((section) => section.heading);
    const existing = new Set(pages.get(slug) ?? []);
    const duplicate = headings.find(
      (heading, at) => existing.has(heading) || headings.indexOf(heading) !== at,
    );
    if (duplicate) throw new Error(`${slug} has duplicate example heading ${duplicate}`);
    pages.set(slug, [...existing, ...headings]);
    for (const section of page.sections) {
      examples.set(`${slug}/${section.heading}`, {
        codeSha256: codeHash(section.code),
        descriptionSha256: descriptionHash(section.description ?? ""),
      });
    }
    index = page.end;
  }

  return { pages, examples };
}

export function buildManifest(source, artifact, glue = Buffer.alloc(0)) {
  const { pages, examples } = parseExampleSource(source);
  return {
    sections: Object.fromEntries(pages),
    parity: {
      version: MANIFEST_VERSION,
      artifactSha256: createHash("sha256").update(artifact).digest("hex"),
      glueSha256: createHash("sha256").update(glue).digest("hex"),
      examples: Object.fromEntries(
        [...examples.keys()].sort().map((key) => [key, examples.get(key)]),
      ),
    },
  };
}

export function run() {
  const sourcePath = argument("--source", galleryComponentsDir(repoRoot));
  const artifactPath = argument(
    "--wasm",
    resolve(webRoot, "public", "gallery", "herogpui_web_bg.wasm"),
  );
  const gluePath = argument("--glue", resolve(webRoot, "public", "gallery", "herogpui_web.js"));
  const { sections, parity } = buildManifest(
    readGallerySource(sourcePath),
    readFileSync(artifactPath),
    readFileSync(gluePath),
  );

  writeFileSync(sectionsOut, `${JSON.stringify(sections, null, 2)}\n`);
  writeFileSync(parityOut, `${JSON.stringify(parity, null, 2)}\n`);
  console.log(
    `wasm-sections.json: ${Object.keys(sections).length} pages, ${Object.values(sections).flat().length} examples`,
  );
  console.log(`wasm-parity.json: artifact ${parity.artifactSha256.slice(0, 12)}`);
  return { sections, parity };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) run();
