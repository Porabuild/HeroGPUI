// What a gallery example needs beyond its own expression to compile on its
// own: the page function's `let` bindings it reads, and the definitions of the
// gallery helper functions it calls. Both are lifted verbatim from the gallery
// source, so the website shows the real code rather than a paraphrase, and
// the compile gate (gallery/build.rs) proves the result builds against the
// public `herogpui` API.

import { readIdent, scanDelimited, skipTrivia, stepOver } from "./rust.mjs";

const KEYWORDS = new Set(["mut", "ref", "_", "let", "move", "self", "Self", "true", "false"]);

/// Identifiers used as values or calls (not `.method`, not `path::`-qualified
/// segments, not inside literals or comments).
export function referencedNames(code) {
  const names = new Set();
  let i = 0;
  while (i < code.length) {
    const stepped = stepOver(code, i);
    if (stepped !== null) {
      // A format string's inline arguments (`"{count} items"`) are uses too.
      if (code[i] === '"' || code[i] === "r") {
        const literal = code.slice(i, stepped).replaceAll("{{", "");
        for (const match of literal.matchAll(/\{([A-Za-z_][A-Za-z0-9_]*)/g)) names.add(match[1]);
      }
      i = stepped;
      continue;
    }
    const ident = readIdent(code, i);
    if (ident && (i === 0 || !/[A-Za-z0-9_]/.test(code[i - 1]))) {
      const before = code.slice(Math.max(0, i - 2), i);
      const qualified = before.endsWith(".") || before === "::";
      if (!qualified) names.add(ident.name ?? code.slice(i, ident.end));
      i = ident.end ?? i + 1;
      continue;
    }
    i += 1;
  }
  return names;
}

/// End of the statement starting at `i`: the index after its top-level `;`,
/// or after the closing `}` of an item (`fn`, block) when there is no `;`.
function statementEnd(src, i, limit, itemBlock = false) {
  let depth = 0;
  let j = i;
  while (j < limit) {
    const stepped = stepOver(src, j);
    if (stepped !== null) {
      j = stepped;
      continue;
    }
    const c = src[j];
    if (c === "(" || c === "[" || c === "{") depth += 1;
    else if (c === ")" || c === "]" || c === "}") {
      depth -= 1;
      if (itemBlock && depth === 0 && c === "}") return j + 1;
    } else if (c === ";" && depth === 0) return j + 1;
    j += 1;
  }
  return -1;
}

/// The identifiers a `let` pattern binds (`let (a, mut b): T = ...`).
function patternNames(pattern) {
  const colon = (() => {
    let depth = 0;
    for (let k = 0; k < pattern.length; k += 1) {
      const c = pattern[k];
      if ("([{".includes(c)) depth += 1;
      else if (")]}".includes(c)) depth -= 1;
      else if (c === ":" && depth === 0 && pattern[k + 1] !== ":") return k;
    }
    return -1;
  })();
  const head = colon === -1 ? pattern : pattern.slice(0, colon);
  return [...head.matchAll(/[A-Za-z_][A-Za-z0-9_]*/g)]
    .map((m) => m[0])
    .filter((name) => !KEYWORDS.has(name) && !/^[A-Z]/.test(name));
}

/// Find the body of the `fn` that encloses `macroStart`.
function enclosingFnBody(src, macroStart) {
  const re = /\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*(?:<[^>{]*>)?\s*\(/g;
  let bodyStart = -1;
  let match;
  while ((match = re.exec(src)) !== null && match.index < macroStart) {
    const params = scanDelimited(src, match.index + match[0].length - 1);
    if (!params) continue;
    const brace = src.indexOf("{", params.end);
    if (brace === -1 || brace >= macroStart) continue;
    // Only a function whose body still spans the invocation encloses it; a
    // local `fn` declared earlier in the page body has already closed.
    const body = scanDelimited(src, brace);
    if (body && body.end > macroStart) bodyStart = brace + 1;
  }
  return bodyStart;
}

/// The `let` statements and local `fn` items a page function declares before
/// its `component_doc_page!` invocation, in source order.
export function pageBindings(src, macroStart) {
  const bodyStart = enclosingFnBody(src, macroStart);
  if (bodyStart === -1) return [];
  const bindings = [];
  let j = bodyStart;
  while (j < macroStart) {
    j = skipTrivia(src, j);
    if (j >= macroStart) break;
    const lineStart = src.lastIndexOf("\n", j - 1) + 1;
    if (/^let\b/.test(src.slice(j, j + 4))) {
      const end = statementEnd(src, j, macroStart);
      if (end === -1) break;
      const text = src.slice(j, end);
      const eq = text.search(/[^=!<>]=[^=>]/);
      const pattern = eq === -1 ? text.slice(3, -1) : text.slice(3, eq + 1);
      bindings.push({ names: patternNames(pattern), text, indent: j - lineStart });
      j = end;
      continue;
    }
    const constMatch = src.slice(j, j + 200).match(/^(?:const|static)\s+([A-Za-z_][A-Za-z0-9_]*)/);
    if (constMatch) {
      const end = statementEnd(src, j, macroStart);
      if (end === -1) break;
      bindings.push({ names: [constMatch[1]], text: src.slice(j, end), indent: j - lineStart });
      j = end;
      continue;
    }
    const fnMatch = src.slice(j, j + 200).match(/^fn\s+([A-Za-z_][A-Za-z0-9_]*)/);
    if (fnMatch) {
      const end = statementEnd(src, j, macroStart, true);
      if (end === -1) break;
      bindings.push({ names: [fnMatch[1]], text: src.slice(j, end), indent: j - lineStart });
      j = end;
      continue;
    }
    // Any other statement (a side effect, a nested item) is not lifted.
    const end = statementEnd(
      src,
      j,
      macroStart,
      src[j] === "{" || /^(if|for|while|match)\b/.test(src.slice(j, j + 6)),
    );
    if (end === -1) break;
    j = end;
  }
  return bindings;
}

/// Remove `column` leading spaces from every line after the first.
export function dedent(text, column) {
  const lines = text.split("\n");
  return [
    lines[0],
    ...lines.slice(1).map((line) => {
      const lead = line.match(/^ */)[0].length;
      return line.slice(Math.min(lead, column));
    }),
  ].join("\n");
}

/// The bindings a snippet reads, transitively, in declaration order.
export function neededBindings(code, bindings) {
  const wanted = new Set();
  let names = referencedNames(code);
  for (let pass = 0; pass < 10; pass += 1) {
    let grew = false;
    bindings.forEach((binding, index) => {
      if (wanted.has(index)) return;
      if (binding.names.some((name) => names.has(name))) {
        wanted.add(index);
        grew = true;
      }
    });
    if (!grew) break;
    names = referencedNames([code, ...[...wanted].map((index) => bindings[index].text)].join("\n"));
  }
  return [...wanted].sort((a, b) => a - b).map((index) => bindings[index]);
}

/// Top-level helper items (`fn`, `const`, `static`) of a gallery module,
/// keyed by name, with their visibility stripped.
export function helperItems(src) {
  const items = new Map();
  // `thread_local! { static NAME: ... }` blocks, keyed by each static inside.
  for (const block of src.matchAll(/^thread_local!\s*\{/gm)) {
    const group = scanDelimited(src, block.index + block[0].length - 1);
    if (!group) continue;
    const text = src
      .slice(block.index, group.end)
      .replace(/\bpub(?:\([a-z]+\))? static/g, "static");
    for (const name of text.matchAll(/\bstatic\s+([A-Za-z_][A-Za-z0-9_]*)/g)) {
      if (!items.has(name[1])) items.set(name[1], text);
    }
  }
  // Gallery-private types a helper relies on (`struct X(usize);`), with every
  // `impl Trait for X` block attached to the struct's own definition.
  const structs = new Map();
  for (const m of src.matchAll(/^(?:pub(?:\([a-z]+\))? )?struct\s+([A-Za-z_][A-Za-z0-9_]*)/gm)) {
    const end = statementEnd(src, m.index, src.length, true);
    if (end !== -1) structs.set(m[1], src.slice(m.index, end).replace(/^pub(?:\([a-z]+\))? /, ""));
  }
  for (const m of src.matchAll(
    /^impl(?:<[^>]*>)?\s+[A-Za-z_][\w:]*(?:<[^>]*>)?\s+for\s+([A-Za-z_][A-Za-z0-9_]*)[^{;]*\{/gm,
  )) {
    if (!structs.has(m[1])) continue;
    const end = statementEnd(src, m.index, src.length, true);
    if (end !== -1) structs.set(m[1], `${structs.get(m[1])}\n${src.slice(m.index, end)}`);
  }
  for (const [name, text] of structs) items.set(name, text);
  const re = /^(?:pub(?:\([a-z]+\))? )?(fn|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)/gm;
  let match;
  while ((match = re.exec(src)) !== null) {
    const start = match.index;
    // Keep the doc comment directly above the item.
    let docStart = start;
    for (;;) {
      const prevEnd = docStart - 1;
      const prevStart = src.lastIndexOf("\n", prevEnd - 1) + 1;
      const line = src.slice(prevStart, prevEnd);
      if (prevEnd <= 0 || !/^\s*(\/\/\/|#\[)/.test(line)) break;
      docStart = prevStart;
    }
    const end = statementEnd(src, start, src.length, match[1] === "fn");
    if (end === -1) continue;
    const text = src
      .slice(docStart, end)
      .replace(/^(\s*)pub(?:\([a-z]+\))? (fn|const|static)/m, "$1$2");
    if (!items.has(match[2])) items.set(match[2], text);
    re.lastIndex = end;
  }
  return items;
}

/// The helper definitions a snippet needs, transitively, in a stable order.
export function neededHelpers(code, helpers, exclude = new Set()) {
  const wanted = [];
  const seen = new Set();
  const visit = (text) => {
    for (const name of referencedNames(text)) {
      if (seen.has(name) || exclude.has(name) || !helpers.has(name)) continue;
      seen.add(name);
      visit(helpers.get(name));
      wanted.push(name);
    }
  };
  visit(code);
  // Several names can share one definition (a `thread_local!` block).
  return [...new Set(wanted.map((name) => helpers.get(name)))];
}
