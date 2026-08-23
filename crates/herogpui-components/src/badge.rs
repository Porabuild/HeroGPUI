//! Badge — port of `@heroui/badge`.

use gpui::{prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window};
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
    count: Option<u32>,
    dot: bool,
    invisible: bool,
    show_outline: bool,
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
            count: None,
            dot: false,
            invisible: false,
            show_outline: false,
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

    /// Numeric badge (`count`); values above 99 render as `99+`.
    pub fn count(mut self, n: u32) -> Self {
        self.count = Some(n);
        self
    }

    /// Small dot without content (`isDot`).
    pub fn dot(mut self) -> Self {
        self.dot = true;
        self
    }

    /// Hides the badge while keeping layout (`isInvisible`).
    pub fn invisible(mut self, v: bool) -> Self {
        self.invisible = v;
        self
    }

    pub fn show_outline(mut self, v: bool) -> Self {
        self.show_outline = v;
        self
    }

    /// Custom badge content instead of a number.
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

        let (size_px, font) = match self.size {
            Size::Sm => (px(16.), px(10.)),
            Size::Md => (px(20.), px(12.)),
            Size::Lg => (px(24.), px(14.)),
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
            .min_w(size_px)
            .h(size_px)
            .px(px(3.))
            .rounded_full()
            .bg(bg)
            .text_color(fg)
            .text_size(font)
            .font_weight(gpui::FontWeight::BOLD)
            .flex()
            .items_center()
            .justify_center()
            .when(self.show_outline, |b| {
                b.border_2().border_color(colors.background)
            })
            .when_some(top, |b, t| b.top(t))
            .when_some(bottom, |b, v| b.bottom(v))
            .when_some(left, |b, l| b.left(l))
            .when_some(right, |b, r| b.right(r));

        if self.dot {
            badge = badge.size(size_px).max_w(size_px).px(px(0.));
        } else if let Some(count) = self.count {
            badge = badge.child(if count > 99 { "99+".to_string() } else { count.to_string() });
        } else if let Some(content) = self.content {
            badge = badge.child(content);
        }

        let hidden = self.invisible;

        gpui::div()
            .relative()
            .flex()
            .children(self.children)
            .when(!hidden, |el| el.child(badge))
    }
}
