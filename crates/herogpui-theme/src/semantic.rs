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

/// How a role resolves `--role-soft-foreground`.
#[derive(Clone, Copy, Debug)]
pub enum SoftForeground {
    /// `--default-soft-foreground: var(--default-foreground)`
    RoleForeground,
    /// `color-mix(in oklab, var(--role) C%, var(--foreground) F%)`. CSS
    /// normalises the weights, so the role contributes `C / (C + F)`.
    Mix { color: f32, foreground: f32 },
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
    /// Share of the role color in `*-soft`, over transparent.
    soft_mix: f32,
    /// Share of the role color in `*-soft-hover`.
    soft_hover_mix: f32,
    /// How `*-soft-foreground` resolves.
    soft_foreground: SoftForeground,
}

impl RoleColor {
    pub fn new(color: Hsla, foreground: Hsla) -> Self {
        Self {
            color,
            foreground,
            hover_mix: 0.10,
            soft_mix: 0.15,
            soft_hover_mix: 0.20,
            soft_foreground: SoftForeground::Mix {
                color: 70.0,
                foreground: 30.0,
            },
        }
    }

    /// `default` mixes only 4% of its foreground on hover.
    pub fn with_hover_mix(mut self, hover_mix: f32) -> Self {
        self.hover_mix = hover_mix;
        self
    }

    /// Sets the shares of the role color in `--role-soft` and
    /// `--role-soft-hover` (`default` sits at 50/60, the rest at 15/20 in
    /// light and 12/16 in dark for the cooler roles).
    pub fn with_soft_mix(mut self, soft: f32, soft_hover: f32) -> Self {
        self.soft_mix = soft;
        self.soft_hover_mix = soft_hover;
        self
    }

    /// `--role-soft-foreground: color-mix(in oklab, var(--role) C%, var(--foreground) F%)`
    pub fn with_soft_foreground_mix(mut self, color: f32, foreground: f32) -> Self {
        self.soft_foreground = SoftForeground::Mix { color, foreground };
        self
    }

    /// `--default-soft-foreground: var(--default-foreground)`
    pub fn with_soft_foreground_role(mut self) -> Self {
        self.soft_foreground = SoftForeground::RoleForeground;
        self
    }

    /// `--color-accent-hover: color-mix(in oklab, var(--accent) 90%, var(--accent-foreground) 10%)`
    pub fn hover(&self) -> Hsla {
        mix_oklab(self.color, self.foreground, self.hover_mix)
    }

    /// `--color-accent-soft: color-mix(in oklab, var(--accent) 15%, transparent)`
    pub fn soft(&self) -> Hsla {
        soft_mix(self.color, self.soft_mix)
    }

    /// `--color-accent-soft-hover: color-mix(in oklab, var(--accent) 20%, transparent)`
    pub fn soft_hover(&self) -> Hsla {
        soft_mix(self.color, self.soft_hover_mix)
    }

