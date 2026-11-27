import assert from "node:assert/strict";
import test from "node:test";

import {
  componentExtensions,
  exampleAnchors,
  exampleMatches,
  extensionSummary,
  headingMatches,
  liveForAllExtensions,
  liveForExtension,
  parseEntry,
  slugifyName,
  splitSection,
  typeIdentifiers,
} from "./extract-extensions.mjs";
import { buildExampleSections } from "../src/app/docs/components/[slug]/examples.ts";

// The same shapes the llms.txt section uses: a qualifier bullet with wrapped
// continuation lines, then a bullet with a nested sub-bullet.
const FIXTURE = `## HeroGPUI extensions

Not HeroUI APIs; each one is an extension in its rustdoc.

- **VirtualList** (\`test-support\` feature) — \`VirtualList::new(id, &handle)\`,
  over a caller-owned handle, which
  picks the mode.
  - \`uniform(count)\` lays every row out
  - \`remeasure()\` re-runs it
- **TitleBar / WindowBorder** — open the window with
  \`TitleBar::window_options()\`.
- **Theme hot reload** (\`watch\` feature, implies \`serde\`; native only) —
  \`watch_themes_dir(dir, |reload, cx| .., cx)\` polls \`dir\`.
`;

test("splitSection isolates the intro and the top-level bullet groups", () => {
  const { intro, bullets } = splitSection(FIXTURE);
  assert.equal(intro, "Not HeroUI APIs; each one is an extension in its rustdoc.");
  assert.equal(bullets.length, 3);
  assert.match(bullets[0][0], /^- \*\*VirtualList\*\*/);
  assert.match(bullets[1][0], /^- \*\*TitleBar \/ WindowBorder\*\*/);
});

test("parseEntry keeps the qualifier backticks and dedents the body", () => {
  const { intro, bullets } = splitSection(FIXTURE);
  const first = parseEntry(bullets[0]);
  assert.equal(first.name, "VirtualList");
  assert.equal(first.qualifier, "(`test-support` feature)");
  assert.equal(
    first.body,
    "`VirtualList::new(id, &handle)`,\nover a caller-owned handle, which\npicks the mode.\n- `uniform(count)` lays every row out\n- `remeasure()` re-runs it",
  );
  const second = parseEntry(bullets[1]);
  assert.equal(second.name, "TitleBar / WindowBorder");
  assert.equal(second.qualifier, "");
  assert.equal(second.body, "open the window with\n`TitleBar::window_options()`.");
  // The em dash may close the head line, with the body on the continuations.
  const third = parseEntry(bullets[2]);
  assert.equal(third.name, "Theme hot reload");
  assert.equal(third.qualifier, "(`watch` feature, implies `serde`; native only)");
  assert.equal(third.body, "`watch_themes_dir(dir, |reload, cx| .., cx)` polls `dir`.");
  assert.ok(intro.length > 0);
});

test("parseEntry finds the closing ** and the em dash outside code spans", () => {
  // The qualifier's `a — b` code span holds an em dash of its own; the real
  // separator is the one outside the backticks.
  const dashed = parseEntry(["- **Sorting** (`a — b` keys) — the real body", "second line"]);
  assert.equal(dashed.name, "Sorting");
  assert.equal(dashed.qualifier, "(`a — b` keys)");
  assert.equal(dashed.body, "the real body\nsecond line");

  // The name's code span holds `**`; the closing bold is the one outside it.
  const starred = parseEntry(["- **Uses `**` in markdown** — body"]);
  assert.equal(starred.name, "Uses `**` in markdown");
  assert.equal(starred.qualifier, "");
  assert.equal(starred.body, "body");

  // No closing ** outside code spans, no entry.
  assert.throws(() => parseEntry(["- **Unfinished `**` name — body"]), /not an extension bullet/);
});

