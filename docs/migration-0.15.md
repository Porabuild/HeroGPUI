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

## `extend::shift_wheel_scroll_x` takes `&ScrollEvent` and `&mut Window`

Following GPUI's rename, the helper's event parameter is the value an
`on_scroll` handler receives. It also takes the handler's window: moving a
`ScrollHandle` offset does not repaint by itself, so in 0.14 a shift+wheel
in the browser moved the content only at the next unrelated redraw.

```rust
// 0.14
.on_scroll_wheel(move |event, _window, _cx| {
    shift_wheel_scroll_x(&handle, event);
})

// 0.15
.on_scroll(move |event, window, _cx| {
    shift_wheel_scroll_x(&handle, event, window);
})
```

## Cmd-vs-Ctrl follows the runtime platform

Select-all and the Home/End chords in `Select`, `ListBox`, `TagGroup` and
`Table` used to pick the macOS convention at compile time
(`cfg!(target_os = "macos")`). They now read `App::platform_info()` at
runtime, matching React Aria: the WebAssembly build running on a Mac uses
Cmd, and the same build elsewhere uses Ctrl. Native builds are unchanged.

## Validation errors wait for a commit

Under the default `ValidationBehavior::Native`, a field's `validate` result
and its built-in constraint errors (a date outside `min_value`/`max_value`,
an unavailable date) used to paint as soon as the field rendered. They now
appear once the user commits the field, as React Aria's do: changing the
value and leaving the field, Enter or a step in NumberField and ColorField,
a toggle in Checkbox and Switch, a calendar pick in DatePicker and
DateRangePicker, or submitting the form. This applies to every field that
takes `validate`, the text fields included, except `InputOTP`: HeroUI's
InputOTP has no React Stately validation state, so its errors stay live.

Submission is unchanged: a native form still blocks on the live result, so
an untouched invalid field still stops the submit and receives focus, and
the submit then shows its error. Controlled `is_invalid` and server
`validation_errors` still show immediately.

If a screen or a test relied on seeing a `validate` message at rest, commit
the field first (interact with it, then move focus away) or pass
`.validation_behavior(ValidationBehavior::Allow)`, which shows every error
live and does not block submission.
