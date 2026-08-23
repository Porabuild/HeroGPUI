//! # HeroGPUI
//!
//! Beautiful, fast and modern cross-platform Rust UI library — a faithful port
//! of [HeroUI v3](https://github.com/heroui-inc/heroui) to
//! [GPUI](https://gpui.rs).
//!
//! This crate is the umbrella facade (like `@heroui/react`):
//!
//! ```no_run
//! use herogpui::prelude::*;
//!
//! // herogpui::components::* — Button, Card, Modal, ...
//! // herogpui::theme::*       — ThemeProvider, ActiveTheme, tokens
//! ```
//!
//! See `crates/herogpui-components` for the full component list and
//! `crates/herogpui-theme` for the token system.

pub use herogpui_components as components;
pub use herogpui_core as core;
pub use herogpui_theme as theme;

/// Convenience prelude re-exporting the most-used items.
pub mod prelude {
    pub use gpui::prelude::*;
    pub use herogpui_components::*;
    pub use herogpui_core::{
        Backdrop, Color, CurrencySign, FieldVariant, NumberFormat, NumberStyle, Orientation,
        Placement, Prominence, SelectionMode, Size, SizeXl, UnitDisplay, Variant,
    };
    pub use herogpui_theme::{
        set_theme, toggle_light_dark, use_theme, ActiveTheme, Appearance, Theme, ThemeProvider,
    };
}
