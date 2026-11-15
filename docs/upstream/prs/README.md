# Upstream PR drafts for Zed GPUI

Ready-to-submit patches for `zed-industries/zed`:
- `01`–`05` carry the deviations HeroGPUI retired when it moved to vanilla
  registry GPUI (`docs/upstream/retired-patches/`, written against `gpui-pre`
  0.3.3), split into one change per PR and re-based onto `zed@d89e9c2`.
- `06`–`08` capture Windows backend (`gpui_windows`) fixes carried in Poratake's
  `gpui-pre-windows` fork, re-based cleanly onto `gpui-pre-windows` `0.3.11`
  (`zed@f816e1f`, the revision HeroGPUI currently pins).

**Base revisions:**
- Patches `01`–`05`: `zed@d89e9c2124b2786a390c7a451c7488601b4da2e1` (`gpui-pre`
  0.3.5), also re-checked against `zed@f816e1f` (`gpui-pre` 0.3.11).
- Patches `06`–`08`: `gpui-pre-windows` `0.3.11` (`zed@f816e1f`).

The sources in those snapshots are byte-identical to the republished crates
except for the republisher's `gpui/src/action.rs` and `gpui_apple/build.rs`
(crate-relative paths and `vendor/gpui`).

Every patch is a plain `git diff` with Zed monorepo paths
(`a/crates/gpui_web/src/events.rs`, ...). None carries an author or commit
message: the submitter commits it under the identity that signs the Zed CLA.

## The patches

| # | Patch | Crates | Size | Still needed at d89e9c2 / f816e1f (0.3.11) |
|---|---|---|---|---|
| 01 | [`01-gpui-aria-current`](01-gpui-aria-current.md) | gpui | 1 file, +23 | Yes: no `aria_current` setter, no node propagation |
| 02 | [`02-gpui-spread-shadow-corner-radii`](02-gpui-spread-shadow-corner-radii.md) | gpui | 1 file, +59/-1 | Yes: `paint_drop_shadows` reuses the element radius |
| 03 | [`03-gpui-rounded-content-masks`](03-gpui-rounded-content-masks.md) | gpui, gpui_wgpu, gpui_apple, gpui_windows, editor, ui, terminal_view | 26 files, +1142/-188 | Yes: `ContentMask` is rectangular only |
| 04 | [`04-gpui-web-shift-wheel-horizontal-scroll`](04-gpui-web-shift-wheel-horizontal-scroll.md) | gpui_web | 1 file, +11/-5 | Yes: `register_wheel` forwards the raw axes |
| 05 | [`05-gpui-web-ime-mirror-resync-after-paste`](05-gpui-web-ime-mirror-resync-after-paste.md) | gpui_web | 1 file, +9 | Yes: neither paste path schedules a mirror sync |
| 06 | [`06-gpui-windows-atlas-texel-clamping`](06-gpui-windows-atlas-texel-clamping.md) | gpui_windows | 1 file, +8/-1 | Yes at `f816e1f`: `polychrome_sprite_fragment` samples `input.tile_position` unclamped |
| 07 | [`07-gpui-windows-mouse-leave-exit`](07-gpui-windows-mouse-leave-exit.md) | gpui_windows | 1 file, +78/-2 | Yes at `f816e1f`: `handle_mouse_leave_msg` omits `MouseMove` / `MouseExited` |
| 08 | [`08-gpui-windows-dib-clipboard`](08-gpui-windows-dib-clipboard.md) | gpui_windows | 1 file, +234/-5 | Yes at `f816e1f`: `write_image` omits `CF_DIB`; `convert_dib_to_bmp` lacks bounds/mask checks |

Patches `01`–`05` are independent of each other: each applies alone to
`d89e9c2`, and all five also apply cumulatively in numeric order (01 and 03
both touch `div.rs`, 02 and 03 both touch `window.rs`, in disjoint hunks).
Patches `06`–`08` each touch a distinct file (`shaders.hlsl`, `events.rs`,
`clipboard.rs`) on `f816e1f` (`0.3.11`) and apply independently or cumulatively
with `01`–`05` (no file overlap except `03` and `06` both touching `shaders.hlsl`
in disjoint areas: `quad` clip logic vs `polychrome_sprite_fragment`).
All PRs can be opened in any order.

[`context-menu-focus-anchor.md`](context-menu-focus-anchor.md) is not an
upstream patch. It is the HeroGPUI-side design for anchoring a
keyboard-opened `ContextMenu` at the focused element, for the components agent.

