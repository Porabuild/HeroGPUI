//! Surface — port of `@heroui/surface`.
//!
//! A container that applies surface-level styling and publishes its variant to
//! descendants. Mirrors the React API: `variant` of
//! `transparent | default | secondary | tertiary`.

use gpui::{
    div, px, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window,
};
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
    padding: Pixels,
    gap: Pixels,
    children: Vec<AnyElement>,
}

impl Surface {
    pub fn new() -> Self {
        Self {
            variant: SurfaceVariant::default(),
            padding: px(24.),
            gap: px(12.),
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: SurfaceVariant) -> Self {
        self.variant = variant;
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
            .rounded(crate::util::container_radius(cx))
            .text_color(colors.foreground);

        el = match self.variant {
            // No fill, so the outline is what marks the surface out — the same
            // treatment `Card`'s `transparent` variant gets.
            SurfaceVariant::Transparent => el
                .border(cx.layout().border_width)
                .border_color(colors.border),
            // Each fill brings its own foreground: `.surface--secondary` is
            // `bg-surface-secondary text-surface-secondary-foreground`, and the
            // text colour was going unset.
            SurfaceVariant::Default => el
                .bg(colors.surface.background)
                .text_color(colors.surface.foreground),
            SurfaceVariant::Secondary => el
                .bg(colors.surface_secondary)
                .text_color(colors.surface_secondary_foreground()),
            SurfaceVariant::Tertiary => el
                .bg(colors.surface_tertiary)
                .text_color(colors.surface_tertiary_foreground()),
        };

        el.children(self.children)
    }
}
