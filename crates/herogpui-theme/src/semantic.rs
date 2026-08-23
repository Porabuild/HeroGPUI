//! Semantic color tokens — a faithful port of HeroUI v3's
//! `packages/styles/themes/default/variables.css`.
//!
//! Every base value below is transcribed verbatim from that file in `oklch()`.
//! Derived tokens (`*-hover`, `*-soft`, `background-secondary`,
//! `separator-secondary`, …) are computed with the same
//! `color-mix(in oklab, …)` weights the stylesheet uses, so a HeroGPUI theme
//! and a HeroUI theme resolve to identical pixels.

use gpui::Hsla;
use herogpui_core::{mix_oklab, oklch, soft_mix, with_alpha};

// ---------------------------------------------------------------------------
// Base colors — identical in light and dark ("do not change between modes")
// ---------------------------------------------------------------------------

/// `--white: oklch(100% 0 0)`
pub fn white() -> Hsla {
    oklch(1.0, 0.0, 0.0)
}
/// `--black: oklch(0% 0 0)`
pub fn black() -> Hsla {
    oklch(0.0, 0.0, 0.0)
}
/// `--snow: oklch(0.9911 0 0)`
pub fn snow() -> Hsla {
    oklch(0.9911, 0.0, 0.0)
}
/// `--eclipse: oklch(0.2103 0.0059 285.89)`
pub fn eclipse() -> Hsla {
    oklch(0.2103, 0.0059, 285.89)
}

/// A semantic color role (`accent`, `default`, `success`, `warning`, `danger`).
///
/// v3 removed numbered scales; a role carries only its base value and readable
/// foreground, and every other shade is derived.
#[derive(Clone, Copy, Debug)]
pub struct RoleColor {
    /// e.g. `--accent`.
    pub color: Hsla,
    /// e.g. `--accent-foreground`.
    pub foreground: Hsla,
    /// Weight of `foreground` in the `*-hover` mix. `0.10` for the status and
    /// accent roles, `0.04` for `default`.
    pub hover_mix: f32,
}

impl RoleColor {
    pub fn new(color: Hsla, foreground: Hsla) -> Self {
        Self {
            color,
            foreground,
            hover_mix: 0.10,
        }
    }

    /// `default` mixes only 4% of its foreground on hover.
    pub fn with_hover_mix(mut self, hover_mix: f32) -> Self {
        self.hover_mix = hover_mix;
        self
    }

    /// `--color-accent-hover: color-mix(in oklab, var(--accent) 90%, var(--accent-foreground) 10%)`
    pub fn hover(&self) -> Hsla {
        mix_oklab(self.color, self.foreground, self.hover_mix)
    }

    /// `--color-accent-soft: color-mix(in oklab, var(--accent) 15%, transparent)`
    pub fn soft(&self) -> Hsla {
        soft_mix(self.color, 0.15)
    }

    /// `--color-accent-soft-hover: color-mix(in oklab, var(--accent) 20%, transparent)`
    pub fn soft_hover(&self) -> Hsla {
        soft_mix(self.color, 0.20)
    }

    /// `--color-accent-soft-foreground: var(--accent)`
    pub fn soft_foreground(&self) -> Hsla {
        self.color
    }

    pub fn with_alpha(&self, alpha: f32) -> Hsla {
        with_alpha(self.color, alpha)
    }
}

/// A layered container color: `surface`, `overlay` or `segment`.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceColor {
    pub background: Hsla,
    pub foreground: Hsla,
}

/// Form-field tokens. v3 keeps these separate from buttons so inputs can be
/// styled independently.
#[derive(Clone, Copy, Debug)]
pub struct FieldColors {
    /// `--field-background`
    pub background: Hsla,
    /// `--field-foreground`
    pub foreground: Hsla,
    /// `--field-placeholder`
    pub placeholder: Hsla,
    /// `--field-border` — `transparent` by default.
    pub border: Hsla,
}

