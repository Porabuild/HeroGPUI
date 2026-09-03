// Extract per-component Rust example snippets into
// web/src/data/rust-examples.json.
//
// Source: gallery/src/pages/components.rs. Every component page is built by
// the `component_doc_page!` macro:
//
//   component_doc_page!("Title", <desc>, <import>, vec![("Heading", expr), …], cx)
//
// The macro stringifies each section expression for display, so the raw
// source text of the expression *is* the example code. Expressions nest
// deeply (closures, macros, nested vec!/format!), so sections are split with
// the shared character-level scanner, which also tracks string/char literals
// and comments.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseGalleryPageImports, parseGalleryPages } from "./lib/gallery-pages.mjs";
import { parseUseStatement, splitUseStatements } from "./lib/imports.mjs";
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
const SOURCE = resolve(repoRoot, "gallery", "src", "pages", "components.rs");
const MOD_SOURCE = resolve(repoRoot, "gallery", "src", "pages", "mod.rs");
const CATALOG = resolve(webRoot, "src", "data", "catalog.json");
const REFERENCE = resolve(webRoot, "src", "data", "reference.json");
const OUT = resolve(webRoot, "src", "data", "rust-examples.json");

const LAYOUT_HELPERS = new Set(["row", "col", "field_col", "spec_row"]);
const SPEC_HELPERS = new Set(["spec", "spec_block"]);
const CORE_IMPORTS = [
  "Backdrop",
  "Color",
  "FieldVariant",
  "Orientation",
  "Placement",
  "Prominence",
  "SelectionMode",
  "Size",
  "SizeXl",
  "Variant",
];
const CONTEXT_HELPERS = [
  "avatar_box",
  "bool_cb",
  "build",
  "color_cb",
  "date_cb",
  "demo_calendar",
  "demo_range",
  "demo_text",
  "f32_cb",
  "language_items",
  "languages",
  "opt_color_cb",
  "opt_date_cb",
  "opt_time_cb",
  "opt_usize_cb",
  "overlay_demo",
  "palette",
  "para",
  "shadow_vis_cb",
  "sort_cb",
  "usize_cb",
  "virtual_names",
  "virtual_picker_items",
  "virtual_user",
  "virtual_users",
  "virtual_users_described",
];

/// Read one balanced `( ... )` group whose `(` is at `i`.
/// Returns { start, end } with end past the `)`, or null.
function scanGroup(src, i) {
  if (src[i] !== "(") return null;
  let depth = 1;
  let j = i + 1;
  while (j < src.length) {
    const stepped = stepOver(src, j);
    if (stepped !== null) {
      j = stepped;
      continue;
    }
    const c = src[j];
    if (c === "(" || c === "[" || c === "{") depth += 1;
    else if (c === ")" || c === "]" || c === "}") {
      depth -= 1;
      if (depth === 0) return { start: i, end: j + 1 };
    }
    j += 1;
  }
  return null;
}

/// Skip a balanced expression starting at `i` (any non-comma token), up to
/// and not including the top-level `,` that follows it. Returns the index of
/// that comma, or -1 on structural failure.
function skipExpr(src, i) {
  let j = skipTrivia(src, i);
  let started = false;
  let depth = 0;
  while (j < src.length) {
    const stepped = stepOver(src, j);
    if (stepped !== null) {
      j = stepped;
      started = true;
      continue;
    }
    const c = src[j];
    if (c === "(" || c === "[" || c === "{") {
      depth += 1;
      started = true;
    } else if (c === ")" || c === "]" || c === "}") {
      depth -= 1;
    } else if (c === "," && depth === 0 && started) {
      return j;
    }
    j += 1;
  }
  return -1;
}

