# Migrating from 0.12 to 0.13

This guide covers only the changes in 0.13 that can break a 0.12 build or
change behavior an application relied on. The full list, including
additions, is in [`CHANGELOG.md`](../CHANGELOG.md).

Update the dependency first:

```toml
herogpui = "0.13"
```

## Theme roles are the typed `Color`

`ThemeBuilder::role`, `ThemeBuilder::role_hover` and `ThemeColors::role` took
the role as a `&str`, and any name they did not know (a typo such as
`"sucess"`, or v2's `"primary"`) silently meant `accent`: the builder
recoloured the accent and the focus ring. They now take
`herogpui::core::Color`, so a misspelt role does not compile.

```rust
// 0.12
Theme::builder("brand", Theme::light())
    .role("success", oklch(0.73, 0.19, 150.0), snow())
    .role_hover("accent", hover);
let soft = cx.colors().role("danger").soft();

// 0.13
use herogpui::core::Color;
Theme::builder("brand", Theme::light())
    .role(Color::Success, oklch(0.73, 0.19, 150.0), snow())
    .role_hover(Color::Accent, hover);
let soft = cx.colors().role(Color::Danger).soft(); // or cx.role(Color::Danger)
```

A role name that arrives as a string (a settings file, a CLI flag) parses
with `FromStr`, which fails on an unknown name instead of falling back:

```rust
let role: Color = name.parse()?; // Err(UnknownColorError { name })
```

`Color::from_token(&str) -> Option<Color>` is the same parse without the
error type. Theme JSON is unchanged: the `roles` map already rejected an
unknown key, and still does.

## `HEROGPUI_REDUCE_MOTION` is no longer read by the library

`ThemeProvider::init` read the `HEROGPUI_REDUCE_MOTION` environment variable
and wrote GPUI's reduced-motion flag from it, overwriting any value the app
had set before initializing. A library reading a hidden configuration
channel is surprising, so it no longer does: `init` leaves the flag alone,
and `set_reduce_motion` is the one way to set it. The gallery keeps
honouring the variable by mapping it itself.

An app that relied on the variable maps it (or, better, its own setting)
explicitly:

```rust
// 0.12: HEROGPUI_REDUCE_MOTION=1 was honoured by ThemeProvider::init.
ThemeProvider::init(cx);

// 0.13
ThemeProvider::init(cx);
if std::env::var("HEROGPUI_REDUCE_MOTION").is_ok_and(|v| v != "0" && v != "false") {
    herogpui::theme::set_reduce_motion(true, cx);
}
```
