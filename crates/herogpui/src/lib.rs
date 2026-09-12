//! # HeroGPUI
//!
//! Beautiful, fast and modern cross-platform Rust UI library — a faithful port
//! of [HeroUI v3](https://github.com/heroui-inc/heroui) to
//! [GPUI](https://gpui.rs).
//!
//! This crate is the single dependency an application needs. GPUI itself ships
//! as a family of crates that move together (`gpui-pre`,
//! `gpui-pre-platform`, and their per-platform backends); this
//! crate depends on the matching set and re-exports it, so `Cargo.toml` names
//! `herogpui` alone and nothing else. `use herogpui::*;` **is** GPUI, and each
//! layer is reachable by name:
//!
//! | Path                        | Crate                 | Feature                    |
//! | --------------------------- | --------------------- | -------------------------- |
//! | `herogpui::*`               | `gpui`                | always                     |
//! | [`platform`], [`application`] | `gpui_platform`     | always                     |
//! | `web`                       | `gpui_platform`       | always, `cfg(wasm)` only   |
//! | [`core`]                    | `herogpui-core`       | `core` (via `theme`)       |
//! | [`theme`]                   | `herogpui-theme`      | `theme` (via `components`) |
//! | [`components`], `herogpui::*` | `herogpui-components` | `components` (**on**)    |
//!
//! Two features carry no layer of their own and only forward to GPUI:
//!
//! | Feature        | Forwards to                                          |
//! | -------------- | ---------------------------------------------------- |
//! | `test-support` | `gpui/test-support`, `gpui_platform/test-support`     |
//! | `profiler`     | `gpui/profiler`                                      |
//! | `serde`        | `herogpui-theme/serde` (`ThemeDocument`)              |
//!
//! There is no separate assets or icons feature: the SVG icons HeroGPUI's own
//! component chrome draws are `&'static str` constants inside
//! `herogpui-components` (see [`components::assets`]), not a separable crate,
//! so they arrive with the `components` feature and cost nothing to a build
//! that leaves that feature off.
//!
//! ## A whole application, one dependency
//!
//! [`application`] opens the platform, [`init`] initializes the enabled
//! layers, and [`HeroGpuiAssets`] supplies the icons. Nothing below names
//! `gpui` or `gpui_platform` as a dependency:
//!
//! ```
//! use herogpui::*;
//!
//! actions!(demo, [Quit]);
//!
//! struct Hello;
//!
//! impl Render for Hello {
//!     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
//!         app_focus_root(
//!             div()
//!                 .size_full()
//!                 .bg(cx.colors().background)
//!                 .text_color(cx.colors().foreground)
//!                 .child(Button::new("save").label("Save changes")),
//!             window,
//!             cx,
//!         )
//!     }
//! }
//!
//! fn open() {
//!     application().with_assets(HeroGpuiAssets).run(|cx: &mut App| {
//!         herogpui::init(cx);
//!         let bounds = Bounds::centered(None, size(px(1280.), px(820.)), cx);
//!         cx.open_window(
//!             WindowOptions {
//!                 window_bounds: Some(WindowBounds::Windowed(bounds)),
//!                 ..Default::default()
//!             },
//!             |_, cx| cx.new(|_| Hello),
//!         )
//!         .expect("failed to open window");
//!     });
//! }
//!
//! fn main() {
//!     // `open` is compiled, not run: a doctest must not open a real window.
//!     let _ = open;
//! }
//! ```
//!
//! ## Two globs, one root
//!
//! The root re-exports GPUI *and* the component library, and eight names exist
//! in both: `ColorSpace`, `FontWeight`, `Menu`, `MenuItem`, `Orientation`,
//! [`Size`], `Surface` and `TextAlign`. At this crate's root the HeroUI v3
//! spelling wins, because that is the one the component builders take —
//! [`Size`] is HeroUI's `sm`/`md`/`lg` scale, not a geometric width/height
//! pair. GPUI's eight keep the `gpui::` path they already have everywhere
//! else: `gpui::Size`, `gpui::FontWeight`, and so on.
//!
//! ```
//! use herogpui::*;
//!
//! // HeroUI's scale at the root...
//! assert_eq!(Size::default(), Size::Md);
//! // ...and GPUI's geometry a path away.
//! let extent: gpui::Size<Pixels> = size(px(320.), px(240.));
//! assert_eq!(extent.width, px(320.));
//! ```
//!
//! [`prelude`] is the narrow import: it carries GPUI's *prelude* rather than
//! all of GPUI, so nothing in it collides with anything else in it.
//!
//! ```
//! use herogpui::prelude::*;
//!
//! assert_eq!(Size::default(), Size::Md);
//! ```

