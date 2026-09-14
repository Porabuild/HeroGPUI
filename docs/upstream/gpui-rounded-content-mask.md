# gpui rounded overflow patch

HeroGPUI uses `gpui-pre` 0.3.3 from the registry. That release's
`overflow_hidden()` content mask is rectangular even when the element has
rounded corners, so a child gradient or image can paint square pixels through
the corner. The fix belongs in the renderer because every component shares
that mask.

The checked-in directories `crates/gpui_pre`, `crates/gpui_pre_wgpu`,
`crates/gpui_pre_apple`, and `crates/gpui_pre_windows` are local forks of the
exact 0.3.3 registry packages. They are selected by the `[patch.crates-io]`
entries at the end of the workspace `Cargo.toml`; no file in Cargo's registry
cache is edited. The lockfile consequently records these packages without a
registry source or checksum.

The intentional fork delta is recorded as unified patches under
`docs/upstream/patches/`:

- `gpui-pre-0.3.3.patch` adds exact `ClipRegion` intersections and immutable
  `RoundedClip` scene nodes. Each primitive references its ancestor chain by
  index; replay remaps those indices into the new frame. Rectangular culling
  bounds never replace an ancestor's original rounded geometry.
- `gpui-pre-wgpu-0.3.3.patch` carries the clip nodes through storage buffers
  and WebGL instance textures, with matching record strides and antialiased
  fragment coverage for every primitive, including subpixel text.
- `gpui-pre-apple-0.3.3.patch` carries the same nodes through the Metal
  renderer, including path intermediate passes, and updates the vendored
  definitions used to generate its bindings.
- `gpui-pre-windows-0.3.3.patch` uploads the node buffer to DirectX and applies
  the same coverage to every masked fragment path, including subpixel text.

Styled backgrounds, borders, and overflow masks derive from the same snapped
outer rectangle, normalized radii, and device-pixel border widths. Unequal
borders produce separate horizontal and vertical inner radii. Those inner
ellipses must not be clamped against the narrower inner rectangle. Rounded
scroll containers retain their corner mask when only one scroll axis is set,
including Dropdown and Select panels. The scrolling behavior follows the
[overflow and corner-clipping rules](https://www.w3.org/TR/css-overflow-3/#corner-clipping).

The same renderer path also expands a drop shadow's corner radii by its spread
before painting. A spread shadow is a larger rounded rectangle in CSS; reusing
the element's original radius makes focus rings and offset gaps look square at
their corners. The element's own radii remain unchanged for inset shading and
for the control's content mask, while negative spreads shrink the shadow curve
and clamp it at zero. This is a renderer-level correction, so every component
that uses GPUI shadows inherits the same rounded focus-ring geometry.

Clip intersections preserve every distinct ancestor curve. Logical corner
containment alone cannot remove one: the conservative device-pixel culling
rectangle may expand across its antialiased edge. Border-only quad strips keep
the same ancestor index when narrowing their rectangular cull. Scene replay
imports each distinct referenced chain once per replay, and scene clearing
releases the frame's nodes.

Regenerate and verify these patches with:

```sh
python3 .shots/gpui_patches.py --write
python3 .shots/gpui_patches.py --check
python3 .shots/gpui_patches.py --self-test
```

The command reads the exact pin and local fork paths from `Cargo.toml`, finds
those published package sources in Cargo's registry cache, and verifies that
applying each patch to a fresh copy reproduces its fork byte for byte. It
rejects missing or stale patches and fuzzy/offset application. Registry cache
metadata and per-package lockfiles are excluded; builds use the workspace
lockfile. On a cold cache it retrieves the exact packages with `cargo info`
outside the patched workspace. It never edits registry sources. CI runs the
same check so dependency edits cannot silently drift from the recorded patch.

When upgrading gpui-pre, keep the patch workflow explicit:

1. Pin the new `gpui-pre`, `gpui-pre-*` family versions together. Do not let a
   new version resolve with the old exact patch fork; the version pins are
   intentionally exact so Cargo cannot silently drop the renderer fix.
2. Copy the new registry package sources into fresh local fork directories,
   then apply the previous version's corresponding patch with `patch -p2` from
   each fork root.
   Resolve source drift deliberately, especially around `ContentMask`, shader
   record layouts, and the Apple generated-header vendor files.
3. Update the four patch files and this note to the new upstream version,
   retain the `[patch.crates-io]` entries, regenerate `Cargo.lock`, and verify
   that `cargo tree -i gpui-pre` reports the local path source.
4. Run the native and wasm checks and the focused regressions below. Rebuild
   the committed WASM artifact with the matching `wasm-bindgen`, regenerate
   its manifests, and verify both WebGPU and WebGL rendering.

The regular component suite includes the actual WGPU shader assembly module
for Naga validation on every host platform. macOS also runs real Metal pixel
tests through `MetalHeadlessRenderer` with runtime shader compilation:

```sh
cargo test -p herogpui-components --test rounded_clip_geometry \
  --test rounded_clip_pixels --test rounded_clip_shaders \
  --test rounded_content_mask --test color_geometry_deep
```

The pixel matrix compares seven content types across thin, tall, wide,
fractional, and asymmetric shapes at 1×, 1.25×, and 2×. Separate regressions
exercise linked elliptical clips after an unrelated node, fractional nested
rectangles through a real Window, and border-strip splitting. Geometry tests
also cover zero-area bounds, scrolling, fractional and unequal borders, and
64-node scene replay. Shader validation covers storage, WebGL, and subpixel
variants; it does not substitute for a Windows runtime check.

The component code may still add a radius to a child when it has a deliberate
different shape (for example, a swatch inside a circular item), but it no
longer relies on those local workarounds to make `overflow_hidden()` rounded.

Styled outer radii use GPUI 0.3.3's `clamp_radii_for_quad_size`: each radius
is limited to half the shortest side, including after device-pixel snapping. A ColorSlider endpoint is 10×20 px,
so a 10 px radius on that cap becomes 5 px, even with the renderer patch active.
ColorSlider therefore clips the whole painted track, including its caps and
checkerboard, in one full-size rounded layer. The thumb and inset edge
shadows remain outside that layer. Regression coverage inspects the painted cap and checkerboard masks for both slider orientations.
Hue caps and the ramp use the same fully saturated display spectrum; other
channels share their ramp endpoint colors. This prevents a color jump where a
cap meets its gradient. The extra solid track outline has been removed: it
left colored pixels beneath its independently antialiased rim. Like the pinned
HeroUI CSS, the complete track and caps use one-pixel, zero-blur inset edge
shadows, painted before the thumb.

The same local `gpui-pre` fork also carries one small accessibility extension:
AccessKit 0.24 exposes `AriaCurrent`, but gpui-pre 0.3.3 does not publish an
`aria_current` builder or copy that field into its node writer. The fork adds
that setter and node propagation; the component layer uses it for the active
page in Pagination and the last crumb in Breadcrumbs. The addition is included
in `docs/upstream/patches/gpui-pre-0.3.3.patch` and must be reapplied when the
exact gpui-pre family is refreshed.