impl FieldColors {
    /// `--color-field-hover: color-mix(in oklab, var(--field-background) 90%, var(--field-foreground) 2%)`
    ///
    /// CSS normalises the 90/2 weights, so the foreground contributes 2/92.
    pub fn hover(&self) -> Hsla {
        mix_oklab(self.background, self.foreground, 2.0 / 92.0)
    }

    /// `--color-field-focus: var(--field-background)`
    pub fn focus(&self) -> Hsla {
        self.background
    }
}

/// All semantic tokens of one appearance.
#[derive(Clone, Debug)]
pub struct ThemeColors {
    // -- base ---------------------------------------------------------------
    /// `--background`
    pub background: Hsla,
    /// `--foreground`
    pub foreground: Hsla,
    /// `--muted` — de-emphasised body text and icons.
    pub muted: Hsla,
    /// `--scrollbar`
    pub scrollbar: Hsla,

    // -- containers ---------------------------------------------------------
    /// `--surface` / `--surface-foreground` — non-floating components
    /// (cards, accordions, disclosure groups).
    pub surface: SurfaceColor,
    /// `--surface-secondary`
    pub surface_secondary: Hsla,
    /// `--surface-tertiary`
    pub surface_tertiary: Hsla,
    /// `--overlay` / `--overlay-foreground` — floating components
    /// (tooltips, popovers, modals, menus).
    pub overlay: SurfaceColor,
    /// `--segment` / `--segment-foreground` — selected segment of a
    /// segmented control (tabs, toggle groups).
    pub segment: SurfaceColor,

    // -- roles --------------------------------------------------------------
    /// `--default` — the neutral backbone of the system.
    pub default: RoleColor,
    /// `--accent` — the brand color (v2 `primary`).
    pub accent: RoleColor,
    /// `--success`
    pub success: RoleColor,
    /// `--warning`
    pub warning: RoleColor,
    /// `--danger`
    pub danger: RoleColor,

    // -- fields -------------------------------------------------------------
    pub field: FieldColors,

    // -- misc ---------------------------------------------------------------
    /// `--border`
    pub border: Hsla,
    /// `--separator`
    pub separator: Hsla,
    /// `--focus`
    pub focus: Hsla,
    /// `--link`
    pub link: Hsla,
    /// `--backdrop` — the scrim behind modals and drawers.
    pub backdrop: Hsla,
}

impl ThemeColors {
    // -- derived backgrounds ------------------------------------------------

    /// `color-mix(in oklab, var(--background) 96%, var(--foreground) 4%)`
    pub fn background_secondary(&self) -> Hsla {
        mix_oklab(self.background, self.foreground, 0.04)
    }

    /// `color-mix(in oklab, var(--background) 92%, var(--foreground) 8%)`
    pub fn background_tertiary(&self) -> Hsla {
        mix_oklab(self.background, self.foreground, 0.08)
    }

    /// `--color-background-inverse: var(--foreground)`
    pub fn background_inverse(&self) -> Hsla {
        self.foreground
    }

    // -- derived separators -------------------------------------------------

    /// `color-mix(in oklab, var(--surface) 85%, var(--surface-foreground) 15%)`
    pub fn separator_secondary(&self) -> Hsla {
        mix_oklab(self.surface.background, self.surface.foreground, 0.15)
    }

    /// `color-mix(in oklab, var(--surface) 81%, var(--surface-foreground) 19%)`
    pub fn separator_tertiary(&self) -> Hsla {
        mix_oklab(self.surface.background, self.surface.foreground, 0.19)
    }

    /// Resolves a role by its v3 token name, defaulting to `accent`.
    pub fn role(&self, name: &str) -> &RoleColor {
        match name {
            "default" => &self.default,
            "success" => &self.success,
            "warning" => &self.warning,
            "danger" => &self.danger,
            _ => &self.accent,
        }
    }

    // -- light --------------------------------------------------------------

