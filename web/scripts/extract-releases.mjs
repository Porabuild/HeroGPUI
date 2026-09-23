// Extract the release notes into web/src/data/releases.json for the
// /docs/releases page.
//
// Source: the repository's own CHANGELOG.md (Keep a Changelog format). Every
// dated `## [X.Y.Z] - YYYY-MM-DD` section becomes one release; `[Unreleased]`
// is skipped. The changelog is the source rather than the GitHub Releases API
// so the data is a pure function of the checkout: extraction and `--check`
// need no network, and CI cannot go flaky on a rate limit. Every GitHub
// release (v0.9.0 onward) is cut from a tag whose notes are this section.
//
// Wrapped list items are joined onto one line, because the page's small
// markdown renderer reads one list item per line.
//
// Usage:
//   node scripts/extract-releases.mjs          # write releases.json
//   node scripts/extract-releases.mjs --check  # exit 1 when it is stale

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const CHANGELOG = resolve(repoRoot, "CHANGELOG.md");
const OUT = resolve(webRoot, "src", "data", "releases.json");

const REPOSITORY = "Porabuild/HeroGPUI";
const SECTION = /^## \[(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\] - (\d{4}-\d{2}-\d{2})\s*$/;
const ITEM = /^(\s*)(?:[-*+]|\d+[.)])\s+/;

/// Join the wrapped continuation lines of a list item onto the item line.
export function unwrapListItems(lines) {
  const out = [];
  let inItem = false;
  for (const line of lines) {
    if (line.trim() === "") {
      inItem = false;
      out.push(line);
    } else if (ITEM.test(line)) {
      inItem = true;
      out.push(line);
    } else if (inItem && /^\s+\S/.test(line)) {
      out[out.length - 1] = `${out[out.length - 1].trimEnd()} ${line.trim()}`;
    } else {
      inItem = false;
      out.push(line);
    }
  }
  return out;
}

export function parseChangelog(text) {
  const lines = text.replace(/\r\n?/g, "\n").split("\n");
  const releases = [];
  let current = null;
  for (const line of lines) {
    if (line.startsWith("## ") || /^\[[^\]]+\]:\s/.test(line)) {
      if (current) releases.push(current);
      current = null;
      const match = line.match(SECTION);
      if (match) current = { version: match[1], date: match[2], lines: [] };
      continue;
    }
    if (current) current.lines.push(line);
  }
  if (current) releases.push(current);
  return releases.map(({ version, date, lines: body }) => ({
    tag: `v${version}`,
    name: `v${version}`,
    publishedAt: date,
    prerelease: version.includes("-"),
    url: `https://github.com/${REPOSITORY}/releases/tag/v${version}`,
    body: unwrapListItems(body).join("\n").trim(),
  }));
}

export function run({ check = false } = {}) {
  const releases = parseChangelog(readFileSync(CHANGELOG, "utf8"));
  const output =
    JSON.stringify({ source: "CHANGELOG.md", repository: REPOSITORY, releases }, null, 2) + "\n";
  if (check) {
    let current = "";
    try {
      current = readFileSync(OUT, "utf8");
    } catch {}
    if (current !== output) {
      console.error("ERROR: releases.json is stale; run `pnpm run extract`");
      process.exitCode = 1;
    }
  } else {
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, output);
  }
  console.log(`releases.json: ${releases.length} releases from CHANGELOG.md`);
  return { releases: releases.length };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
