//! Motion primitives shared by every animated component.
//!
//! HeroUI v3 drives animation from data attributes: overlays fade in on
//! `[data-entering]`, buttons scale on `[data-pressed]`, and everything is
//! suppressed when the user asks for reduced motion — with no opt-in required
//! from the caller.
//!
//! This module is the gpui equivalent. Components call [`entering`] instead of
//! reaching for `with_animation` directly, so the reduced-motion check and the
//! duration/easing live in exactly one place.

use std::time::Duration;

use gpui::{
    ease_out_quint, AnimationExt, AnyElement, App, ElementId, IntoElement,
    StatefulInteractiveElement, StyleRefinement, Styled,
};
use herogpui_theme::ActiveTheme;

/// `[data-entering]` duration — `duration-200` in the v3 stylesheet.
pub const ENTERING_MS: u64 = 200;

/// Duration of a hover/press colour transition.
pub const TRANSITION_MS: u64 = 150;

/// The scale v3 applies to a pressed control (`transform: scale(0.97)`).
pub const PRESSED_SCALE: f32 = 0.97;

/// The inset that shrinks a control of `height` by [`PRESSED_SCALE`] about its
/// centre.
pub fn pressed_inset(height: gpui::Pixels) -> gpui::Pixels {
    gpui::px(f32::from(height) * (1.0 - PRESSED_SCALE) / 2.0)
}

fn shrink(value: gpui::Pixels, by: gpui::Pixels) -> gpui::Pixels {
    gpui::px((f32::from(value) - f32::from(by)).max(0.0))
}

/// `value` scaled by [`PRESSED_SCALE`].
fn scaled(value: gpui::Pixels) -> gpui::Pixels {
    gpui::px(f32::from(value) * PRESSED_SCALE)
}

/// Everything a pressed control scales down.
#[derive(Clone, Copy, Debug)]
pub struct PressBox {
    pub height: gpui::Pixels,
    /// Horizontal padding for a control that sizes to its content, or `None`
    /// for one with a fixed width.
    pub padding_x: Option<gpui::Pixels>,
    /// Fixed width, for a square icon-only control.
    pub width: Option<gpui::Pixels>,
    /// Minimum width, which has to scale too or it pins the box at full size.
    pub min_width: Option<gpui::Pixels>,
    pub text_size: gpui::Pixels,
    pub line_height: gpui::Pixels,
    pub gap: gpui::Pixels,
    pub radius: gpui::Pixels,
    /// False for a full-width control, whose width is its parent's: a
    /// horizontal margin there would overflow rather than inset.
    pub shrink_x: bool,
}

/// Applies v3's `[data-pressed]` press.
///
/// gpui 0.2.2 has no transform for a div — only `paint_svg` takes a
/// transformation matrix — so `scale(0.97)` is reproduced by scaling everything
/// the control is made of: its height, padding, gap, corner radius **and type
/// size**, with margins absorbing what the box gives up so the outer footprint
/// is unchanged and a press never reflows its neighbours.
///
/// Scaling the type is what makes this a real scale rather than an inset: gpui
/// takes fractional font sizes, so the glyphs shrink with the box. Two
/// differences from a CSS transform remain: a label wider than the control's
/// `min_w` narrows the control by ~3% of that overflow, because gpui cannot
/// shrink text without affecting layout; and an icon child keeps its size,
/// since its dimensions belong to the caller.
///
/// Returns `el` untouched under reduced motion.
pub fn pressed(
    el: gpui::Stateful<gpui::Div>,
    b: PressBox,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    if cx.reduce_motion() {
        return el;
    }
    let inset = pressed_inset(b.height);
    el.active(move |s: StyleRefinement| {
        let s = s
            .h(shrink(b.height, inset + inset))
            .mt(inset)
            .mb(inset)
            .text_size(scaled(b.text_size))
            .line_height(scaled(b.line_height))
            .gap(scaled(b.gap))
            .rounded(scaled(b.radius));
        match (b.width, b.shrink_x) {
            // Fixed width: shrink it directly.
            (Some(w), _) => s.w(shrink(w, inset + inset)).ml(inset).mr(inset),
            // Content width: the padding gives way to the margin, and any
            // minimum width scales with it.
            (None, true) => {
                let s = match b.padding_x {
                    Some(px_) => s.px(shrink(px_, inset)).ml(inset).mr(inset),
                    None => s.ml(inset).mr(inset),
                };
                match b.min_width {
                    Some(w) => s.min_w(shrink(w, inset + inset)),
                    None => s,
                }
            }
            // Full width: leave the horizontal axis alone.
            (None, false) => s,
        }
    })
}

/// The scale v3's `zoom-in-90` starts an entering overlay at.
pub const ZOOM_FROM: f32 = 0.90;

/// Everything an entering overlay grows from [`ZOOM_FROM`] to full size.
///
/// Every field is optional because the overlays differ in what they know about
/// themselves: a `Modal` has a width, a `Popover` only its padding, type and
/// corner radius. Whatever is supplied is scaled; whatever is not keeps its
/// size, so a panel sized by its content grows by its chrome alone.
#[derive(Clone, Copy, Debug, Default)]
pub struct ZoomBox {
    pub width: Option<gpui::Pixels>,
    pub height: Option<gpui::Pixels>,
    pub padding_x: Option<gpui::Pixels>,
    pub padding_y: Option<gpui::Pixels>,
    pub gap: Option<gpui::Pixels>,
    pub text_size: Option<gpui::Pixels>,
    pub line_height: Option<gpui::Pixels>,
    pub radius: Option<gpui::Pixels>,
}

