# 01: gpui: Add `aria_current` accessibility builder

Patch: [`01-gpui-aria-current.patch`](01-gpui-aria-current.patch). Base:
`zed@d89e9c2`. File: `crates/gpui/src/elements/div.rs` (+23).

`git apply --check`: OK on `/tmp/zedapply` (gpui-pre 0.3.5 crate dirs) and on
`zed@d89e9c2`. Tested with `cargo test -p gpui-pre --lib --features
test-support`: 324 passed, including the new test.

Still needed at d89e9c2: yes. `AriaProperties` has no `current` field,
`StatefulInteractiveElement` has no setter, and `write_a11y_info` never calls
`Node::set_aria_current`.

---

**PR title:** `gpui: Add aria_current accessibility builder`

**PR body:**

AccessKit (0.24, as pinned by gpui) models `aria-current` as
`Node::set_aria_current(AriaCurrent)`, but GPUI's accessibility builders on
`StatefulInteractiveElement` have no way to set it. Elements that are the
current item in a set cannot say so. Examples are the active page in a
pagination control, the last crumb of a breadcrumb trail, the current step of
a wizard, or the selected link in a nav bar. Screen readers then do not
announce "current page" and similar.

This adds the builder next to the existing `aria_selected`/`aria_expanded`
setters, following their exact pattern:

- `AriaProperties` gains `current: Option<accesskit::AriaCurrent>`;
- `StatefulInteractiveElement::aria_current(AriaCurrent)` sets it;
- `Interactivity::write_a11y_info` forwards it with `node.set_aria_current`.

It includes a unit test alongside the other `write_a11y_info` tests.

Motivation: HeroGPUI (a GPUI port of the HeroUI component library) exposes
`aria-current="page"` on its Pagination and Breadcrumbs components, matching
React Aria. It carried this exact change as a local fork of GPUI, and
documents the state as unsupported on vanilla GPUI.

Testing: `cargo test -p gpui`, plus the new
`elements::div::tests::test_write_a11y_info_aria_current`. There is no
behavior change for elements that do not call the builder.

Release Notes:

- N/A

---

HeroGPUI follow-up once released: implement `a11y_current` in
`crates/herogpui-components/src/a11y.rs` (currently deliberately unwritten)
and drop the `aria-current` row from its omissions table.
