// Extract the repository's upgrade guides (docs/migration-*.md) into
// web/src/data/migrations.json for the /docs/migration pages.
//
// The guides are the source rather than a hand-copied page, the way
// extract-releases.mjs treats CHANGELOG.md: extraction and `--check` are a
// pure function of the checkout and need no network. Each file becomes one
// guide: `version` is the minor version in the file name ("0.15"), `title`
// the first `# ` heading, `slug` the route segment with the dot replaced
// ("0-15"), and `body` the markdown after the title.
//
// The body's links are rewritten for the site:
//   - `../CHANGELOG.md`                  -> /docs/releases;
//   - `../CHANGELOG.md#0110---2026-09-22` -> /docs/releases#v0.11.0 — the
//     fragment's leading version digits pick the release whose tag matches
//     (the releases page ids its headings by tag); anything that does not
//     resolve to exactly one release throws;
//   - `migration-0.13.md` (+ anchor)     -> /docs/migration/0-13 (+ anchor);
//   - any other relative repo link       -> an absolute GitHub blob URL on
//     `master`, resolved from the guide's own directory (`docs/`), the same
//     base the site's source links use;
//   - in-page `#anchor` links are kept, and links inside fenced code blocks
//     are never touched.
//
// Usage:
//   node scripts/extract-migrations.mjs          # write migrations.json
//   node scripts/extract-migrations.mjs --check  # exit 1 when it is stale

import { readFileSync, readdirSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const DOCS_DIR = resolve(repoRoot, "docs");
const OUT = resolve(webRoot, "src", "data", "migrations.json");

const REPOSITORY = "https://github.com/Porabuild/HeroGPUI";
const FILE_NAME = /^migration-(\d+\.\d+)\.md$/;
const H1 = /^# (.+)$/;
const MARKDOWN_LINK = /(\[[^\]\n]*\]\()([^)\n]+)(\))/g;

/// A heading's in-page anchor, GitHub-style: lowercase, punctuation stripped
/// except `-` and `_`, every space a hyphen (no collapsing — GitHub emits one
/// hyphen per space, so "a / b" anchors as "a--b").
export function slugifyHeading(text) {
  return text
    .toLowerCase()
    .replace(/[^\w- ]/g, "")
    .replaceAll(" ", "-");
}

/// Split a link target into its path and its `#anchor` ("" when absent).
function splitAnchor(target) {
  const at = target.indexOf("#");
  return at < 0 ? [target, ""] : [target.slice(0, at), target.slice(at)];
}

/// Resolve a repo link written from a guide in `docs/` to its
/// repository-root-relative path (`../CHANGELOG.md` -> `CHANGELOG.md`).
function resolveRepoPath(href) {
  const parts = [];
  for (const part of `docs/${href}`.split("/")) {
    if (part === "" || part === ".") continue;
    if (part === "..") parts.pop();
    else parts.push(part);
  }
  return parts.join("/");
}

/// The releases page renders `<h2 id={release.tag}>` per release
/// (`releases/page.tsx`), so a CHANGELOG anchor must map to the tag, not stay
/// verbatim: a CHANGELOG heading anchors its release as the version's digits
/// (`0110---2026-09-22`), and `v0.11.0` is the tag with those digits. Tags are
/// keyed by digits once, on first use; two tags reducing to the same digits
/// (`0.1.10` / `0.11.0`) would make the mapping ambiguous, so that shape is
/// refused here rather than silently picking one.
const RELEASES_PATH = resolve(webRoot, "src", "data", "releases.json");
let releasesByDigits = null;

function releaseTagForChangelogFragment(fragment) {
  if (releasesByDigits === null) {
    releasesByDigits = new Map();
    const { releases } = JSON.parse(readFileSync(RELEASES_PATH, "utf8"));
    for (const release of releases) {
      const digits = release.tag.replace(/^v/, "").replaceAll(".", "");
      releasesByDigits.set(digits, [...(releasesByDigits.get(digits) ?? []), release.tag]);
    }
  }
  const digits = fragment.match(/^\d+/)?.[0];
  const tags = digits === undefined ? undefined : releasesByDigits.get(digits);
  if (digits === undefined || !tags || tags.length !== 1) {
    const why =
      digits === undefined
        ? "the fragment does not start with a version's digits"
        : tags
          ? `it matches several releases (${tags.join(", ")})`
          : `no releases.json release has version digits "${digits}"`;
    throw new Error(`CHANGELOG anchor "#${fragment}" does not resolve to one release: ${why}`);
  }
  return tags[0];
}

