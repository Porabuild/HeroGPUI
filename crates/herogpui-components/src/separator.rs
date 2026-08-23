//! Separator — port of `@heroui/separator` (v3, formerly `Divider`).
//!
//! `variant` pairs with the surrounding [`Surface`](crate::surface::Surface)
//! prominence so the line stays visible as the container gets more prominent.

use gpui::{div, App, IntoElement, Pixels, RenderOnce, Styled, Window};
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
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            variant: SeparatorVariant::default(),
            inset_y: gpui::px(0.),
            inset_x: gpui::px(0.),
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

        let el = div()
            .my(self.inset_y)
            .mx(self.inset_x)
            .flex_shrink_0()
            .rounded(crate::util::hairline_radius(cx))
            .bg(color);

        match self.orientation {
            Orientation::Horizontal => el.w_full().h(weight),
            Orientation::Vertical => el.h_full().w(weight),
        }
    }
}