// Everything in GPUI itself, so `use herogpui::*;` is enough to get started.
// With the `test-support` feature the glob also carries GPUI's `test`
// attribute macro, so a test module has to import explicitly (or add
// `use core::prelude::v1::test;`) to keep the built-in `#[test]`.
pub use ::gpui::*;

// The crate name, so code written with `gpui::…` paths still resolves when
// GPUI is reached only through this facade. It is not decoration: *every*
// derive macro in GPUI's `gpui_macros` surface -- `Action`, `IntoElement`,
// `Render`, `AppContext`, `VisualContext`, `register_action` -- expands to
// hard-coded absolute `gpui::…` paths (`gpui::IntoElement`,
// `gpui::private::inventory`, ...). A proc macro cannot spell `$crate`, so
// there is nothing upstream can do about it and nothing this crate can rewrite
// either. `use herogpui::*;` puts the name `gpui` in the caller's module, which
// is what makes those expansions resolve. `herogpui::*` is the documented way;
// a caller who imports selectively instead needs `use herogpui::gpui;` beside
// it before deriving.
#[doc(hidden)]
pub use ::gpui;
pub use ::gpui_platform as platform;
pub use ::gpui_platform::application;

/// The layer this facade adds on top of GPUI: HeroUI v3's components.
///
/// Every item here is also re-exported at the crate root.
#[cfg(feature = "components")]
pub use ::herogpui_components as components;
#[cfg(feature = "components")]
pub use ::herogpui_components::*;
// The eight names GPUI and HeroUI v3 both spell. Two globs would leave each of
// these an ambiguous re-export that silently resolves to whichever glob rustc
// read first (GPUI's), so every one is named explicitly here -- an explicit
// re-export shadows a glob -- and HeroGPUI wins at HeroGPUI's own root. That is
// the direction the component builders need: `Button::new("x").size(Size::Sm)`
// takes HeroUI's `sm`/`md`/`lg` scale, not a geometric width/height pair. GPUI's
// counterparts keep the `gpui::` path they already have in GPUI's own docs and
// in every source file in this repository: `gpui::Size`, `gpui::FontWeight`,
// `gpui::TextAlign`, `gpui::Menu`, `gpui::MenuItem`, `gpui::Surface`,
// `gpui::ColorSpace`, `gpui::Orientation`.
#[cfg(feature = "components")]
pub use ::herogpui_components::{ColorSpace, FontWeight, Menu, MenuItem, Surface, TextAlign};
/// HeroUI v3's shared vocabularies and OKLCH colour math. No GPUI state.
#[cfg(feature = "core")]
pub use ::herogpui_core as core;
// The v3 vocabulary at the root. `Size` and `Orientation` are two of the eight
// shadowed names above and are listed here rather than beside the other six
// because they live in `herogpui-core`, so they reach the root with the `core`
// feature and without any of the components.
#[cfg(feature = "core")]
pub use ::herogpui_core::{
    Backdrop, Color, CurrencySign, FieldVariant, NumberFormat, NumberStyle, Orientation, Placement,
    Prominence, SelectionMode, Size, SizeXl, UnitDisplay, Variant,
};
/// The semantic and layout token layer. [`ThemeProvider`] is a GPUI global;
/// read it through [`ActiveTheme`].
#[cfg(feature = "theme")]
pub use ::herogpui_theme as theme;
// `cx.colors()` is a trait method, so `ActiveTheme` has to be in scope for the
// root glob to be usable at all; the rest is what an application calls to
// install or switch a theme.
#[cfg(feature = "theme")]
pub use ::herogpui_theme::{
    follow_system_appearance, set_reduce_motion, set_theme, stop_following_system_appearance,
    toggle_light_dark, toggle_reduce_motion, use_theme, ActiveTheme, Appearance, ButtonStyle,
    ComponentColor, ComponentTheme, ComponentThemes, MenuStyle, SelectStyle, SliderStyle,
    SwitchStyle, TextFieldStyle, Theme, ThemeProvider,
};
#[cfg(feature = "serde")]
pub use ::herogpui_theme::{ThemeDocument, ThemeDocumentError};

