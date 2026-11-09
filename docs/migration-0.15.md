# Migrating from 0.14 to 0.15

This guide covers only the changes in 0.15 that can break a 0.14 build or
change behavior an application relied on. The full list, including
additions, is in [`CHANGELOG.md`](../CHANGELOG.md).

Update the dependency first:

```toml
herogpui = "0.15"
```

## Upgrade checklist

Work through these as you bump the dependency:

- **Build changes** — your code may stop compiling:
  [Rust 1.99 and GPUI `gpui-pre` 0.3.11](#rust-199-and-gpui-gpui-pre-0311),
  [`is_command_palette_shortcut`](#is_command_palette_shortcut-takes-app) and
  [`shift_wheel_scroll_x`](#extendshift_wheel_scroll_x-takes-scrollevent-and-mut-window)
  signature changes, and
  [`Checkbox`/`Switch` `'static` labels](#checkbox-and-switch-labels-must-be-static).
- **Visible changes to check** — walk the affected screens:
  [Drawer handle](#drawer-draws-its-handle-only-when-composed),
  [Calendar year picker](#calendar-and-rangecalendar-the-year-picker-is-opt-in),
  [DateField hint](#datefield-no-longer-shows-a-format-hint),
  [floating popovers and InputGroup insets](#layout-floating-popovers-and-inputgroup-insets),
  and the hover-only Toast stack
  ([interaction changes](#list-and-overlay-interaction-changes)).
- **Behaviour tests may pin** — replay key sequences and pointer paths:
  [validation timing](#validation-errors-wait-for-a-commit),
  [list/overlay keyboard and pointer behaviour](#list-and-overlay-interaction-changes),
  and the animation clock
  ([GPUI 0.3.11](#rust-199-and-gpui-gpui-pre-0311)).
- **Recommended** —
  [name icon-only controls](#recommended-name-icon-only-controls).

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

## Checkbox and Switch labels must be `'static`

`Checkbox::label` and `Switch::label` now take `impl IntoElement + 'static`
(0.14: `impl IntoElement`) so the component can read a plain-string label as
the accessible name. Strings, `SharedString` and GPUI elements are already
`'static`, so most code compiles unchanged; a label type that borrows data
needs owned data (`SharedString`/`String`).

A plain string label now also becomes the control's accessible name, so a
screen reader announces it. A composed element label does not — the element
could be anything — so give those an explicit `aria_label`:

```rust
// a string label names the checkbox
Checkbox::new("agree").label("I agree to the terms");

// an element label leaves the name empty — aria_label supplies it
Checkbox::new("agree")
    .label(div().child("I agree"))
    .aria_label("Agree to the terms");
```

`Switch::label` behaves the same way.

## Drawer draws its handle only when composed

In 0.14 every Drawer painted its drag handle automatically. In 0.15 the
handle is the opt-in `DrawerHandle` part — compose it where you want the
drag bar, the way HeroUI v3.2.7's examples compose `Drawer.Handle`.
Without it the drawer has no handle and no drag-to-dismiss affordance:

```rust
// 0.14 — the handle was always drawn
Drawer::new()
    .id("filters")
    .is_open(open)
    .child("Filters");

// 0.15 — compose DrawerHandle to draw it
Drawer::new()
    .id("filters")
    .is_open(open)
    .child(DrawerHandle::new())
    .child("Filters");
```

## Calendar and RangeCalendar: the year picker is opt-in

In 0.14 a `Calendar` or `RangeCalendar` heading was always the interactive
year-picker trigger. In 0.15 a plain heading is a `Calendar.Heading` label;
compose the picker with `.year_picker(true)`, or use the open-state props —
`is_year_picker_open`, `default_year_picker_open` and
`on_year_picker_open_change` imply it, since a picker state with no trigger
could never open. `DatePicker` and `DateRangePicker` turn it on themselves,
matching their v3 popovers, so picker users need no change:

```rust
// 0.14 — the heading always opened the year grid
Calendar::new(state).default_value(date)

// 0.15 — opt in to the trigger
Calendar::new(state).year_picker(true).default_value(date)

// or drive it controlled — these imply year_picker(true)
Calendar::new(state)
    .is_year_picker_open(open)
    .on_year_picker_open_change(handler)
```

`RangeCalendar` takes the same builders.

## DateField no longer shows a format hint

In 0.14 a `DateField` with no description of its own showed the locale's
date-format hint (for example "MM/DD/YYYY") as its description. In 0.15 it
shows nothing unless you set one. Restore a fixed string with
`.description(..)` — note it does not follow the locale the way the old
default did:

```rust
DateField::new(state).description("MM/DD/YYYY")
```

## Layout: floating popovers and InputGroup insets

Two layout changes can move pixels an app positioned by hand:

- Popover panels now float above later page content instead of reserving
  layout height, matching how an absolutely-positioned popup behaves. If you
  stacked content under a popover expecting it to push the page down, the
  page no longer moves.
- `InputGroup` prefix and suffix slots now own the 12px inset, so an icon or
  text addon is padded like the field's own text. `InputAddon` looks the
  same because its own padding moved to the slot; the change shows only if
  you passed a custom element (say, an icon) that added its own horizontal
  padding to line up with the input — drop that padding, the slot supplies
  it now:

```rust
// 0.14 — the app padded its own icon addon to match the field inset
InputGroup::new()
    .label("Website")
    .prefix(gpui::div().px(px(12.)).child(Icon::new(IconName::Globe)))
    .input(input)

// 0.15 — the slot owns the inset; pass the bare icon
InputGroup::new()
    .label("Website")
    .prefix(Icon::new(IconName::Globe))
    .input(input)
```

## List and overlay interaction changes

Select, ComboBox, Dropdown, Autocomplete and the overlay stack now match
React Aria's keyboard, pointer and typeahead behaviour. Most apps see no
difference; tests that replay exact key sequences or assert hover/cursor
state may need updating:

- Select, ComboBox and Dropdown open with the selected item highlighted, as
  React Aria's list autofocus does. With no selection, Select and Dropdown
  open onto the first item from Enter, Space or ArrowDown, and onto the last
  from ArrowUp.
- The Dropdown trigger opens on ArrowDown and ArrowUp, and on the Alt+Arrow
  chords; a long-press trigger opens from the keyboard only through its Alt
  chords.
- Hovering a Select, ComboBox or Autocomplete option moves the keyboard
  cursor to it, unless the cursor was placed from the keyboard.
- Removing the highlighted item from an open Select, ComboBox or
  Autocomplete moves the highlight to the next enabled item, or the previous
  one, instead of clearing it.
- Dropdown's list cursor follows its item by key: it stays on the same row
  when items reorder and clears when that row becomes disabled.
- List typeahead ignores Ctrl, Alt and platform chords, so a shortcut like
  Ctrl+B is no longer read as typing the letter B.
- A press outside an open ComboBox closes it and also reaches the control
  underneath, matching React Aria's non-modal popover.
- The Toast stack expands only while the pointer is over it or focus is
  inside it, and collapses when both leave.

## Recommended: name icon-only controls

GPUI derives no accessible name from an element's text children, so a button
that shows only an icon is announced as nothing. `aria_label(..)` sets the
name explicitly — it was added in 0.15 to Button, ToggleButton, Checkbox,
Switch, Select, Slider, ColorSlider, ColorField, InputOTP, Input, TextField
and TextArea, and it wins over any derived label:

```rust
Button::new("search").aria_label("Search")
```

Give every icon-only control one.