impl ZoomBox {
    /// The box for a floating panel: its padding and corner radius, with no
    /// fixed extent.
    pub fn panel(padding_y: gpui::Pixels, radius: gpui::Pixels) -> Self {
        Self {
            padding_y: Some(padding_y),
            radius: Some(radius),
            ..Default::default()
        }
    }

    pub fn padding_x(mut self, padding_x: gpui::Pixels) -> Self {
        self.padding_x = Some(padding_x);
        self
    }

    /// Adds a fixed width, for a panel that has one.
    pub fn sized(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Adds the panel's type size, which grows with the box.
    pub fn text(mut self, text_size: gpui::Pixels) -> Self {
        self.text_size = Some(text_size);
        self
    }
}

fn lerp(value: gpui::Pixels, factor: f32) -> gpui::Pixels {
    gpui::px(f32::from(value) * factor)
}

/// v3's `[data-entering]` in full: `zoom-in-90 fade-in-0 duration-200`.
///
/// gpui 0.2.2 has no transform for a div, so the zoom is reproduced the same
/// way [`pressed`] reproduces `scale(0.97)` — by growing the metrics the panel
/// is made of, including its **type size**, which gpui accepts fractionally.
/// What a real `scale()` would also carry, and this does not, is a child whose
/// size the caller fixed: an icon or an image inside the panel keeps its size
/// while the chrome around it grows.
///
/// Returns `el` untouched under reduced motion.
pub fn entering_zoom<E>(el: E, id: impl Into<ElementId>, b: ZoomBox, cx: &App) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if cx.reduce_motion() {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(ENTERING_MS)).with_easing(ease_out_quint()),
        move |el, delta| {
            let f = ZOOM_FROM + (1.0 - ZOOM_FROM) * delta;
            let mut el = el.opacity(delta);
            if let Some(w) = b.width {
                el = el.w(lerp(w, f));
            }
            if let Some(h) = b.height {
                el = el.h(lerp(h, f));
            }
            if let Some(p) = b.padding_x {
                el = el.px(lerp(p, f));
            }
            if let Some(p) = b.padding_y {
                el = el.py(lerp(p, f));
            }
            if let Some(g) = b.gap {
                el = el.gap(lerp(g, f));
            }
            if let Some(t) = b.text_size {
                el = el.text_size(lerp(t, f));
            }
            if let Some(l) = b.line_height {
                el = el.line_height(lerp(l, f));
            }
            if let Some(r) = b.radius {
                el = el.rounded(lerp(r, f));
            }
            el
        },
    )
    .into_any_element()
}

/// Applies the v3 overlay entry animation: a 200ms ease-out fade.
///
/// The fade alone, for a panel with no metrics worth growing. Prefer
/// [`entering_zoom`], which adds v3's `zoom-in-90`. Returns `el` untouched when
/// the app has reduced motion enabled.
pub fn entering<E>(el: E, id: impl Into<ElementId>, cx: &App) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if cx.reduce_motion() {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(ENTERING_MS)).with_easing(ease_out_quint()),
        |el, delta| el.opacity(delta),
    )
    .into_any_element()
}

/// Like [`entering`] but for content that also slides in — used by `Drawer`,
/// which enters from a window edge.
///
/// `travel` is the distance in pixels the panel covers; it is applied as a
/// margin that relaxes to zero.
pub fn entering_from<E>(
    el: E,
    id: impl Into<ElementId>,
    edge: Edge,
    travel: gpui::Pixels,
    cx: &App,
) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if cx.reduce_motion() {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(ENTERING_MS)).with_easing(ease_out_quint()),
        move |el, delta| {
            let remaining = travel * (1.0 - delta);
            let el = el.opacity(delta);
            match edge {
                Edge::Left => el.ml(-remaining),
                Edge::Right => el.mr(-remaining),
                Edge::Top => el.mt(-remaining),
                Edge::Bottom => el.mb(-remaining),
            }
        },
    )
    .into_any_element()
}

/// Which window edge a sliding panel enters from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn press_inset_matches_the_scale() {
        // scale(0.97) on a 40px control moves each edge in by 1.5% of 40.
        // `1.0 - 0.97` is not exact in f32, so compare with a tolerance.
        assert!((f32::from(pressed_inset(px(40.))) - 0.6).abs() < 1e-4);
        assert!((f32::from(pressed_inset(px(32.))) - 0.48).abs() < 1e-4);
    }

    #[test]
    fn press_preserves_the_outer_footprint() {
        // The margin the box gains is exactly what its height gives up, so a
        // press never moves a neighbour.
        for h in [32.0f32, 40.0, 48.0] {
            let inset = f32::from(pressed_inset(px(h)));
            let shrunk = f32::from(shrink(px(h), pressed_inset(px(h)) + pressed_inset(px(h))));
            assert!((shrunk + inset * 2.0 - h).abs() < 1e-4, "footprint changed at {h}");
        }
    }

    #[test]
    fn shrink_never_goes_negative() {
        assert_eq!(f32::from(shrink(px(1.), px(4.))), 0.0);
    }
}
