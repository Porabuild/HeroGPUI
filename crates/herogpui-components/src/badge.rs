//! Badge — port of `@heroui/badge`.

use gpui::{
    prelude::*, px, AnyElement, App, DefiniteLength, Hsla, IntoElement, ParentElement, RenderOnce,
    Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::{ActiveTheme, ThemeColors};

/// Where the badge is anchored on its child (`placement`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgePlacement {
    TopLeft,
    #[default]
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Visual style of a badge (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Filled with the badge color.
    #[default]
    Primary,
    /// Filled with `default`, colored text.
    Secondary,
    /// Filled with `--{role}-soft` (the default role mixes at 50%), labelled
    /// in the soft foreground.
    Soft,
}

impl BadgeVariant {
    pub const ALL: [BadgeVariant; 3] = [
        BadgeVariant::Primary,
        BadgeVariant::Secondary,
        BadgeVariant::Soft,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BadgeVariant::Primary => "Primary",
            BadgeVariant::Secondary => "Secondary",
            BadgeVariant::Soft => "Soft",
        }
    }
}

/// v3's `Badge.Anchor` — the positioning wrapper (`.badge-anchor`) that owns
/// the anchored element and the [`Badge`] pointing at it:
///
/// ```ignore
/// BadgeAnchor::new()
///     .child(avatar)
///     .child(Badge::new().child(BadgeLabel::new().child("5")))
/// ```
///
/// `.badge-anchor` is `relative inline-flex shrink-0`. GPUI 0.2.2 has no
/// inline-flex, so the wrapper hugs its content only inside a flex parent,
/// and the ported `flex_shrink_0` keeps it from compressing in an overflowing
/// row. The badge positions itself against this wrapper.
#[derive(IntoElement)]
pub struct BadgeAnchor {
    children: Vec<AnyElement>,
}

impl BadgeAnchor {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for BadgeAnchor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for BadgeAnchor {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BadgeAnchor {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        gpui::div()
            .relative()
            .flex()
            .flex_shrink_0()
            .debug_selector(|| "badge-anchor".to_owned())
            .children(self.children)
    }
}

/// v3's `Badge.Label` — the badge's text slot. `.badge__label` is `px-0.5`,
/// the only horizontal padding in v3's badge sheet; the badge itself has
/// none.
///
/// v3's root auto-wraps plain string and number children in this part. GPUI
/// elements carry no runtime type a parent can intercept, so the port cannot
/// reproduce that auto-wrap: plain [`Badge`] children draw inside the badge
/// without the label padding, and this explicit part is the seam that carries
/// the pinned padding.
#[derive(IntoElement)]
pub struct BadgeLabel {
    children: Vec<AnyElement>,
}

impl BadgeLabel {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for BadgeLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for BadgeLabel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for BadgeLabel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        gpui::div()
            .debug_selector(|| "badge-label".to_owned())
            .px(px(2.))
            .children(self.children)
    }
}

/// HeroUI Badge — v3's `Badge.Root`, the indicator positioned against a
/// [`BadgeAnchor`]. Its [`ParentElement`] children are the badge's own
/// content: text goes through [`BadgeLabel`], and a badge with no children
/// renders as a dot — the dot is the omitted label, not a separate mode.
#[derive(IntoElement)]
pub struct Badge {
    color: Color,
    variant: BadgeVariant,
    size: Size,
    placement: BadgePlacement,
    children: Vec<AnyElement>,
}

