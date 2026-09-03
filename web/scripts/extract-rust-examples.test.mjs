import assert from "node:assert/strict";
import test from "node:test";

import { humanizeGalleryIds } from "./extract-rust-examples.mjs";

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
