// Extract HeroGPUI reference metadata into web/src/data/reference.json.
//
// Source: gallery/src/pages/reference_metadata.rs — uniform, checked-in Rust
// const literals (ApiDoc / PartDoc / StateDoc / StyleDoc / ReferenceMetadata).
// The file is read with the shared character-level scanner so escaped quotes
// and commas inside strings cannot split a field.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseGalleryPages } from "./lib/gallery-pages.mjs";
import {
  readIdent,
  readStringLiteral,
  scanDelimited,
  skipTrivia,
  slugify,
  splitTopLevelCommas,
  stepOver,
} from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const SOURCE = resolve(repoRoot, "gallery", "src", "pages", "reference_metadata.rs");
const MOD_SOURCE = resolve(repoRoot, "gallery", "src", "pages", "mod.rs");
const OUT = resolve(webRoot, "src", "data", "reference.json");

const STATUS = {
  "ImplementationStatus::Implemented": "implemented",
  "ImplementationStatus::Partial": "partial",
  "ImplementationStatus::Unavailable": "unavailable",
};

/// Find every top-level `const NAME: Type = Init;` in the file. Returns a map
/// of name -> { typeStart, typeEnd, initStart, initEnd } (absolute indices).
function collectConsts(src) {
  const consts = new Map();
  let i = 0;
  while (i < src.length) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    if (
      src.startsWith("const", i) &&
      (i === 0 || !/[A-Za-z0-9_]/.test(src[i - 1])) &&
      !/[A-Za-z0-9_]/.test(src[i + 5] ?? "")
    ) {
      const name = readIdent(src, skipTrivia(src, i + 5));
      if (!name) {
        i += 5;
        continue;
      }
      let j = skipTrivia(src, name.end);
      if (src[j] !== ":") {
        i = name.end;
        continue;
      }
      j = skipTrivia(src, j + 1);
      // Type text: up to the top-level `=`.
      const typeStart = j;
      let depth = 0;
      while (j < src.length) {
        const steppedType = stepOver(src, j);
        if (steppedType !== null) {
          j = steppedType;
          continue;
        }
        const c = src[j];
        if (c === "(" || c === "[" || c === "{") depth += 1;
        else if (c === ")" || c === "]" || c === "}") depth -= 1;
        else if (c === "=" && depth === 0) break;
        j += 1;
      }
      const typeEnd = j;
      j = skipTrivia(src, j + 1);
      // Initializer: balanced delimiters, ending at `;` at depth 0.
      const initStart = j;
      depth = 0;
      let done = false;
      while (j < src.length) {
        const steppedInit = stepOver(src, j);
        if (steppedInit !== null) {
          j = steppedInit;
          continue;
        }
        const c = src[j];
        if (c === "(" || c === "[" || c === "{") depth += 1;
        else if (c === ")" || c === "]" || c === "}") depth -= 1;
        else if (c === ";" && depth === 0) {
          done = true;
          break;
        }
        j += 1;
      }
      if (!done) {
        throw new Error(`const ${name.name}: initializer never terminates`);
      }
      consts.set(name.name, { typeStart, typeEnd, initStart, initEnd: j });
      i = j;
      continue;
    }
    i += 1;
  }
  return consts;
}

/// Parse a struct literal (`TypeName { field: value, ... }`) starting at the
/// type identifier. Returns { type: string, fields: Map<string, [start, end]> }
/// or null.
function parseStructLiteral(src, i, consts) {
  const typeName = readIdent(src, i);
  if (!typeName) return null;
  let j = skipTrivia(src, typeName.end);
  if (src[j] !== "{") return null;
  const body = scanDelimited(src, j);
  if (!body) return null;
  const fields = new Map();
  for (const part of splitTopLevelCommas(src, j + 1, body.end - 1)) {
    const p = skipTrivia(src, part.start);
    const field = readIdent(src, p);
    if (!field) continue;
    let k = skipTrivia(src, field.end);
    if (src[k] !== ":") continue;
    k = skipTrivia(src, k + 1);
    fields.set(field.name, [k, part.end]);
  }
  return { type: typeName.name, fields, end: body.end };
}

