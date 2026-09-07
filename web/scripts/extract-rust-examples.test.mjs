import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";

import {
  documentationParity,
  humanizeGalleryIds,
  normalizeCollapsedItem,
  parseInvocation,
  separateExampleDescription,
} from "./extract-rust-examples.mjs";
import { MANIFEST_VERSION, buildManifest, parseExampleSource } from "./extract-wasm-sections.mjs";
import { readGalleryComponentSource } from "./lib/gallery-source.mjs";

test("documentationParity keeps component examples and reference metadata in sync", () => {
  assert.deepEqual(documentationParity(["button", "date-field"], ["button", "date-field"]), {
    missingReference: [],
    missingExamples: [],
  });
  assert.deepEqual(documentationParity(["button", "new-page"], ["button", "old-page"]), {
    missingReference: ["new-page"],
    missingExamples: ["old-page"],
  });
});

test("wasm section manifest matches generated component examples", () => {
  const examples = JSON.parse(
    readFileSync(resolve(import.meta.dirname, "../src/data/rust-examples.json"), "utf8"),
  );
  const wasmSections = JSON.parse(
    readFileSync(resolve(import.meta.dirname, "../src/data/wasm-sections.json"), "utf8"),
  );

  assert.deepEqual(Object.keys(wasmSections).sort(), Object.keys(examples).sort());

  const missing = [];
  for (const [slug, headings] of Object.entries(wasmSections)) {
    const documented = new Set(examples[slug]?.map((example) => example.heading) ?? []);
    const live = new Set(headings);
    assert.ok(documented.size > 0, `${slug} has wasm examples but no component documentation`);
    for (const heading of headings) {
      assert.ok(documented.has(heading), `${slug}/${heading} is not generated documentation`);
    }
    for (const heading of documented) {
      if (!live.has(heading)) missing.push(`${slug}/${heading}`);
    }
  }

  // The gallery has one source, so every documented example is compiled into
  // the artifact. A new entry here means the manifest was not regenerated
  // after the gallery changed, and a page would ship documentation the
  // browser cannot render.
  assert.deepEqual(missing.sort(), []);
});

test("wasm parity manifest pins the native source and compiled artifact", () => {
  const parity = JSON.parse(
    readFileSync(resolve(import.meta.dirname, "../src/data/wasm-parity.json"), "utf8"),
  );
  const nativeSource = readGalleryComponentSource(resolve(import.meta.dirname, "../.."));
  const native = parseExampleSource(nativeSource);
  const artifact = readFileSync(
    resolve(import.meta.dirname, "../public/gallery/herogpui_web_bg.wasm"),
  );
  const glue = readFileSync(resolve(import.meta.dirname, "../public/gallery/herogpui_web.js"));

  assert.equal(parity.version, MANIFEST_VERSION);
  assert.equal(createHash("sha256").update(artifact).digest("hex"), parity.artifactSha256);
  assert.equal(createHash("sha256").update(glue).digest("hex"), parity.glueSha256);
  assert.deepEqual(Object.keys(parity.examples).sort(), [...native.examples.keys()].sort());
  for (const [key, example] of native.examples) {
    assert.equal(parity.examples[key]?.codeSha256, example.codeSha256, `${key} code changed`);
    assert.equal(
      parity.examples[key]?.descriptionSha256,
      example.descriptionSha256,
      `${key} description changed`,
    );
  }
});

test("the manifest rejects unparseable pages and duplicate selector headings", () => {
  assert.throws(
    () => parseExampleSource('component_doc_page!("Broken"'),
    /could not parse component page.*unbalanced macro arguments/,
  );
  const duplicate = `component_doc_page!(
    "Button",
    "Press an action.",
    "use herogpui::Button;",
    vec![("Usage", Button::new("one")), ("Usage", Button::new("two"))],
    cx,
  )`;
  assert.throws(() => parseExampleSource(duplicate), /button has duplicate example heading Usage/);
});

