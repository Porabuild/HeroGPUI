//! The `Theme` type plus a builder for custom themes.
//!
//! Mirrors how v3 themes are authored: override a handful of base CSS variables
//! and let every hover / soft / surface-level value derive from them.

use gpui::{Hsla, Pixels, SharedString};

use crate::layout::LayoutTheme;
use crate::semantic::{SurfaceColor, ThemeColors};

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

    /// Sets a role's base value and foreground. Like overriding a CSS
    /// variable, the role's hover and soft mix weights carry over — only the
    /// inputs change. `--focus` tracks `--accent` unless it is overridden
    /// afterwards.
    pub fn role(mut self, name: &str, color: Hsla, foreground: Hsla) -> Self {
        match name {
            "default" => {
                self.theme.colors.default.color = color;
                self.theme.colors.default.foreground = foreground;
                self.theme.colors.field.background = color;
            }
            "success" => {
                self.theme.colors.success.color = color;
                self.theme.colors.success.foreground = foreground;
            }
            "warning" => {
                self.theme.colors.warning.color = color;
                self.theme.colors.warning.foreground = foreground;
            }
            "danger" => {
                self.theme.colors.danger.color = color;
                self.theme.colors.danger.foreground = foreground;
            }
            _ => {
                self.theme.colors.accent.color = color;
                self.theme.colors.accent.foreground = foreground;
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

#[cfg(test)]
mod tests {
    use super::*;
    use herogpui_core::{mix_oklab, oklch};

    #[test]
    fn overriding_a_role_keeps_its_soft_semantics() {
        // v3's soft variables are `color-mix`es of the role variables: an
        // override replaces the input, never the weights.
        let theme = Theme::builder("brand", Theme::light())
            .role("success", oklch(0.55, 0.18, 145.0), oklch(1.0, 0.0, 0.0))
            .build();
        assert!((theme.colors.success.soft().a - 0.15).abs() < 1e-4);
        assert!((theme.colors.success.soft_hover().a - 0.20).abs() < 1e-4);
        assert_eq!(
            theme
                .colors
                .success
                .soft_foreground(theme.colors.foreground),
            mix_oklab(
                theme.colors.success.color,
                theme.colors.foreground,
                80.0 / 140.0
            )
        );
    }

    #[test]
    fn a_default_role_override_keeps_the_half_strength_soft() {
        let theme = Theme::builder("brand", Theme::light())
            .role(
                "default",
                oklch(0.90, 0.01, 286.0),
                oklch(0.20, 0.01, 286.0),
            )
            .build();
        assert!((theme.colors.default.soft().a - 0.50).abs() < 1e-4);
        assert!((theme.colors.default.soft_hover().a - 0.60).abs() < 1e-4);
        assert_eq!(
            theme
                .colors
                .default
                .soft_foreground(theme.colors.foreground),
            theme.colors.default.foreground
        );
    }
}
