// Pin the web-gallery artifact's key to the gallery source, and record which
// example headings the embed can render.
//
//   node scripts/extract-wasm-sections.mjs
//
// Writes `src/data/wasm-sections.json` (consumed by the component page to
// decide which headings get a live embed) and `src/data/wasm-parity.json`.
//
// There is one gallery source. `crates/herogpui-web` is a workspace member
// that links the `herogpui-gallery` library and compiles for wasm32, so the
// browser runs the same `gallery/src/pages/components/` the native binary
// does.
//
// The ~19 MB artifact itself is not committed: CI builds it from the checkout
// and publishes it keyed by `inputsSha256`, and the site build downloads the
// artifact for its own key (`scripts/gallery-artifact.mjs`). The manifest
// records what that key is made of:
//
//   * every wasm build input (`inputsSha256`): the component, theme, core,
//     facade, web-entry and gallery sources plus the workspace manifests and
//     lockfile -- the artifact key; and
//   * every example body (`examples`), so a change to what the page's code
//     block shows is visible in review next to the key it moves.
//
// `--check` recomputes both and fails when either is stale (it is in
// `pnpm run extract:check`, so CI runs it). The fix is always
// `pnpm run wasm:manifest`; no artifact rebuild is needed, CI does that.

import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { parseInvocation } from "./extract-rust-examples.mjs";
import { galleryComponentsDir, readGallerySource } from "./lib/gallery-source.mjs";
import { skipTrivia, slugify, stepOver } from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const sectionsOut = resolve(webRoot, "src", "data", "wasm-sections.json");
const parityOut = resolve(webRoot, "src", "data", "wasm-parity.json");

export const MANIFEST_VERSION = 4;

// The inputs of `cargo build -p herogpui-web`: every file under these roots
// (tests, docs and licence texts excluded) plus the workspace-level files.
const INPUT_FILES = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml"];
const INPUT_ROOTS = [
  "crates/herogpui-core",
  "crates/herogpui-theme",
  "crates/herogpui-components",
  "crates/herogpui",
  "crates/herogpui-web",
  "gallery",
];
const SKIPPED_DIRS = new Set(["tests", "target", "node_modules"]);
const SKIPPED_FILE = /(?:\.md|^LICENSE|^NOTICE|\.source\.ttf)$/;
const TEXT_FILE = /\.(?:rs|toml|html|json|svg|css|js|txt)$|^Cargo\.lock$/;

function walk(root, dir, out) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = resolve(dir, entry.name);
    if (entry.isDirectory()) {
      if (!SKIPPED_DIRS.has(entry.name)) walk(root, path, out);
    } else if (entry.isFile() && !SKIPPED_FILE.test(entry.name)) {
      out.push(relative(root, path).split("\\").join("/"));
    }
  }
}

/// SHA-256 over every wasm build input: sorted repository-relative paths, each
/// with the hash of its bytes (text files with CRLF folded to LF, so a Windows
/// checkout hashes the same as CI's).
export function inputsHash(root) {
  const files = INPUT_FILES.filter((file) => {
    try {
      return statSync(resolve(root, file)).isFile();
    } catch {
      return false;
    }
  });
  for (const dir of INPUT_ROOTS) walk(root, resolve(root, dir), files);
  const hash = createHash("sha256");
  for (const file of [...new Set(files)].sort()) {
    let bytes = readFileSync(resolve(root, file));
    const name = file.split("/").pop();
    if (TEXT_FILE.test(name)) bytes = Buffer.from(bytes.toString("utf8").replace(/\r\n/g, "\n"));
    hash.update(`${file}\0${createHash("sha256").update(bytes).digest("hex")}\n`);
  }
  return hash.digest("hex");
}

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
// so reformatting the gallery does not read as an example change in the
// manifest diff while a real edit to the code still does.
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

export function buildManifest(source, inputs = "") {
  const { pages, examples } = parseExampleSource(source);
  return {
    sections: Object.fromEntries(pages),
    parity: {
      version: MANIFEST_VERSION,
      inputsSha256: inputs,
      examples: Object.fromEntries(
        [...examples.keys()].sort().map((key) => [key, examples.get(key)]),
      ),
    },
  };
}

export function run({ check = false } = {}) {
  const sourcePath = argument("--source", galleryComponentsDir(repoRoot));
  const { sections, parity } = buildManifest(readGallerySource(sourcePath), inputsHash(repoRoot));
  const sectionsText = `${JSON.stringify(sections, null, 2)}\n`;
  const parityText = `${JSON.stringify(parity, null, 2)}\n`;

  if (check) {
    const current = JSON.parse(readFileSync(parityOut, "utf8"));
    const stale = [];
    if (readFileSync(sectionsOut, "utf8") !== sectionsText) stale.push("example headings");
    if (current.version !== parity.version) stale.push("manifest version");
    if (current.inputsSha256 !== parity.inputsSha256)
      stale.push("wasm build inputs (Rust sources, manifests or lockfile)");
    if (JSON.stringify(current.examples) !== JSON.stringify(parity.examples))
      stale.push("example bodies");
    if (stale.length) {
      console.error(
        `ERROR: src/data/wasm-parity.json / wasm-sections.json are stale (${stale.join(", ")} changed). ` +
          "Run `pnpm run wasm:manifest`; CI builds and publishes the matching artifact.",
      );
      process.exitCode = 1;
    } else {
      console.log(`wasm-parity.json: current (artifact key ${parity.inputsSha256.slice(0, 16)})`);
    }
    return { sections, parity };
  }

  writeFileSync(sectionsOut, sectionsText);
  writeFileSync(parityOut, parityText);
  console.log(
    `wasm-sections.json: ${Object.keys(sections).length} pages, ${Object.values(sections).flat().length} examples`,
  );
  console.log(`wasm-parity.json: artifact key ${parity.inputsSha256.slice(0, 16)}`);
  return { sections, parity };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