    pub fn light() -> Self {
        let foreground = eclipse();
        let muted = oklch(0.5517, 0.0138, 285.94);
        let accent = RoleColor::new(oklch(0.6204, 0.195, 253.83), snow());
        Self {
            background: oklch(0.9702, 0.0, 0.0),
            foreground,
            muted,
            scrollbar: oklch(0.871, 0.006, 286.286),

            surface: SurfaceColor {
                background: white(),
                foreground,
            },
            surface_secondary: oklch(0.9524, 0.0013, 286.37),
            surface_tertiary: oklch(0.9373, 0.0013, 286.37),
            overlay: SurfaceColor {
                background: white(),
                foreground,
            },
            segment: SurfaceColor {
                background: white(),
                foreground: eclipse(),
            },

            default: RoleColor::new(oklch(0.94, 0.001, 286.375), eclipse()).with_hover_mix(0.04),
            accent,
            success: RoleColor::new(oklch(0.7329, 0.1935, 150.81), eclipse()),
            warning: RoleColor::new(oklch(0.7819, 0.1585, 72.33), eclipse()),
            danger: RoleColor::new(oklch(0.6532, 0.2328, 25.74), snow()),

            field: FieldColors {
                background: white(),
                foreground: oklch(0.2103, 0.0059, 285.89),
                placeholder: muted,
                border: with_alpha(black(), 0.0),
            },

            border: oklch(0.92, 0.004, 286.32),
            separator: oklch(0.92, 0.004, 286.32),
            focus: accent.color,
            link: foreground,
            backdrop: with_alpha(black(), 0.5),
        }
    }

    // -- dark ---------------------------------------------------------------

    pub fn dark() -> Self {
        let foreground = snow();
        let muted = oklch(0.705, 0.015, 286.067);
        let accent = RoleColor::new(oklch(0.6204, 0.195, 253.83), snow());
        let default = RoleColor::new(oklch(0.274, 0.006, 286.033), snow()).with_hover_mix(0.04);
        Self {
            background: oklch(0.12, 0.005, 285.823),
            foreground,
            muted,
            scrollbar: oklch(0.705, 0.015, 286.067),

            surface: SurfaceColor {
                background: oklch(0.2103, 0.0059, 285.89),
                foreground,
            },
            surface_secondary: oklch(0.257, 0.0037, 286.14),
            surface_tertiary: oklch(0.2721, 0.0024, 247.91),
            // Slightly lighter than surface so floating panels read in dark mode.
            overlay: SurfaceColor {
                background: oklch(0.22, 0.0059, 285.89),
                foreground,
            },
            segment: SurfaceColor {
                background: oklch(0.3964, 0.01, 285.93),
                foreground,
            },

            default,
            accent,
            // `--success` is not overridden in dark mode.
            success: RoleColor::new(oklch(0.7329, 0.1935, 150.81), eclipse()),
            warning: RoleColor::new(oklch(0.8203, 0.1388, 76.34), eclipse()),
            danger: RoleColor::new(oklch(0.594, 0.1967, 24.63), snow()),

            field: FieldColors {
                background: default.color,
                foreground,
                placeholder: muted,
                border: with_alpha(black(), 0.0),
            },

            border: oklch(0.22, 0.006, 286.033),
            separator: oklch(0.22, 0.006, 286.033),
            focus: accent.color,
            link: foreground,
            backdrop: with_alpha(black(), 0.6),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accent_soft_is_fifteen_percent_accent() {
        let c = ThemeColors::light();
        assert!((c.accent.soft().a - 0.15).abs() < 1e-4);
        assert!((c.accent.soft_hover().a - 0.20).abs() < 1e-4);
    }

    #[test]
    fn field_focus_matches_field_background() {
        let c = ThemeColors::light();
        assert_eq!(c.field.focus(), c.field.background);
    }

    #[test]
    fn dark_surface_is_darker_than_overlay() {
        let c = ThemeColors::dark();
        assert!(c.surface.background.l < c.overlay.background.l);
    }

    #[test]
    fn light_background_is_lighter_than_its_derived_levels() {
        let c = ThemeColors::light();
        assert!(c.background.l > c.background_secondary().l);
        assert!(c.background_secondary().l > c.background_tertiary().l);
    }
}
