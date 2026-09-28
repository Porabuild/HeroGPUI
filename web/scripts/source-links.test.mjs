// Checks the component pages' docs.rs and GitHub source links
// (src/lib/source-links.ts) against the crate they point into: every
// catalog component resolves to a module that exists in
// crates/herogpui-components/src and a `pub struct` of the linked name, and
// DIRECTORY_MODULES matches the crate's directory modules.
//
//   node --test scripts/source-links.test.mjs

import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { componentSourceLinks, DIRECTORY_MODULES, REPOSITORY } from "../src/lib/source-links.ts";

const webRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(webRoot, "..");
const catalog = JSON.parse(readFileSync(join(webRoot, "src/data/catalog.json"), "utf8"));

function rustFiles(path) {
  if (!statSync(path).isDirectory()) return [path];
  return readdirSync(path)
    .filter((name) => name.endsWith(".rs"))
    .map((name) => join(path, name));
}

test("every component page links a module and struct that exist", () => {
  const slugs = Object.keys(catalog.components);
  assert.ok(slugs.length > 0, "catalog.json lists no components");
  for (const slug of slugs) {
    const component = catalog.components[slug];
    const links = componentSourceLinks({
      slug,
      title: component.title,
      importLine: component.importLine,
      version: catalog.version,
    });
    assert.ok(
      links,
      `${slug}: no source links derived from ${JSON.stringify(component.importLine)}`,
    );
    const path = join(repoRoot, links.sourcePath);
    assert.ok(existsSync(path), `${slug}: ${links.sourcePath} does not exist`);
    const declares = rustFiles(path).some((file) =>
      new RegExp(`\\bpub struct ${links.type}\\b`).test(readFileSync(file, "utf8")),
    );
    assert.ok(declares, `${slug}: no \`pub struct ${links.type}\` in ${links.sourcePath}`);
    assert.ok(links.github.startsWith(`${REPOSITORY}/`), links.github);
    assert.ok(links.github.includes(`/v${catalog.version}/`), links.github);
  }
});

test("DIRECTORY_MODULES matches the crate's directory modules", () => {
  const src = join(repoRoot, "crates/herogpui-components/src");
  const directories = readdirSync(src).filter(
    (name) => statSync(join(src, name)).isDirectory() && existsSync(join(src, name, "mod.rs")),
  );
  assert.deepEqual([...DIRECTORY_MODULES].sort(), directories.sort());
});
