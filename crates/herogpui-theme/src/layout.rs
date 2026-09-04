//! Layout tokens — a faithful port of HeroUI v3's non-color custom properties
//! from `packages/styles/themes/default/variables.css`.
//!
//! v3 replaced v2's size-named tokens (`radius-small`, `box-shadow-medium`)
//! with a single `--radius` base plus calculated steps, and with
//! component-semantic shadows (`--surface-shadow`, `--overlay-shadow`,
//! `--field-shadow`).

use gpui::{point, px, BoxShadow, Pixels};

/// How a [`Skeleton`](../herogpui_components/struct.Skeleton.html) animates by
/// default (`--skeleton-animation`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonAnimation {
    #[default]
    Shimmer,
    Pulse,
    None,
}

/// Spacing, radius, border and shadow tokens shared by all components.
#[derive(Clone, Debug)]
pub struct LayoutTheme {
    /// `--spacing: 0.25rem`
    pub spacing: Pixels,

    /// `--radius: 0.5rem` — the base every other radius is calculated from.
    pub radius: Pixels,
    /// `--field-radius: calc(var(--radius) * 1.5)`
    pub field_radius: Pixels,

    /// `--border-width: 1px`
    pub border_width: Pixels,
    /// `--field-border-width: 0px`
    pub field_border_width: Pixels,

    /// `--disabled-opacity: 0.5`
    pub disabled_opacity: f32,
    /// `--ring-offset-width: 2px`
    pub ring_offset_width: Pixels,

    /// `--surface-shadow` — cards, accordions and other inline containers.
    pub surface_shadow: Vec<BoxShadow>,
    /// `--overlay-shadow` — tooltips, popovers, modals and menus.
    pub overlay_shadow: Vec<BoxShadow>,
    /// `--field-shadow` — inputs and other form controls.
    pub field_shadow: Vec<BoxShadow>,

    /// `--skeleton-animation`
    pub skeleton_animation: SkeletonAnimation,
    /// `--tooltip-delay: 1500ms`
    pub tooltip_delay_ms: u64,
    /// `--tooltip-close-delay: 500ms`
    pub tooltip_close_delay_ms: u64,
    /// The hairline a floating panel draws instead of a border.
    ///
    /// v3 gives its panels no border at all: light mode separates them with
    /// `--overlay-shadow`, and dark mode adds `0 0 1px 0 rgba(255,255,255,.3)
    /// **inset**` -- a one-pixel highlight just inside the edge. gpui has no
    /// inset shadow, so the closest reproduction is a one-pixel border in that
    /// colour, and in light mode there is none.
    pub overlay_hairline: Option<gpui::Hsla>,
}

impl Default for LayoutTheme {
    fn default() -> Self {
        Self::light()
    }
}

impl LayoutTheme {
    pub fn light() -> Self {
        Self {
            // `0 2px 4px 0 rgba(0,0,0,.04), 0 1px 2px 0 rgba(0,0,0,.06),
            //  0 0 1px 0 rgba(0,0,0,.06)`
            surface_shadow: vec![
                shadow(0., 2., 4., 0.04),
                shadow(0., 1., 2., 0.06),
                shadow(0., 0., 1., 0.06),
            ],
            // `0 2px 8px 0 rgba(0,0,0,.06), 0 -6px 12px 0 rgba(0,0,0,.03),
            //  0 14px 28px 0 rgba(0,0,0,.08)` -- three, and the middle one
            // throws its blur *upward*, which is what keeps a panel from
            // looking pasted onto the page.
            overlay_shadow: vec![
                shadow(0., 2., 8., 0.06),
                shadow(0., -6., 12., 0.03),
                shadow(0., 14., 28., 0.08),
            ],
            field_shadow: vec![
                shadow(0., 2., 4., 0.04),
                shadow(0., 1., 2., 0.06),
                shadow(0., 0., 1., 0.06),
            ],
            ..Self::common()
        }
    }

    pub fn dark() -> Self {
        // Dark mode drops all three shadows in v3.
        Self {
            surface_shadow: Vec::new(),
            overlay_shadow: Vec::new(),
            field_shadow: Vec::new(),
            // `--overlay-shadow: 0 0 1px 0 rgba(255,255,255,.3) inset` is the
            // only shadow dark mode keeps, and it is what separates a panel from
            // the page now that both are the same colour.
            overlay_hairline: Some(gpui::hsla(0., 0., 1., 0.3)),
            ..Self::common()
        }
    }

    fn common() -> Self {
        let radius = px(8.0);
        Self {
            spacing: px(4.0),
            radius,
            field_radius: radius * 1.5,
            border_width: px(1.0),
            field_border_width: px(0.0),
            disabled_opacity: 0.5,
            ring_offset_width: px(2.0),
            surface_shadow: Vec::new(),
            overlay_shadow: Vec::new(),
            field_shadow: Vec::new(),
            skeleton_animation: SkeletonAnimation::Shimmer,
            tooltip_delay_ms: 1500,
            tooltip_close_delay_ms: 500,
            overlay_hairline: None,
        }
    }

    /// `--radius-xs: calc(var(--radius) * 0.25)`
    pub fn radius_xs(&self) -> Pixels {
        self.radius * 0.25
    }
    /// `--radius-sm: calc(var(--radius) * 0.5)`
    pub fn radius_sm(&self) -> Pixels {
        self.radius * 0.5
    }
    /// `--radius-md: calc(var(--radius) * 0.75)`
    pub fn radius_md(&self) -> Pixels {
        self.radius * 0.75
    }
    /// `--radius-lg: calc(var(--radius) * 1)`
    pub fn radius_lg(&self) -> Pixels {
        self.radius
    }
    /// `--radius-xl: calc(var(--radius) * 1.5)`
    pub fn radius_xl(&self) -> Pixels {
        self.radius * 1.5
    }
    /// `--radius-2xl: calc(var(--radius) * 2)`
    pub fn radius_2xl(&self) -> Pixels {
        self.radius * 2.0
    }
    /// `--radius-3xl: calc(var(--radius) * 3)`
    pub fn radius_3xl(&self) -> Pixels {
        self.radius * 3.0
    }
    /// `--radius-4xl: calc(var(--radius) * 4)`
    pub fn radius_4xl(&self) -> Pixels {
        self.radius * 4.0
    }

    /// A radius capped the way v3 caps its own: `min(32px, ..)`.
    ///
    /// v3 wraps every `rounded-*` and `rounded-full` in `min()` so a theme with
    /// an oversized `--radius` cannot distort a component — the corner stops
    /// growing before it swallows the box.
    pub fn capped(&self, radius: Pixels) -> Pixels {
        radius.min(px(32.0))
    }
}

fn shadow(x: f32, y: f32, blur: f32, alpha: f32) -> BoxShadow {
    BoxShadow {
        inset: false,
        color: gpui::hsla(0.0, 0.0, 0.0, alpha),
        offset: point(px(x), px(y)),
        blur_radius: px(blur),
        spread_radius: px(0.),
    }
}