    /// `--color-accent-soft-foreground: color-mix(in oklab, var(--accent) 70%, var(--foreground) 30%)`
    ///
    /// The mixing roles blend against the page foreground, so pass the live
    /// `ThemeColors::foreground` — a `ThemeBuilder::foreground` override then
    /// flows through without rebuilding the theme.
    pub fn soft_foreground(&self, page_foreground: Hsla) -> Hsla {
        match self.soft_foreground {
            SoftForeground::RoleForeground => self.foreground,
            SoftForeground::Mix { color, foreground } => {
                mix_oklab(self.color, page_foreground, color / (color + foreground))
            }
        }
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

impl SurfaceColor {
    /// `--surface-hover: color-mix(in oklab, var(--surface) 92%, var(--surface-foreground) 8%)`
    pub fn hover(&self) -> Hsla {
        mix_oklab(self.background, self.foreground, 0.08)
    }
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

    /// `--field-border-hover: color-mix(in oklab, var(--field-border) 88%, var(--field-foreground) 10%)`
    ///
    /// The weights sum to 98, which CSS normalises, so the foreground
    /// contributes 10/98. Invisible while `--field-border-width` is 0, and the
    /// token exists for a caller who gives their fields a border.
    pub fn border_hover(&self) -> Hsla {
        mix_oklab(self.border, self.foreground, 10.0 / 98.0)
    }

    /// `--field-border-focus: color-mix(in oklab, var(--field-border) 74%, var(--field-foreground) 22%)`
    pub fn border_focus(&self) -> Hsla {
        mix_oklab(self.border, self.foreground, 22.0 / 96.0)
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

    // -- derived borders ----------------------------------------------------

    /// `--border-secondary: color-mix(in oklab, var(--surface) 78%, var(--surface-foreground) 22%)`
    pub fn border_secondary(&self) -> Hsla {
        mix_oklab(self.surface.background, self.surface.foreground, 0.22)
    }

    /// `--border-tertiary: color-mix(in oklab, var(--surface) 66%, var(--surface-foreground) 34%)`
    pub fn border_tertiary(&self) -> Hsla {
        mix_oklab(self.surface.background, self.surface.foreground, 0.34)
    }

    /// `--surface-secondary-foreground: var(--foreground)`
    ///
    /// v3 gives the secondary and tertiary surfaces their own foreground
    /// variables, both defaulting to the page's, so a caller who repaints one of
    /// those surfaces has somewhere to put the matching text colour.
    pub fn surface_secondary_foreground(&self) -> Hsla {
        self.foreground
    }

    /// `--surface-tertiary-foreground: var(--foreground)`
    pub fn surface_tertiary_foreground(&self) -> Hsla {
        self.foreground
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
        let accent = RoleColor::new(oklch(0.6204, 0.195, 253.83), snow())
            .with_soft_mix(0.15, 0.20)
            .with_soft_foreground_mix(70.0, 30.0);
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

            default: RoleColor::new(oklch(0.94, 0.001, 286.375), eclipse())
                .with_hover_mix(0.04)
                .with_soft_mix(0.50, 0.60)
                .with_soft_foreground_role(),
            accent,
            success: RoleColor::new(oklch(0.7329, 0.1935, 150.81), eclipse())
                .with_soft_foreground_mix(80.0, 60.0),
            warning: RoleColor::new(oklch(0.7819, 0.1585, 72.33), eclipse())
                .with_soft_foreground_mix(80.0, 70.0),
            danger: RoleColor::new(oklch(0.6532, 0.2328, 25.74), snow())
                .with_soft_foreground_mix(70.0, 40.0),

            field: FieldColors {
                background: white(),
                foreground: oklch(0.2103, 0.0059, 285.89),
                placeholder: muted,
                border: with_alpha(black(), 0.0),
            },

            // `--border` is a step darker than `--separator`: 90% against 92%.
            // Both had been transcribed as the separator's value.
            border: oklch(0.9, 0.004, 286.32),
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
        let accent = RoleColor::new(oklch(0.6204, 0.195, 253.83), snow())
            .with_soft_mix(0.12, 0.16)
            .with_soft_foreground_mix(80.0, 30.0);
        let default = RoleColor::new(oklch(0.274, 0.006, 286.033), snow())
            .with_hover_mix(0.04)
            .with_soft_mix(0.50, 0.60)
            .with_soft_foreground_role();
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
            // `--overlay` *is* `--surface` in dark mode. This used to lighten it
            // "so floating panels read", which is the kind of improvement the
            // token values are not allowed to make: a v3 dark popover is the
            // colour of a v3 dark card, and the shadow is what separates them.
            overlay: SurfaceColor {
                background: oklch(0.2103, 0.0059, 285.89),
                foreground,
            },
            segment: SurfaceColor {
                background: oklch(0.3964, 0.01, 285.93),
                foreground,
            },

            default,
            accent,
            // `--success` is not overridden in dark mode; only its soft shares
            // are (12/16 over transparent, foreground at 80/30).
            success: RoleColor::new(oklch(0.7329, 0.1935, 150.81), eclipse())
                .with_soft_mix(0.12, 0.16)
                .with_soft_foreground_mix(80.0, 30.0),
            warning: RoleColor::new(oklch(0.8203, 0.1388, 76.34), eclipse())
                .with_soft_mix(0.12, 0.16)
                .with_soft_foreground_mix(80.0, 30.0),
            danger: RoleColor::new(oklch(0.594, 0.1967, 24.63), snow())
                .with_soft_foreground_mix(80.0, 30.0),

            field: FieldColors {
                // `--field-background: oklch(0.2103 0.0059 285.89)` -- the
                // surface colour, not `--default`, which is two steps lighter.
                background: oklch(0.2103, 0.0059, 285.89),
                foreground,
                placeholder: muted,
                border: with_alpha(black(), 0.0),
            },

            // `--border: oklch(28% ..)`, `--separator: oklch(25% ..)`. Both were
            // one value here, and both too dark.
            border: oklch(0.28, 0.006, 286.033),
            separator: oklch(0.25, 0.006, 286.033),
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
    fn the_soft_matrix_matches_the_pinned_stylesheet() {
        // Weights transcribed from `variables.css` at v3.2.4. `None` means
        // `--role-soft-foreground: var(--role-foreground)`; `Some` is the
        // `color-mix(in oklab, var(--role) C%, var(--foreground) F%)` pair.
        let light = ThemeColors::light();
        let dark = ThemeColors::dark();
        for (colors, role_name, soft, soft_hover, foreground) in [
            (&light, "default", 0.50, 0.60, None),
            (&light, "accent", 0.15, 0.20, Some((70.0, 30.0))),
            (&light, "success", 0.15, 0.20, Some((80.0, 60.0))),
            (&light, "warning", 0.15, 0.20, Some((80.0, 70.0))),
            (&light, "danger", 0.15, 0.20, Some((70.0, 40.0))),
            (&dark, "default", 0.50, 0.60, None),
            (&dark, "accent", 0.12, 0.16, Some((80.0, 30.0))),
            (&dark, "success", 0.12, 0.16, Some((80.0, 30.0))),
            (&dark, "warning", 0.12, 0.16, Some((80.0, 30.0))),
            (&dark, "danger", 0.15, 0.20, Some((80.0, 30.0))),
        ] {
            let role = colors.role(role_name);
            assert!(
                (role.soft().a - soft).abs() < 1e-4,
                "{role_name} soft alpha"
            );
            assert!(
                (role.soft_hover().a - soft_hover).abs() < 1e-4,
                "{role_name} soft-hover alpha"
            );
            match foreground {
                Some((color, page)) => assert_eq!(
                    role.soft_foreground(colors.foreground),
                    mix_oklab(role.color, colors.foreground, color / (color + page)),
                    "{role_name} soft-foreground mix"
                ),
                None => assert_eq!(
                    role.soft_foreground(colors.foreground),
                    role.foreground,
                    "{role_name} soft-foreground follows the role"
                ),
            }
        }
    }

    #[test]
    fn a_custom_foreground_stays_live_in_the_soft_foreground_mix() {
        let base = ThemeColors::light();
        let custom = ThemeColors {
            foreground: oklch(0.30, 0.05, 120.0),
            ..base
        };
        // The mixing roles resolve against the page foreground passed in, so a
        // `ThemeBuilder::foreground` override flows through at render time.
        assert_eq!(
            custom.accent.soft_foreground(custom.foreground),
            mix_oklab(custom.accent.color, custom.foreground, 0.70)
        );
        assert_ne!(
            custom.accent.soft_foreground(custom.foreground),
            base.accent.soft_foreground(base.foreground)
        );
        // `--default-soft-foreground: var(--default-foreground)` does not mix,
        // so it follows the role's own foreground instead of the page's.
        assert_eq!(
            custom.default.soft_foreground(custom.foreground),
            custom.default.foreground
        );
    }

    #[test]
    fn the_derived_borders_step_away_from_the_surface() {
        let c = ThemeColors::light();
        // `--border-secondary` and `--border-tertiary` mix further from the
        // surface than `--separator-secondary` does, so a border reads stronger
        // than a rule at the same step.
        let steps = [
            c.separator_secondary(),
            c.border_secondary(),
            c.border_tertiary(),
        ];
        for pair in steps.windows(2) {
            assert!(
                pair[1].l < pair[0].l,
                "each step is darker than the last on a light surface"
            );
        }
    }

    #[test]
    fn a_secondary_surface_keeps_the_page_foreground() {
        let c = ThemeColors::light();
        assert_eq!(c.surface_secondary_foreground(), c.foreground);
        assert_eq!(c.surface_tertiary_foreground(), c.foreground);
    }

    #[test]
    fn a_field_border_mixes_toward_its_own_foreground() {
        let c = ThemeColors::light();
        // Both are mixes of `--field-border` toward `--field-foreground`, and
        // focus mixes further than hover.
        let border = c.field.border;
        assert_ne!(c.field.border_hover(), border);
        assert_ne!(c.field.border_focus(), c.field.border_hover());
    }

    #[test]
    fn field_focus_matches_field_background() {
        let c = ThemeColors::light();
        assert_eq!(c.field.focus(), c.field.background);
    }

    #[test]
    #[ignore = "v3 gives dark mode one colour for both; the shadow separates them"]
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