### What was dropped from the retired patch set (`01`–`05`)

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
- **Nothing was obsoleted by 0.3.5 — or by 0.3.10 or 0.3.11.** All five changes
  in `01`–`05` remain missing at `d89e9c2` and `f816e1f` (0.3.11), and all
  three Windows fixes in `06`–`08` remain missing at `f816e1f` (0.3.11).

### Added while re-basing

- 02 gained a regression test and its own `Window::painted_shadows`
  test-support accessor. The retired patch had the fix but no gpui-side test.
- 03 gained `ContentMask::new(bounds)` and `ClipRegion::from_bounds` (a
  `const fn`). It also gained the monorepo call-site updates the new
  `ContentMask` fields make necessary: `editor`, `ui` and `terminal_view`,
  six files, +13/-10. Without them the PR does not build Zed.
- 05 was rewritten and re-verified against 0.3.5's `ime_mirror.rs` (see its
  description for which paste paths 0.3.5 already covers by accident).
- 08 was re-based onto `0.3.11` (which already carries the `"PNG"` clipboard
  write path) with `rustfmt --edition 2024` formatting and neutral upstream
  `GPUI_ISOLATED_CLIPBOARD_TEST` env naming.

## Verification record

Setup for `01`–`05`: `/tmp/zedapply/crates/<zed dir>/` held verbatim copies
of the five 0.3.5 crate directories (`gpui-pre` -> `gpui`, `gpui-pre-web` ->
`gpui_web`, `gpui-pre-wgpu` -> `gpui_wgpu`, `gpui-pre-apple` -> `gpui_apple`,
`gpui-pre-windows` -> `gpui_windows`), committed in a fresh `git init`. A
shallow `git fetch --depth 1 origin d89e9c2...` of `zed-industries/zed` was
also checked, because 03 touches crates outside the GPUI family.

Setup for `06`–`08`: a temporary scratch repo at `/tmp/gpui-win-check/crates/gpui_windows`
held a verbatim copy of `gpui-pre-windows 0.3.11` (`zed@f816e1f`).

`git apply --check`, each patch alone:

| Patch | Scratch crate tree | `zed@d89e9c2` |
|---|---|---|
| 01 | OK (0.3.5) | OK |
| 02 | OK (0.3.5) | OK |
| 03 | OK for `--include='crates/gpui*'` (20 files) | OK (26 files) |
| 04 | OK (0.3.5) | OK |
| 05 | OK (0.3.5) | OK |
| 06 | OK (0.3.11 `gpui_windows`) | — |
| 07 | OK (0.3.11 `gpui_windows`) | — |
| 08 | OK (0.3.11 `gpui_windows`) | — |

All five `01`–`05` applied sequentially succeed on both `0.3.5` and
`zed@d89e9c2`; all three `06`–`08` applied sequentially succeed on `0.3.11`
`gpui_windows`.

Build and test on the patched `0.3.5` crates (`01`–`05`, macOS arm64):

| Check | Result |
|---|---|
| `cargo test -p gpui-pre --lib --features test-support` | 324 passed, 0 failed (`test_write_a11y_info_aria_current`, `test_spread_shadow_corner_radii_follow_the_spread`, `nested_clips_preserve_both_original_shapes`, `rounded_content_mask_intersection_keeps_the_original_curve`) |
| 02 alone on the base: `cargo test ... spread_shadow` | passed |
| `cargo test -p gpui-pre-wgpu --lib` | 32 passed (naga shader validation and stride checks) |
| `cargo check -p gpui-pre-apple --features runtime_shaders` | OK |
| `cargo check -p gpui-pre-windows --target x86_64-pc-windows-msvc` | OK |
| `cargo +nightly check -p gpui-pre-web -p gpui-pre-wgpu --target wasm32-unknown-unknown` | OK |
| `cargo clippy` of gpui, gpui_wgpu, gpui_apple, gpui_windows (`d89e9c2` lints) | clean |

Build check on the patched `0.3.11` `gpui_windows` crate (`06`–`08` applied, macOS arm64 with Homebrew `llvm-rc`):

| Check | Result |
|---|---|
| `cargo check -p gpui-pre-windows --tests --target x86_64-pc-windows-msvc --offline` | OK (clean Rust & test compilation for `events.rs` and `clipboard.rs`) |

What was **not** verified locally, which the submitter must cover:

- The Metal shader source was not compiled offline: this machine has no
  Metal toolchain (`xcodebuild -downloadComponent MetalToolchain`). A normal
  macOS build of `gpui_apple` compiles it.
- The HLSL (`03` and `06`) is compiled only by a Windows build of `gpui_windows`.
- The `editor`, `ui` and `terminal_view` call-site updates in 03 were only
  formatted and `git apply`-checked, not compiled (`cargo check -p editor -p ui -p terminal_view` covers them).
- Runtime execution of `06`–`08` on Windows, including `cargo test -p gpui_windows`,
  `GPUI_ISOLATED_CLIPBOARD_TEST=1`, fractional-DPI atlas inspection, `WM_MOUSELEAVE`
  hover exit, and third-party app clipboard paste.

### Re-checked against Zed `main` (2026-11-08)

Against `zed@d2a4fb6ded7c470be25d1541281da3882495b65b` (`main` on
2026-11-08), from a shallow sparse clone of the touched crates, `git apply --check`, each of `01`–`05` alone:

| Patch | Applies | Still needed on `main` |
|---|---|---|
| 01 | Yes | Yes: no `aria_current` anywhere under `crates/gpui/src` |
| 02 | Only with reduced context (`-C1`): `window.rs` moved around the hunks | Yes: `paint_drop_shadows` now also passes `element_bounds` and `element_corner_radii`, but the shadow's own `corner_radii` is still the element's, not grown by the spread |
| 03 | No: conflicts in `view.rs`, `window.rs` and `gpui_apple/src/metal_renderer.rs`, even at `-C1` | Yes: `ContentMask` is still `{ bounds }` only |
| 04 | Yes | Yes: `register_wheel` still forwards the raw axes |
| 05 | Yes | Yes: `register_paste` (now with image pastes) still schedules no IME-mirror sync |

Nothing has been submitted yet: each PR needs an account that has signed the Zed CLA.

## How a maintainer submits them

Each PR needs a GitHub account that has signed the
[Zed CLA](https://zed.dev/cla). The CLA bot comments on the first PR if it is
missing.

1. Fork `zed-industries/zed` and clone the fork.
2. Check out the base (`d89e9c2124b2786a390c7a451c7488601b4da2e1` for `01`–`05`, or `f816e1f` / current `main` for `06`–`08`) and branch from it, one branch per patch:

   ```sh
   git fetch origin d89e9c2124b2786a390c7a451c7488601b4da2e1
   git switch -c gpui-aria-current d89e9c2124b2786a390c7a451c7488601b4da2e1
   git apply --index /path/to/HeroGPUI/docs/upstream/prs/01-gpui-aria-current.patch
   git commit -m "gpui: Add aria_current accessibility builder"
   ```

   The patches are plain diffs rather than mailboxes, so use `git apply`, not
   `git am`. Use the PR title from the matching `.md` as the commit subject.
3. Re-base onto current `main`. Zed moves fast, and `crates/gpui*` changes
   between snapshots, so conflicts are possible, especially for 03:

   ```sh
   git fetch upstream main && git rebase upstream/main
   ```

   After re-basing 03, grep the whole tree again for
   `ContentMask {`, `overflow_mask(` and `Hitbox {` outside `crates/gpui*`.
   Any new struct-literal site needs `ContentMask::new(..)` or
   `..Default::default()`.
4. Run the checks for the touched crates. Zed's CI runs the full set:

   ```sh
   cargo test -p gpui                                   # 01, 02, 03
   cargo test -p gpui_wgpu                              # 03 (naga shader validation)
   cargo check -p gpui_apple -p gpui_macos              # 03, on macOS (compiles shaders.metal)
   cargo check -p gpui_windows                          # 03, 06, 07, 08 on Windows (compiles shaders.hlsl)
   cargo test -p gpui_windows                           # 07, 08 on Windows
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
   (WebGPU and WebGL2). For 06–08, verify atlas edge sampling at 125%/150% DPI,
   window-exit hover clearing, and third-party clipboard paste on Windows.
6. Push the branch to the fork and open the PR against `zed-industries/zed`
   `main`. Use the title and body from the matching `.md`, including its
   `Release Notes:` block. Zed's PR template requires that block, and `- N/A`
   is the accepted value for changes users do not see.

When a PR merges and reaches a `gpui-pre` release that HeroGPUI adopts, delete
the matching workaround (each `.md` names it), and remove the corresponding
part of `docs/upstream/retired-patches/`.