/// Resolve a field value range to a JS value:
/// string literals -> string, enum paths -> "ImplementationStatus::X",
/// const references -> resolved through `consts`, `&[...]` -> array,
/// struct literals -> { type, fields }.
function evalValue(src, start, end, consts, depth = 0) {
  const i = skipTrivia(src, start);
  if (i >= end) return { kind: "empty" };
  if (src[i] === '"') {
    const s = readStringLiteral(src, i);
    if (!s) return { kind: "unparsed", text: src.slice(i, end) };
    return { kind: "string", value: s.value };
  }
  if (src[i] === "&") {
    return evalValue(src, skipTrivia(src, i + 1), end, consts, depth);
  }
  if (src[i] === "[") {
    const body = scanDelimited(src, i);
    if (!body) return { kind: "unparsed", text: src.slice(i, end) };
    const items = splitTopLevelCommas(src, i + 1, body.end - 1).map((part) =>
      evalValue(src, part.start, part.end, consts, depth),
    );
    return { kind: "array", items };
  }
  const ident = readIdent(src, i);
  if (!ident) return { kind: "unparsed", text: src.slice(i, end) };
  let j = skipTrivia(src, ident.end);
  if (src[j] === "{") {
    const struct = parseStructLiteral(src, i, consts);
    return struct ? { kind: "struct", ...struct } : { kind: "unparsed", text: src.slice(i, end) };
  }
  if (src[j] === ":" && src[j + 1] === ":") {
    const sub = readIdent(src, skipTrivia(src, j + 2));
    if (sub) return { kind: "enum", path: `${ident.name}::${sub.name}` };
  }
  if (src[j] === "!") {
    // Macro invocation (e.g. concat!) — not expected in values.
    return { kind: "macro", name: ident.name };
  }
  if (consts.has(ident.name)) {
    if (depth > 4) return { kind: "cycle", name: ident.name };
    const c = consts.get(ident.name);
    return evalValue(src, c.initStart, c.initEnd, consts, depth + 1);
  }
  return { kind: "ident", name: ident.name };
}

/// Resolve a `&["a", "b"]` value (inline or via const reference) to string[].
function evalStringArray(src, value, consts) {
  if (value.kind === "array") {
    return value.items.map((item) => {
      if (item.kind !== "string") throw new Error("non-string element in string array");
      return item.value;
    });
  }
  if (value.kind === "unparsed") {
    throw new Error(`cannot parse string array: ${value.text.slice(0, 80)}`);
  }
  throw new Error(`expected string array, got ${value.kind}`);
}

/// Resolve a `&[ApiDoc { ... }, ...]` value to an array of parsed struct
/// literals.
function evalStructArray(src, value, consts) {
  if (value.kind === "array") {
    return value.items.map((item) => {
      if (item.kind !== "struct") throw new Error("non-struct element in struct array");
      return item;
    });
  }
  if (value.kind === "unparsed") {
    throw new Error(`cannot parse struct array: ${value.text.slice(0, 80)}`);
  }
  throw new Error(`expected struct array, got ${value.kind}`);
}

function requireString(src, fields, name, constName) {
  const range = fields.get(name);
  if (!range) throw new Error(`${constName}: missing field \`${name}\``);
  const value = evalValue(src, range[0], range[1], new Map());
  if (value.kind !== "string") {
    throw new Error(`${constName}: field \`${name}\` is ${value.kind}, expected string`);
  }
  return value.value;
}

function rowStatus(value, context) {
  if (value.kind === "enum" && STATUS[value.path]) return STATUS[value.path];
  throw new Error(`${context}: unknown status ${JSON.stringify(value)}`);
}

