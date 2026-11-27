import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

import {
  parseGuide,
  rewriteLinkTarget,
  rewriteLinks,
  slugifyHeading,
} from "./extract-migrations.mjs";

const DOCS = fileURLToPath(new URL("../../docs/", import.meta.url));

// The guide shape: one `# ` title, a toml fence, an h2, and the three link
// kinds the extractor rewrites plus the two it keeps.
const FIXTURE = `# Migrating from 0.12 to 0.13

The full list is in [\`CHANGELOG.md\`](../CHANGELOG.md#0130---2026-10-03).

## \`VirtualListScroll\` is \`#[non_exhaustive]\`

See [the 0.11 guide](migration-0.11.md#motion-api-lives-under-anim) and
[contributing](../CONTRIBUTING.md); in-page [anchors](#virtuallistscroll-is-non_exhaustive)
stay, as do [web](https://example.com/x) and [site](/docs/releases) links.

\`\`\`rust
// a fence must not be rewritten, even though it holds link punctuation
let url = "CHANGELOG.md](../CHANGELOG.md)";
let pairs: [(&str, u8); 2] = [("a", 1), ("b", 2)];
\`\`\`
`;

test("parseGuide takes version and slug from the file name and the title from the first # heading", () => {
  const guide = parseGuide("migration-0.13.md", FIXTURE);
  assert.equal(guide.version, "0.13");
  assert.equal(guide.slug, "0-13");
  assert.equal(guide.title, "Migrating from 0.12 to 0.13");
  assert.ok(!guide.body.includes("Migrating from"), "the body starts after the title");
  assert.ok(guide.body.startsWith("The full list"));
});

test("parseGuide rejects a non-guide file name and a guide without a title", () => {
  assert.throws(() => parseGuide("roadmap.md", "# Roadmap"), /not a migration guide/);
  assert.throws(() => parseGuide("migration-0.13.md", "no heading here"), /no "# " title/);
});

test("slugifyHeading reproduces the anchors the guides link to", () => {
  // Every expected anchor here is a link target found in the real guides.
  assert.equal(
    slugifyHeading("Rust 1.99 and GPUI `gpui-pre` 0.3.11"),
    "rust-199-and-gpui-gpui-pre-0311",
  );
  assert.equal(
    slugifyHeading("`is_command_palette_shortcut` takes `&App`"),
    "is_command_palette_shortcut-takes-app",
  );
  assert.equal(
    slugifyHeading("`extend::shift_wheel_scroll_x` takes `&ScrollEvent` and `&mut Window`"),
    "extendshift_wheel_scroll_x-takes-scrollevent-and-mut-window",
  );
  assert.equal(
    slugifyHeading("Checkbox and Switch labels must be `'static`"),
    "checkbox-and-switch-labels-must-be-static",
  );
  assert.equal(
    slugifyHeading("Calendar and RangeCalendar: the year picker is opt-in"),
    "calendar-and-rangecalendar-the-year-picker-is-opt-in",
  );
  assert.equal(
    slugifyHeading("`VirtualListScroll` is `#[non_exhaustive]`"),
    "virtuallistscroll-is-non_exhaustive",
  );
  assert.equal(
    slugifyHeading("`Modal::on_close` / `Drawer::on_close` receive `DismissReason`"),
    "modalon_close--draweron_close-receive-dismissreason",
    "one hyphen per space, no collapsing",
  );
});

test("rewriteLinkTarget rewrites the changelog, sibling guides and other repo links", () => {
  assert.equal(rewriteLinkTarget("../CHANGELOG.md"), "/docs/releases");
  assert.equal(
    rewriteLinkTarget("../CHANGELOG.md#0110---2026-09-22"),
    "/docs/releases#v0.11.0",
    "a CHANGELOG release anchor becomes the releases page's tag id",
  );
  assert.equal(rewriteLinkTarget("../CHANGELOG.md#0130---2026-10-03"), "/docs/releases#v0.13.0");
  assert.equal(rewriteLinkTarget("migration-0.13.md"), "/docs/migration/0-13");
  assert.equal(
    rewriteLinkTarget("migration-0.13.md#virtual-list"),
    "/docs/migration/0-13#virtual-list",
  );
  assert.equal(
    rewriteLinkTarget("../CONTRIBUTING.md"),
    "https://github.com/Porabuild/HeroGPUI/blob/master/CONTRIBUTING.md",
    "any other relative link becomes a master blob URL",
  );
});

test("rewriteLinkTarget throws on a CHANGELOG anchor no release matches", () => {
  assert.throws(
    () => rewriteLinkTarget("../CHANGELOG.md#9999---2026-01-01"),
    /no releases.json release has version digits "9999"/,
  );
  assert.throws(
    () => rewriteLinkTarget("../CHANGELOG.md#known-issues"),
    /does not start with a version's digits/,
  );
});

test("rewriteLinkTarget keeps in-page, absolute and site-absolute links verbatim", () => {
  assert.equal(rewriteLinkTarget("#rust-199-and-gpui-gpui-pre-0311"), null);
  assert.equal(rewriteLinkTarget("https://example.com/x"), null);
  assert.equal(rewriteLinkTarget("http://example.com"), null);
  assert.equal(rewriteLinkTarget("//example.com/x"), null);
  assert.equal(rewriteLinkTarget("mailto:hi@example.com"), null);
  assert.equal(rewriteLinkTarget("/docs/releases"), null);
});

test("rewriteLinks rewrites links outside fences and leaves fence contents alone", () => {
  const body = rewriteLinks(FIXTURE.split("\n").slice(1).join("\n"));
  assert.ok(body.includes("](/docs/releases#v0.13.0)"));
  assert.ok(body.includes("](/docs/migration/0-11#motion-api-lives-under-anim)"));
  assert.ok(body.includes("](https://github.com/Porabuild/HeroGPUI/blob/master/CONTRIBUTING.md)"));
  assert.ok(body.includes("](#virtuallistscroll-is-non_exhaustive)"), "in-page anchors stay");
  assert.ok(body.includes("](https://example.com/x)"));
  assert.ok(body.includes("](/docs/releases)"), "a site link stays verbatim");
  assert.ok(
    body.includes('"CHANGELOG.md](../CHANGELOG.md)"'),
    "the rust fence keeps its link-shaped string untouched",
  );
});

test("every in-page anchor a guide links to resolves to one of its headings", () => {
  const names = readdirSync(DOCS).filter((name) => /^migration-\d+\.\d+\.md$/.test(name));
  assert.ok(names.length > 0, "the repository ships migration guides");
  let linked = 0;
  for (const name of names) {
    const guide = parseGuide(name, readFileSync(`${DOCS}${name}`, "utf8"));
    const ids = new Set(
      [...guide.body.matchAll(/^#{1,4} (.+)$/gm)].map(([, text]) =>
        slugifyHeading(text.replace(/\s*#*\s*$/, "")),
      ),
    );
    const anchors = [...guide.body.matchAll(/\]\(#([^)\n]+)\)/g)].map((match) => match[1]);
    linked += anchors.length;
    const missing = [...new Set(anchors)].filter((anchor) => !ids.has(anchor));
    assert.deepEqual(
      missing,
      [],
      `${name}: anchors with no heading on the page (${ids.size} headings)`,
    );
    console.log(
      `${name}: ${anchors.length} in-page links, ${ids.size} heading anchors, all resolve`,
    );
  }
  assert.ok(linked > 0, "the guides do use in-page anchors");
});
