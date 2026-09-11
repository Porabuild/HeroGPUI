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
    /// The corner radius, in place of the owning `hairline_radius` helper.
    radius: Option<Pixels>,
    children: Vec<AnyElement>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            id: "skeleton".into(),
            w: None,
            h: Some(px(24.)),
            animation_type: None,
            radius: None,
            children: Vec::new(),
            sx: None,
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

    /// The corner radius, in place of the owning `hairline_radius` helper. Not
    /// a v3 prop; the removed v2 `radius` prop is prohibited and this is a
    /// per-component repository extension.
    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the skeleton's root element after every value the component
    /// and the active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(crate::util::capture_sx(style));
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
            .rounded(
                self.radius
                    .unwrap_or_else(|| crate::util::hairline_radius(cx)),
            )
            .overflow_hidden()
            .when_some(self.w, |el, w| el.w(w))
            .when_some(self.h, |el, h| el.h(h))
            .when(!self.children.is_empty(), |el| {
                el.child(div().opacity(0.).children(self.children))
            });

        match animation {
            SkeletonAnimation::None => crate::util::apply_sx(base, &self.sx).into_any_element(),
            // `with_animation` hands back an `AnimationElement`, which has no
            // style of its own to refine, so the slot lands on the box the
            // animation wraps: the pulse keeps driving opacity, everything
            // else the caller set holds.
            SkeletonAnimation::Pulse => crate::util::apply_sx(base, &self.sx)
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
                crate::util::apply_sx(base.child(band), &self.sx).into_any_element()
            }
        }
    }
}
