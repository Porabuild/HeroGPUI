// Parse gallery/src/pages/mod.rs into the page registry the catalog and the
// reference extractor share: enum variants with their `// Category` comments,
// plus the display titles and descriptions from `impl Page`.

import {
  readIdent,
  readStringLiteral,
  scanDelimited,
  skipTrivia,
  slugify,
  stepOver,
} from "./rust.mjs";

/// Find the first occurrence of `needle` at top level (not inside a literal
/// or comment). Returns the index, or -1.
function findTopLevel(src, needle, from = 0) {
  let i = from;
  while (i < src.length) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    if (
      src.startsWith(needle, i) &&
      (i === 0 || !/[A-Za-z0-9_]/.test(src[i - 1])) &&
      !/[A-Za-z0-9_]/.test(src[i + needle.length] ?? "")
    ) {
      return i;
    }
    i += 1;
  }
  return -1;
}

/// Collect the string literals of a match-arm value (which may be a plain
/// literal, a `{ ... }` block, or a `concat!(...)` invocation) joined into one
/// string.
function armValueStrings(src, start, end) {
  let out = "";
  let i = start;
  while (i < end) {
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      // stepOver skips literals without exposing their text; re-read it.
      if (src[i] === '"') {
        const s = readStringLiteral(src, i);
        if (s) {
          out += s.value;
          i = s.end;
          continue;
        }
      }
      i = stepped;
      continue;
    }
    i += 1;
  }
  return out;
}

/// Parse `match self { Page::X => <value>, ... }` arms starting at the `match`
/// keyword. Returns a Map of variant name -> arm value range.
function parseMatchArms(src, matchKw) {
  let i = skipTrivia(src, matchKw + "match".length);
  // Skip the scrutinee (`self`) up to the `{`.
  while (i < src.length && src[i] !== "{") {
    const stepped = stepOver(src, i);
    i = stepped !== null ? stepped : i + 1;
  }
  const body = scanDelimited(src, i);
  if (!body) throw new Error("mod.rs: `match self {` block not terminated");
  const arms = new Map();
  let j = i + 1;
  while (j < body.end - 1) {
    j = skipTrivia(src, j);
    if (j >= body.end - 1) break;
    if (!src.startsWith("Page::", j)) {
      const stepped = stepOver(src, j);
      j = stepped !== null ? stepped : j + 1;
      continue;
    }
    const variant = readIdent(src, j + "Page::".length);
    if (!variant) {
      j += "Page::".length;
      continue;
    }
    let k = skipTrivia(src, variant.end);
    if (!src.startsWith("=>", k)) {
      j = variant.end;
      continue;
    }
    k = skipTrivia(src, k + 2);
    // Value: everything until a top-level `,`, the closing brace, or the
    // next arm's `Page::` (a `{ ... }` block value closes at its own `}`,
    // so bracket depth alone cannot delimit it).
    const valueStart = k;
    let depth = 0;
    while (k < body.end - 1) {
      const stepped = stepOver(src, k);
      if (stepped !== null) {
        k = stepped;
        continue;
      }
      const c = src[k];
      if (c === "(" || c === "[" || c === "{") depth += 1;
      else if (c === ")" || c === "]" || c === "}") {
        if (depth === 0) break;
        depth -= 1;
      } else if ((c === "," && depth === 0) || (depth === 0 && src.startsWith("Page::", k))) {
        break;
      }
      k += 1;
    }
    arms.set(variant.name, { start: valueStart, end: k });
    j = k;
  }
  return arms;
}

/// Parse the `pub enum Page { ... }` body: ordered variants with the most
/// recent preceding `// Category` comment (doc comments `///` are ignored).
function parseEnumVariants(src) {
  const enumKw = findTopLevel(src, "enum Page");
  if (enumKw === -1) throw new Error("mod.rs: `enum Page` not found");
  const brace = skipTrivia(src, enumKw + "enum Page".length);
  const body = scanDelimited(src, brace);
  if (!body) throw new Error("mod.rs: `enum Page` body not terminated");

  const variants = [];
  let category = null;
  let i = brace + 1;
  while (i < body.end - 1) {
    const c = src[i];
    // `// Category` comments; `///` doc comments are not categories.
    if (c === "/" && src[i + 1] === "/" && src[i + 2] !== "/") {
      const lineEnd = src.indexOf("\n", i);
      const text = src.slice(i + 2, lineEnd === -1 ? body.end : lineEnd).trim();
      if (text) category = text;
      i = lineEnd === -1 ? body.end : lineEnd + 1;
      continue;
    }
    if (c === "/" && src[i + 1] === "/" && src[i + 2] === "/") {
      const lineEnd = src.indexOf("\n", i);
      i = lineEnd === -1 ? body.end : lineEnd + 1;
      continue;
    }
    const stepped = stepOver(src, i);
    if (stepped !== null) {
      i = stepped;
      continue;
    }
    if (/[A-Za-z_]/.test(c)) {
      const ident = readIdent(src, i);
      let j = skipTrivia(src, ident.end);
      // A variant is an identifier followed by `,` (or the closing brace).
      if (src[j] === "," || src[j] === "}") {
        variants.push({ name: ident.name, category });
        i = src[j] === "," ? j + 1 : j;
        continue;
      }
      i = ident.end;
      continue;
    }
    i += 1;
  }
  return variants;
}

/// Parse mod.rs into ordered pages:
/// [{ variant, category, title, description, titleSlug }].
export function parseGalleryPages(src) {
  const variants = parseEnumVariants(src);

  const titleKw = findTopLevel(src, "fn title");
  if (titleKw === -1) throw new Error("mod.rs: `fn title` not found");
  const titles = parseMatchArms(src, findTopLevel(src, "match", titleKw));

  const descKw = findTopLevel(src, "fn description");
  if (descKw === -1) throw new Error("mod.rs: `fn description` not found");
  const descriptions = parseMatchArms(src, findTopLevel(src, "match", descKw));

  return variants.map((v) => {
    const titleRange = titles.get(v.name);
    const descRange = descriptions.get(v.name);
    if (!titleRange) throw new Error(`mod.rs: no title arm for Page::${v.name}`);
    if (!descRange) throw new Error(`mod.rs: no description arm for Page::${v.name}`);
    const title = armValueStrings(src, titleRange.start, titleRange.end);
    const description = armValueStrings(src, descRange.start, descRange.end)
      .replace(/\s+/g, " ")
      .trim();
    return { ...v, title, description, titleSlug: slugify(title) };
  });
}

/// Parse `Page::X => "use ..."` arms from `Page::import_line`.
///
/// The generated catalog/reference data is the normal source for imports, but
/// a few gallery-only pages intentionally have no reference metadata yet. The
/// page registry remains the authoritative fallback for those canonical lines.
export function parseGalleryPageImports(src) {
  const importKw = findTopLevel(src, "fn import_line");
  if (importKw === -1) throw new Error("mod.rs: `fn import_line` not found");
  const matchKw = findTopLevel(src, "match", importKw);
  if (matchKw === -1) throw new Error("mod.rs: import-line match not found");
  const arms = parseMatchArms(src, matchKw);
  return new Map(
    [...arms.entries()].map(([variant, range]) => [
      variant,
      armValueStrings(src, range.start, range.end).trim(),
    ]),
  );
}