impl Badge {
    pub fn new() -> Self {
        // v3's table gives `color` a default of `"default"`, which is the
        // gray `.badge--default`; the seed used to be `Danger`.
        Self {
            color: Color::Default,
            variant: BadgeVariant::Primary,
            size: Size::Md,
            placement: BadgePlacement::TopRight,
            children: Vec::new(),
        }
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn variant(mut self, v: BadgeVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn placement(mut self, p: BadgePlacement) -> Self {
        self.placement = p;
        self
    }
}

impl Default for Badge {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Badge {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Resolves one badge's paint pair against v3.2.4's `badge.css` cascade: the
/// base rule, the `.badge--{color}` foreground classes, and the compound
/// variant×color rules. Every badge carries the page-background ring, so
/// unlike a chip this always paints a fill.
fn paint(colors: &ThemeColors, variant: BadgeVariant, color: Color) -> (Hsla, Hsla) {
    let role = colors.role(color.token());
    let muted_foreground = || {
        if color == Color::Default {
            colors.default.foreground
        } else {
            role.soft_foreground(colors.foreground)
        }
    };
    match variant {
        BadgeVariant::Primary => (role.color, role.foreground),
        BadgeVariant::Secondary => (colors.default.color, muted_foreground()),
        BadgeVariant::Soft => (role.soft(), muted_foreground()),
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();

        // `.badge` is `min-h-7 min-w-7 rounded-3xl text-xs leading-[1.34]`,
        // `--lg` is `min-h-8 min-w-8 rounded-2xl text-sm leading-[1.43]` and
        // `--sm` is `min-h-4 min-w-4 rounded-xl text-[10px] leading-[1.34]`.
        // The radius is a *step per size*, not a pill: a large badge is a
        // rounded rectangle, which `rounded_full` could not draw, and its box
        // was 24px where v3 asks for 32. The leadings are Tailwind's unitless
        // multipliers, resolved against the badge's own text size.
        let (size_px, font, radius, leading) = match self.size {
            Size::Sm => (
                px(16.),
                px(10.),
                crate::util::small_radius(cx),
                DefiniteLength::Fraction(1.34),
            ),
            Size::Md => (
                px(28.),
                px(12.),
                crate::util::control_radius(cx),
                DefiniteLength::Fraction(1.34),
            ),
            Size::Lg => (
                px(32.),
                px(14.),
                crate::util::soft_radius(cx),
                DefiniteLength::Fraction(1.43),
            ),
        };

        // Each placement class sits at its corner with `top/right/bottom/left:
        // 0` and then translates itself `±25%` of its own box outward — an
        // overhang of a quarter of the badge: 4px sm, 7px md, 8px lg.
        // `size / 2 - 4` was only ever right for sm.
        //
        // The port can only afford a quarter of the *min* box: GPUI 0.2.2 has
        // no div-level transform, and a percentage `top`/`right` inset would
        // resolve against the containing block, not the badge itself. A badge
        // grown past its min box (a longer label) therefore overhangs less
        // than v3's quarter-of-own-box translate would; only dot and
        // min-box-fitting badges are exact.
        let offset = size_px / -4.0;
        let (top, bottom, left, right) = match self.placement {
            BadgePlacement::TopRight => (Some(offset), None, None, Some(offset)),
            BadgePlacement::TopLeft => (Some(offset), None, Some(offset), None),
            BadgePlacement::BottomRight => (None, Some(offset), None, Some(offset)),
            BadgePlacement::BottomLeft => (None, Some(offset), Some(offset), None),
        };

        let (bg, fg) = paint(colors, self.variant, self.color);

        let badge = gpui::div()
            .absolute()
            .debug_selector(|| "badge".to_owned())
            // `min-h`/`min-w`, not a fixed box: a badge with a longer label
            // grows sideways rather than clipping.
            .min_w(size_px)
            .min_h(size_px)
            .gap(px(2.))
            .rounded(radius)
            .bg(bg)
            .text_color(fg)
            .text_size(font)
            .line_height(leading)
            .font_weight(gpui::FontWeight::MEDIUM)
            .flex()
            // `shrink-0` is pinned on `.badge` itself, not only the anchor.
            // It is inert here — every placement class makes the badge
            // absolute, out of the flex flow — but it is v3's declared value.
            .flex_shrink_0()
            .items_center()
            .justify_center()
            // v3 rings every anchored badge against the page background with
            // `border: 1px solid var(--background)`; there is no prop, because
            // without it the badge and its anchor bleed together.
            .border_1()
            .border_color(colors.background)
            .when_some(top, |b, t| b.top(t))
            .when_some(bottom, |b, v| b.bottom(v))
            .when_some(left, |b, l| b.left(l))
            .when_some(right, |b, r| b.right(r));

        // v3's root renders its children in the badge itself; with no
        // children — the omitted label — the badge is a dot, a circle at the
        // badge size.
        if self.children.is_empty() {
            badge.size(size_px).max_w(size_px)
        } else {
            badge.children(self.children)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pure variant×color paint matrix of `badge.css`: the base rule,
    /// the `.badge--{color}` foreground classes, and the compound
    /// `.badge--{variant}.badge--{color}` cells, over both appearances. The
    /// headless test window cannot sample a fill, so the cascade is pinned
    /// here instead.
    #[test]
    fn paint_matrix_matches_the_badge_css_cascade() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for color in Color::ALL {
                let role = colors.role(color.token());

                // `.badge--primary.badge--{color}` fills with the role itself
                // and labels in the role foreground.
                assert_eq!(
                    paint(&colors, BadgeVariant::Primary, color),
                    (role.color, role.foreground),
                    "primary×{color:?} must fill with the role and label in its foreground"
                );

                // Secondary keeps the base `--badge-bg: var(--default)`
                // whatever the colour; only the label changes, to the
                // soft foreground (`.badge--default` labels in
                // `--default-foreground`).
                assert_eq!(
                    paint(&colors, BadgeVariant::Secondary, color).0,
                    colors.default.color,
                    "secondary×{color:?} must keep the default fill"
                );
                assert_eq!(
                    paint(&colors, BadgeVariant::Secondary, color).1,
                    if color == Color::Default {
                        colors.default.foreground
                    } else {
                        role.soft_foreground(colors.foreground)
                    },
                    "secondary×{color:?} must label in the colour's soft foreground"
                );

                // Soft fills with `--{color}-soft`, lighter than the role
                // itself, and labels in the soft foreground.
                assert_eq!(
                    paint(&colors, BadgeVariant::Soft, color),
                    (role.soft(), role.soft_foreground(colors.foreground)),
                    "soft×{color:?} must fill with the soft mix and label in the soft foreground"
                );
                assert!(
                    role.soft() != role.color,
                    "the soft fill of {color:?} must not equal the solid fill"
                );
            }
        }

        // The roles are distinct fills: no colour may borrow another's
        // primary background.
        let colors = ThemeColors::light();
        for (a, b) in [
            (Color::Default, Color::Accent),
            (Color::Accent, Color::Success),
            (Color::Success, Color::Warning),
            (Color::Warning, Color::Danger),
        ] {
            assert_ne!(
                colors.role(a.token()).color,
                colors.role(b.token()).color,
                "the {a:?} and {b:?} roles must not share a fill"
            );
        }
    }
}
