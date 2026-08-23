//! Surface — port of `@heroui/surface`.
//!
//! A container that applies surface-level styling and publishes its variant to
//! descendants. Mirrors the React API: `variant` of
//! `transparent | default | secondary | tertiary`.

use gpui::{div, px, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window};
use herogpui_theme::ActiveTheme;

/// Prominence level of a surface (`variant` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceVariant {
    /// No background — for overlays and cards with a custom background.
    Transparent,
    /// `bg-surface`.
    #[default]
    Default,
    /// `bg-surface-secondary`.
    Secondary,
    /// `bg-surface-tertiary`.
    Tertiary,
}

/// HeroUI Surface.
#[derive(IntoElement)]
pub struct Surface {
    variant: SurfaceVariant,
    radius: Pixels,
    padding: Pixels,
    gap: Pixels,
    bordered: bool,
    children: Vec<AnyElement>,
}

impl Surface {
    pub fn new() -> Self {
        Self {
            variant: SurfaceVariant::default(),
            radius: px(24.),
            padding: px(24.),
            gap: px(12.),
            bordered: false,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: SurfaceVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = radius.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }

    /// Draws a 1px border — the documented pairing for the transparent variant.
    pub fn bordered(mut self, v: bool) -> Self {
        self.bordered = v;
        self
    }
}

impl Default for Surface {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Surface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Surface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let mut el = div()
            .flex()
            .flex_col()
            .gap(self.gap)
            .p(self.padding)
            .rounded(self.radius)
            .text_color(colors.foreground);

        el = match self.variant {
            SurfaceVariant::Transparent => el,
            SurfaceVariant::Default => el.bg(colors.surface.background),
            SurfaceVariant::Secondary => el.bg(colors.surface_secondary),
            SurfaceVariant::Tertiary => el.bg(colors.surface_tertiary),
        };

        if self.bordered {
            el = el.border_1().border_color(colors.border);
        }

        el.children(self.children)
    }
}
