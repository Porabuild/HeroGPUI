//! Skeleton — port of `@heroui/skeleton` (v3).
//!
//! `animationType` selects between the sweeping shimmer, a pulse, and no
//! motion; it defaults to the theme's `--skeleton-animation` token.

use std::time::Duration;

use gpui::{
    div, prelude::*, px, Animation, AnimationExt, AnyElement, App, ElementId, IntoElement,
    ParentElement, Pixels, RenderOnce, Styled, Window,
};
use herogpui_theme::{ActiveTheme, SkeletonAnimation};

/// Loading placeholder (`Skeleton`).
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    w: Option<Pixels>,
    h: Option<Pixels>,
    /// `animationType`. `None` defers to `--skeleton-animation`.
    animation_type: Option<SkeletonAnimation>,
    children: Vec<AnyElement>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            id: "skeleton".into(),
            w: None,
            h: Some(px(24.)),
            animation_type: None,
            children: Vec::new(),
        }
    }

    /// Distinct id per skeleton; required when several animate on one page.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn w(mut self, v: impl Into<Pixels>) -> Self {
        self.w = Some(v.into());
        self
    }

    pub fn h(mut self, v: impl Into<Pixels>) -> Self {
        self.h = Some(v.into());
        self
    }

    pub fn animation_type(mut self, animation: SkeletonAnimation) -> Self {
        self.animation_type = Some(animation);
        self
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Skeleton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        // `.skeleton` is `bg-surface-tertiary/70`, not the solid token: the
        // placeholder is meant to read as a tint of whatever it sits on. This
        // painted it opaque, so every skeleton came out the full tertiary fill
        // -- (234,234,235) on the light page against v3's (237,237,238).
        let base_color = colors.surface_tertiary.alpha(0.7);
        // Reduced motion collapses every animation type to `None`.
        let animation = if ActiveTheme::reduce_motion(cx) {
            SkeletonAnimation::None
        } else {
            self.animation_type
                .unwrap_or(cx.layout().skeleton_animation)
        };

        let base = div()
            .bg(base_color)
            .rounded(crate::util::hairline_radius(cx))
            .overflow_hidden()
            .when_some(self.w, |el, w| el.w(w))
            .when_some(self.h, |el, h| el.h(h))
            .when(!self.children.is_empty(), |el| {
                el.child(div().opacity(0.).children(self.children))
            });

        match animation {
            SkeletonAnimation::None => base.into_any_element(),
            SkeletonAnimation::Pulse => base
                .with_animation(
                    self.id,
                    Animation::new(Duration::from_millis(1600)).repeat(),
                    move |el, delta| {
                        let t = (delta * std::f32::consts::TAU).sin();
                        el.opacity(0.55 + 0.25 * t)
                    },
                )
                .into_any_element(),
            // A highlight band sweeping left to right, like v3's shimmer. The
            // band itself is animated, so its position moves rather than its
            // size.
            SkeletonAnimation::Shimmer => {
                let highlight = colors.background;
                let band = div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .w(gpui::relative(0.35))
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(highlight.alpha(0.0), 0.0),
                        gpui::linear_color_stop(highlight.alpha(0.7), 0.5),
                    ))
                    .with_animation(
                        self.id,
                        Animation::new(Duration::from_millis(1400)).repeat(),
                        |el, delta| el.left(gpui::relative(delta)),
                    );
                base.child(band).into_any_element()
            }
        }
    }
}
