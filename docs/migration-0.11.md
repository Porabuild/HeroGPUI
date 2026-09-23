# Migrating from 0.10 to 0.11

0.11.0 moves the parity target to HeroUI v3.2.6 and hardens the public API.
This guide covers only the changes that can break a 0.10 build or change
what a callback receives. The full list, including additions, is in
[`CHANGELOG.md`](../CHANGELOG.md#0110---2026-09-22).

Update the dependency first:

```toml
herogpui = "0.11"
```

## `use_theme` and `ThemeProvider::set_active` return `Result`

An unregistered id is now refused and leaves the active theme untouched. In
0.10 it was installed anyway and the next frame panicked. Ids are
case-sensitive.

```rust
// 0.10
use_theme("dark", cx);

// 0.11
use_theme("dark", cx)?; // or .expect(..), or handle UnknownThemeError
// A built-in id you know is registered:
let _ = use_theme("dark", cx);
```

`UnknownThemeError { id }` carries the rejected id.

## `Modal::on_close` / `Drawer::on_close` receive `DismissReason`

The callback no longer gets a `&ClickEvent`. Escape and backdrop dismissal
used to fabricate one with `ClickEvent::default()`. `modal::OnClose` changes
to match.

```rust
// 0.10
Modal::new("m").on_close(|_ev: &ClickEvent, window, cx| { /* ... */ })

// 0.11
Modal::new("m").on_close(|reason: &DismissReason, window, cx| {
    match reason {
        DismissReason::Backdrop => { /* e.g. confirm unsaved changes */ }
        _ => { /* CloseButton, Escape, Drag */ }
    }
})
```

`DismissReason` is `#[non_exhaustive]`, so a `match` needs a `_` arm.

## `#[non_exhaustive]` theme types, `TabItem` and render-prop payloads

`Theme`, `ThemeColors`, `FieldColors`, `LayoutTheme`, `ComponentThemes`,
`ComponentTheme`, the `*Style` recipes (`ButtonStyle`, `SliderStyle`,
`SwitchStyle`, `SelectStyle`, `MenuStyle`, `TextFieldStyle`), `TabItem` and
every render-prop payload (`InteractiveState`, `FieldFocus`,
`SelectionValue`, the `*State` / `*RenderState` structs) can no longer be
built with a struct literal outside HeroGPUI, not even with
`..Default::default()`. Start from a constructor, builder or `Default`, then
assign fields. Reading fields and destructuring with `..` still work.

```rust
// 0.10
let theme = Theme { id: "brand".into(), ..Theme::light() };

// 0.11
let mut theme = Theme::light();
theme.id = "brand".into();

let button = ButtonStyle::default().radius(px(4.)); // recipe builders
let tab = TabItem::new("a", "A").is_disabled(true); // TabItem builders
```

A `match` or `let` on a payload needs a trailing `..`:

```rust
let CheckboxState { is_selected, .. } = state;
```

## `herogpui_components::util` is private

The helpers supported for custom widgets move to `extend`
(`herogpui::extend`). The render-prop payloads move to the crate root.

```rust
// 0.10
use herogpui_components::util::{control_radius, focus_visible, FieldFocus};

// 0.11
use herogpui::extend::{control_radius, focus_visible};
use herogpui::FieldFocus;
```

`extend` holds `control_radius`, `soft_radius`, `small_radius`,
`mark_radius`, `key_radius`, `hairline_radius`, `micro_radius`,
`field_radius`, `container_radius`, `FIELD_HEIGHT`, `FIELD_TEXT`,
`FIELD_ICON`, `cursor_interactive`, `interactive_cursor`, `focus_visible`,
`set_focus_visible`, `inner_fill_radius`, `shift_wheel_scroll_x` and
`app_focus_root`. Every other former `util` item is no longer public, and
`prominence_bg`, `placed_panel`, `placed_field_panel` and `focusable` are
deleted.

## Motion API lives under `anim`

`pub use anim::*` is gone from the crate root.

```rust
// 0.10
use herogpui::TRANSITION_MS;

// 0.11
use herogpui::anim::TRANSITION_MS;
```

## `Separator` takes no children

The content mode (`Separator::new().child("OR")`) is removed because v3's
separator is childless. Lay out the label between two separators instead:

```rust
// 0.10
Separator::new().child("OR")

// 0.11
div()
    .flex()
    .items_center()
    .gap_2()
    .child(div().flex_1().child(Separator::new()))
    .child("OR")
    .child(div().flex_1().child(Separator::new()))
```

## Behavior changes that need no code change

- Backspace, Delete and Left/Right in text fields step by grapheme cluster,
  so an emoji, a flag or a letter with combining marks is one caret stop.
- Component chrome strings resolve through `i18n`. The en-US output is
  unchanged unless you call `i18n::set_locale`.
- `herogpui::gpui` is documented as the stable path to GPUI items.
  `use herogpui::*;` still carries all of GPUI.
