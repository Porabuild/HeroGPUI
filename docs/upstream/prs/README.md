# Upstream PR drafts for Zed GPUI

Ready-to-submit patches for `zed-industries/zed`, carrying the deviations
HeroGPUI retired when it moved to vanilla registry GPUI
(`docs/upstream/retired-patches/`, written against `gpui-pre` 0.3.3), split
into one change per PR and re-based onto the revision HeroGPUI pins.

**Base revision:** `zed@d89e9c2124b2786a390c7a451c7488601b4da2e1`, which is
what `gpui-pre` 0.3.5 snapshots (every `gpui-pre-*` 0.3.5 manifest records it
under `[package.metadata.gpui-pre] zed-rev`). The `crates/gpui`, `gpui_web`,
`gpui_wgpu`, `gpui_apple` and `gpui_windows` sources in that revision are
byte-identical to the 0.3.5 crates except for two files the republisher
rewrote (`gpui/src/action.rs`, `gpui_apple/build.rs`: crate-relative paths and
the `vendor/gpui` directory). Neither is touched by a patch except
`gpui_apple/build.rs` in 03, and 03 was generated against the Zed tree.

Every patch is a plain `git diff` with Zed monorepo paths
(`a/crates/gpui_web/src/events.rs`, ...). None carries an author or commit
message: the submitter commits it under the identity that signs the Zed CLA.

## The patches

| # | Patch | Crates | Size | Still needed at d89e9c2 |
|---|---|---|---|---|
| 01 | [`01-gpui-aria-current`](01-gpui-aria-current.md) | gpui | 1 file, +23 | Yes: no `aria_current` setter, no node propagation |
| 02 | [`02-gpui-spread-shadow-corner-radii`](02-gpui-spread-shadow-corner-radii.md) | gpui | 1 file, +59/-1 | Yes: `paint_drop_shadows` reuses the element radius |
| 03 | [`03-gpui-rounded-content-masks`](03-gpui-rounded-content-masks.md) | gpui, gpui_wgpu, gpui_apple, gpui_windows, editor, ui, terminal_view | 26 files, +1142/-188 | Yes: `ContentMask` is rectangular only |
| 04 | [`04-gpui-web-shift-wheel-horizontal-scroll`](04-gpui-web-shift-wheel-horizontal-scroll.md) | gpui_web | 1 file, +11/-5 | Yes: `register_wheel` forwards the raw axes |
| 05 | [`05-gpui-web-ime-mirror-resync-after-paste`](05-gpui-web-ime-mirror-resync-after-paste.md) | gpui_web | 1 file, +9 | Yes: neither paste path schedules a mirror sync |

The patches are independent of each other: each applies alone to
`d89e9c2`, and all five also apply cumulatively in numeric order (01 and 03
both touch `div.rs`, 02 and 03 both touch `window.rs`, in disjoint hunks).
They can be opened as five PRs in any order.

[`context-menu-focus-anchor.md`](context-menu-focus-anchor.md) is not an
upstream patch. It is the HeroGPUI-side design for anchoring a
keyboard-opened `ContextMenu` at the focused element, for the components agent.

### What was dropped from the retired patch set

- **Packaging hunks.** The `[workspace]` tables, `[package.metadata.*]`
  tables, and the `gpui-pre-web` `default = []` feature deviation edit
  republished manifests only. The Zed manifests differ, and nothing upstream
  needs them.
- **`gpui_apple/vendor/gpui/*`.** That directory exists only in the
  republished crate. In Zed, `build.rs` reads `../gpui`, so 03's `scene.rs`,
  `clip.rs` and `window.rs` changes reach the Metal bindings with no copy.
- **The `gpui_wgpu` `shaders.rs` extraction.** HeroGPUI moved the WGSL
  constants into a new module so its own test could `#[path]`-include them.
  Upstream keeps them in `wgpu_renderer.rs`, and its existing naga validation
  tests (with `naga` already a dev-dependency in Zed) cover the changed shaders.
- **Formatting noise.** The 0.3.3 patches reordered imports under the 2021
  style edition. Everything here is formatted with Zed's `rustfmt.toml`
  (`edition = "2024"`, `style_edition = "2024"`), and `rustfmt --check` passes
  on every touched file. One import the old reorder hunk had re-added
  (`use futures::FutureExt;`) was removed, because 0.3.5 no longer uses it.
- **Nothing was obsoleted by 0.3.5.** All five changes are still missing at
  `d89e9c2`. The 0.3.3-to-0.3.5 re-base was a three-way merge. It conflicted
  only in import blocks, in `gpui/src/elements/div.rs` and
  `gpui/src/window.rs`.

### Added while re-basing

- 02 gained a regression test and its own `Window::painted_shadows`
  test-support accessor. The retired patch had the fix but no gpui-side test.
- 03 gained `ContentMask::new(bounds)` and `ClipRegion::from_bounds` (a
  `const fn`). It also gained the monorepo call-site updates the new
  `ContentMask` fields make necessary: `editor`, `ui` and `terminal_view`,
  six files, +13/-10. Without them the PR does not build Zed.
- 05 was rewritten and re-verified against 0.3.5's `ime_mirror.rs` (see its
  description for which paste paths 0.3.5 already covers by accident).

## Verification record

Setup: `/tmp/zedapply/crates/<zed dir>/` holds verbatim copies of the five
0.3.5 crate directories (`gpui-pre` -> `gpui`, `gpui-pre-web` -> `gpui_web`,
`gpui-pre-wgpu` -> `gpui_wgpu`, `gpui-pre-apple` -> `gpui_apple`,
`gpui-pre-windows` -> `gpui_windows`), committed in a fresh `git init`. A
shallow `git fetch --depth 1 origin d89e9c2...` of `zed-industries/zed` was
also checked, because 03 touches crates outside the GPUI family.