test("slugifyName splits camelCase into kebab-case and dashes non-alphanumeric runs", () => {
  assert.equal(slugifyName("VirtualList"), "virtual-list");
  assert.equal(slugifyName("ContextMenu"), "context-menu");
  assert.equal(slugifyName("ResizablePanelGroup"), "resizable-panel-group");
  assert.equal(slugifyName("TreeView"), "tree-view");
  assert.equal(slugifyName("TitleBar / WindowBorder"), "title-bar-window-border");
  assert.equal(slugifyName("CommandPalette"), "command-palette");
  assert.equal(slugifyName("HoverCard"), "hover-card");
  assert.equal(slugifyName("MenuBar"), "menu-bar");
  assert.equal(slugifyName("i18n"), "i18n", "a lowercase/digit run has no boundary to split");
  assert.equal(slugifyName("Shared builder traits"), "shared-builder-traits");
  assert.equal(slugifyName("Theme hot reload"), "theme-hot-reload");
});

test("typeIdentifiers keeps only PascalCase name parts", () => {
  assert.deepEqual(typeIdentifiers("TitleBar / WindowBorder"), ["TitleBar", "WindowBorder"]);
  assert.deepEqual(typeIdentifiers("Toolbar extras"), []);
  assert.deepEqual(typeIdentifiers("i18n"), []);
  assert.deepEqual(typeIdentifiers("HoverCard"), ["HoverCard"]);
});

test("exampleMatches requires Ident:: in code or the whole word in imports", () => {
  assert.ok(exampleMatches("VirtualList", { code: 'VirtualList::new("list")' }));
  assert.ok(
    exampleMatches("VirtualList", { code: "", imports: "use herogpui::prelude::VirtualList;" }),
  );
  assert.ok(
    exampleMatches("VirtualList", { code: "not_a_VirtualList::new()", imports: "" }),
    "the code check is a plain `Ident::` substring, as specified",
  );
  assert.ok(
    !exampleMatches("VirtualList", { code: "// VirtualList mentioned in prose", imports: "" }),
    "code must call the type, not merely mention it",
  );
  assert.ok(
    !exampleMatches("VirtualList", {
      code: "",
      imports: "use herogpui::prelude::VirtualListDerive;",
    }),
    "an identifier nested inside a larger import name is not a match",
  );
});

test("headingMatches strips non-alphanumerics and compares case-insensitively", () => {
  assert.ok(headingMatches("CommandPalette", "Command Palette"));
  assert.ok(headingMatches("VirtualList", "virtual list!"));
  assert.ok(!headingMatches("HoverCard", "Hover Cards"));
  assert.ok(!headingMatches("HoverCard", "Card"), "the whole heading must name the identifier");
  assert.ok(!headingMatches("HoverCard", ""), "an empty heading names nothing");
});

test("exampleAnchors reproduces buildExampleSections, including -2/-3 dedupes", () => {
  const headings = ["Usage", "Sidebar", "Usage", "Sidebar", "Icon-Only Toolbar Segments"];
  const sections = buildExampleSections(headings.map((heading) => ({ heading, code: "" })));
  assert.deepEqual(
    exampleAnchors(headings),
    sections.map((section) => section.id),
  );
  assert.deepEqual(exampleAnchors(headings), [
    "rust-usage",
    "rust-sidebar",
    "rust-usage-2",
    "rust-sidebar-2",
    "rust-icon-only-toolbar-segments",
  ]);
});

