// Structural parsing of Rust `use` statements for the example extractor.
//
// addImports() in extract-rust-examples.mjs needs to reconcile a
// component's canonical import (from catalog.json / reference.json /
// gallery/src/pages/mod.rs) with names inferred from the snippet body, and
// the canonical line is a real `use` statement — sometimes a bare
// `use a::b::Name;`, sometimes a braced `use a::b::{X, Y};`. Regex-matching
// only the braced form (the previous implementation) silently treats every
// bare canonical import as contributing zero "already imported" names, so a
// symbol named again by the snippet gets a second, conflicting import. This
// module parses either form (plus `as` renames and globs, for completeness)
// so the generator can reason about "what name is already bound, and by
// which path" uniformly.

import { splitTopLevelCommas } from "./rust.mjs";

/// Split a block of `use` statement text on top-level `;`. `use` bodies
/// contain no string/char literals or comments, so a plain scan is safe.
export function splitUseStatements(text) {
  const stmts = [];
  let start = 0;
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] === ";") {
      const stmt = text.slice(start, i + 1).trim();
      if (stmt) stmts.push(stmt);
      start = i + 1;
    }
  }
  const rest = text.slice(start).trim();
  if (rest) stmts.push(rest);
  return stmts;
}

function parseNameItem(item) {
  const trimmed = item.trim();
  const asMatch = trimmed.match(/^(\S+)\s+as\s+(\S+)$/);
  if (asMatch) return { imported: asMatch[1], local: asMatch[2] };
  return { imported: trimmed, local: trimmed };
}

/// Parse one `use <body>;` statement. Returns `{ path, isGlob, entries }`,
/// where `entries` is `[{ imported, local }]` (braces expanded, one entry
/// per bound name; empty for a glob). `path` is everything before the final
/// segment/group, with no trailing `::`.
export function parseUseStatement(stmt) {
  const m = stmt.match(/^use\s+([\s\S]*);\s*$/);
  if (!m) throw new Error(`not a \`use\` statement: ${stmt}`);
  const body = m[1].trim();
  if (body === "*" || body.endsWith("::*")) {
    const path = body === "*" ? "" : body.slice(0, -"::*".length);
    return { path, isGlob: true, entries: [] };
  }
  const braceStart = body.indexOf("{");
  if (braceStart !== -1) {
    const braceEnd = body.lastIndexOf("}");
    const path = body.slice(0, braceStart).replace(/::$/, "");
    const inner = body.slice(braceStart + 1, braceEnd);
    const entries = splitTopLevelCommas(inner, 0, inner.length).map((part) =>
      parseNameItem(inner.slice(part.start, part.end)),
    );
    return { path, isGlob: false, entries };
  }
  const lastSep = body.lastIndexOf("::");
  const path = lastSep === -1 ? "" : body.slice(0, lastSep);
  const tail = lastSep === -1 ? body : body.slice(lastSep + 2);
  return { path, isGlob: false, entries: [parseNameItem(tail)] };
}