/// Parse one `component_doc_page!` invocation whose `component_doc_page` token
/// starts at `i`. Returns { title, sections: [{ heading, code }], end } on
/// success, or { error, end } with end = -1 when the page cannot be parsed.
function parseInvocation(src, i) {
  let skipped = 0;
  let j = skipTrivia(src, i + "component_doc_page".length);
  if (src[j] !== "!" || src[j + 1] !== "(") {
    return { error: "macro invocation not followed by `(`", end: -1 };
  }
  const outer = scanGroup(src, j + 1);
  if (!outer) return { error: "unbalanced macro arguments", end: -1 };

  // Argument 1: title (must be a string literal).
  j = skipTrivia(src, j + 2);
  const titleLit = readStringLiteral(src, j);
  if (!titleLit) {
    return { error: "first argument is not a string literal", end: -1 };
  }
  const title = titleLit.value;
  j = skipTrivia(src, titleLit.end);
  if (src[j] !== ",") return { error: "expected `,` after title", end: -1 };

  // Arguments 2 (description) and 3 (import line): skip.
  for (let arg = 0; arg < 2; arg += 1) {
    j = skipExpr(src, j + 1);
    if (j === -1) return { error: "unbalanced description/import argument", end: -1 };
  }

  // `vec![ … ]` of sections.
  j = skipTrivia(src, j + 1);
  if (!src.startsWith("vec", j)) {
    return { error: "expected `vec![…]` of sections", end: -1 };
  }
  j = skipTrivia(src, j + 3);
  if (src[j] !== "!" || src[j + 1] !== "[") {
    return { error: "expected `vec![…]` of sections", end: -1 };
  }
  j = skipTrivia(src, j + 2);
  const sections = [];
  while (j < outer.end) {
    j = skipTrivia(src, j);
    if (src[j] === "]" || j >= outer.end) break;
    if (src[j] !== "(") {
      return { error: "section list is not a tuple sequence", end: -1 };
    }
    const tuple = scanGroup(src, j);
    if (!tuple) return { error: "unbalanced section tuple", end: -1 };

    // ("Heading", expr)
    let k = skipTrivia(src, j + 1);
    const heading = readStringLiteral(src, k);
    if (heading) {
      k = skipTrivia(src, heading.end);
      if (src[k] !== ",") {
        return { error: "expected `,` after section heading", end: -1 };
      }
      const exprStart = skipTrivia(src, k + 1);
      // The expression runs to the tuple's closing paren (tuple.end - 1).
      const exprEnd = tuple.end - 1;
      // The macro allows a trailing comma inside the tuple
      // (`($heading, $body $(,)?)`); it is separator syntax, not code.
      let code = src.slice(exprStart, exprEnd).trim();
      if (code.endsWith(",")) code = code.slice(0, -1).trimEnd();
      sections.push({
        heading: heading.value,
        code,
        baseIndent: sourceLineIndent(src, exprStart),
      });
    } else {
      // A non-literal heading cannot be shown on the site; skip the section
      // but keep the rest of the page.
      skipped += 1;
    }
    j = tuple.end;
    j = skipTrivia(src, j);
    if (src[j] === ",") {
      j += 1;
    }
  }
  return { title, sections, skipped, end: outer.end };
}

/// Bracket-balance a snippet with the scanner. A truncated extraction shows up
/// as an imbalance, so every emitted snippet is checked before writing.
function isBalanced(code) {
  const stack = [];
  let i = 0;
  while (i < code.length) {
    const stepped = stepOver(code, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    const c = code[i];
    if (c === "(" || c === "[" || c === "{") stack.push(c);
    else if (c === ")" || c === "]" || c === "}") {
      const want = { "(": ")", "[": "]", "{": "}" }[stack.pop()];
      if (want !== c) return false;
    }
    i += 1;
  }
  return stack.length === 0;
}

/// Return the indentation of the source line containing `index`.
function sourceLineIndent(src, index) {
  const lineStart = src.lastIndexOf("\n", index - 1) + 1;
  const prefix = src.slice(lineStart, index);
  return prefix.match(/^[ \t]*/)?.[0].length ?? 0;
}

/// Protect literal/comment spans so indentation cleanup cannot rewrite the
/// contents of a raw string or a multiline comment.
function protectedSpans(src) {
  const spans = [];
  let i = 0;
  while (i < src.length) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      spans.push({ start: i, end: stepped });
      i = stepped;
      continue;
    }
    i += 1;
  }
  return spans;
}

