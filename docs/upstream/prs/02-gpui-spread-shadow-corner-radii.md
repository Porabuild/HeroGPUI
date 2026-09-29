# 02: gpui: Grow spread shadow corner radii with the spread

Patch: [`02-gpui-spread-shadow-corner-radii.patch`](02-gpui-spread-shadow-corner-radii.patch).
Base: `zed@d89e9c2`. File: `crates/gpui/src/window.rs` (+59/-1). Of that, the
fix is 9 lines, a test-support accessor is 7, and the test is 43.

`git apply --check`: OK on `/tmp/zedapply` and on `zed@d89e9c2`. Tested
alone on the base, and with the full series:
`test_spread_shadow_corner_radii_follow_the_spread` passes.

Still needed at d89e9c2: yes. `Window::paint_drop_shadows` dilates the shadow
bounds by `spread_radius`, but passes the element's unchanged
`corner_radii`.

---

**PR title:** `gpui: Grow spread shadow corner radii with the spread`

**PR body:**

`Window::paint_drop_shadows` dilates a drop shadow's bounds by its
`spread_radius` but keeps the element's corner radii. In CSS
([css-backgrounds-3, shadow shape](https://www.w3.org/TR/css-backgrounds-3/#shadow-shape)),
a spread shadow's corner curve grows with it. The shadow is `spread` larger on
every side, so each radius grows by `spread` and clamps at zero for negative
spreads. CSS also softens the growth of radii smaller than the spread. This
change uses the plain additive rule, which matches CSS exactly whenever the
radius is at least the spread (the focus-ring case), and is never squarer
than the element.

Keeping the element radius makes the outside of any spread shadow visibly
squarer than the element. It is most obvious on focus rings drawn as
`0 0 0 2px` spread shadows, where the ring's corners look pinched against a
rounded control. Component libraries currently work around this by drawing
rings as bordered overlays or pre-rendered SVGs.

This change:

- computes `corner_radii.map(|r| (r + spread_radius).max(0))` for each drop
  shadow. Inset shadows go through `paint_inset_shadows` and are untouched,
  and `element_corner_radii` is unchanged too;
- adds a `#[cfg(any(test, feature = "test-support"))] Window::painted_shadows()`
  accessor, mirroring the existing `painted_quads()`;
- adds a test that paints an 8px-radius element with a +4px and a -12px
  spread shadow and asserts radii of 12px and 0px.

Blur-only shadows (`spread_radius == 0`) are unchanged. This is the common
case for elevation shadows in Zed's UI.

Motivation: HeroGPUI, a GPUI port of HeroUI v3, whose focus rings are CSS
`ring-2`/`ring-offset-2` spread shadows. It carried this fix in a local GPUI
fork, and without it currently draws the ring band as an SVG overlay (see
`crates/herogpui-components/src/util.rs`, `focus_ring_overlay`).

Testing: `cargo test -p gpui`. Also visually: a rounded button with
`shadow(vec![BoxShadow { spread_radius: px(2.), .. }])` now has concentric
corners.

Release Notes:

- N/A

---

HeroGPUI follow-up once released: re-evaluate whether the SVG ring band in
`util::focus_ring_overlay` can return to `focus_ring_shadows` for symmetric
corners. The overlay also exists for the soft blur profile, so this is a
design decision, not an automatic deletion.
