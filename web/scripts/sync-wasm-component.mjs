// Reconcile a component implementation between the native crate and the
// wasm32 migration crate.
//
//   node scripts/sync-wasm-component.mjs report
//   node scripts/sync-wasm-component.mjs sync <file.rs> [--wasm-root D:/herogpui-wasm]
//
// The website executes the migration build while it documents the native
// source, so a component that diverges ships behaviour the docs do not
// describe. The migration targets an older GPUI than this workspace, and the
// difference is a short, mechanical vocabulary -- signature changes and struct
// fields, not design. `report` applies that vocabulary to each native file and
// tells ADAPTED (already current) from STALE (genuinely behind); `sync`
// rewrites one file. The wasm compiler is the gate for anything the
// vocabulary does not cover.

import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

function flag(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index === -1) return fallback;
  const value = process.argv[index + 1];
  return !value || value.startsWith("--") ? true : value;
}

// Components the migration cannot take from native as-is, keyed by the reason.
// A dependency the migration workspace does not carry is not a GPUI version
// difference the vocabulary below can absorb, so syncing one only produces an
// unresolved-crate build. Removing an entry means adding those dependencies to
// the migration workspace and proving they build for wasm32 -- which is what
// closed the date and time entries that used to sit here.
const BLOCKED = {};

const wasmRoot = flag("--wasm-root", "D:/herogpui-wasm");
const nativeDir = join(repoRoot, "crates", "herogpui-components", "src");
const wasmDir = join(wasmRoot, "crates", "herogpui-components", "src");

// GPUI 0.2.2 in this workspace against the older GPUI the migration pins.
const ADAPTATIONS = [
  // `UniformList::track_scroll` took the handle by value.
  [/\.track_scroll\(([a-z_][A-Za-z_0-9]*)\)/g, ".track_scroll(&$1)"],
  // `flex_grow` and `flex_shrink` took their factor before it defaulted to one.
  [/\.flex_grow\(\)/g, ".flex_grow(1.)"],
  [/\.flex_shrink\(\)/g, ".flex_shrink(1.)"],
  // The rest of the focus surface took the app context too.
  [/window\.focus_next\(\)/g, "window.focus_next(cx)"],
  [/window\.focus_prev\(\)/g, "window.focus_prev(cx)"],
  [/window\.blur\(\)/g, "window.blur(cx)"],
  [/\.focus\(window\)/g, ".focus(window, cx)"],
  // `Styled::text_style` answered the refinement itself, not an Option.
  [/\.text_style\(\)\s*\n\s*\.get_or_insert_with\(Default::default\)/g, ".text_style()"],
];

// `Window::focus` took the app context before it was threaded through. Rewrite
// every call rather than guessing which closures own a `cx`: where one is not
// in scope the wasm compiler names the line, and that caller needs a real look
// instead of a silent skip.
function addFocusContext(source) {
  const focused = source.replace(
    /window\.focus\(([^;()]*(?:\([^()]*\))?[^;()]*)\)/g,
    (match, argument) => (/,\s*cx\s*$/.test(argument) ? match : `window.focus(${argument}, cx)`),
  );
  // The context the older signature needs is the parameter the native closure
  // discards, so a closure that focuses has to name it.
  return focused.replace(
    /\|([^|\n]*\bwindow\b[^|\n]*), _\|(.{0,600}?window\.focus\()/gs,
    "|$1, cx|$2",
  );
}

// `Entity::update` answered the value itself before it answered a Result, so
// the native `let Ok(x) = ... else { return };` guard has nothing to unwrap.
function adaptEntityUpdate(source) {
  return source.replace(
    /let Ok\((\w+)\) = (\w+\.update\(cx, .*?\}\)) else \{\s*return;\s*\};/gs,
    "let $1 = $2;",
  );
}

// A handle read out of an entity borrows the context the older `focus`
// signature also needs, so the handle has to be cloned out first.
function adaptEntityFocus(source) {
  return source.replace(
    /^([ \t]*)(\w+)\.read\(cx\)\.(\w+)\.focus\(window, cx\);[ \t]*\r?$/gm,
    "$1let focus_handle = $2.read(cx).$3.clone();\r\n$1focus_handle.focus(window, cx);",
  );
}

// `ScrollHandle::max_offset` answered a `Point` before it answered a `Size`.
function adaptScrollMaxOffset(source) {
  if (!source.includes("max_offset()")) return source;
  return source.replace(/\bmax\.width\b/g, "max.x").replace(/\bmax\.height\b/g, "max.y");
}

