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
import { liftDescriptions } from "./lift-wasm-descriptions.mjs";
import { buildParity, introducedDrift, parseExampleSource } from "./extract-wasm-sections.mjs";

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

  // Every documented example now compiles into the migration artifact. A new
  // entry here means a page shipped documentation the browser cannot render.
  assert.deepEqual(missing.sort(), []);
});

test("wasm parity manifest pins the native source and compiled artifact", () => {
  const parity = JSON.parse(
    readFileSync(resolve(import.meta.dirname, "../src/data/wasm-parity.json"), "utf8"),
  );
  const nativeSource = readFileSync(
    resolve(import.meta.dirname, "../../gallery/src/pages/components.rs"),
    "utf8",
  );
  const native = parseExampleSource(nativeSource);
  const artifact = readFileSync(
    resolve(import.meta.dirname, "../public/gallery/herogpui_web_bg.wasm"),
  );
  const glue = readFileSync(resolve(import.meta.dirname, "../public/gallery/herogpui_web.js"));

  assert.equal(parity.version, 1);
  assert.equal(createHash("sha256").update(artifact).digest("hex"), parity.artifactSha256);
  assert.equal(createHash("sha256").update(glue).digest("hex"), parity.glueSha256);
  assert.deepEqual(Object.keys(parity.examples).sort(), [...native.examples.keys()].sort());
  for (const [key, example] of native.examples) {
    assert.equal(parity.examples[key]?.nativeCodeSha256, example.codeSha256, `${key} code changed`);
    assert.equal(
      parity.examples[key]?.descriptionSha256,
      example.descriptionSha256,
      `${key} description changed`,
    );
  }
  assert.deepEqual(parity.missing, []);
  assert.deepEqual(parity.extra, []);
  assert.deepEqual(parity.descriptionDrift, []);
  assert.deepEqual(
    parity.codeDrift,
    Object.entries(parity.examples)
      .filter(([, example]) =>
        Boolean(
          example.nativeCodeSha256 &&
          example.wasmCodeSha256 &&
          example.nativeCodeSha256 !== example.wasmCodeSha256,
        ),
      )
      .map(([key]) => key),
  );
});

test("wasm parity rejects duplicate selector headings and detects description drift", () => {
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

  const native = duplicate
    .replace(', ("Usage", Button::new("two"))', "")
    .replace(
      '("Usage", Button::new("one"))',
      '("Usage", "Current description.", Button::new("one"))',
    );
  const wasm = native.replace("Current description.", "Stale description.");
  assert.deepEqual(buildParity(native, wasm, Buffer.from("wasm")).parity.descriptionDrift, [
    "button/Usage",
  ]);
  assert.deepEqual(introducedDrift(["button/Usage"], ["button/Usage", "input/Usage"]), [
    "input/Usage",
  ]);

  const formattingOnly = native.replace('Button::new("one")', ' Button::new(  "one", ) ');
  assert.deepEqual(buildParity(native, formattingOnly, Buffer.from("wasm")).parity.codeDrift, []);
  const changedString = native.replace('Button::new("one")', 'Button::new("o ne")');
  assert.deepEqual(buildParity(native, changedString, Buffer.from("wasm")).parity.codeDrift, [
    "button/Usage",
  ]);
});

test("liftDescriptions moves static copy outside the wasm specimen", () => {
  const source = `component_doc_page!(
    "Button",
    "Press an action.",
    "use herogpui::Button;",
    vec![(
      "Usage",
      col(vec![
        para("Choose an action before continuing.", cx),
        Button::new("save").label("Save").into_any_element(),
      ]),
    )],
    cx,
  )`;

  const first = liftDescriptions(source);
  assert.equal(first.lifted, 1);
  assert.match(first.output, /"Usage",\s+"Choose an action before continuing\.",/);
  assert.doesNotMatch(first.output, /para\(/);
  assert.equal(liftDescriptions(first.output).lifted, 0);
});

test("liftDescriptions rejects setup blocks that need explicit migration", () => {
  const source = `component_doc_page!(
    "Form",
    "Submit fields.",
    "use herogpui::Form;",
    vec![("Server Errors", {
      let form = Form::new();
      col(vec![para("Server errors stay visible.", cx), form.into_any_element()])
    })],
    cx,
  )`;

  assert.throws(
    () => liftDescriptions(source),
    /Form\/Server Errors has prose inside a setup block/,
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
