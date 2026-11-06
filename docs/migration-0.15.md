# Migrating from 0.14 to 0.15

This guide covers only the changes in 0.15 that can break a 0.14 build or
change behavior an application relied on. The full list, including
additions, is in [`CHANGELOG.md`](../CHANGELOG.md).

Update the dependency first:

```toml
herogpui = "0.15"
```

## Rust 1.99 and GPUI `gpui-pre` 0.3.11

The workspace (and the published crates' `rust-version`) moved from Rust
1.98 to 1.99, and the re-exported GPUI moved from `gpui-pre` 0.3.5 to
0.3.11 (`zed@f816e1f`). Code that names GPUI types through `herogpui::*`
sees the 0.3.11 API: among others, `ScrollWheelEvent` is now `ScrollEvent`
(with `on_scroll_wheel` renamed to `on_scroll`), `Modifiers::secondary()`
takes `&App`, `Keystroke::parse` takes a `&'static str` (use
`Keystroke::parse_with_os` for runtime strings), and `Animation` measures
time on the background executor's clock instead of the wall clock — in
tests, `executor().advance_clock(..)` now drives animations
deterministically, and sleeping the thread no longer does.

On macOS the `block` 0.1.6 future-incompatibility warning is gone: the
0.3.11 macOS graph contains no `metal`, `cocoa` or `block`.

## `is_command_palette_shortcut` takes `&App`

GPUI's `Modifiers::secondary()` now resolves Cmd-vs-Ctrl from the runtime
platform, so the helper needs the app context:

```rust
// 0.14
if is_command_palette_shortcut(&event.keystroke) { /* ... */ }

// 0.15
if is_command_palette_shortcut(&event.keystroke, cx) { /* ... */ }
```

## `extend::shift_wheel_scroll_x` takes `&ScrollEvent`

Following GPUI's rename, the helper's event parameter is the value an
`on_scroll` handler receives:

```rust
// 0.14
.on_scroll_wheel(move |event, _window, _cx| {
    shift_wheel_scroll_x(&handle, event);
})

// 0.15
.on_scroll(move |event, _window, _cx| {
    shift_wheel_scroll_x(&handle, event);
})
```

## Cmd-vs-Ctrl follows the runtime platform

Select-all and the Home/End chords in `Select`, `ListBox`, `TagGroup` and
`Table` used to pick the macOS convention at compile time
(`cfg!(target_os = "macos")`). They now read `App::platform_info()` at
runtime, matching React Aria: the WebAssembly build running on a Mac uses
Cmd, and the same build elsewhere uses Ctrl. Native builds are unchanged.