/// The site URL for one markdown link target, or null to keep it verbatim
/// (absolute http(s), protocol-relative, mailto, site-absolute and in-page
/// `#anchor` links are all kept as the renderer already handles them).
export function rewriteLinkTarget(target) {
  if (/^(?:https?:)?\/\//i.test(target) || /^(?:mailto|tel):/i.test(target)) return null;
  if (target.startsWith("#") || target.startsWith("/")) return null;
  const [path, anchor] = splitAnchor(target);
  const migration = path.match(FILE_NAME);
  if (migration) {
    return `/docs/migration/${migration[1].replaceAll(".", "-")}${anchor}`;
  }
  const repoPath = resolveRepoPath(path);
  if (repoPath === "CHANGELOG.md") {
    if (anchor === "") return "/docs/releases";
    return `/docs/releases#${releaseTagForChangelogFragment(anchor.slice(1))}`;
  }
  return `${REPOSITORY}/blob/master/${repoPath}${anchor}`;
}

/// Rewrite the markdown links of one guide body, outside fenced code blocks.
export function rewriteLinks(body) {
  let inFence = false;
  return body
    .replace(/\r\n?/g, "\n")
    .split("\n")
    .map((line) => {
      if (line.trimStart().startsWith("```")) {
        inFence = !inFence;
        return line;
      }
      if (inFence) return line;
      return line.replace(MARKDOWN_LINK, (all, head, target, tail) => {
        const rewritten = rewriteLinkTarget(target.trim());
        return rewritten === null ? all : `${head}${rewritten}${tail}`;
      });
    })
    .join("\n");
}

/// One migration-<version>.md file's text -> { version, slug, title, body }.
/// The title is the first `# ` heading; the body is the markdown after it,
/// with links rewritten for the site.
export function parseGuide(fileName, text) {
  const name = fileName.match(FILE_NAME);
  if (!name) throw new Error(`not a migration guide file name: ${fileName}`);
  const lines = text.replace(/\r\n?/g, "\n").split("\n");
  const titleAt = lines.findIndex((line) => H1.test(line));
  if (titleAt < 0) throw new Error(`${fileName} has no "# " title heading`);
  const version = name[1];
  return {
    version,
    slug: version.replaceAll(".", "-"),
    title: lines[titleAt].match(H1)[1].trim(),
    body: rewriteLinks(
      lines
        .slice(titleAt + 1)
        .join("\n")
        .trim(),
    ),
  };
}

/// Newest version first: "0.15" before "0.13" before "0.11".
function newestFirst(a, b) {
  const [aMajor, aMinor] = a.split(".").map(Number);
  const [bMajor, bMinor] = b.split(".").map(Number);
  return bMajor - aMajor || bMinor - aMinor;
}

export function run({ check = false } = {}) {
  const names = readdirSync(DOCS_DIR).filter((name) => FILE_NAME.test(name));
  if (names.length === 0) throw new Error("docs/ has no migration-*.md guides");
  names.sort((a, b) => newestFirst(a.match(FILE_NAME)[1], b.match(FILE_NAME)[1]));
  const guides = names.map((name) =>
    parseGuide(name, readFileSync(resolve(DOCS_DIR, name), "utf8")),
  );
  const output =
    JSON.stringify({ source: "docs/migration-*.md", repository: REPOSITORY, guides }, null, 2) +
    "\n";
  if (check) {
    let current = "";
    try {
      current = readFileSync(OUT, "utf8");
    } catch {}
    if (current !== output) {
      console.error("ERROR: migrations.json is stale; run `pnpm run extract`");
      process.exitCode = 1;
    }
  } else {
    mkdirSync(dirname(OUT), { recursive: true });
    writeFileSync(OUT, output);
  }
  console.log(`migrations.json: ${guides.length} upgrade guides from docs/migration-*.md`);
  return { guides: guides.length };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