test("the manifest hashes example code past formatting but not past edits", () => {
  const source = `component_doc_page!(
    "Button",
    "Press an action.",
    "use herogpui::Button;",
    vec![("Usage", "Current description.", Button::new("one"))],
    cx,
  )`;
  const baseline = buildManifest(source, Buffer.from("wasm")).parity;
  assert.deepEqual(baseline.sections ?? undefined, undefined);
  assert.equal(baseline.version, MANIFEST_VERSION);

  // Reformatting the gallery must not invalidate the committed artifact: a
  // 19 MB rebuild for a moved comma is a rebuild nobody does, and a manifest
  // people stop regenerating stops guarding anything.
  const formattingOnly = source.replace('Button::new("one")', ' Button::new(  "one", ) ');
  assert.equal(
    buildManifest(formattingOnly, Buffer.from("wasm")).parity.examples["button/Usage"].codeSha256,
    baseline.examples["button/Usage"].codeSha256,
  );

  // A real edit inside a string literal must, because the browser would
  // render the old text next to the new code block.
  const changedString = source.replace('Button::new("one")', 'Button::new("o ne")');
  assert.notEqual(
    buildManifest(changedString, Buffer.from("wasm")).parity.examples["button/Usage"].codeSha256,
    baseline.examples["button/Usage"].codeSha256,
  );

  const changedDescription = source.replace("Current description.", "New description.");
  assert.notEqual(
    buildManifest(changedDescription, Buffer.from("wasm")).parity.examples["button/Usage"]
      .descriptionSha256,
    baseline.examples["button/Usage"].descriptionSha256,
  );

  // The artifact and glue are pinned too, so swapping the binary without
  // regenerating is caught the same way.
  assert.notEqual(
    buildManifest(source, Buffer.from("other wasm")).parity.artifactSha256,
    baseline.artifactSha256,
  );
});

test("parseInvocation reads explicit section descriptions", () => {
  const source = `component_doc_page!(
    "Date Field",
    "Edit a date.",
    "use herogpui::DateField;",
    vec![
      ("Usage", "Uses the system format.", DateField::new(value)),
      ("Disabled", DateField::new(value).is_disabled(true)),
    ],
    cx,
  )`;

  assert.deepEqual(parseInvocation(source, 0).sections, [
    {
      heading: "Usage",
      description: "Uses the system format.",
      code: "DateField::new(value)",
      baseIndent: 6,
    },
    {
      heading: "Disabled",
      description: undefined,
      code: "DateField::new(value).is_disabled(true)",
      baseIndent: 6,
    },
  ]);
});

test("humanizeGalleryIds preserves multiline constructor indentation", () => {
  const source = `gpui::div()
    .child(Dropdown::new(
        "dd-trigger-dd",
        Button::new("dd-trigger").label("Actions"),
        items,
    ))`;

  assert.equal(
    humanizeGalleryIds(source, "dropdown").code,
    `gpui::div()
    .child(Dropdown::new(
        "dropdown-trigger-dd",
        Button::new("dropdown-trigger").label("Actions"),
        items,
    ))`,
  );
});

test("humanizeGalleryIds preserves collection keys and their selected values", () => {
  for (const item of ["ListBoxItem", "MenuItem"]) {
    const source = `${item}::new("new-file", "New file");
h::${item}::new("edit-file", "Edit file");
${item}::new(
    "delete-file", "Delete file"
);
list.selected_keys(["new-file", "edit-file"]);`;
    assert.deepEqual(humanizeGalleryIds(source, "list-box"), {
      code: source,
      changed: false,
    });
  }
});

test("separateExampleDescription lifts direct static gallery copy", () => {
  const source = `col(vec![
    Button::new("save").label("Save").into_any_element(),
    para(
        "The value is stored by the caller and \\
         submitted with the form.",
        cx,
    ),
])`;

  assert.deepEqual(separateExampleDescription(source), {
    description: "The value is stored by the caller and submitted with the form.",
    code: `col(vec![
    Button::new("save").label("Save").into_any_element(),
])`,
  });
});

test("separateExampleDescription preserves component content and dynamic output", () => {
  const nested = `Card::new().child(para("Card body", cx))`;
  assert.deepEqual(separateExampleDescription(nested), {
    description: undefined,
    code: nested,
  });

  const dynamic = `col(vec![para(&format!("Value: {value}"), cx)])`;
  assert.deepEqual(separateExampleDescription(dynamic), {
    description: undefined,
    code: dynamic,
  });
});

test("normalizeCollapsedItem keeps standard method-chain indentation", () => {
  assert.equal(
    normalizeCollapsedItem(`DatePicker::new(calendar)
        .label("Date")
        .is_disabled(true)`),
    `DatePicker::new(calendar)
    .label("Date")
    .is_disabled(true)`,
  );
});