/// Rule: remove the indentation of the macro expression's source line from
/// every following source line, preserving relative Rust indentation. This
/// removes the gallery column while leaving literal contents untouched.
function normalizeIndent(code, baseIndent) {
  const spans = protectedSpans(code);
  const lines = code.replace(/\r\n?/g, "\n").split("\n");
  let offset = 0;
  const normalized = lines.map((line, lineIndex) => {
    const lineStart = offset;
    offset += line.length + 1;
    if (lineIndex === 0 || line.trim() === "") return line;
    const prefix = line.match(/^[ \t]*/)?.[0] ?? "";
    if (
      !prefix ||
      spans.some((span) => span.start < lineStart + prefix.length && span.end > lineStart)
    ) {
      return line;
    }
    return line.slice(Math.min(baseIndent, prefix.length));
  });
  return normalized.join("\n").trim();
}

function callAt(src, start, name) {
  if (!src.startsWith(name, start)) return null;
  if (start > 0 && /[A-Za-z0-9_]/.test(src[start - 1])) return null;
  if (/[A-Za-z0-9_]/.test(src[start + name.length] ?? "")) return null;
  let open = skipTrivia(src, start + name.length);
  if (src[open] !== "(") return null;
  const group = scanDelimited(src, open);
  if (!group) return null;
  return { start, end: group.end, group };
}

export function namedCalls(src, names) {
  const calls = [];
  let i = 0;
  while (i < src.length) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    const ident = readIdent(src, i);
    if (ident && names.has(ident.name)) {
      const call = callAt(src, i, ident.name);
      if (call) calls.push({ ...call, name: ident.name });
      i = ident.end;
      continue;
    }
    i += 1;
  }
  return calls;
}

function callArgs(src, call) {
  return splitTopLevelCommas(src, call.group.start + 1, call.group.end - 1).map((part) => ({
    ...part,
    text: src.slice(part.start, part.end).trim(),
  }));
}

function topLevelCall(src, names) {
  const start = skipTrivia(src, 0);
  const ident = readIdent(src, start);
  if (!ident || !names.has(ident.name)) return null;
  const call = callAt(src, start, ident.name);
  if (!call || skipTrivia(src, call.end) !== src.length) return null;
  return { ...call, name: ident.name, args: callArgs(src, call) };
}

function vecMacro(src) {
  const start = skipTrivia(src, 0);
  const ident = readIdent(src, start);
  if (!ident || ident.name !== "vec") return null;
  const bang = skipTrivia(src, ident.end);
  if (src[bang] !== "!") return null;
  const open = skipTrivia(src, bang + 1);
  if (src[open] !== "[") return null;
  const group = scanDelimited(src, open);
  if (!group || skipTrivia(src, group.end) !== src.length) return null;
  return {
    items: splitTopLevelCommas(src, group.start + 1, group.end - 1).map((part) =>
      src.slice(part.start, part.end).trim(),
    ),
  };
}

function indentReplacement(text, column) {
  const prefix = " ".repeat(column);
  return text
    .split("\n")
    .map((line, index) => (index === 0 ? line : prefix + line))
    .join("\n");
}

function indentNested(text, amount = 4) {
  return text
    .split("\n")
    .map((line, index) => (index === 0 ? line : " ".repeat(amount) + line))
    .join("\n");
}