function extract() {
  const src = readFileSync(SOURCE, "utf8");
  const consts = collectConsts(src);

  // Reference entries must be keyed by the slug the site uses for the page,
  // which comes from `Page::title()` in mod.rs — not from the `page` field,
  // which holds the enum-variant spelling ("FieldSlots", not "Label &
  // Messages"). The `page` value is the variant name, so join on it.
  const titleByVariant = new Map(
    parseGalleryPages(readFileSync(MOD_SOURCE, "utf8")).map((page) => [page.name, page.title]),
  );

  const all = consts.get("ALL");
  if (!all) throw new Error("const ALL not found in reference_metadata.rs");
  // ALL is an array of const *names* (DROPDOWN, BUTTON, …). Read them as raw
  // identifiers — evalValue would eagerly resolve each name to its struct.
  const allInit = skipTrivia(src, all.initStart);
  let bracket = skipTrivia(src, src[allInit] === "&" ? allInit + 1 : allInit);
  if (src[bracket] !== "[") {
    throw new Error("const ALL: expected `&[...]` initializer");
  }
  const allBody = scanDelimited(src, bracket);
  if (!allBody) throw new Error("const ALL: unterminated array");
  const names = splitTopLevelCommas(src, bracket + 1, allBody.end - 1).map((part) => {
    const text = src.slice(part.start, part.end).trim();
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(text)) {
      throw new Error(`const ALL: unexpected element ${JSON.stringify(text)}`);
    }
    return text;
  });

  const entries = {};
  const warnings = [];
  let apiRows = 0;
  let partsRows = 0;
  let statesRows = 0;
  let stylingRows = 0;

  for (const name of names) {
    const decl = consts.get(name);
    if (!decl) {
      warnings.push(`const ${name} (listed in ALL) not found`);
      continue;
    }
    const root = evalValue(src, decl.initStart, decl.initEnd, consts);
    if (root.kind !== "struct" || root.type !== "ReferenceMetadata") {
      warnings.push(`const ${name}: expected ReferenceMetadata struct, got ${root.kind}`);
      continue;
    }
    const constName = name;
    const page = requireString(src, root.fields, "page", constName);
    let entry;
    try {
      const requiredPartsValue = evalValue(src, ...root.fields.get("required_parts"), consts);
      const apiValue = evalValue(src, ...root.fields.get("api"), consts);
      const partsValue = evalValue(src, ...root.fields.get("parts"), consts);
      const statesValue = evalValue(src, ...root.fields.get("states"), consts);
      const stylingValue = evalValue(src, ...root.fields.get("styling"), consts);

      const api = evalStructArray(src, apiValue, consts).map((row, n) => {
        const status = rowStatus(
          evalValue(src, ...row.fields.get("status"), new Map()),
          `${constName}.api[${n}]`,
        );
        return {
          owner: requireString(src, row.fields, "owner", `${constName}.api[${n}]`),
          prop: requireString(src, row.fields, "prop", `${constName}.api[${n}]`),
          type: requireString(src, row.fields, "ty", `${constName}.api[${n}]`),
          default: requireString(src, row.fields, "default", `${constName}.api[${n}]`),
          description: requireString(src, row.fields, "description", `${constName}.api[${n}]`),
          rustOwner: requireString(src, row.fields, "rust_owner", `${constName}.api[${n}]`),
          rust: requireString(src, row.fields, "rust", `${constName}.api[${n}]`),
          status,
        };
      });
      const parts = evalStructArray(src, partsValue, consts).map((row, n) => ({
        name: requireString(src, row.fields, "name", `${constName}.parts[${n}]`),
        slot: requireString(src, row.fields, "slot", `${constName}.parts[${n}]`),
        description: requireString(src, row.fields, "description", `${constName}.parts[${n}]`),
        rustOwner: requireString(src, row.fields, "rust_owner", `${constName}.parts[${n}]`),
        status: rowStatus(
          evalValue(src, ...row.fields.get("status"), new Map()),
          `${constName}.parts[${n}]`,
        ),
      }));
      const states = evalStructArray(src, statesValue, consts).map((row, n) => ({
        state: requireString(src, row.fields, "state", `${constName}.states[${n}]`),
        selector: requireString(src, row.fields, "selector", `${constName}.states[${n}]`),
        description: requireString(src, row.fields, "description", `${constName}.states[${n}]`),
        rust: requireString(src, row.fields, "rust", `${constName}.states[${n}]`),
        status: rowStatus(
          evalValue(src, ...row.fields.get("status"), new Map()),
          `${constName}.states[${n}]`,
        ),
      }));
      const styling = evalStructArray(src, stylingValue, consts).map((row, n) => ({
        token: requireString(src, row.fields, "class_or_token", `${constName}.styling[${n}]`),
        description: requireString(src, row.fields, "description", `${constName}.styling[${n}]`),
        rust: requireString(src, row.fields, "rust", `${constName}.styling[${n}]`),
        status: rowStatus(
          evalValue(src, ...row.fields.get("status"), new Map()),
          `${constName}.styling[${n}]`,
        ),
      }));

      entry = {
        page,
        importLine: requireString(src, root.fields, "import_line", constName),
        version: requireString(src, root.fields, "version", constName),
        docsSource: requireString(src, root.fields, "docs_source", constName),
        apiSource: requireString(src, root.fields, "api_source", constName),
        styleSource: requireString(src, root.fields, "style_source", constName),
        requiredParts: evalStringArray(src, requiredPartsValue, consts),
        api,
        parts,
        states,
        styling,
      };
      apiRows += api.length;
      partsRows += parts.length;
      statesRows += states.length;
      stylingRows += styling.length;
    } catch (err) {
      warnings.push(`const ${name} (${page}): ${err.message}`);
      continue;
    }
    const title = titleByVariant.get(page);
    if (!title) {
      warnings.push(
        `reference page "${page}" has no Page::${page} variant in mod.rs; keying by slug of the page field`,
      );
    }
    entries[slugify(title ?? page)] = entry;
  }

  return {
    entries,
    warnings,
    counts: {
      entries: Object.keys(entries).length,
      allNames: names.length,
      apiRows,
      partsRows,
      statesRows,
      stylingRows,
    },
  };
}

export function run({ check = false } = {}) {
  const { entries, warnings, counts } = extract();

  if (warnings.length) {
    for (const w of warnings) console.error(`WARNING: ${w}`);
  }
  if (counts.entries !== counts.allNames) {
    console.error(
      `WARNING: ALL lists ${counts.allNames} entries but only ${counts.entries} were extracted`,
    );
  }
  if (counts.entries < counts.allNames) {
    console.error("ERROR: reference extraction is incomplete; refusing to write partial data");
    process.exitCode = 1;
    return counts;
  }

  const output = JSON.stringify(entries, null, 2) + "\n";
  if (check) {
    let current = "";
    try {
      current = readFileSync(OUT, "utf8");
    } catch {}
    if (current !== output) {
      console.error("ERROR: reference.json is stale; run `pnpm run extract`");
      process.exitCode = 1;
    }
  } else {
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, output);
  }
  console.log(
    `reference.json: ${counts.entries} entries, ` +
      `${counts.apiRows} api rows, ${counts.partsRows} parts rows, ` +
      `${counts.statesRows} states rows, ${counts.stylingRows} styling rows`,
  );
  return counts;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