// `BoxShadow` gained `inset` after the migration baseline. Brace matching, not
// a pattern: a shadow literal nests calls, and the older struct rejects both a
// missing field and a duplicated one.
function addBoxShadowInset(source) {
  const marker = "BoxShadow {";
  let out = "";
  let index = 0;
  for (;;) {
    const start = source.indexOf(marker, index);
    if (start === -1) return out + source.slice(index);
    let depth = 0;
    let end = start + marker.length - 1;
    for (; end < source.length; end += 1) {
      if (source[end] === "{") depth += 1;
      else if (source[end] === "}") {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    const body = source.slice(start, end);
    out += source.slice(index, start);
    if (/\binset\b/.test(body)) {
      out += body;
    } else {
      const indent = /\n([ \t]*)$/.exec(body)?.[1] ?? "";
      out += `${body.replace(/\s+$/, "")}\n${indent}    inset: false,\n${indent}`;
    }
    index = end;
  }
}

function adapt(source) {
  const rewritten = ADAPTATIONS.reduce(
    (text, [pattern, replacement]) => text.replace(pattern, replacement),
    source,
  );
  return adaptEntityUpdate(
    adaptEntityFocus(adaptScrollMaxOffset(addBoxShadowInset(addFocusContext(rewritten)))),
  );
}

const [mode, file] = process.argv.slice(2);

if (mode === "report") {
  const rows = [];
  for (const name of readdirSync(nativeDir).filter((entry) => entry.endsWith(".rs"))) {
    const native = readFileSync(join(nativeDir, name), "utf8");
    const wasmPath = join(wasmDir, name);
    if (!existsSync(wasmPath)) {
      rows.push([Number.POSITIVE_INFINITY, "ABSENT", name]);
      continue;
    }
    const wasm = readFileSync(wasmPath, "utf8");
    if (wasm === native) {
      rows.push([0, "IDENTICAL", name]);
      continue;
    }
    if (wasm === adapt(native)) {
      rows.push([0, "ADAPTED", name]);
      continue;
    }
    if (BLOCKED[name]) {
      rows.push([Number.POSITIVE_INFINITY, "BLOCKED", name]);
      continue;
    }
    // Lines the migration is missing once the vocabulary is applied.
    const wasmLines = new Set(wasm.split("\n").map((line) => line.trim()));
    const behind = adapt(native)
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line && !wasmLines.has(line)).length;
    rows.push([behind, "STALE", name]);
  }
  rows.sort((a, b) => a[0] - b[0] || a[2].localeCompare(b[2]));
  const counts = {};
  for (const [behind, kind, name] of rows) {
    counts[kind] = (counts[kind] ?? 0) + 1;
    if (kind === "STALE") console.log(`${String(behind).padStart(5)}  ${kind}  ${name}`);
    if (kind === "ABSENT") console.log(`    -  ${kind}  ${name}`);
    if (kind === "BLOCKED") console.log(`    -  ${kind}  ${name} -- ${BLOCKED[name]}`);
  }
  console.log(
    `\n${counts.IDENTICAL ?? 0} identical, ${counts.ADAPTED ?? 0} adapted, ` +
      `${counts.STALE ?? 0} stale, ${counts.BLOCKED ?? 0} blocked, ` +
      `${counts.ABSENT ?? 0} absent`,
  );
} else if (mode === "sync") {
  if (!file) throw new Error("usage: sync-wasm-component.mjs sync <file.rs>");
  const nativePath = join(nativeDir, file);
  if (!existsSync(nativePath)) throw new Error(`${nativePath} does not exist`);
  if (BLOCKED[file] && !flag("--force", false)) {
    throw new Error(`${file} is blocked: ${BLOCKED[file]}. Pass --force once that is resolved.`);
  }
  const adapted = adapt(readFileSync(nativePath, "utf8"));
  const wasmPath = join(wasmDir, file);
  const before = existsSync(wasmPath) ? readFileSync(wasmPath, "utf8") : "";
  if (before === adapted) {
    console.log(`already synced ${file}`);
  } else {
    writeFileSync(wasmPath, adapted);
    console.log(`synced ${file} (${before ? "rewritten" : "created"})`);
  }
} else {
  throw new Error("usage: sync-wasm-component.mjs (report|sync <file.rs>)");
}
