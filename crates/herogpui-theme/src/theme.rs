//! The `Theme` type plus a builder for custom themes.
//!
//! Mirrors how v3 themes are authored: override a handful of base CSS variables
//! and let every hover / soft / surface-level value derive from them.

use gpui::{Hsla, Pixels, SharedString, WindowAppearance};

use crate::layout::LayoutTheme;
use crate::semantic::{SurfaceColor, ThemeColors};

/// Visual appearance of a theme (`color-scheme`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Appearance {
    Light,
    Dark,
}

/// Collapses the OS appearance onto the two `color-scheme` values v3 has.
///
/// The vibrant variants are macOS `NSAppearanceNameVibrant{Light,Dark}`: they
/// are still light and dark, and forgetting them is the classic bug here
/// because they only appear on a vibrancy-enabled window, never in a test.
/// The match is deliberately exhaustive rather than `_ => Light`, so a future
/// GPUI variant fails the build instead of silently painting light tokens over
/// a dark desktop.
impl From<WindowAppearance> for Appearance {
    fn from(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::Light,
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::Dark,
        }
    }
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
    ///
    /// A JSON document of the same sparse overrides lives behind this crate's
    /// `serde` feature (`ThemeDocument`): it applies through this builder so
    /// derived hover / soft mixes stay live.
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

    /// Sets the cursor every interactive control shows on hover. Defaults to
    /// [`gpui::CursorStyle::PointingHand`], v3's `cursor: pointer`.
    pub fn cursor_interactive(mut self, cursor: gpui::CursorStyle) -> Self {
        self.theme.layout.cursor_interactive = cursor;
        self
    }

    /// Sets the opacity a hovered `Tabs` item drops to. Finite values are
    /// clamped to `0..=1`; a non-finite value keeps the default `0.7`.
    pub fn tabs_hover_opacity(mut self, v: f32) -> Self {
        self.theme.layout.tabs_hover_opacity = if v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            0.7
        };
        self
    }

    /// Sets the warm window after the pointer leaves a tooltip during which
    /// the next tip opens without its delay. A per-tooltip close delay
    /// extends the window via `max()`.
    pub fn tooltip_cooldown_ms(mut self, ms: u64) -> Self {
        self.theme.layout.tooltip_cooldown_ms = ms;
        self
    }

    /// Sets how long a `DropdownTrigger::LongPress` waits before it opens.
    pub fn long_press_ms(mut self, ms: u64) -> Self {
        self.theme.layout.long_press_ms = ms;
        self
    }

    /// Sets the background fade duration of `anim::hover_fade`. Zero resolves
    /// immediately, like reduced motion.
    pub fn hover_fade_ms(mut self, ms: u64) -> Self {
        self.theme.layout.hover_fade_ms = ms;
        self
    }

    /// `--tooltip-delay`: how long a hover waits before the tip opens.
    pub fn tooltip_delay_ms(mut self, ms: u64) -> Self {
        self.theme.layout.tooltip_delay_ms = ms;
        self
    }

    /// `--tooltip-close-delay`: the per-tooltip close delay default.
    pub fn tooltip_close_delay_ms(mut self, ms: u64) -> Self {
        self.theme.layout.tooltip_close_delay_ms = ms;
        self
    }

    // -- base colors --------------------------------------------------------

    pub fn background(mut self, c: Hsla) -> Self {
        self.theme.colors.background = c;
        self
    }

    pub fn foreground(mut self, c: Hsla) -> Self {
        self.theme.colors.foreground = c;
        self.theme.colors.scrollbar = herogpui_core::with_alpha(c, 0.15);
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
    use herogpui_core::{mix_oklab, oklch, with_alpha};

    #[test]
    fn the_builder_overrides_the_interactive_cursor_and_nothing_else() {
        let base = Theme::light();
        assert_eq!(
            base.layout.cursor_interactive,
            gpui::CursorStyle::PointingHand
        );

        let theme = Theme::builder("arrow", base.clone())
            .cursor_interactive(gpui::CursorStyle::Arrow)
            .build();

        assert_eq!(theme.layout.cursor_interactive, gpui::CursorStyle::Arrow);
        assert_eq!(theme.layout.radius, base.layout.radius);
        assert!(
            (theme.layout.disabled_opacity - base.layout.disabled_opacity).abs() < f32::EPSILON
        );
        assert_eq!(theme.colors.background, base.colors.background);
    }

    #[test]
    fn customisation_tokens_flow_through_the_builder_and_clamp() {
        let theme = Theme::builder("custom", Theme::light())
            .tabs_hover_opacity(2.0)
            .tooltip_cooldown_ms(250)
            .long_press_ms(350)
            .hover_fade_ms(0)
            .tooltip_delay_ms(50)
            .tooltip_close_delay_ms(75)
            .build();
        assert!((theme.layout.tabs_hover_opacity - 1.0).abs() < 1e-6);
        assert_eq!(theme.layout.tooltip_cooldown_ms, 250);
        assert_eq!(theme.layout.long_press_ms, 350);
        assert_eq!(theme.layout.hover_fade_ms, 0);
        assert_eq!(theme.layout.tooltip_delay_ms, 50);
        assert_eq!(theme.layout.tooltip_close_delay_ms, 75);

        let nan = Theme::builder("nan", Theme::light())
            .tabs_hover_opacity(f32::NAN)
            .build();
        assert!((nan.layout.tabs_hover_opacity - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn overriding_foreground_recomputes_scrollbar_without_changing_other_tokens() {
        let base = Theme::light();
        let foreground = oklch(0.30, 0.05, 120.0);
        let background = oklch(0.90, 0.01, 286.0);
        let unchanged = (
            base.colors.muted,
            base.colors.border,
            base.colors.separator,
            base.colors.focus,
            base.layout.radius,
            base.layout.disabled_opacity,
        );
        let theme = Theme::builder("brand", base)
            .background(background)
            .foreground(foreground)
            .build();

        assert_eq!(
            (
                theme.colors.scrollbar,
                theme.colors.background,
                theme.colors.muted,
                theme.colors.border,
                theme.colors.separator,
                theme.colors.focus,
                theme.layout.radius,
                theme.layout.disabled_opacity,
            ),
            (
                with_alpha(foreground, 0.15),
                background,
                unchanged.0,
                unchanged.1,
                unchanged.2,
                unchanged.3,
                unchanged.4,
                unchanged.5,
            )
        );
    }

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
                60.0 / 140.0
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
