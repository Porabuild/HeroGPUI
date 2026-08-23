//! The `Theme` type plus a builder for custom themes.
//!
//! Mirrors how v3 themes are authored: override a handful of base CSS variables
//! and let every hover / soft / surface-level value derive from them.

use gpui::{Hsla, Pixels, SharedString};

use crate::layout::LayoutTheme;
use crate::semantic::{RoleColor, SurfaceColor, ThemeColors};

/// Visual appearance of a theme (`color-scheme`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

/// A complete HeroUI v3 theme: semantic colors plus layout tokens.
#[derive(Clone, Debug)]
pub struct Theme {
    pub id: SharedString,
    pub appearance: Appearance,
    pub colors: ThemeColors,
    pub layout: LayoutTheme,
}

impl Theme {
    /// The default light theme.
    pub fn light() -> Self {
        Self {
            id: "light".into(),
            appearance: Appearance::Light,
            colors: ThemeColors::light(),
            layout: LayoutTheme::light(),
        }
    }

    /// The default dark theme.
    pub fn dark() -> Self {
        Self {
            id: "dark".into(),
            appearance: Appearance::Dark,
            colors: ThemeColors::dark(),
            layout: LayoutTheme::dark(),
        }
    }

    /// Starts a custom theme extending `base` — the equivalent of overriding
    /// CSS variables under a `[data-theme]` selector.
    pub fn builder(id: impl Into<SharedString>, base: Theme) -> ThemeBuilder {
        ThemeBuilder { theme: base }.id(id)
    }

    pub fn is_dark(&self) -> bool {
        self.appearance == Appearance::Dark
    }
}

/// Builder for custom themes.
pub struct ThemeBuilder {
    theme: Theme,
}

impl ThemeBuilder {
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.theme.id = id.into();
        self
    }

    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.theme.appearance = appearance;
        self
    }

    // -- layout -------------------------------------------------------------

    /// Sets `--radius`; `--field-radius` follows as `radius * 1.5` unless it is
    /// overridden afterwards with [`field_radius`](Self::field_radius).
    pub fn radius(mut self, radius: Pixels) -> Self {
        self.theme.layout.radius = radius;
        self.theme.layout.field_radius = radius * 1.5;
        self
    }

    pub fn field_radius(mut self, radius: Pixels) -> Self {
        self.theme.layout.field_radius = radius;
        self
    }

    pub fn border_width(mut self, width: Pixels) -> Self {
        self.theme.layout.border_width = width;
        self
    }

    pub fn disabled_opacity(mut self, v: f32) -> Self {
        self.theme.layout.disabled_opacity = v;
        self
    }

    // -- base colors --------------------------------------------------------

    pub fn background(mut self, c: Hsla) -> Self {
        self.theme.colors.background = c;
        self
    }

    pub fn foreground(mut self, c: Hsla) -> Self {
        self.theme.colors.foreground = c;
        self
    }

    pub fn muted(mut self, c: Hsla) -> Self {
        self.theme.colors.muted = c;
        self
    }

    pub fn border(mut self, c: Hsla) -> Self {
        self.theme.colors.border = c;
        self
    }

    /// Sets `--separator`. Defaults to the same value as `--border`.
    pub fn separator(mut self, c: Hsla) -> Self {
        self.theme.colors.separator = c;
        self
    }

    pub fn focus(mut self, c: Hsla) -> Self {
        self.theme.colors.focus = c;
        self
    }

    pub fn link(mut self, c: Hsla) -> Self {
        self.theme.colors.link = c;
        self
    }

    pub fn backdrop(mut self, c: Hsla) -> Self {
        self.theme.colors.backdrop = c;
        self
    }

    // -- containers ---------------------------------------------------------

    pub fn surface(mut self, background: Hsla, foreground: Hsla) -> Self {
        self.theme.colors.surface = SurfaceColor {
            background,
            foreground,
        };
        self
    }

    pub fn surface_levels(mut self, secondary: Hsla, tertiary: Hsla) -> Self {
        self.theme.colors.surface_secondary = secondary;
        self.theme.colors.surface_tertiary = tertiary;
        self
    }

    pub fn overlay(mut self, background: Hsla, foreground: Hsla) -> Self {
        self.theme.colors.overlay = SurfaceColor {
            background,
            foreground,
        };
        self
    }

    pub fn segment(mut self, background: Hsla, foreground: Hsla) -> Self {
        self.theme.colors.segment = SurfaceColor {
            background,
            foreground,
        };
        self
    }

    // -- roles --------------------------------------------------------------

    /// Sets a role's base value and foreground. `--focus` tracks `--accent`
    /// unless it is overridden afterwards.
    pub fn role(mut self, name: &str, color: Hsla, foreground: Hsla) -> Self {
        let value = RoleColor::new(color, foreground);
        match name {
            "default" => {
                self.theme.colors.default = value.with_hover_mix(0.04);
                self.theme.colors.field.background = color;
            }
            "success" => self.theme.colors.success = value,
            "warning" => self.theme.colors.warning = value,
            "danger" => self.theme.colors.danger = value,
            _ => {
                self.theme.colors.accent = value;
                self.theme.colors.focus = color;
            }
        }
        self
    }

    /// Sets `--accent` and `--accent-foreground`, deriving the foreground for
    /// readability when it is not supplied.
    pub fn accent(self, color: Hsla) -> Self {
        let fg = herogpui_core::readable_color(color);
        self.role("accent", color, fg)
    }

    // -- fields -------------------------------------------------------------

    pub fn field(mut self, background: Hsla, foreground: Hsla) -> Self {
        self.theme.colors.field.background = background;
        self.theme.colors.field.foreground = foreground;
        self
    }

    pub fn field_placeholder(mut self, c: Hsla) -> Self {
        self.theme.colors.field.placeholder = c;
        self
    }

    pub fn field_border(mut self, c: Hsla) -> Self {
        self.theme.colors.field.border = c;
        self
    }

    pub fn build(self) -> Theme {
        self.theme
    }
}