/// GPUI's web platform entry points, on `wasm32` only.
///
/// `gpui_platform` grows these items under `cfg(target_family = "wasm")` and
/// pulls in GPUI's web backend itself, so this module is a rename rather than
/// a dependency of its own — it is inert on every native target, and it does
/// not depend on the in-progress `herogpui-web` browser entry crate.
#[cfg(target_family = "wasm")]
pub mod web {
    pub use ::gpui_platform::{
        application_with_web_backend, single_threaded_web, web_init, WebBackendPreference,
    };
}

/// Defines unit-struct actions without requiring the caller to depend on GPUI
/// under the crate name `gpui`.
///
/// GPUI's own `actions!` expands its derive as the absolute path
/// `gpui::Action`, which resolves only where the caller has a crate or import
/// literally named `gpui` in scope. Through this facade that is true after
/// `use herogpui::*;` and false after a selective import, so the macro would
/// work or break depending on how the caller wrote an unrelated `use` line.
/// This copy spells the derive `$crate::Action`, which resolves from anywhere.
///
/// ```
/// use herogpui::actions;
///
/// actions!(my_app, [Save, Quit]);
///
/// # fn main() {
/// use herogpui::Action as _;
/// assert_eq!(Save.name(), "my_app::Save");
/// # }
/// ```
#[macro_export]
macro_rules! actions {
    ($namespace:path, [ $( $(#[$attr:meta])* $name:ident),* $(,)? ]) => {
        $(
            #[derive(
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
                ::std::default::Default,
                ::std::fmt::Debug,
                $crate::Action
            )]
            #[action(namespace = $namespace)]
            $(#[$attr])*
            pub struct $name;
        )*
    };
    ([ $( $(#[$attr:meta])* $name:ident),* $(,)? ]) => {
        $(
            #[derive(
                ::std::clone::Clone,
                ::std::cmp::PartialEq,
                ::std::default::Default,
                ::std::fmt::Debug,
                $crate::Action
            )]
            $(#[$attr])*
            pub struct $name;
        )*
    };
}

/// Initializes every enabled layer. Call it once, inside
/// [`application`]`().run(..)`, before opening a window.
///
/// With the `theme` feature — which `components`, and therefore the default
/// feature set, turns on — this registers the light and dark themes as a GPUI
/// global, exactly as [`theme::ThemeProvider::init`] does. Use
/// [`theme::ThemeProvider::init_with`] instead of this function to start from a
/// custom [`theme::Theme`]. Without `theme` there is nothing to initialize and
/// this is a no-op, so an application can call it unconditionally.
///
/// The embedded icons are registered separately, on the `Application` rather
/// than the `App`: `application().with_assets(HeroGpuiAssets)`.
#[cfg_attr(not(feature = "theme"), expect(unused_variables))]
pub fn init(cx: &mut App) {
    #[cfg(feature = "theme")]
    ::herogpui_theme::ThemeProvider::init(cx);
}

/// Convenience prelude re-exporting the most-used items.
///
/// This is the narrow import: GPUI's own prelude (its traits, `div`, and the
/// styling methods) plus HeroGPUI's components and vocabulary — but not the
/// whole of GPUI, so nothing here is ambiguous with anything else here.
pub mod prelude {
    pub use gpui::prelude::*;
    #[cfg(feature = "components")]
    pub use herogpui_components::*;
    #[cfg(feature = "core")]
    pub use herogpui_core::{
        Backdrop, Color, CurrencySign, FieldVariant, NumberFormat, NumberStyle, Orientation,
        Placement, Prominence, SelectionMode, Size, SizeXl, UnitDisplay, Variant,
    };
    #[cfg(feature = "theme")]
    pub use herogpui_theme::{
        follow_system_appearance, set_reduce_motion, set_theme, stop_following_system_appearance,
        toggle_light_dark, toggle_reduce_motion, use_theme, ActiveTheme, Appearance, ButtonStyle,
        ComponentColor, ComponentTheme, ComponentThemes, MenuStyle, SelectStyle, SliderStyle,
        SwitchStyle, TextFieldStyle, Theme, ThemeProvider,
    };
}
