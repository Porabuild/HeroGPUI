//! Badge — port of `@heroui/badge`.

use gpui::{
    prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

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
    /// The color at 15% with colored text.
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

/// HeroUI Badge wrapper.
#[derive(IntoElement)]
pub struct Badge {
    color: Color,
    variant: BadgeVariant,
    size: Size,
    placement: BadgePlacement,
    content: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl Badge {
    pub fn new() -> Self {
        Self {
            color: Color::Danger,
            variant: BadgeVariant::Primary,
            size: Size::Md,
            placement: BadgePlacement::TopRight,
            content: None,
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

    /// The badge's own content — v3's `<Badge>5</Badge>` children.
    ///
    /// This builder's [`ParentElement`] children are the *anchor* instead
    /// (v3's `Badge.Anchor`), since one struct stands in for both parts.
    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
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

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let colors = cx.colors();

        // `.badge` is `min-h-7 min-w-7 rounded-3xl text-xs`, `--lg` is
        // `min-h-8 min-w-8 rounded-2xl text-sm` and `--sm` is `min-h-4 min-w-4
        // rounded-xl text-[10px]`. The radius is a *step per size*, not a pill:
        // a large badge is a rounded rectangle, which `rounded_full` could not
        // draw, and its box was 24px where v3 asks for 32.
        let (size_px, font, radius) = match self.size {
            Size::Sm => (px(16.), px(10.), crate::util::small_radius(cx)),
            Size::Md => (px(28.), px(12.), crate::util::control_radius(cx)),
            Size::Lg => (px(32.), px(14.), crate::util::soft_radius(cx)),
        };

        let offset = size_px / -2.0 + px(4.);
        let (top, bottom, left, right) = match self.placement {
            BadgePlacement::TopRight => (Some(offset), None, None, Some(offset)),
            BadgePlacement::TopLeft => (Some(offset), None, Some(offset), None),
            BadgePlacement::BottomRight => (None, Some(offset), None, Some(offset)),
            BadgePlacement::BottomLeft => (None, Some(offset), Some(offset), None),
        };

        let (bg, fg) = match self.variant {
            BadgeVariant::Primary => (sem.color, sem.foreground),
            BadgeVariant::Secondary => (
                colors.default.color,
                if self.color == Color::Default {
                    colors.default.foreground
                } else {
                    sem.soft_foreground()
                },
            ),
            BadgeVariant::Soft => (sem.soft(), sem.soft_foreground()),
        };

        let mut badge = gpui::div()
            .absolute()
            // `min-h`/`min-w`, not a fixed box: a badge with a longer label
            // grows sideways rather than clipping.
            .min_w(size_px)
            .min_h(size_px)
            .gap(px(2.))
            .rounded(radius)
            .bg(bg)
            .text_color(fg)
            .text_size(font)
            .font_weight(gpui::FontWeight::MEDIUM)
            .flex()
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

        // No content is v3's dot badge: a circle at the badge size.
        badge = match self.content {
            // `.badge__label` is `px-0.5`, which is the only horizontal padding
            // in v3's sheet -- the badge itself has none.
            Some(content) => badge.child(gpui::div().px(px(2.)).child(content)),
            None => badge.size(size_px).max_w(size_px),
        };

        gpui::div()
            .relative()
            .flex()
            .children(self.children)
            .child(badge)
    }
}