/// Replace all disjoint innermost named calls in one pass so nested gallery
/// helpers never invalidate the source ranges used for an outer replacement.
function replaceCalls(src, names, replacementFor) {
  let code = src;
  let changed = false;
  for (let pass = 0; pass < 20; pass += 1) {
    const calls = namedCalls(code, names);
    const candidates = calls
      .map((call) => {
        const replacement = replacementFor(call, code, callArgs(code, call));
        return replacement === null ? null : { call, replacement };
      })
      .filter(Boolean)
      .filter(
        ({ call }) =>
          !calls.some(
            (other) => other !== call && other.start < call.start && other.end >= call.end,
          ),
      )
      .sort((a, b) => b.call.start - a.call.start);
    if (!candidates.length) break;
    for (const { call, replacement } of candidates) {
      const column = call.start - (code.lastIndexOf("\n", call.start - 1) + 1);
      const indented = indentReplacement(replacement, column);
      code = code.slice(0, call.start) + indented + code.slice(call.end);
      changed = true;
    }
  }
  return { code, changed };
}

function replaceOutsideToken(src, token, replacement) {
  let code = "";
  let i = 0;
  while (i < src.length) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      code += src.slice(i, stepped);
      i = stepped;
      continue;
    }
    if (src.startsWith(token, i)) {
      code += replacement;
      i += token.length;
    } else {
      code += src[i];
      i += 1;
    }
  }
  return code;
}

function replaceAliases(code) {
  let out = "";
  let i = 0;
  while (i < code.length) {
    const stepped = stepOver(code, i);
    if (stepped !== null) {
      out += code.slice(i, stepped);
      i = stepped;
      continue;
    }
    const atAliasBoundary = i === 0 || !/[A-Za-z0-9_]/.test(code[i - 1]);
    if (atAliasBoundary && code.startsWith("h::icons::", i)) {
      out += "icons::";
      i += "h::icons::".length;
    } else if (atAliasBoundary && code.startsWith("h::", i)) {
      out += "";
      i += "h::".length;
    } else {
      out += code[i];
      i += 1;
    }
  }
  return out;
}

