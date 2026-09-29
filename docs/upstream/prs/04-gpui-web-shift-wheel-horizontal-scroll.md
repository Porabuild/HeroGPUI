# 04: gpui_web: Treat shift+wheel as horizontal scrolling

Patch: [`04-gpui-web-shift-wheel-horizontal-scroll.patch`](04-gpui-web-shift-wheel-horizontal-scroll.patch).
Base: `zed@d89e9c2`. File: `crates/gpui_web/src/events.rs` (+11/-5).

`git apply --check`: OK on `/tmp/zedapply` and on `zed@d89e9c2`.
`cargo +nightly check -p gpui-pre-web --target wasm32-unknown-unknown`
passes. `rustfmt --check` is clean.

Still needed at d89e9c2: yes. `register_wheel` builds `ScrollDelta` from the
raw `deltaX`/`deltaY` whatever the modifiers.

---

**PR title:** `gpui_web: Treat shift+wheel as horizontal scrolling`

**PR body:**

A mouse wheel reports only `deltaY`. On every desktop platform, shift+wheel
scrolls horizontally, and GPUI's native platforms already implement that:

- `crates/gpui_windows/src/events.rs` (the `WM_MOUSEWHEEL` handler switches
  axis on `modifiers.shift`);
- `crates/gpui_linux/src/linux/wayland/client.rs` and
  `crates/gpui_linux/src/linux/x11/client.rs` ("When shift is held down,
  vertical scrolling turns into horizontal scrolling");
- on macOS, AppKit delivers shift+wheel as `scrollingDeltaX`.

The web platform forwards the DOM axes unchanged. Whether shift+wheel
arrives horizontal therefore depends on the browser and OS: some combinations
already report it as `deltaX`, and others deliver a plain `deltaY` with
`shiftKey` set. In the second case, a horizontally scrollable GPUI region (an
overflowing tab strip, a wide table, a code block) cannot be scrolled with a
mouse wheel at all.

This swaps the axes in `register_wheel` when shift is held **and**
`deltaX == 0.0`. A trackpad's real horizontal gesture, and browsers that
already swap the axes for shift+wheel, report a non-zero `deltaX`, so they
are left untouched. The change cannot double-swap and is a no-op everywhere
except the wheel-only case. It applies to both `DOM_DELTA_LINE` and
`DOM_DELTA_PIXEL`. `modifiers` is already computed a few lines above
(`modifiers_from_wheel_event`), so no new helper or import is needed.

Motivation: HeroGPUI, a GPUI component library whose browser gallery runs on
`gpui_web`. Its Tabs overflow, Table and code-block regions were unreachable
with a mouse wheel in the browser. It carried this change in a local fork of
`gpui_web`, and now works around it per component
(`crates/herogpui-components/src/util.rs`, `shift_wheel_scroll_x`, compiled
only for wasm32).

Testing: in a browser build of a GPUI app with a horizontal
`overflow_x_scroll()` region:

- mouse wheel with shift scrolls it horizontally;
- mouse wheel without shift still scrolls vertically;
- a trackpad horizontal swipe is unchanged, with and without shift;
- a browser that already reports shift+wheel as `deltaX` behaves as before.

Native targets are not affected (`gpui_web` is wasm-only).

Release Notes:

- N/A

---

HeroGPUI follow-up once released: delete `util::shift_wheel_scroll_x` and
its call sites.
