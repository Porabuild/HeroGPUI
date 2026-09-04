// Lift static top-level `para(...)` copy into component section metadata.
// The wasm migration source predates the native gallery's description schema;
// run this before rebuilding the artifact so preview canvases contain controls,
// not documentation prose.

import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { parseInvocation, separateExampleDescription } from "./extract-rust-examples.mjs";

function rustString(value) {
  return JSON.stringify(value);
}

export function liftDescriptions(source) {
  const replacements = [];
  let index = 0;

  while ((index = source.indexOf("component_doc_page!", index)) !== -1) {
    const page = parseInvocation(source, index);
    if (!page) {
      index += "component_doc_page!".length;
      continue;
    }
    let searchFrom = index;
    for (const section of page.sections) {
      if (section.description) continue;
      const separated = separateExampleDescription(section.code);
      if (!separated.description) continue;
      // Block-bodied examples can contain a nested `col` after setup code;
      // their body span is not safe to rewrite mechanically.
      if (!/^col\s*\(/.test(section.code)) {
        throw new Error(
          `${page.title}/${section.heading} has prose inside a setup block; move it to the section description explicitly`,
        );
      }
      const bodyStart = source.indexOf(section.code, searchFrom);
      if (bodyStart === -1 || bodyStart >= page.end) {
        throw new Error(`could not locate ${page.title}/${section.heading}`);
      }
      const lineStart = source.lastIndexOf("\n", bodyStart) + 1;
      const indent = source.slice(lineStart, bodyStart);
      replacements.push({
        start: bodyStart,
        end: bodyStart + section.code.length,
        text: `${rustString(separated.description)},\n${indent}${separated.code}`,
      });
      searchFrom = bodyStart + section.code.length;
    }
    index = page.end;
  }

  let output = source;
  for (const replacement of replacements.toReversed()) {
    output = output.slice(0, replacement.start) + replacement.text + output.slice(replacement.end);
  }
  return { output, lifted: replacements.length };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const sourcePath = process.argv[2];
  if (!sourcePath) {
    throw new Error("usage: node scripts/lift-wasm-descriptions.mjs <components.rs>");
  }
  const source = readFileSync(sourcePath, "utf8");
  const { output, lifted } = liftDescriptions(source);
  if (lifted) writeFileSync(sourcePath, output);
  console.log(`lifted ${lifted} wasm example descriptions`);
}