function collectAliases(code) {
  const names = new Set();
  let i = 0;
  while (i < code.length) {
    const stepped = stepOver(code, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    const atAliasBoundary = i === 0 || !/[A-Za-z0-9_]/.test(code[i - 1]);
    if (atAliasBoundary && code.startsWith("h::icons::", i)) {
      names.add("icons");
      i += "h::icons::".length;
      continue;
    }
    if (atAliasBoundary && code.startsWith("h::", i)) {
      const ident = readIdent(code, i + 3);
      if (ident) names.add(ident.name);
      i += 3;
      continue;
    }
    i += 1;
  }
  return names;
}

/// Rule: the gallery's `spec` helpers only add captions/layout around a
/// specimen; retain the actual specimen expression and discard that chrome.
function removeSpecHelpers(code) {
  return replaceCalls(code, SPEC_HELPERS, (_call, src, args) => {
    if (args.length !== 3) return null;
    return args[1].text;
  });
}

function publicLayout(name, arg) {
  const styles =
    name === "row" || name === "spec_row"
      ? [".flex()", ".flex_wrap()", ".w_full()", ".items_start()", ".gap(px(12.))"]
      : name === "field_col"
        ? [".flex()", ".flex_col()", ".w(px(256.))", ".gap(px(12.))"]
        : [".flex()", ".flex_col()", ".items_start()", ".gap(px(12.))"];
  return [
    "gpui::div()",
    ...styles.map((style) => `    ${style}`),
    `    .children(${indentNested(arg)})`,
  ].join("\n");
}

/// Rule: multi-child gallery layout helpers become the same public GPUI
/// container operations, preserving their actual spacing and flex behavior.
export function replaceLayoutHelpers(code) {
  return replaceCalls(code, LAYOUT_HELPERS, (call, src, args) => {
    if (args.length !== 1) return null;
    return publicLayout(call.name, args[0].text);
  });
}

/// Rule: these two simple gallery sizing helpers expand to their equivalent
/// public GPUI builders; their width is part of the rendered specimen.
function replaceSizingHelpers(code) {
  return replaceCalls(code, new Set(["demo_field", "fixed_demo"]), (call, src, args) => {
    if (call.name === "demo_field" && args.length === 1) {
      return [
        "gpui::div()",
        "    .w(px(256.))",
        "    .flex()",
        "    .flex_col()",
        `    .child(${indentNested(args[0].text)})`,
      ].join("\n");
    }
    if (call.name === "fixed_demo" && args.length === 2) {
      // `fixed_demo(width: f32, …)` wraps its width in `px` internally
      // (gallery/src/pages/components.rs), so the expansion must too —
      // `Into<Length>` does not accept a bare float.
      return [
        "gpui::div()",
        `    .w(px(${args[0].text}))`,
        `    .child(${indentNested(args[1].text)})`,
      ].join("\n");
    }
    return null;
  });
}

/// Rule: `icon(path, cx)` is a gallery-only wrapper for this exact public SVG
/// builder. Expand it so icon examples retain their real rendering code.
function replaceIconHelpers(code) {
  return replaceCalls(code, new Set(["icon"]), (_call, src, args) => {
    if (args.length !== 2) return null;
    return [
      "gpui::svg()",
      "    .size(px(16.))",
      `    .path(${args[0].text})`,
      `    .text_color(${args[1].text}.colors().foreground)`,
    ].join("\n");
  });
}

/// Rule: `el_id` only materialises a name for the gallery helper; its argument
/// already implements the public `Into<ElementId>` contract.
function removeElementIdHelpers(code) {
  return replaceCalls(code, new Set(["el_id"]), (_call, src, args) =>
    args.length === 1 ? args[0].text : null,
  );
}

/// Rule: shorten obvious generated gallery ids while preserving a unique
/// suffix. Required component ids stay present, but no gallery shorthand or
/// section-only `usage` name is exposed to the reader.
function humanizeIdText(text, pageSlug) {
  if (text === pageSlug || text.startsWith(`${pageSlug}-`)) return text;
  const match = text.match(/^([a-z][a-z0-9]*)-(.+)$/);
  if (!match) return text;
  const [, , rest] = match;
  const parts = rest.split("-");
  const suffix =
    parts[0] === "usage"
      ? []
      : ["v", "s", "i", "io", "soc"].includes(parts[0])
        ? parts.slice(1)
        : parts;
  return [pageSlug, ...suffix].join("-");
}

function rewriteIdExpression(expr, pageSlug) {
  const trimmedStart = expr.search(/\S/);
  if (trimmedStart === -1) return null;
  const start = trimmedStart;
  if (expr[start] === '"') {
    const literal = readStringLiteral(expr, start);
    if (!literal) return null;
    const bodyStart = start + 1;
    const body = expr.slice(bodyStart, literal.end - 1);
    const next = humanizeIdText(body, pageSlug);
    if (next === body) return null;
    return expr.slice(0, bodyStart) + next + expr.slice(literal.end - 1);
  }
  const formatStart = expr.indexOf("format!", start);
  if (formatStart !== start) return null;
  const literalStart = expr.indexOf('"', formatStart);
  if (literalStart === -1) return null;
  const literal = readStringLiteral(expr, literalStart);
  if (!literal) return null;
  const bodyStart = literalStart + 1;
  const body = expr.slice(bodyStart, literal.end - 1);
  const next = humanizeIdText(body, pageSlug);
  if (next === body) return null;
  return expr.slice(0, bodyStart) + next + expr.slice(literal.end - 1);
}

function humanizeGalleryIds(code, pageSlug) {
  return replaceCalls(code, new Set(["id", "new"]), (call, src, args) => {
    if (args.length === 0) return null;
    const next = rewriteIdExpression(args[0].text, pageSlug);
    if (next === null) return null;
    const argStart = args[0].start + src.slice(args[0].start, args[0].end).search(/\S/);
    return src.slice(call.start, argStart) + next + src.slice(args[0].end, call.end);
  });
}

function stripTerminalAnyElement(code) {
  const suffix = ".into_any_element()";
  let terminal = -1;
  let i = 0;
  while (i < code.length) {
    const stepped = stepOver(code, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    if (code.startsWith(suffix, i) && skipTrivia(code, i + suffix.length) === code.length) {
      terminal = i;
      break;
    }
    i += 1;
  }
  return terminal === -1 ? code : code.slice(0, terminal).trimEnd();
}

function cleanExample(rawCode, baseIndent, pageSlug) {
  const aliases = collectAliases(rawCode);
  let code = normalizeIndent(rawCode, baseIndent);
  for (let pass = 0; pass < 20; pass += 1) {
    const before = code;
    const top = topLevelCall(code, LAYOUT_HELPERS);
    if (top) {
      const args = top.args;
      const vector = args.length === 1 ? vecMacro(args[0].text) : null;
      if (vector?.items.length === 1) {
        code = vector.items[0];
      }
    }
    const spec = topLevelCall(code, SPEC_HELPERS);
    if (spec && spec.args.length === 3) code = spec.args[1].text;
    code = removeSpecHelpers(code).code;
    code = removeElementIdHelpers(code).code;
    code = replaceIconHelpers(code).code;
    code = replaceSizingHelpers(code).code;
    code = replaceLayoutHelpers(code).code;
    code = replaceOutsideToken(code, ".els()", "");
    code = replaceAliases(code);
    code = humanizeGalleryIds(code, pageSlug).code;
    code = stripTerminalAnyElement(code);
    if (code === before) break;
  }
  return { code: code.trim(), aliases };
}

function canonicalImports() {
  const catalog = JSON.parse(readFileSync(CATALOG, "utf8"));
  const reference = JSON.parse(readFileSync(REFERENCE, "utf8"));
  const mod = readFileSync(MOD_SOURCE, "utf8");
  const pages = parseGalleryPages(mod);
  const pageImports = parseGalleryPageImports(mod);
  const bySlug = new Map();
  for (const page of pages) {
    const candidates = [
      catalog.components?.[page.titleSlug]?.importLine,
      reference[page.titleSlug]?.importLine,
      pageImports.get(page.name),
    ].filter((line) => typeof line === "string" && line.trim() !== "");
    const line = candidates[0] ?? "";
    if (candidates.some((candidate) => candidate !== line)) {
      throw new Error(`import line mismatch for ${page.titleSlug}`);
    }
    bySlug.set(page.titleSlug, line);
  }
  return bySlug;
}

function importNamesFromCode(code, aliases) {
  const names = new Set(aliases);
  for (const name of CORE_IMPORTS) {
    if (new RegExp(`\\b${name}\\s*::`).test(code)) names.add(name);
  }
  return names;
}

// Rule: modules of herogpui-components that `herogpui::prelude` does NOT
// expose as a glob. Verified against crates/herogpui-components/src/lib.rs:
// every `pub mod` there is re-exported with a matching `pub use <mod>::*;`
// except `list_nav` (no re-export at all) and `util` (only `app_focus_root`
// is re-exported by name); `gallery_source` is additionally cfg-gated
// behind a feature the docs build does not enable. Every other component
// module, plus the herogpui_core prop-vocabulary types and the
// herogpui_theme surface, is confirmed re-exported by
// crates/herogpui/src/lib.rs's `pub mod prelude`, so those symbols default
// to the prelude path below.
const NON_PRELUDE_COMPONENT_MODULES = new Set(["list_nav", "util", "gallery_source"]);
const PRELUDE_EXCEPTIONS_BY_MODULE = new Map([["util", new Set(["app_focus_root"])]]);

// Rule: prefer `herogpui::prelude` over a specific `herogpui::components::*`
// path whenever the prelude genuinely re-exports that symbol (see above), so
// a name never keeps a module path the prelude has already made redundant.
// A symbol from an excluded module keeps its own full module path, since
// the prelude does not export it.
function preludePathFor(path, name) {
  const match = path.match(/^herogpui::components::([a-z_0-9]+)$/);
  if (!match) return path;
  const mod = match[1];
  if (!NON_PRELUDE_COMPONENT_MODULES.has(mod)) return "herogpui::prelude";
  return PRELUDE_EXCEPTIONS_BY_MODULE.get(mod)?.has(name) ? "herogpui::prelude" : path;
}

// Rule: every name the snippet needs — whether declared by the page's own
// canonical import or inferred from the code — is bound to exactly ONE
// path, never two. Canonical's own path wins when it names the symbol (it
// is the authoritative source for its own page); anything canonical
// doesn't mention is either a herogpui_core prop-vocabulary type (detected
// via CORE_IMPORTS) or a herogpui_components item referenced through the
// gallery's `h::` shorthand (an alias) — both re-exported by the prelude,
// so unmatched names default there. `preludePathFor` then collapses
// specific module paths onto the prelude wherever that is valid.
//
// This is what reconciles a canonical `herogpui::components::meter::Meter`
// against a snippet that also needs `Meter`: the previous implementation
// recognised a canonical name as "already imported" only when it appeared
// inside a `{...}` group, so a bare single-name canonical import (the
// common case — every page except Button) was never recognised as
// covering its own symbol, and that symbol was added a second time under
// `herogpui::prelude`, binding one name to two paths (`error[E0252]`).
function resolveImportPaths(canonical, code, aliases) {
  const path = new Map(); // local name -> path
  const globs = [];
  if (canonical) {
    for (const stmt of splitUseStatements(canonical)) {
      const parsed = parseUseStatement(stmt);
      if (parsed.isGlob) {
        globs.push(parsed.path);
        continue;
      }
      for (const { local } of parsed.entries) path.set(local, parsed.path);
    }
  }
  for (const name of importNamesFromCode(code, aliases)) {
    if (!path.has(name)) path.set(name, "herogpui::prelude");
  }
  // Rule: `cx.colors()` is an `ActiveTheme` trait method — the call site
  // never names the trait, so a snippet can need it in scope with no
  // path-shaped token to detect. The prelude re-exports the trait by name,
  // so bind it like any other import unless a canonical `herogpui::prelude`
  // glob already covers it. (Corpus check: `.colors()` is the only trait
  // method the examples use, always on a context value.)
  if (
    !path.has("ActiveTheme") &&
    !globs.includes("herogpui::prelude") &&
    /\.colors\(\)/.test(code)
  ) {
    path.set("ActiveTheme", "herogpui::prelude");
  }
  for (const [name, p] of path) path.set(name, preludePathFor(p, name));
  return { path, globs };
}

function addImports(canonical, code, aliases) {
  const { path, globs } = resolveImportPaths(canonical, code, aliases);

  // Rule: merge every name sharing a path onto one line; braces only when a
  // path ends up with more than one name, so a single import keeps the
  // plain `use path::Name;` form already used throughout the corpus.
  const groups = new Map();
  for (const [name, p] of path) {
    if (!groups.has(p)) groups.set(p, []);
    groups.get(p).push(name);
  }
  const herogpuiLines = globs.map((g) => `use ${g}::*;`);
  for (const [p, names] of groups) {
    names.sort();
    herogpuiLines.push(
      names.length > 1 ? `use ${p}::{${names.join(", ")}};` : `use ${p}::${names[0]};`,
    );
  }

  const gpuiLines = [];
  if (
    /\bgpui::/.test(code) ||
    /\.(?:child|children|flex|flex_col|flex_wrap|items_start|gap|when|on_press|on_click|into_any_element|size|w|h|px|text_color|rounded|path)\s*\(/.test(
      code,
    )
  ) {
    gpuiLines.push("use gpui::prelude::*;");
  }
  if (/\bpx\s*\(/.test(code)) gpuiLines.push("use gpui::px;");

  const otherLines = [];
  if (/\bDuration\s*::/.test(code)) otherLines.push("use std::time::Duration;");

  // Rule: stable order — herogpui first, then gpui, then anything else;
  // alphabetical within each group (a glob's `path::*` sorts like any other
  // line text, e.g. `components::` before `prelude::`).
  herogpuiLines.sort();
  gpuiLines.sort();
  otherLines.sort();
  return [...herogpuiLines, ...gpuiLines, ...otherLines].join("\n");
}

export function run({ check = false } = {}) {
  const src = readFileSync(SOURCE, "utf8");
  const importsByPage = canonicalImports();
  const pages = new Map();
  /** @type {{ page: string, reason: string }[]} */
  const skippedPages = [];
  let skippedSections = 0;
  let totalSnippets = 0;
  let unbalanced = 0;
  const retainedHelpers = new Map();

  let i = 0;
  while (i < src.length) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    if (src.startsWith("component_doc_page", i) && (i === 0 || !/[A-Za-z0-9_]/.test(src[i - 1]))) {
      // Only `component_doc_page!(` is an invocation; the macro definition
      // (`macro_rules!`) and the plain `component_doc_page(` fn call share
      // the token and are walked past.
      const bang = skipTrivia(src, i + "component_doc_page".length);
      if (src[bang] !== "!") {
        i += "component_doc_page".length;
        continue;
      }
      const result = parseInvocation(src, i);
      if (result.end === -1) {
        skippedPages.push({ page: "(unknown)", reason: result.error });
        i += "component_doc_page".length;
        continue;
      }
      skippedSections += result.skipped;
      const slug = slugify(result.title);
      if (!pages.has(slug)) pages.set(slug, []);
      // A page body may legitimately skip a section with a non-literal
      // heading; anything extracted counts.
      if (result.sections.length === 0) {
        skippedPages.push({
          page: result.title,
          reason: "no sections with literal headings could be read",
        });
      }
      for (const section of result.sections) {
        const cleaned = cleanExample(section.code, section.baseIndent, slug);
        if (!isBalanced(cleaned.code)) {
          unbalanced += 1;
          console.error(
            `WARNING: ${result.title} / "${section.heading}": extracted code is not balanced; keeping it but review`,
          );
        }
        const helpers = CONTEXT_HELPERS.filter((helper) =>
          new RegExp(`\\b${helper}\\s*\\(`).test(cleaned.code),
        );
        for (const helper of helpers) {
          if (!retainedHelpers.has(helper)) retainedHelpers.set(helper, []);
          retainedHelpers.get(helper).push(`${slug}/${section.heading}`);
        }
        pages.get(slug).push({
          heading: section.heading,
          imports: addImports(importsByPage.get(slug) ?? "", cleaned.code, cleaned.aliases),
          code: cleaned.code,
        });
        totalSnippets += 1;
      }
      i = result.end;
      continue;
    }
    i += 1;
  }

  const data = Object.fromEntries([...pages.entries()]);
  const output = JSON.stringify(data, null, 2) + "\n";
  if (check) {
    let current = "";
    try {
      current = readFileSync(OUT, "utf8");
    } catch {}
    if (current !== output) {
      console.error(
        "ERROR: rust-examples.json is stale; run `pnpm run extract`",
      );
      process.exitCode = 1;
    }
  } else {
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, output);
  }

  console.log(
    `rust-examples.json: ${pages.size} pages, ${totalSnippets} snippets` +
      (unbalanced ? `, ${unbalanced} unbalanced` : "") +
      (skippedPages.length ? `, ${skippedPages.length} skipped pages` : "") +
      (skippedSections ? `, ${skippedSections} skipped sections` : ""),
  );
  for (const skip of skippedPages) {
    console.log(`  skipped: ${skip.page} — ${skip.reason}`);
  }
  if (retainedHelpers.size) {
    console.log("  retained context helpers (not safely expandable without inventing state/data):");
    for (const [helper, snippets] of retainedHelpers) {
      console.log(`    ${helper}: ${snippets.join(", ")}`);
    }
  }
  return { pages: pages.size, snippets: totalSnippets, skippedPages };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
