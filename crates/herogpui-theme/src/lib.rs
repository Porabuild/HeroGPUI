//! HeroUI design tokens and theming for GPUI.
//!
//! This crate is a faithful port of HeroUI v3's `packages/styles`: the semantic
//! OKLCH color tokens for the light and dark appearances, the layout tokens
//! (radius, border width, shadows), and a global [`ThemeProvider`] with an
//! [`ActiveTheme`] accessor trait.

mod components;
mod layout;
mod provider;
mod semantic;
mod theme;
#[cfg(feature = "serde")]
mod theme_document;
#[cfg(feature = "serde")]
mod theme_registry;

pub use components::*;
pub use layout::*;
pub use provider::*;
pub use semantic::*;
pub use theme::*;
#[cfg(feature = "serde")]
pub use theme_document::{
    ColorPair, RoleOverride, Roles, SurfaceLevels, ThemeDocument, ThemeDocumentError,
};
#[cfg(feature = "serde")]
pub use theme_registry::{
    load_themes_dir, presets, register_theme_json, ThemeLoadError, THEME_SCHEMA,
};
