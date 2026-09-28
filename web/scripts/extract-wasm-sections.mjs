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
// repository. The manifest pins three things together:
//
//   * the artifact and glue bytes (`artifactSha256`, `glueSha256`);
//   * every example body (`examples`), so the page's code block and the live
//     embed cannot disagree; and
//   * every wasm build input (`inputsSha256`): the component, theme, core,
//     facade, web-entry and gallery sources plus the workspace manifests and
//     lockfile. A component-behaviour change that leaves every example body
//     untouched still invalidates the committed artifact, and this catches it.
//
// `--check` recomputes all of it and fails when any part is stale (it is in
// `pnpm run extract:check`, so CI runs it). The fix is always the same: rebuild
// the artifact (root AGENTS.md) and run `pnpm run wasm:manifest`.

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

export const MANIFEST_VERSION = 3;

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

export function buildManifest(source, artifact, glue = Buffer.alloc(0), inputs = "") {
  const { pages, examples } = parseExampleSource(source);
  return {
    sections: Object.fromEntries(pages),
    parity: {
      version: MANIFEST_VERSION,
      artifactSha256: createHash("sha256").update(artifact).digest("hex"),
      glueSha256: createHash("sha256").update(glue).digest("hex"),
      inputsSha256: inputs,
      examples: Object.fromEntries(
        [...examples.keys()].sort().map((key) => [key, examples.get(key)]),
      ),
    },
  };
}

export function run({ check = false } = {}) {
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
    inputsHash(repoRoot),
  );
  const sectionsText = `${JSON.stringify(sections, null, 2)}\n`;
  const parityText = `${JSON.stringify(parity, null, 2)}\n`;

  if (check) {
    const current = JSON.parse(readFileSync(parityOut, "utf8"));
    const stale = [];
    if (readFileSync(sectionsOut, "utf8") !== sectionsText) stale.push("example headings");
    if (current.version !== parity.version) stale.push("manifest version");
    if (current.artifactSha256 !== parity.artifactSha256) stale.push("artifact bytes");
    if (current.glueSha256 !== parity.glueSha256) stale.push("glue bytes");
    if (current.inputsSha256 !== parity.inputsSha256)
      stale.push("wasm build inputs (Rust sources, manifests or lockfile)");
    if (JSON.stringify(current.examples) !== JSON.stringify(parity.examples))
      stale.push("example bodies");
    if (stale.length) {
      console.error(
        `ERROR: the committed web-gallery artifact is stale (${stale.join(", ")} changed). ` +
          "Rebuild it (root AGENTS.md, nightly wasm build + wasm-bindgen) and run `pnpm run wasm:manifest`.",
      );
      process.exitCode = 1;
    } else {
      console.log(`wasm-parity.json: current (artifact ${parity.artifactSha256.slice(0, 12)})`);
    }
    return { sections, parity };
  }

  writeFileSync(sectionsOut, sectionsText);
  writeFileSync(parityOut, parityText);
  console.log(
    `wasm-sections.json: ${Object.keys(sections).length} pages, ${Object.values(sections).flat().length} examples`,
  );
  console.log(`wasm-parity.json: artifact ${parity.artifactSha256.slice(0, 12)}`);
  return { sections, parity };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
