// Extract the `## HeroGPUI extensions` section of the repository's llms.txt
// into web/src/data/extensions.json for the /docs/extensions page.
//
// The section is the single source of truth for the HeroGPUI APIs that have no
// HeroUI counterpart. It is deliberately kept out of `reference_metadata`, so
// this extractor reads the published llms.txt directly: the page and the
// public API description cannot drift apart while `--check` gates both.
//
// The bodies stay markdown and render through the release notes renderer
// (`src/app/docs/releases/markdown.tsx`), which reads one list item per line —
// so bullet continuation lines are kept as their own dedented lines rather
// than joined, and nested `  - ` sub-bullets stay markdown list items.
//
// Each entry also records `live`: the component pages that demonstrate the
// extension in a generated example, anchored with the same scheme
// `buildExampleSections` uses on the component pages, so a link lands on the
// exact example heading. An entry matches examples by Rust type identifier;
// an entry with no identifier ("Table data extras") whose first word is a
// component's catalog title matches that component's "HeroGPUI extension"
// examples, minus the examples an identifier entry already claimed.
//
// The JSON also carries `componentExtensions`: every extension-marked example
// no entry's `live` list claims, each with a one-sentence `summary` cut from
// its description — the page's "In component pages" section.
//
// Usage:
//   node scripts/extract-extensions.mjs          # write extensions.json
//   node scripts/extract-extensions.mjs --check  # exit 1 when it is stale

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const LLMSTXT = resolve(repoRoot, "llms.txt");
const CATALOG = resolve(webRoot, "src", "data", "catalog.json");
const EXAMPLES = resolve(webRoot, "src", "data", "rust-examples.json");
const OUT = resolve(webRoot, "src", "data", "extensions.json");

const SECTION_HEADING = "## HeroGPUI extensions";
const BULLET = /^- \*\*(.+)/;
/// A name part is a Rust type identifier only when it is a single PascalCase
/// word: "TitleBar / WindowBorder" yields two, "Toolbar extras" none.
const TYPE_IDENTIFIER = /^[A-Z][A-Za-z]+$/;
const MAX_LIVE = 6;
/// An example description that marks a HeroGPUI-only example, the marker the
/// identifier-less entries' third live rule links ("HeroGPUI extension, not a
/// HeroUI v3 example …").
const EXTENSION_DESCRIPTION = /^HeroGPUI extension/;

