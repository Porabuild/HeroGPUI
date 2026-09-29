# 03: gpui: Clip `overflow_hidden` content to rounded corners

Patch: [`03-gpui-rounded-content-masks.patch`](03-gpui-rounded-content-masks.patch).
Base: `zed@d89e9c2`. 26 files, +1142/-188:

- `crates/gpui`: `clip.rs` (new), `scene.rs`, `window.rs`, `style.rs`,
  `view.rs`, `debug_overlay.rs`, `elements/{div,list,uniform_list}.rs`,
  `gpui.rs`;
- `crates/gpui_wgpu`: `wgpu_renderer.rs`, `shaders.wgsl`,
  `shaders_storage.wgsl`, `shaders_webgl.wgsl`, `shaders_subpixel.wgsl`;
- `crates/gpui_apple`: `metal_renderer.rs`, `shaders.metal`, `build.rs`;
- `crates/gpui_windows`: `directx_renderer.rs`, `shaders.hlsl`;
- call sites: `crates/editor/src/{element.rs, element/header.rs,
  edit_prediction.rs, split_editor_view.rs}`,
  `crates/terminal_view/src/terminal_element.rs`,
  `crates/ui/src/components/scrollbar.rs` (+13/-10).

`git apply --check`: OK on `zed@d89e9c2` (all 26 files). On `/tmp/zedapply`
(0.3.5 crate dirs), OK with `--include='crates/gpui*'` (20 files; the six
call-site files are not in the crate tree). Build and test results are in
[README.md](README.md#verification-record). In short: gpui has 324 tests
passing, gpui_wgpu has 32 passing (including naga validation of all three
WGSL variants), the apple, windows and wasm checks pass, and clippy is clean.
The Metal and HLSL shader sources and the call-site crates were not compiled
locally.

Still needed at d89e9c2: yes. `ContentMask<P>` has only `bounds`,
`Style::overflow_mask` returns a rectangle, and every renderer clips with
`clip_distances` against a rectangle.

This is one PR, not four, because the GPU record layout of `ContentMask`
changes. Splitting gpui from the renderers would leave intermediate commits
that do not build.

---

**PR title:** `gpui: Clip overflow_hidden content to rounded corners`

**PR body:**

`overflow_hidden()` on an element with `rounded(..)` corners clips its
children to the element's *rectangle*. A child with its own background (an
image, a gradient, a hovered list row, a header strip) paints square pixels
through the rounded corner. Today the only workaround is to repeat the
parent's radius on every child that touches a corner. That breaks as soon as
the child is scrolled, inset by a border, or not flush with the corner.

This makes content masks rounded, end to end.

**Model (`crates/gpui`).**

- `ContentMask<P>` gains `corner_radii: Corners<P>`, `clip_index: u32` and a
  padding word. `ContentMask::new(bounds)` constructs the old rectangular
  mask.
- New `ClipRegion` (`clip.rs`): a rectangular cull `bounds` plus the list of
  `RoundedClip`s (bounds, separate horizontal/vertical `Corners` radii, parent
  index) that still constrain it. `intersect` never approximates a curve or
  moves it to the intersection's bounds. A narrow child inside a rounded
  parent keeps the parent's original corner geometry, so nested masks compose
  exactly. The window's content-mask stack is now a stack of `ClipRegion`s.
- `Scene` interns the rounded clips of each primitive's mask as an immutable
  parent-linked chain (`Scene::rounded_clips`, `insert_clip`). Identical
  chains share nodes within a frame, and `replay` re-imports the chains a
  reused range references. A primitive's `ContentMask::clip_index` points at
  the innermost node, and zero means "rectangular, use the inline bounds and
  radii" (the previous behavior and the common case).
