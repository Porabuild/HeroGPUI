//! HeroUI design tokens and theming for GPUI.
//!
//! This crate is a faithful port of HeroUI v3's `packages/styles`: the semantic
//! OKLCH color tokens for the light and dark appearances, the layout tokens
//! (radius, border width, shadows), and a global [`ThemeProvider`] with an
//! [`ActiveTheme`] accessor trait.

mod layout;
mod provider;
mod semantic;
mod theme;

pub use layout::*;
pub use provider::*;
pub use semantic::*;
pub use theme::*;