test("liveForExtension orders by catalog order, caps at six, dedupes anchors", () => {
  const catalog = {
    categories: [
      { name: "Collections", slug: "collections", components: ["dropdown"] },
      { name: "Overlays", slug: "overlays", components: ["modal", "tooltip"] },
    ],
    components: {
      dropdown: { slug: "dropdown", title: "Dropdown" },
      modal: { slug: "modal", title: "Modal" },
      tooltip: { slug: "tooltip", title: "Tooltip" },
    },
  };
  const examplesBySlug = {
    // Later in the object than tooltip, but dropdown comes first in the
    // catalog, so its matches must sort first.
    tooltip: [
      { heading: "Hover Card", code: 'HoverCard::new("hover")', imports: "" },
      { heading: "Hover Card", code: 'HoverCard::new("hover-2")', imports: "" },
    ],
    dropdown: [
      { heading: "Context Menu", code: "", imports: "use herogpui::prelude::ContextMenu;" },
    ],
    // The heading rule alone: "Command Palette" names CommandPalette although
    // neither code nor imports mention it (the modal page's real case).
    modal: [{ heading: "Command Palette", code: 'Button::new("open")', imports: "" }],
    unlisted: [{ heading: "Hover Card", code: 'HoverCard::new("x")', imports: "" }],
  };
  const live = liveForExtension("HoverCard", examplesBySlug, catalog);
  // "unlisted" has no catalog entry, so it contributes nothing even though its
  // code matches, and a tooltip example matches by both code and heading but
  // yields one link each.
  assert.deepEqual(live, [
    { component: "tooltip", title: "Tooltip", heading: "Hover Card", anchor: "rust-hover-card" },
    { component: "tooltip", title: "Tooltip", heading: "Hover Card", anchor: "rust-hover-card-2" },
  ]);
  const palette = liveForExtension("CommandPalette", examplesBySlug, catalog);
  assert.deepEqual(palette, [
    {
      component: "modal",
      title: "Modal",
      heading: "Command Palette",
      anchor: "rust-command-palette",
    },
  ]);

  // The cap: eight matching, catalog-listed components yield six links.
  const eight = Array.from({ length: 8 }, (_, index) => `demo-${index}`);
  const many = {
    categories: [{ name: "All", slug: "all", components: eight }],
    components: Object.fromEntries(eight.map((slug) => [slug, { slug, title: `Demo ${slug}` }])),
  };
  const examples = Object.fromEntries(
    eight.map((slug) => [
      slug,
      [{ heading: "Hover Card", code: `HoverCard::new("${slug}")`, imports: "" }],
    ]),
  );
  assert.equal(liveForExtension("HoverCard", examples, many).length, 6);
});

test("liveForAllExtensions links title-named entries to extension examples, minus claimed ones", () => {
  const catalog = {
    categories: [{ name: "Data", slug: "data", components: ["table", "toolbar"] }],
    components: {
      table: { slug: "table", title: "Table" },
      toolbar: { slug: "toolbar", title: "Toolbar" },
    },
  };
  const examplesBySlug = {
    toolbar: [
      { heading: "Usage", code: "", imports: "", description: "" },
      {
        heading: "Sized Controls",
        code: "",
        imports: "",
        description: "HeroGPUI extension, not a HeroUI v3 example (after gpui-kit's)",
      },
      {
        heading: "Title Bar",
        code: "TitleBar::window_options()",
        imports: "",
        description: "HeroGPUI extension, not a HeroUI v3 example",
      },
    ],
    table: [
      {
        heading: "Column Reordering",
        code: "",
        imports: "",
        description: "HeroGPUI extension, not a HeroUI v3 example",
      },
      { heading: "Usage", code: "", imports: "", description: "" },
      {
        heading: "Frozen Columns",
        code: "",
        imports: "",
        description: "HeroGPUI extension, not a HeroUI v3 example",
      },
    ],
  };
  const live = liveForAllExtensions(
    ["TitleBar / WindowBorder", "Toolbar extras", "Table data extras", "Theme hot reload"],
    examplesBySlug,
    catalog,
  );
  // The identifier rule claims the toolbar's Title Bar example by heading.
  assert.deepEqual(live[0], [
    { component: "toolbar", title: "Toolbar", heading: "Title Bar", anchor: "rust-title-bar" },
  ]);
  // "Toolbar extras" -> Sized Controls only: Title Bar is already claimed, and
  // plain examples without the description marker are not linked.
  assert.deepEqual(live[1], [
    {
      component: "toolbar",
      title: "Toolbar",
      heading: "Sized Controls",
      anchor: "rust-sized-controls",
    },
  ]);
  // "Table data extras" -> the extension examples in example order.
  assert.deepEqual(live[2], [
    {
      component: "table",
      title: "Table",
      heading: "Column Reordering",
      anchor: "rust-column-reordering",
    },
    {
      component: "table",
      title: "Table",
      heading: "Frozen Columns",
      anchor: "rust-frozen-columns",
    },
  ]);
  // A first word that names no catalog title links nothing.
  assert.deepEqual(live[3], []);
});

