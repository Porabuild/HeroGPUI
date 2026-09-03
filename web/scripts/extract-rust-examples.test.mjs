import assert from "node:assert/strict";
import test from "node:test";

import {
  documentationParity,
  humanizeGalleryIds,
  normalizeCollapsedItem,
  separateExampleDescription,
} from "./extract-rust-examples.mjs";

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
