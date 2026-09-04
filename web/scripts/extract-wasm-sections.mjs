// Record the component examples compiled into the checked-in wasm artifact.
// Run this against the wasm migration worktree whenever that artifact changes.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseInvocation } from "./extract-rust-examples.mjs";
import { slugify } from "./lib/rust.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const out = resolve(scriptDir, "..", "src", "data", "wasm-sections.json");

function sourceArgument() {
  const index = process.argv.indexOf("--source");
  if (index === -1 || !process.argv[index + 1]) {
    throw new Error("usage: node scripts/extract-wasm-sections.mjs --source <components.rs>");
  }
  return resolve(process.argv[index + 1]);
}

const source = readFileSync(sourceArgument(), "utf8");
const pages = new Map();
let index = 0;

while ((index = source.indexOf("component_doc_page!", index)) !== -1) {
  const page = parseInvocation(source, index);
  if (!page) {
    index += "component_doc_page!".length;
    continue;
  }
  pages.set(
    slugify(page.title),
    page.sections.map((section) => section.heading),
  );
  index = page.end;
}

const data = Object.fromEntries(pages);
writeFileSync(out, `${JSON.stringify(data, null, 2)}\n`);
console.log(
  `wasm-sections.json: ${pages.size} pages, ${[...pages.values()].flat().length} examples`,
);
