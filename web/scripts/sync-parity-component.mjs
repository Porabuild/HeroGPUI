// Per-component native/WASM parity loop helper.
//
//   node scripts/sync-parity-component.mjs plan <slug>
//     List every drifted example on one component page with a line diff and a
//     composition-only verdict. Composition-only means the native body adds no
//     new component API calls beyond layout helpers, placeholders, and common
//     builder props, so it is a transplant candidate.
//
//   node scripts/sync-parity-component.mjs sync <slug> [--composition-only] [--keys a,b]
//     Transplant native example bodies into the WASM migration source with
//     exact-substring guards (each WASM body must occur exactly once).
//     Writes the WASM source in place; rebuilding the artifact, regenerating
//     the manifest, and committing stay in .shots/parity-loop.ps1 so every
//     step remains inspectable.
//
// Never edits native sources or generated JSON. The WASM compiler plus
// extract-wasm-sections.mjs (run without --accept-drift) are the real gates:
// unknown APIs fail the build and new drift fails generation.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { parseInvocation } from "./extract-rust-examples.mjs";
import { slugify } from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..", "..");

const NATIVE_DEFAULT = resolve(repoRoot, "gallery", "src", "pages", "components.rs");
const WASM_DEFAULT = "D:/herogpui-wasm/gallery/src/pages/components.rs";

// Added-line patterns that never need a component API port: layout helpers,
// specimen-width wrappers, status prose, placeholders, and builder props the
// WASM migration already supports (proven by prior transplant batches).
const COMPOSITION_RE =
  /placeholder|field_col|demo_field|fixed_demo|para\(|into_any_element|gpui::div|\.flex\(|\.gap\(|items_start|\.children\(|\.child\(|w\(px\(|variant\(|description\(|label\(|error_message|is_required|is_disabled|is_clearable|input_type|full_width|row_height|show_value|selection_mode|SelectionMode|format_options|NumberFormat|\.color\(|\.size\(|default_value|\.value\(|padding\(|FieldVariant|el_id|opt_usize_cb|demo_text|validate\(|\.on_selection_change|selected|select_lang|notify\(\)|\.els\(\)|\.map\(|\.iter\(|languages\(\)|virtual_names\(\)|\.into\(\)|"/;

// Added lines that always need a real component port first.
const API_GAP_RE =
  /fn |struct |impl |set_demo_value|set_demo_text_value|demo_value\(|demo_text_value\(|on_resize|remove_content|\.width\(px|allows_resizing|escape|render_fn|render-prop|RenderProp/;

// Bare closers and dimension literals carry no API meaning.
const NOISE_RE = /^[\s()[\]{},.\d_"']+$/;

function flag(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index === -1) return fallback;
  if (name === "--composition-only") return true;
  if (!process.argv[index + 1]) throw new Error(`${name} requires a value`);
  return process.argv[index + 1].startsWith("--") ? fallback : resolve(process.argv[index + 1]);
}

function loadSections(source) {
  const sections = new Map();
  let index = 0;
  while ((index = source.indexOf("component_doc_page!", index)) !== -1) {
    const page = parseInvocation(source, index);
    if (!page || page.end === -1) throw new Error(`could not parse component page at ${index}`);
    const slug = slugify(page.title);
    for (const section of page.sections) {
      const key = `${slug}/${section.heading}`;
      if (!sections.has(key)) sections.set(key, section);
    }
    index = page.end;
  }
  return sections;
}

function diffLines(wasmCode, nativeCode) {
  const removed = wasmCode.split("\n").map((line) => line.trim()).filter(Boolean);
  const nativeLines = nativeCode.split("\n").map((line) => line.trim()).filter(Boolean);
  const removedSet = new Set(removed);
  const nativeSet = new Set(nativeLines);
  return {
    added: nativeLines.filter((line) => !removedSet.has(line)),
    dropped: removed.filter((line) => !nativeSet.has(line)),
  };
}

function verdict(added) {
  const meaningful = added.filter((line) => line && !NOISE_RE.test(line));
  const gaps = meaningful.filter((line) => API_GAP_RE.test(line));
  if (gaps.length) return { kind: "API-GAP", evidence: gaps };
  const exotic = meaningful.filter((line) => !COMPOSITION_RE.test(line));
  if (exotic.length) return { kind: "NEEDS-REVIEW", evidence: exotic };
  return { kind: "COMPOSITION-ONLY", evidence: [] };
}

function driftKeys(parity, slug) {
  return (parity.codeDrift ?? []).filter((key) => key.startsWith(`${slug}/`)).sort();
}

const [mode, slug] = process.argv.slice(2);
if (!mode || !slug || mode.startsWith("--") || slug.startsWith("--")) {
  throw new Error("usage: sync-parity-component.mjs (plan|sync) <slug> [--composition-only] [--keys a,b] [--native-source p] [--wasm-source p]");
}
const nativePath = flag("--native-source", NATIVE_DEFAULT);
const wasmPath = flag("--wasm-source", WASM_DEFAULT);
const parityPath = resolve(scriptDir, "..", "src", "data", "wasm-parity.json");

const nativeSource = readFileSync(nativePath, "utf8");
let wasmSource = readFileSync(wasmPath, "utf8");
const native = loadSections(nativeSource);
const wasm = loadSections(wasmSource);
const parity = JSON.parse(readFileSync(parityPath, "utf8"));
const keys = driftKeys(parity, slug);
if (!keys.length) {
  console.log(`no code drift for ${slug}`);
  process.exit(0);
}

if (mode === "plan") {
  for (const key of keys) {
    const { added, dropped } = diffLines(wasm.get(key).code, native.get(key).code);
    const result = verdict(added);
    console.log(`### ${key} -> ${result.kind} (+${added.length}/-${dropped.length})`);
    for (const line of result.evidence.slice(0, 8)) console.log(`    | ${line.slice(0, 140)}`);
  }
} else if (mode === "sync") {
  const onlyKeys = flag("--keys", null);
  const wanted = onlyKeys
    ? String(onlyKeys).split(",").map((key) => (key.includes("/") ? key : `${slug}/${key}`))
    : keys;
  const compositionOnly = flag("--composition-only", false);
  let done = 0;
  for (const key of wanted) {
    if (!keys.includes(key)) throw new Error(`${key} is not a known drift key`);
    const nativeCode = native.get(key).code;
    const wasmCode = wasm.get(key).code;
    if (wasmCode === nativeCode) {
      console.log(`already synced ${key}`);
      done += 1;
      continue;
    }
    if (compositionOnly) {
      const result = verdict(diffLines(wasmCode, nativeCode).added);
      if (result.kind !== "COMPOSITION-ONLY") {
        console.log(`skip ${key} (${result.kind})`);
        continue;
      }
    }
    const occurrences = wasmSource.split(wasmCode).length - 1;
    if (occurrences !== 1) throw new Error(`${key}: wasm body occurs ${occurrences}x, refusing`);
    wasmSource = wasmSource.replace(wasmCode, nativeCode);
    done += 1;
    console.log(`transplanted ${key}`);
  }
  writeFileSync(wasmPath, wasmSource);
  console.log(`wrote ${wasmPath} (${done} transplanted)`);
} else {
  throw new Error(`unknown mode ${mode}`);
}
