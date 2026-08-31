// One entry point for the offline data pipeline. Runs the extractors in
// dependency order — reference first (the catalog joins against it for
// importLine), then the catalog, the Rust snippets, and the screenshot copy —
// and prints a single summary.
//
//   node scripts/build-data.mjs

import { pathToFileURL } from "node:url";

import { run as copyShots } from "./copy-shots.mjs";
import { run as extractCatalog } from "./extract-catalog.mjs";
import { run as extractReference } from "./extract-reference.mjs";
import { run as extractRustExamples } from "./extract-rust-examples.mjs";

/// Run one pipeline step and stop when it failed (the extractors signal by
/// setting process.exitCode and refusing to write partial data).
function step(name, fn) {
  const before = process.exitCode;
  const result = fn();
  if (process.exitCode !== undefined && process.exitCode !== before && process.exitCode !== 0) {
    throw new Error(`${name} failed; stopping build-data`);
  }
  return result;
}

export function run() {
  const reference = step("extract-reference", () => extractReference());
  const catalog = step("extract-catalog", () => extractCatalog());
  const rust = step("extract-rust-examples", () => extractRustExamples());
  const shots = step("copy-shots", () => copyShots());

  const apiRows =
    reference.apiRows + reference.partsRows + reference.statesRows + reference.stylingRows;

  console.log("── build-data summary ────────────────────────────────");
  console.log(
    `reference.json       ${reference.entries} components, ${apiRows} api/parts/states/styling rows`,
  );
  console.log(
    `catalog.json         ${Object.keys(catalog.components).length} components ` +
      `in ${catalog.categories.length} categories`,
  );
  console.log(`rust-examples.json   ${rust.pages} components, ${rust.snippets} snippets`);
  console.log(
    `public/shots         ${shots.count} screenshots (${(shots.bytes / (1024 * 1024)).toFixed(1)} MB)`,
  );
  console.log("──────────────────────────────────────────────────────");
  return { reference, catalog, rust, shots };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run();
}
