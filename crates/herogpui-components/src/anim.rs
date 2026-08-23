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

/// Applies v3's `[data-pressed]` press to a control that sizes to its content.
///
/// gpui 0.2.2 has no transform for a div — only `paint_svg` takes a
/// transformation matrix, so quads and text cannot be scaled. The box is
/// therefore shrunk geometrically: horizontal padding gives way to an equal
/// margin, and the height loses what the vertical margin gains. The outer
/// footprint is identical, so pressing never moves a neighbour, and the visible
/// control contracts about its centre. The glyphs keep their size, which is the
/// one visible difference from `scale(0.97)`.
///
/// `shrink_x` should be false for a full-width control, whose width is already
/// its parent's: adding a horizontal margin there would overflow instead of
/// inset.
///
/// Returns `el` untouched under reduced motion.
pub fn pressed_padded(
    el: gpui::Stateful<gpui::Div>,
    height: gpui::Pixels,
    padding_x: gpui::Pixels,
    shrink_x: bool,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    if cx.reduce_motion() {
        return el;
    }
    let inset = pressed_inset(height);
    el.active(move |s: StyleRefinement| {
        let s = s.h(shrink(height, inset + inset)).mt(inset).mb(inset);
        if shrink_x {
            s.px(shrink(padding_x, inset)).ml(inset).mr(inset)
        } else {
            s
        }
    })
}

/// Applies the press to a control with a fixed size, such as a square
/// icon-only button. See [`pressed_padded`] for why this is geometric.
pub fn pressed_fixed(
    el: gpui::Stateful<gpui::Div>,
    height: gpui::Pixels,
    width: gpui::Pixels,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    if cx.reduce_motion() {
        return el;
    }
    let inset = pressed_inset(height);
    el.active(move |s: StyleRefinement| {
        s.h(shrink(height, inset + inset))
            .w(shrink(width, inset + inset))
            .mt(inset)
            .mb(inset)
            .ml(inset)
            .mr(inset)
    })
}

/// Applies the v3 overlay entry animation: a 200ms ease-out fade.
///
/// v3 pairs the fade with `zoom-in-90`, which gpui 0.2.2 cannot express for a
/// div, so this is the fade alone. Returns `el` untouched when the app has
/// reduced motion enabled.
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
