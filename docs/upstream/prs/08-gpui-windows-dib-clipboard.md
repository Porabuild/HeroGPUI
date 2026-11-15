# 08: gpui_windows: Write `CF_DIB` alongside PNG and harden `convert_dib_to_bmp`

Patch: [`08-gpui-windows-dib-clipboard.patch`](08-gpui-windows-dib-clipboard.patch).
Base: `gpui-pre-windows 0.3.11` (`zed@f816e1f`). File: `crates/gpui_windows/src/clipboard.rs` (+234/-5).

`git apply --check`: OK on the `gpui-pre-windows 0.3.11` source tree (`crates/gpui_windows`).
Cross-compile `cargo check -p gpui-pre-windows --tests --target x86_64-pc-windows-msvc`: OK (with `llvm-rc`), including the new `#[cfg(test)]` suite.

> **Verification scope:** Native clipboard-format availability and PNG/`CF_DIB` round-trips are the coverage intended by the `#[ignore]`d Windows test included in the patch; live Windows execution of this re-based patch and pasting into third-party Windows apps (Paint, Word, Slack, browsers) remain unverified locally.

Still needed at `f816e1f` (`gpui-pre 0.3.11`): yes. `0.3.11` writes the native image format plus PNG, but does not write `CF_DIB`, and `convert_dib_to_bmp` has unchecked `u32` arithmetic and treats `compression == 3` (`BI_BITFIELDS`) identically across extended v4/v5 headers that already include color masks inside `header_size`.

---

**PR title:** `gpui_windows: Write CF_DIB on image copy and fix DIB-to-BMP conversion`

**PR body:**

On Windows, many native and legacy desktop applications look for `CF_DIB` when pasting images from the clipboard and ignore custom `"PNG"` formats. `write_image` currently publishes `"PNG"` for non-SVG images, leaving apps that only read `CF_DIB` unable to paste.

Additionally, `convert_dib_to_bmp` (used when reading `CF_DIB` back):
1. Adds `12` bytes of `BI_BITFIELDS` color masks whenever `compression == 3`, even when `header_size > 40` (`BITMAPV4HEADER` / `BITMAPV5HEADER` already embed the masks within `header_size`), shifting the pixel offset by 12 bytes.
2. Uses unchecked `u32` addition/multiplication for `color_table_size`, `pixel_offset`, and `file_size`, and does not reject malformed DIB buffers whose computed `pixel_offset > file_size`.

This patch:
- decodes non-SVG raster images once (`decode_image`) and encodes both `CF_DIB` (stripping the 14-byte BMP file header) and PNG alongside native formats in `write_image`;
- narrows the `BI_BITFIELDS` 12-byte mask adjustment to `compression == 3 && header_size == 40` while honoring `colors_used * 4` palette entries separately for all headers (including `16`/`32`-bit extended v4/v5 DIBs with non-zero `biClrUsed`);
- uses checked arithmetic (`checked_mul`, `checked_add`, `u32::try_from`) in `convert_dib_to_bmp` and returns `None` if `pixel_offset > file_size`;
- adds unit tests covering embedded-mask v4/v5 DIBs, appended-mask v3 `BITMAPINFOHEADER` DIBs, bottom-up and top-down v4 32-bit DIBs with explicit palette entries (`biClrUsed > 0`), PNG-to-DIB pixel fidelity (8-bit and 16-bit PNGs), overflowing headers, and an `#[ignore]`d native clipboard round-trip test gated on `GPUI_ISOLATED_CLIPBOARD_TEST=1`.

Motivation: HeroGPUI and downstream Windows apps (including Poratake) copying images to the Windows clipboard for interoperability with native Windows consumers.

Testing:
- `cargo test -p gpui_windows` (5 pure-Rust `convert_to_dib` / `convert_dib_to_bmp` unit tests run unconditionally);
- on an isolated Windows session: `GPUI_ISOLATED_CLIPBOARD_TEST=1 cargo test -p gpui_windows native_clipboard_round_trips_text_png_and_dib -- --ignored`, plus manual paste verification into third-party Windows apps.

Release Notes:

- N/A

---

HeroGPUI follow-up once released: remove the corresponding local `clipboard.rs` workaround in downstream Windows forks (`gpui-pre-windows`).