- `Style::overflow_mask` returns the rounded padding box. Radii are inset by
  the visible border widths, per axis, so an unequal border gives the
  elliptical inner corners CSS specifies
  ([css-backgrounds-3, corner shaping](https://www.w3.org/TR/css-backgrounds-3/#corner-shaping)).
  A scroll container keeps its rounded mask even when only one axis scrolls
  ([css-overflow-3, corner clipping](https://www.w3.org/TR/css-overflow-3/#corner-clipping)).
  Background, border and mask now derive from one device-pixel-snapped
  geometry (`Style::paint_geometry`), so the mask's curve lines up with the
  painted border's curve at fractional scale factors.
- Paths now get the snapped mask too (`paint_path` previously stored the
  unsnapped logical mask).

**Renderers.** Each renderer uploads `Scene::rounded_clips` once per frame
and evaluates mask coverage per fragment for every primitive kind. The kinds
are quads, shadows, path rasterization, underlines, mono, subpixel and poly
sprites, and surfaces. Coverage is antialiased
(`saturate(0.5 - signed_distance)`) against every node in the chain:

- `gpui_wgpu`: a storage buffer at `@group(3)`, and for WebGL2 a
  `texture_2d<u32>` at the same group using the existing instance-texture
  transport. The clip records are written first so the texture is addressed
  from zero. The WebGL record strides grow accordingly, and
  `webgl_record_sizes_match_shader_word_strides` is updated.
- `gpui_apple`: fragment buffer 8. `build.rs` exports `RoundedClip_ScaledPixels`
  to the generated bindings and reads the new `clip.rs`.
- `gpui_windows`: a `StructuredBuffer<RoundedClip>` at `t2`, grown to the next
  power of two as needed.

**API changes** (all inside the GPUI family except the literals):

| Before | After |
|---|---|
| `ContentMask { bounds }` literal | `ContentMask::new(bounds)` or `..Default::default()` |
| `Window::content_mask() -> ContentMask<Pixels>` | `-> ClipRegion` (`.bounds` still works) |
| `Window::with_content_mask(Option<ContentMask<Pixels>>, ..)` | `Option<impl Into<ClipRegion>>` (existing callers compile unchanged) |
| `ContentMask::intersect -> ContentMask` | `-> ClipRegion` |
| `Style::overflow_mask(bounds, rem_size)` | `(bounds, rem_size, scale_factor) -> Option<ClipRegion>` |
| `Hitbox::content_mask: ContentMask<Pixels>` | `ClipRegion` (`ClipRegion::from_bounds` is `const`) |
| `Window::defer_draw(.., Option<ContentMask<Pixels>>)` | `Option<ClipRegion>` |

In the monorepo, only 13 lines in `editor`, `ui` and `terminal_view` needed
updating, all mechanical. Test-support gains `Window::painted_clips()`.

**Cost.** Unclipped and rectangularly clipped primitives take the
`clip_index == 0` path: one extra flat varying set and a rounded-rect distance
with zero radii. Primitives under rounded masks walk their chain, which is
usually one or two nodes deep. Every primitive's GPU record grows by six
words (radii, index, padding). A path-rasterization vertex grows by ten words,
because it now carries the whole mask.

**Tests.**

- `clip::tests::nested_clips_preserve_both_original_shapes` compares
  `ClipRegion::contains` of intersections against both originals over a pixel
  grid.
- `window::tests::rounded_content_mask_intersection_keeps_the_original_curve`.
- The `gpui_wgpu` naga validation tests cover the updated storage-buffer,
  WebGL and subpixel shader variants.

Suggested manual check on each backend (Metal, Vulkan/DX12 via wgpu,
DirectX, WebGPU and WebGL2): a `rounded(px(16.)).overflow_hidden()` parent
with a full-bleed image child, a scrolled list inside a rounded
`overflow_y_scroll()` panel, and an unequal-border (`border_l_4().border_t_1()`)
rounded box with a filled child. Compare at 1x, 1.25x and 2x.

Motivation: HeroGPUI is a GPUI port of the HeroUI v3 component library, where
cards, menus, selects, drawers and avatars all rely on CSS
`overflow: hidden` + `border-radius`. It carried this change as a local fork
of gpui, gpui_wgpu, gpui_apple and gpui_windows (with Metal pixel tests at 1x,
1.25x and 2x across seven content types). It currently mitigates the
missing renderer support per component (`util::inner_fill_radius`).

Release Notes:

- N/A

(If maintainers prefer a user-facing line: "Fixed content drawing past
rounded corners in panels and popovers.")

---

HeroGPUI follow-up once released: remove the per-component radius
workarounds built on `util::inner_fill_radius` and re-enable the retired
rounded-clip regression tests.
