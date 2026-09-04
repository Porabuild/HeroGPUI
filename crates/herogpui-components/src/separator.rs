//! Separator — port of `@heroui/separator` (v3, formerly `Divider`).
//!
//! `variant` pairs with the surrounding [`Surface`](crate::surface::Surface)
//! prominence so the line stays visible as the container gets more prominent.

use gpui::{div, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window};
use herogpui_core::Orientation;
use herogpui_theme::ActiveTheme;

/// Visual variant of a separator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorVariant {
    /// `--separator`
    #[default]
    Default,
    /// `color-mix(in oklab, surface 85%, surface-foreground 15%)`
    Secondary,
    /// `color-mix(in oklab, surface 81%, surface-foreground 19%)`
    Tertiary,
}

impl SeparatorVariant {
    pub const ALL: [SeparatorVariant; 3] = [
        SeparatorVariant::Default,
        SeparatorVariant::Secondary,
        SeparatorVariant::Tertiary,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SeparatorVariant::Default => "Default",
            SeparatorVariant::Secondary => "Secondary",
            SeparatorVariant::Tertiary => "Tertiary",
        }
    }
}

/// HeroUI Separator.
#[derive(IntoElement)]
pub struct Separator {
    orientation: Orientation,
    variant: SeparatorVariant,
    inset_y: Pixels,
    inset_x: Pixels,
    /// v3 composes content *inside* a separator (`<Separator>OR</Separator>`),
    /// which turns it into `.separator__container`: a line, the content, a line.
    content: Vec<AnyElement>,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            variant: SeparatorVariant::default(),
            inset_y: gpui::px(0.),
            inset_x: gpui::px(0.),
            content: Vec::new(),
        }
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn variant(mut self, variant: SeparatorVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Vertical inset. gpui has no `className`, so the margin v3 sets with
    /// `my-*` is a builder here.
    pub fn my(mut self, v: impl Into<Pixels>) -> Self {
        self.inset_y = v.into();
        self
    }

    /// Horizontal inset — the `mx-*` counterpart of [`Separator::my`].
    pub fn mx(mut self, v: impl Into<Pixels>) -> Self {
        self.inset_x = v.into();
        self
    }
}

impl ParentElement for Separator {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.content.extend(elements);
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let weight = cx.layout().border_width;
        let color = match self.variant {
            SeparatorVariant::Default => colors.separator,
            SeparatorVariant::Secondary => colors.separator_secondary(),
            SeparatorVariant::Tertiary => colors.separator_tertiary(),
        };

        let horizontal = self.orientation == Orientation::Horizontal;
        let radius = crate::util::hairline_radius(cx);
        let line = move || {
            let el = div().flex_shrink_0().rounded(radius).bg(color);
            if horizontal {
                // `.separator__line` is `shrink-0 grow`, so a line beside
                // content takes the space the content leaves.
                el.h(weight).flex_grow(1.)
            } else {
                // `.separator--vertical` is `min-h-2`: a vertical rule between
                // two inline items still draws when its row is shorter.
                el.w(weight).min_h(gpui::px(8.)).flex_grow(1.)
            }
        };

        if !self.content.is_empty() {
            // `.separator__container` is `flex items-center gap-3`, row when
            // horizontal and a centred column when vertical.
            let mut el = div()
                .my(self.inset_y)
                .mx(self.inset_x)
                .flex()
                .items_center()
                .gap(gpui::px(12.));
            el = if horizontal {
                el.w_full().flex_row()
            } else {
                el.h_full().flex_col().justify_center()
            };
            return el
                .child(line())
                .child(
                    // `.separator__content` is a centred, non-wrapping run of
                    // `--muted` text.
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .text_center()
                        .whitespace_nowrap()
                        .text_color(colors.muted)
                        .children(self.content),
                )
                .child(line())
                .into_any_element();
        }

        let el = div()
            .my(self.inset_y)
            .mx(self.inset_x)
            .flex_shrink_0()
            .rounded(radius)
            .bg(color);

        match self.orientation {
            Orientation::Horizontal => el.w_full().h(weight),
            Orientation::Vertical => el.h_full().min_h(gpui::px(8.)).w(weight),
        }
        .into_any_element()
    }
}
