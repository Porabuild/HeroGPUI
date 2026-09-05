// Record the component examples compiled into the checked-in wasm artifact.
// Run this against the wasm migration worktree whenever that artifact changes.

import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { parseInvocation } from "./extract-rust-examples.mjs";
import { skipTrivia, slugify, stepOver } from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const sectionsOut = resolve(webRoot, "src", "data", "wasm-sections.json");
const parityOut = resolve(webRoot, "src", "data", "wasm-parity.json");

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

function sortedDifference(left, right) {
  return [...left.keys()].filter((key) => !right.has(key)).sort();
}

export function introducedDrift(previous, current) {
  const known = new Set(previous);
  return current.filter((key) => !known.has(key));
}

export function buildParity(nativeSource, wasmSource, artifact, glue = Buffer.alloc(0)) {
  const native = parseExampleSource(nativeSource);
  const wasm = parseExampleSource(wasmSource);
  const keys = [...new Set([...native.examples.keys(), ...wasm.examples.keys()])].sort();
  const examples = {};
  const codeDrift = [];
  const descriptionDrift = [];

  for (const key of keys) {
    const nativeExample = native.examples.get(key);
    const wasmExample = wasm.examples.get(key);
    examples[key] = {
      ...(nativeExample ? { nativeCodeSha256: nativeExample.codeSha256 } : {}),
      ...(wasmExample && (!nativeExample || nativeExample.codeSha256 !== wasmExample.codeSha256)
        ? { wasmCodeSha256: wasmExample.codeSha256 }
        : {}),
      descriptionSha256:
        nativeExample?.descriptionSha256 ?? wasmExample?.descriptionSha256 ?? descriptionHash(""),
    };
    if (nativeExample && wasmExample && nativeExample.codeSha256 !== wasmExample.codeSha256) {
      codeDrift.push(key);
    }
    if (
      nativeExample &&
      wasmExample &&
      nativeExample.descriptionSha256 !== wasmExample.descriptionSha256
    ) {
      descriptionDrift.push(key);
    }
  }

  return {
    sections: Object.fromEntries(wasm.pages),
    parity: {
      version: 1,
      artifactSha256: createHash("sha256").update(artifact).digest("hex"),
      glueSha256: createHash("sha256").update(glue).digest("hex"),
      examples,
      missing: sortedDifference(native.examples, wasm.examples),
      extra: sortedDifference(wasm.examples, native.examples),
      codeDrift,
      descriptionDrift,
    },
  };
}

export function run() {
  const wasmSourcePath = argument("--source");
  if (!wasmSourcePath) {
    throw new Error(
      "usage: node scripts/extract-wasm-sections.mjs --source <components.rs> [--native-source <components.rs>] [--wasm <artifact.wasm>] [--glue <bindgen.js>] [--accept-drift]",
    );
  }
  const nativeSourcePath = argument(
    "--native-source",
    resolve(repoRoot, "gallery", "src", "pages", "components.rs"),
  );
  const artifactPath = argument(
    "--wasm",
    resolve(webRoot, "public", "gallery", "herogpui_web_bg.wasm"),
  );
  const gluePath = argument("--glue", resolve(webRoot, "public", "gallery", "herogpui_web.js"));
  const { sections, parity } = buildParity(
    readFileSync(nativeSourcePath, "utf8"),
    readFileSync(wasmSourcePath, "utf8"),
    readFileSync(artifactPath),
    readFileSync(gluePath),
  );

  if (parity.descriptionDrift.length) {
    throw new Error(`wasm example descriptions drifted: ${parity.descriptionDrift.join(", ")}`);
  }
  if (existsSync(parityOut) && !process.argv.includes("--accept-drift")) {
    const previous = JSON.parse(readFileSync(parityOut, "utf8"));
    const introduced = introducedDrift(previous.codeDrift ?? [], parity.codeDrift);
    if (introduced.length) {
      throw new Error(
        `new native/wasm example drift: ${introduced.join(", ")}. Sync the examples or rerun with --accept-drift only for a reviewed GPUI-version adaptation.`,
      );
    }
  }

  writeFileSync(sectionsOut, `${JSON.stringify(sections, null, 2)}\n`);
  writeFileSync(parityOut, `${JSON.stringify(parity, null, 2)}\n`);
  console.log(
    `wasm-sections.json: ${Object.keys(sections).length} pages, ${Object.values(sections).flat().length} examples`,
  );
  console.log(
    `wasm-parity.json: ${parity.codeDrift.length} code drifts, ${parity.descriptionDrift.length} description drifts`,
  );
  return { sections, parity };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) run();