/// The page-anchor slug for a name: kebab-case — a dash at every
/// lowercase-or-digit -> uppercase boundary splits camelCase before the whole
/// name is lowercased, every remaining non-alphanumeric run becomes one dash,
/// and dashes at the ends are trimmed ("VirtualList" -> "virtual-list",
/// "TitleBar / WindowBorder" -> "title-bar-window-border", "i18n" -> "i18n").
export function slugifyName(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/// Split the section body into the intro markdown and the raw line groups of
/// the top-level bullets. Throws when the section is missing — the repository
/// contract guarantees it, and a silent empty output would gut the page.
export function splitSection(text) {
  const lines = text.replace(/\r\n?/g, "\n").split("\n");
  const start = lines.findIndex((line) => line.trimEnd() === SECTION_HEADING);
  if (start < 0) throw new Error(`llms.txt has no "${SECTION_HEADING}" section`);
  let end = lines.length;
  for (let i = start + 1; i < lines.length; i += 1) {
    if (lines[i].startsWith("## ")) {
      end = i;
      break;
    }
  }
  const intro = [];
  const bullets = [];
  let current = null;
  for (const line of lines.slice(start + 1, end)) {
    if (BULLET.test(line)) {
      current = [line];
      bullets.push(current);
    } else if (current) {
      current.push(line);
    } else {
      intro.push(line);
    }
  }
  if (bullets.length === 0) throw new Error(`the "${SECTION_HEADING}" section lists no extensions`);
  return { intro: intro.join("\n").trim(), bullets };
}

/// Drop the bullet's two-space indent from a continuation line. Nested
/// `  - ` sub-bullets surface as top-level markdown list items; deeper
/// indents keep their remainder.
function dedent(line) {
  if (line.trim() === "") return "";
  return line.startsWith("  ") ? line.slice(2) : line.replace(/^\s+/, "");
}

/// The first index of `needle` in `line` at or after `from` that sits outside
/// backtick code spans (backtick parity tracked across the whole line), -1
/// when there is none. A name may hold `` `**` `` and a qualifier a
/// `` `a — b` `` code span, so neither the closing `**` nor the ` — `
/// separator may be found inside one.
function indexOfOutsideCode(line, needle, from = 0) {
  let inCode = false;
  for (let i = 0; i < line.length; i += 1) {
    if (line[i] === "`") inCode = !inCode;
    else if (i >= from && !inCode && line.startsWith(needle, i)) return i;
  }
  return -1;
}

/// The ` — ` head/body separator in the text after the name: an em dash with
/// whitespace before it and whitespace or end-of-line after, outside code
/// spans. A dash inside a `` `a — b` `` code span does not split the entry.
function emDashIndex(rest) {
  let from = 0;
  for (;;) {
    const at = indexOfOutsideCode(rest, "—", from);
    if (at < 0) return -1;
    if (at > 0 && /\s/.test(rest[at - 1]) && (at + 1 === rest.length || /\s/.test(rest[at + 1]))) {
      return at;
    }
    from = at + 1;
  }
}

/// One top-level bullet group -> { name, qualifier, body }. `qualifier` is the
/// text between the closing `**` and the ` — ` em dash (may be empty, keeps
/// its backticks); `body` is the markdown after it, continuations dedented.
/// The em dash may also close the head line itself, with the body entirely on
/// the continuation lines. Both the closing `**` and the em dash are searched
/// outside backtick code spans.
export function parseEntry(lines) {
  const head = lines[0];
  if (!BULLET.test(head)) throw new Error(`not an extension bullet: ${head}`);
  const close = indexOfOutsideCode(head, "**", 5);
  if (close < 0) throw new Error(`not an extension bullet: ${head}`);
  const name = head.slice(4, close);
  const rest = head.slice(close + 2);
  const dash = emDashIndex(rest);
  const qualifier = (dash < 0 ? rest : rest.slice(0, dash)).trim();
  const body = [dash < 0 ? "" : rest.slice(dash + 1), ...lines.slice(1).map(dedent)]
    .join("\n")
    .trim();
  return { name, qualifier, body };
}

/// The Rust type identifiers a name names: its ` / `-separated parts that are
/// single PascalCase words.
export function typeIdentifiers(name) {
  return name.split(" / ").filter((part) => TYPE_IDENTIFIER.test(part));
}

/// True when the example demonstrates the identifier: its code calls
/// `${Ident}::`, or its imports name `${Ident}` as a whole word.
export function exampleMatches(identifier, example) {
  const word = new RegExp(`(^|[^A-Za-z0-9_])${identifier}([^A-Za-z0-9_]|$)`);
  return (example.code ?? "").includes(`${identifier}::`) || word.test(example.imports ?? "");
}

/// Also a match when the example's heading names the identifier: with every
/// non-alphanumeric removed it equals the identifier case-insensitively
/// ("Command Palette" -> "commandpalette" matches CommandPalette).
export function headingMatches(identifier, heading) {
  return (heading ?? "").replace(/[^A-Za-z0-9]/g, "").toLowerCase() === identifier.toLowerCase();
}

function normalizeHeading(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

/// The example anchors for one component page, in example order — the exact
/// ids `buildExampleSections` (src/app/docs/components/[slug]/examples.ts)
/// assigns, including the `-2`/`-3` suffixes repeated headings earn.
export function exampleAnchors(headings) {
  const used = new Set();
  return headings.map((heading, index) => {
    const base = normalizeHeading(heading).replace(/\s+/g, "-") || `example-${index + 1}`;
    let id = `rust-${base}`;
    let counter = 2;
    while (used.has(id)) {
      id = `rust-${base}-${counter}`;
      counter += 1;
    }
    used.add(id);
    return id;
  });
}

function componentOrder(catalog) {
  const order = new Map();
  for (const category of catalog.categories ?? []) {
    for (const slug of category.components ?? []) {
      if (!order.has(slug)) order.set(slug, order.size);
    }
  }
  return order;
}

function componentTitles(catalog) {
  const titles = new Map();
  for (const [slug, component] of Object.entries(catalog.components ?? {})) {
    if (typeof component?.title === "string" && component.title !== "")
      titles.set(slug, component.title);
  }
  return titles;
}

function componentSlugsByTitle(catalog) {
  const byTitle = new Map();
  for (const [slug, component] of Object.entries(catalog.components ?? {})) {
    if (typeof component?.title !== "string" || component.title === "") continue;
    const slugs = byTitle.get(component.title) ?? [];
    slugs.push(slug);
    byTitle.set(component.title, slugs);
  }
  return byTitle;
}

/// Sort the matched examples by the component's catalog order, then example
/// order, and cap at MAX_LIVE so one entry cannot flood the page; project to
/// the link shape the page and JSON carry.
function sortedLive(matches, catalog) {
  const order = componentOrder(catalog);
  matches.sort(
    (a, b) =>
      (order.get(a.component) ?? Number.POSITIVE_INFINITY) -
        (order.get(b.component) ?? Number.POSITIVE_INFINITY) || a.index - b.index,
  );
  return matches.slice(0, MAX_LIVE).map(({ component, title, heading, anchor }) => ({
    component,
    title,
    heading,
    anchor,
  }));
}

/// Where an extension is demonstrated live: the generated examples whose code
/// or imports name one of the extension's type identifiers, or whose heading
/// names one (an example matching both ways counts once).
export function liveForExtension(name, examplesBySlug, catalog) {
  const identifiers = typeIdentifiers(name);
  if (identifiers.length === 0) return [];
  const titles = componentTitles(catalog);
  const matches = [];
  for (const [slug, examples] of Object.entries(examplesBySlug)) {
    const title = titles.get(slug);
    if (title === undefined) continue;
    const anchors = exampleAnchors(examples.map((example) => example.heading ?? ""));
    examples.forEach((example, index) => {
      const matchesExtension = identifiers.some(
        (identifier) =>
          exampleMatches(identifier, example) || headingMatches(identifier, example.heading ?? ""),
      );
      if (!matchesExtension) return;
      matches.push({
        component: slug,
        title,
        heading: example.heading ?? "",
        anchor: anchors[index],
        index,
      });
    });
  }
  return sortedLive(matches, catalog);
}

/// The third live rule, for an entry with no type identifier whose first word
/// is a component's catalog title ("Table data extras" -> Table): that
/// component's examples whose description starts with "HeroGPUI extension",
/// minus the ones `claimed` — the component/anchor pairs another entry's
/// identifier or heading rule already linked (the toolbar's "Title Bar"
/// example belongs to "TitleBar / WindowBorder", not "Toolbar extras").
export function liveByExtensionDescription(name, examplesBySlug, catalog, claimed) {
  const firstWord = name.split(/\s+/)[0] ?? "";
  const slugs = componentSlugsByTitle(catalog).get(firstWord) ?? [];
  if (slugs.length === 0) return [];
  const titles = componentTitles(catalog);
  const matches = [];
  for (const slug of slugs) {
    const examples = examplesBySlug[slug] ?? [];
    const anchors = exampleAnchors(examples.map((example) => example.heading ?? ""));
    examples.forEach((example, index) => {
      if (!EXTENSION_DESCRIPTION.test(example.description ?? "")) return;
      const anchor = anchors[index];
      if (claimed.has(`${slug}\u0000${anchor}`)) return;
      matches.push({
        component: slug,
        title: titles.get(slug) ?? "",
        heading: example.heading ?? "",
        anchor,
        index,
      });
    });
  }
  return sortedLive(matches, catalog);
}

/// The live links for every entry name, in order. Pass one applies the
/// identifier/heading rule and collects what it claimed; pass two gives the
/// identifier-less entries the description rule against that claim set.
export function liveForAllExtensions(names, examplesBySlug, catalog) {
  const live = Array.from({ length: names.length });
  const claimed = new Set();
  names.forEach((name, index) => {
    const links =
      typeIdentifiers(name).length > 0 ? liveForExtension(name, examplesBySlug, catalog) : [];
    live[index] = links;
    for (const link of links) claimed.add(`${link.component}\u0000${link.anchor}`);
  });
  names.forEach((name, index) => {
    if (typeIdentifiers(name).length === 0) {
      live[index] = liveByExtensionDescription(name, examplesBySlug, catalog, claimed);
    }
  });
  return live;
}

/// The text up to and including its first '. ' outside backtick code spans
/// ("`x()` does a thing. Then more" -> "`x()` does a thing."), or the whole
/// text when none follows — the last sentence of a description ends the
/// string, with no trailing space to find.
export function firstSentence(text) {
  const at = indexOfOutsideCode(text, ". ", 1);
  return at < 0 ? text : text.slice(0, at + 1);
}

/// The one-sentence summary for an extension-marked example description: the
/// text after the description's first ': ' — both marker spellings end there,
/// "HeroGPUI extension, not a HeroUI v3 example" and its
/// "(after gpui-kit's …)" variant — cut to its first sentence. Backticks
/// stay, so the page's markdown renderer shows the code spans.
export function extensionSummary(description) {
  const text = description ?? "";
  const at = text.indexOf(": ");
  return at < 0 ? "" : firstSentence(text.slice(at + 2));
}

/// The `componentExtensions` items: every example whose description starts
/// with the "HeroGPUI extension" marker and that no entry's `live` list
/// already claims (`claimed` holds `component\0anchor` pairs, exactly what
/// `liveForAllExtensions` linked). Sorted by catalog component order, then
/// example order, like the live lists; uncapped, so a new component-page
/// extension example always lands on the page.
export function componentExtensions(examplesBySlug, catalog, claimed) {
  const order = componentOrder(catalog);
  const titles = componentTitles(catalog);
  const matches = [];
  for (const [slug, examples] of Object.entries(examplesBySlug)) {
    const title = titles.get(slug);
    if (title === undefined) continue;
    const anchors = exampleAnchors(examples.map((example) => example.heading ?? ""));
    examples.forEach((example, index) => {
      if (!EXTENSION_DESCRIPTION.test(example.description ?? "")) return;
      const anchor = anchors[index];
      if (claimed.has(`${slug}\u0000${anchor}`)) return;
      matches.push({
        component: slug,
        title,
        heading: example.heading ?? "",
        anchor,
        index,
        summary: extensionSummary(example.description ?? ""),
      });
    });
  }
  matches.sort(
    (a, b) =>
      (order.get(a.component) ?? Number.POSITIVE_INFINITY) -
        (order.get(b.component) ?? Number.POSITIVE_INFINITY) || a.index - b.index,
  );
  return matches.map(({ component, title, heading, anchor, summary }) => ({
    component,
    title,
    heading,
    anchor,
    summary,
  }));
}

export function run({ check = false } = {}) {
  const { intro, bullets } = splitSection(readFileSync(LLMSTXT, "utf8"));
  const catalog = JSON.parse(readFileSync(CATALOG, "utf8"));
  const examplesBySlug = JSON.parse(readFileSync(EXAMPLES, "utf8"));

  const parsed = bullets.map(parseEntry);
  const live = liveForAllExtensions(
    parsed.map((entry) => entry.name),
    examplesBySlug,
    catalog,
  );
  const entries = parsed.map((entry, index) => ({
    name: entry.name,
    qualifier: entry.qualifier,
    slug: slugifyName(entry.name),
    body: entry.body,
    live: live[index],
  }));

  const seen = new Set();
  for (const entry of entries) {
    if (seen.has(entry.slug)) throw new Error(`duplicate extension slug: ${entry.slug}`);
    seen.add(entry.slug);
  }

  // The live lists are the claim set: an example one of them links is the
  // entry's demonstration, and `componentExtensions` takes only the rest.
  const claimed = new Set();
  for (const entry of entries) {
    for (const link of entry.live) claimed.add(`${link.component}\u0000${link.anchor}`);
  }
  const extras = componentExtensions(examplesBySlug, catalog, claimed);

  const output =
    JSON.stringify({ source: "llms.txt", intro, entries, componentExtensions: extras }, null, 2) +
    "\n";
  if (check) {
    let current = "";
    try {
      current = readFileSync(OUT, "utf8");
    } catch {}
    if (current !== output) {
      console.error("ERROR: extensions.json is stale; run `pnpm run extract`");
      process.exitCode = 1;
    }
  } else {
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, output);
  }
  const liveCount = entries.reduce((sum, entry) => sum + entry.live.length, 0);
  console.log(
    `extensions.json: ${entries.length} extensions, ${liveCount} live links, ${extras.length} component examples from llms.txt`,
  );
  return { extensions: entries.length, live: liveCount, componentExtensions: extras.length };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
