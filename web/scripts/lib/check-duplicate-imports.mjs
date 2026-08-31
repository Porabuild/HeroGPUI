// Independent duplicate-import checker for web/src/data/rust-examples.json.
//
// Deliberately self-contained: it does NOT import extract-rust-examples.mjs
// or lib/imports.mjs. Sharing a `use`-statement parser with the generator
// would let a bug in that parser hide from both sides at once. This file
// re-derives statement/brace/rename/glob parsing from scratch so it is a
// genuine second opinion, not a mirror of the code it is checking.
//
// A snippet fails to compile with `error[E0252]` when the SAME local name is
// bound more than once by named (non-glob) imports — whether via two
// different paths or the same path twice. Rules applied here:
//   - Split on top-level `;` to get individual `use ...;` statements.
//   - Expand brace groups recursively (`use a::{B, c::{D, E}};`), so every
//     bound name is counted individually, not the group as a whole (the
//     task notes a first attempt got this wrong by double-counting groups).
//   - `use a::X as Y;` counts the LOCAL name `Y`, since that is what the
//     snippet body actually refers to.
//   - A glob (`use a::*;`) never conflicts with anything and is never
//     itself counted as a bound name.

import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const DATA = resolve(scriptDir, "..", "..", "src", "data", "rust-examples.json");

function splitTopComma(s) {
  const parts = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < s.length; i += 1) {
    const c = s[i];
    if (c === "{" || c === "(" || c === "[") depth += 1;
    else if (c === "}" || c === ")" || c === "]") depth -= 1;
    else if (c === "," && depth === 0) {
      parts.push(s.slice(start, i));
      start = i + 1;
    }
  }
  const last = s.slice(start);
  if (last.trim() !== "") parts.push(last);
  return parts;
}

/// Expand one use-tree fragment (the part after `use `, minus the trailing
/// `;`), returning the bound local names and any glob paths within it.
function expandUseTree(body) {
  const trimmed = body.trim();
  if (trimmed === "*" || trimmed.endsWith("::*")) {
    return { globs: [trimmed], names: [] };
  }
  const braceStart = trimmed.indexOf("{");
  if (braceStart !== -1 && trimmed.endsWith("}")) {
    const inner = trimmed.slice(braceStart + 1, -1);
    const globs = [];
    const names = [];
    for (const item of splitTopComma(inner)) {
      const r = expandUseTree(item);
      globs.push(...r.globs);
      names.push(...r.names);
    }
    return { globs, names };
  }
  if (trimmed === "") return { globs: [], names: [] };
  const asMatch = trimmed.match(/^(.*)\sas\s+(\S+)$/s);
  const target = asMatch ? asMatch[1].trim() : trimmed;
  // A bare item may have no `::` at all (a plain name inside a brace group,
  // or a path-less `use Name;`) — lastIndexOf then returns -1, and the whole
  // target IS the name (there is no path prefix to strip).
  const sepIdx = target.lastIndexOf("::");
  const local = asMatch ? asMatch[2].trim() : sepIdx === -1 ? target : target.slice(sepIdx + 2);
  return { globs: [], names: [local] };
}

function splitStatements(importsText) {
  const stmts = [];
  let start = 0;
  for (let i = 0; i < importsText.length; i += 1) {
    if (importsText[i] === ";") {
      const stmt = importsText.slice(start, i).trim();
      if (stmt !== "") stmts.push(stmt);
      start = i + 1;
    }
  }
  const rest = importsText.slice(start).trim();
  if (rest !== "") stmts.push(rest);
  return stmts;
}

/// Return the local names bound more than once by `importsText` (empty if
/// the import block is valid and minimal with respect to duplicate names).
export function duplicateNames(importsText) {
  const counts = new Map();
  for (const stmt of splitStatements(importsText)) {
    const body = stmt.replace(/^use\s+/, "");
    const { names } = expandUseTree(body);
    for (const name of names) counts.set(name, (counts.get(name) ?? 0) + 1);
  }
  return [...counts.entries()].filter(([, n]) => n > 1).map(([name]) => name);
}

export function checkFile(path = DATA) {
  const data = JSON.parse(readFileSync(path, "utf8"));
  const pageSlugs = Object.keys(data);
  let total = 0;
  const offenders = [];
  for (const slug of pageSlugs) {
    for (const section of data[slug]) {
      total += 1;
      const dups = duplicateNames(section.imports ?? "");
      if (dups.length) offenders.push({ slug, heading: section.heading, names: dups });
    }
  }
  return { pages: pageSlugs.length, total, offenders };
}

export function run() {
  const { pages, total, offenders } = checkFile();
  console.log(
    `${total} examples across ${pages} components; ${offenders.length} import a symbol more than once`,
  );
  for (const o of offenders) {
    console.log(`  ${o.slug} / "${o.heading}": ${o.names.join(", ")}`);
  }
  return { pages, total, offenders };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run();
}