`git apply --check`, each patch alone:

| Patch | `/tmp/zedapply` (0.3.5 crates) | `zed@d89e9c2` |
|---|---|---|
| 01 | OK | OK |
| 02 | OK | OK |
| 03 | OK for `--include='crates/gpui*'` (20 files; the 6 `editor`/`ui`/`terminal_view` files are not in the crate tree) | OK (26 files) |
| 04 | OK | OK |
| 05 | OK | OK |

All five applied in order 01 to 05 succeed on both trees.

Build and test, on a scratch workspace of the patched 0.3.5 crates (all five
applied; `cargo +1.98.0 ... -j 6 --offline`, macOS arm64):

| Check | Result |
|---|---|
| `cargo test -p gpui-pre --lib --features test-support` | 324 passed, 0 failed. Includes the new `test_write_a11y_info_aria_current`, `test_spread_shadow_corner_radii_follow_the_spread`, `nested_clips_preserve_both_original_shapes` and `rounded_content_mask_intersection_keeps_the_original_curve` |
| 02 alone on the base: `cargo test ... spread_shadow` | passed |
| `cargo test -p gpui-pre-wgpu --lib` | 32 passed. Includes naga validation of the storage-buffer, WebGL and subpixel WGSL variants, and `webgl_record_sizes_match_shader_word_strides` with the new strides |
| `cargo check -p gpui-pre-apple --features runtime_shaders` | OK (the cbindgen bindings see `RoundedClip_ScaledPixels`) |
| `cargo check -p gpui-pre-windows --target x86_64-pc-windows-msvc` | OK (Rust side; see limits below) |
| `cargo +nightly check -p gpui-pre-web -p gpui-pre-wgpu --target wasm32-unknown-unknown` | OK |
| `cargo clippy` of gpui, gpui_wgpu, gpui_apple, gpui_windows, with Zed's denied lints (`redundant_clone`, `declare_interior_mutable_const`, `dbg_macro`, `todo`) | clean |

What was **not** verified locally, which the submitter must cover:

- The Metal shader source was not compiled offline: this machine has no
  Metal toolchain (`xcodebuild -downloadComponent MetalToolchain`). A normal
  macOS build of `gpui_apple` compiles it.
- The HLSL is compiled only by a Windows build of `gpui_windows`.
- The `editor`, `ui` and `terminal_view` call-site updates in 03 are
  mechanical, but they were only formatted and `git apply`-checked, not
  compiled. `cargo check -p editor -p ui -p terminal_view` in the Zed checkout
  covers them.

## How a maintainer submits them

Each PR needs a GitHub account that has signed the
[Zed CLA](https://zed.dev/cla). The CLA bot comments on the first PR if it is
missing.

1. Fork `zed-industries/zed` and clone the fork.
2. Check out the base and branch from it, one branch per patch:

   ```sh
   git fetch origin d89e9c2124b2786a390c7a451c7488601b4da2e1
   git switch -c gpui-aria-current d89e9c2124b2786a390c7a451c7488601b4da2e1
   git apply --index /path/to/HeroGPUI/docs/upstream/prs/01-gpui-aria-current.patch
   git commit -m "gpui: Add aria_current accessibility builder"
   ```

   The patches are plain diffs rather than mailboxes, so use `git apply`, not
   `git am`. Use the PR title from the matching `.md` as the commit subject.
3. Re-base onto current `main`. Zed moves fast, and `crates/gpui*` changes
   between snapshots, so conflicts are likely, especially for 03:

   ```sh
   git fetch upstream main && git rebase upstream/main
   ```

   After re-basing 03, grep the whole tree again for
   `ContentMask {`, `overflow_mask(` and `Hitbox {` outside `crates/gpui*`.
   Any new struct-literal site needs `ContentMask::new(..)` or
   `..Default::default()`.
4. Run the checks for the touched crates. Zed's CI runs the full set.

   ```sh
   cargo test -p gpui                                   # 01, 02, 03
   cargo test -p gpui_wgpu                              # 03 (naga shader validation)
   cargo check -p gpui_apple -p gpui_macos              # 03, on macOS (compiles shaders.metal)
   cargo check -p gpui_windows                          # 03, on Windows (compiles shaders.hlsl)
   cargo check -p editor -p ui -p terminal_view         # 03 call sites
   cargo check -p gpui_web --target wasm32-unknown-unknown   # 04, 05 (nightly; see below)
   ./script/clippy
   cargo fmt --all -- --check
   ```

   `gpui_web` on wasm32 needs nightly, because the `multithreaded` default
   feature pulls in `wasm_thread` (`#![feature]`), unless Zed's own wasm CI
   recipe says otherwise.
5. For 03, run the visual checks its description lists on at least macOS
   (Metal), Linux or Windows (wgpu Vulkan/DX12, or DirectX), and a browser
   (WebGPU and WebGL2).
6. Push the branch to the fork and open the PR against `zed-industries/zed`
   `main`. Use the title and body from the matching `.md`, including its
   `Release Notes:` block. Zed's PR template requires that block, and `- N/A`
   is the accepted value for changes users do not see.

When a PR merges and reaches a `gpui-pre` release that HeroGPUI adopts, delete
the matching workaround (each `.md` names it), and remove the corresponding
part of `docs/upstream/retired-patches/`.