test("extensionSummary cuts to the first sentence outside backticks", () => {
  assert.equal(
    extensionSummary(
      "HeroGPUI extension, not a HeroUI v3 example: `fit_panel_width(true)` measures options. `intrinsic_panel_width` exposes the same measurement.",
    ),
    "`fit_panel_width(true)` measures options.",
    "the first sentence keeps its period and its code spans",
  );
  // The "(after gpui-kit's …): " variant ends at the same first ': ', and
  // `Toolbar::size`'s `::` is not a ': ' of its own.
  assert.equal(
    extensionSummary(
      "HeroGPUI extension, not a HeroUI v3 example (after gpui-kit's toolbar): `Toolbar::size` sizes every control. More text.",
    ),
    "`Toolbar::size` sizes every control.",
  );
  // No '. ' outside code spans: the whole text is the one sentence.
  assert.equal(
    extensionSummary(
      "HeroGPUI extension: `presenter_mode(TooltipPresenterMode::SeparateWindow)` routes the tooltip into a secondary window.",
    ),
    "`presenter_mode(TooltipPresenterMode::SeparateWindow)` routes the tooltip into a secondary window.",
  );
  // A '. ' inside a code span does not cut.
  assert.equal(
    extensionSummary("HeroGPUI extension: `a.b. c` keeps going"),
    "`a.b. c` keeps going",
  );
  assert.equal(extensionSummary("no marker colon here"), "");
  assert.equal(extensionSummary(undefined), "");
});

test("componentExtensions keeps the extension examples no live list claimed", () => {
  const catalog = {
    categories: [{ name: "Data", slug: "data", components: ["toolbar", "select"] }],
    components: {
      toolbar: { slug: "toolbar", title: "Toolbar" },
      select: { slug: "select", title: "Select" },
    },
  };
  const examplesBySlug = {
    // Select precedes toolbar in the object, but the catalog order sorts the
    // toolbar's item first.
    select: [
      {
        heading: "Intrinsic Panel Width",
        code: "",
        description:
          "HeroGPUI extension, not a HeroUI v3 example: `fit_panel_width(true)` measures options.",
      },
      { heading: "Usage", code: "", description: "" },
    ],
    toolbar: [
      {
        heading: "Title Bar",
        code: "TitleBar::window_options()",
        description:
          "HeroGPUI extension, not a HeroUI v3 example: `TitleBar` is custom window chrome. More.",
      },
      {
        heading: "Sized Controls",
        code: "",
        description:
          "HeroGPUI extension, not a HeroUI v3 example (after gpui-kit's toolbar): `Toolbar::size` sizes every control. More.",
      },
    ],
    unlisted: [
      {
        heading: "Orphan",
        code: "",
        description: "HeroGPUI extension, not a HeroUI v3 example: no catalog title, no item.",
      },
    ],
  };
  const extras = componentExtensions(
    examplesBySlug,
    catalog,
    new Set(["toolbar\u0000rust-title-bar"]),
  );
  assert.deepEqual(extras, [
    {
      component: "toolbar",
      title: "Toolbar",
      heading: "Sized Controls",
      anchor: "rust-sized-controls",
      summary: "`Toolbar::size` sizes every control.",
    },
    {
      component: "select",
      title: "Select",
      heading: "Intrinsic Panel Width",
      anchor: "rust-intrinsic-panel-width",
      summary: "`fit_panel_width(true)` measures options.",
    },
  ]);
  // Nothing claimed: every marker example survives, in catalog order.
  const all = componentExtensions(examplesBySlug, catalog, new Set());
  assert.deepEqual(
    all.map((item) => item.anchor),
    ["rust-title-bar", "rust-sized-controls", "rust-intrinsic-panel-width"],
  );
});
