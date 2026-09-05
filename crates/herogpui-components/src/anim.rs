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
    AnimationExt, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, StyleRefinement, Styled,
};
use herogpui_theme::ActiveTheme;

/// `[data-entering]` duration for the common case — most overlays are
/// `duration-150`. Panels and `Autocomplete` are 250; see [`Motion`].
pub const ENTERING_MS: u64 = 150;

/// How long `.button`'s fill takes to change, read from its own declaration:
/// `background-color 100ms var(--ease-out)`.
///
/// 150ms is the commonest duration across v3's sheets (88 declarations to
/// 100ms's 33), but the button states its own, and this is the button's.
pub const TRANSITION_MS: u64 = 100;

/// How long a press takes: `transform 250ms var(--ease-smooth)`.
///
/// Recorded rather than used — gpui's `active` is a style swap with no
/// timeline, so [`pressed`] arrives in one frame. See the note there.
pub const PRESS_MS: u64 = 250;

/// `progress-bar-indeterminate`: one sweep every 1.5 seconds.
pub const PROGRESS_BAR_INDETERMINATE_MS: u64 = 1500;
/// `.progress-bar__fill` width transition duration.
pub const PROGRESS_BAR_FILL_MS: u64 = 300;

/// v3's indeterminate ProgressBar curve.
pub fn progress_bar_indeterminate_ease() -> impl Fn(f32) -> f32 {
    |t| cubic_bezier(0.65, 0.0, 0.35, 1.0, t)
}

/// `@keyframes progress-circle-spin`: one linear turn per second.
pub const PROGRESS_CIRCLE_SPIN_MS: u64 = 1000;

/// Rotation for one `progress-circle-spin` iteration, in radians.
pub fn progress_circle_spin_turn(delta: f32) -> f32 {
    delta.clamp(0.0, 1.0) * std::f32::consts::TAU
}

/// Evaluates a CSS `cubic-bezier(x1, y1, x2, y2)` at `t`.
///
/// v3 names its curves in `--ease-*` tokens and gpui takes an arbitrary easing
/// function, so the real curves can be used rather than approximated by
/// whichever of gpui's two built-ins looks closest.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    // A cubic Bezier from (0,0) to (1,1); `t` is the x we want a y for, so the
    // curve parameter has to be solved for first.
    let bez = |a: f32, b: f32, u: f32| {
        let v = 1.0 - u;
        3.0 * v * v * u * a + 3.0 * v * u * u * b + u * u * u
    };
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    let mut u = t;
    // Bisection: monotonic in x, and 24 halvings is well under a pixel.
    for _ in 0..24 {
        let x = bez(x1, x2, u);
        if x < t {
            lo = u;
        } else {
            hi = u;
        }
        u = (lo + hi) * 0.5;
    }
    bez(y1, y2, u)
}

/// v3's `--ease-out` — Tailwind's default, `cubic-bezier(0, 0, 0.2, 1)`.
pub fn ease_out() -> impl Fn(f32) -> f32 {
    |t| cubic_bezier(0.0, 0.0, 0.2, 1.0, t)
}

/// One of v3's `--ease-*` curves.
///
/// Named rather than passed as a closure so a [`Motion`] stays `Copy` and can be
/// a `const`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    /// `--ease-out`, Tailwind's default: `cubic-bezier(0, 0, 0.2, 1)`.
    Out,
    /// `--ease-smooth`, CSS `ease`: `cubic-bezier(0.25, 0.1, 0.25, 1)`.
    Smooth,
    /// `--ease-out-quad`: `cubic-bezier(0.25, 0.46, 0.45, 0.94)`.
    OutQuad,
    /// `--ease-out-fluid`: `cubic-bezier(0.32, 0.72, 0, 1)`.
    OutFluid,
    Linear,
}

impl Curve {
    pub fn at(self, t: f32) -> f32 {
        match self {
            Curve::Out => cubic_bezier(0.0, 0.0, 0.2, 1.0, t),
            Curve::Smooth => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Curve::OutQuad => cubic_bezier(0.25, 0.46, 0.45, 0.94, t),
            Curve::OutFluid => cubic_bezier(0.32, 0.72, 0.0, 1.0, t),
            Curve::Linear => t,
        }
    }
}

/// The duration, scale and curve v3 declares for one overlay's transition.
///
/// v3 does **not** animate every overlay the same way, which is what reading the
/// guide rather than the stylesheets had suggested. Each surface names its own
/// `duration-*`, `ease-*` and `zoom-*`, and a modal panel even *shrinks* in from
/// 105% rather than growing from 90%. The constants below are transcribed one
/// per group, and `anim_audit.py` checks them against the CSS.
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub ms: u64,
    /// The scale the animation starts at (entering) or ends at (exiting).
    /// `1.0` means no scaling — a fade alone.
    pub scale: f32,
    pub curve: Curve,
}

impl Motion {
    /// `duration-250 ease-out-quad zoom-in-105` — `Modal` and `AlertDialog`
    /// panels, which settle *down* onto the page.
    pub const PANEL_IN: Motion = Motion {
        ms: 250,
        scale: 1.05,
        curve: Curve::OutQuad,
    };
    /// `duration-100 ease-out-quad zoom-out-95`.
    pub const PANEL_OUT: Motion = Motion {
        ms: 100,
        scale: 0.95,
        curve: Curve::OutQuad,
    };

    /// `duration-150 ease-out fade-in-0` — the backdrop behind a panel, which
    /// only fades.
    pub const BACKDROP_IN: Motion = Motion {
        ms: 150,
        scale: 1.0,
        curve: Curve::Out,
    };
    /// `duration-100 ease-out fade-out-0`.
    pub const BACKDROP_OUT: Motion = Motion {
        ms: 100,
        scale: 1.0,
        curve: Curve::Out,
    };

    /// `duration-150 ease-smooth zoom-in-90` — `Popover`, `Dropdown`, `Tooltip`.
    pub const POPOVER_IN: Motion = Motion {
        ms: 150,
        scale: 0.90,
        curve: Curve::Smooth,
    };
    /// `duration-150 ease-smooth zoom-in-95` — `Select`, `ComboBox`, the date
    /// and colour pickers, which start closer to full size.
    pub const LIST_IN: Motion = Motion {
        ms: 150,
        scale: 0.95,
        curve: Curve::Smooth,
    };
    /// `duration-100 ease-smooth zoom-out-95` — the exit both share.
    pub const LIST_OUT: Motion = Motion {
        ms: 100,
        scale: 0.95,
        curve: Curve::Smooth,
    };

    /// `.disclosure__content` is a *transition*, not an `animate-in`: `height
    /// 200ms ease-out-quad, opacity 200ms ease-out`. gpui cannot animate a
    /// height it has not measured, so the panel fades at that duration and
    /// curve rather than sliding.
    pub const DISCLOSURE: Motion = Motion {
        ms: 200,
        scale: 1.0,
        curve: Curve::OutQuad,
    };

    /// `translate 250ms cubic-bezier(0.32, 0.72, 0, 1)` — the drawer's slide,
    /// which `drawer.css` gives its own `--drawer-enter-*` tokens.
    pub const DRAWER_IN: Motion = Motion {
        ms: 250,
        scale: 1.0,
        curve: Curve::OutFluid,
    };
    /// `--drawer-exit-duration: 200ms`, same curve.
    pub const DRAWER_OUT: Motion = Motion {
        ms: 200,
        scale: 1.0,
        curve: Curve::OutFluid,
    };

    /// `duration-250 ease-out-fluid zoom-in-95` — `Autocomplete` alone.
    pub const FLUID_IN: Motion = Motion {
        ms: 250,
        scale: 0.95,
        curve: Curve::OutFluid,
    };
    /// `duration-100 ease-out-quad zoom-out-95`.
    pub const FLUID_OUT: Motion = Motion {
        ms: 100,
        scale: 0.95,
        curve: Curve::OutQuad,
    };
}

/// v3's `--ease-smooth`, which is CSS `ease`: `cubic-bezier(0.25, 0.1, 0.25, 1)`.
pub fn ease_smooth() -> impl Fn(f32) -> f32 {
    |t| cubic_bezier(0.25, 0.1, 0.25, 1.0, t)
}

/// The scale v3 applies to a pressed control (`transform: scale(0.97)`).
pub const PRESSED_SCALE: f32 = 0.97;

/// The other scales v3 presses with: a menu row and a pagination link squeeze
/// less than a button, a calendar cell and a radio control more, and a range
/// calendar cell most.
pub const PRESSED_SCALE_SUBTLE: f32 = 0.98;
pub const PRESSED_SCALE_FIRM: f32 = 0.96;
pub const PRESSED_SCALE_DEEP: f32 = 0.95;
pub const PRESSED_SCALE_RANGE: f32 = 0.9;

/// The inset that shrinks a control of `height` by a scale about its
/// centre.
pub fn pressed_inset(height: gpui::Pixels) -> gpui::Pixels {
    inset_for(height, PRESSED_SCALE)
}

/// The inset that shrinks `height` by `scale`, centred.
fn inset_for(height: gpui::Pixels, scale: f32) -> gpui::Pixels {
    gpui::px(f32::from(height) * (1.0 - scale) / 2.0)
}

fn shrink(value: gpui::Pixels, by: gpui::Pixels) -> gpui::Pixels {
    gpui::px((f32::from(value) - f32::from(by)).max(0.0))
}

/// `value` scaled by `scale`.
fn scaled_by(value: gpui::Pixels, scale: f32) -> gpui::Pixels {
    gpui::px(f32::from(value) * scale)
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
    /// How far the press scales. v3 uses 0.97 for a button, 0.98 for a menu row,
    /// 0.96 and 0.95 for the smaller controls, so it is per control rather than
    /// one constant.
    pub scale: f32,
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
/// **The press arrives in one frame.** v3 declares
/// `transform 250ms var(--ease-smooth)` ([`PRESS_MS`]), but gpui's `active` is a
/// style swap with no timeline to animate along, so the scale lands instantly
/// and springs back instantly. Everything about the geometry matches; only the
/// quarter-second ramp is missing.
///
/// Returns `el` untouched under reduced motion.
pub fn pressed(el: gpui::Stateful<gpui::Div>, b: PressBox, cx: &App) -> gpui::Stateful<gpui::Div> {
    pressed_with_optional_background(el, b, None, cx)
}

/// Applies the same press geometry and an active-state background in one
/// refinement, for controls whose CSS changes both on `[data-pressed]`.
pub fn pressed_with_background(
    el: gpui::Stateful<gpui::Div>,
    b: PressBox,
    background: gpui::Hsla,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    pressed_with_optional_background(el, b, Some(background), cx)
}

fn pressed_with_optional_background(
    el: gpui::Stateful<gpui::Div>,
    b: PressBox,
    background: Option<gpui::Hsla>,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    if ActiveTheme::reduce_motion(cx) {
        return match background {
            Some(background) => el.active(move |style| style.bg(background)),
            None => el,
        };
    }
    let inset = inset_for(b.height, b.scale);
    el.active(move |s: StyleRefinement| {
        let s = match background {
            Some(background) => s.bg(background),
            None => s,
        };
        let s = s
            .h(shrink(b.height, inset + inset))
            .mt(inset)
            .mb(inset)
            .text_size(scaled_by(b.text_size, b.scale))
            .line_height(scaled_by(b.line_height, b.scale))
            .gap(scaled_by(b.gap, b.scale))
            .rounded(scaled_by(b.radius, b.scale));
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

/// `[data-exiting]` duration. Every overlay in v3 leaves in `duration-100`.
pub const EXITING_MS: u64 = 100;

/// Everything an entering overlay grows from `ZOOM_FROM` to full size.
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
    pub padding_top: Option<gpui::Pixels>,
    pub padding_bottom: Option<gpui::Pixels>,
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
pub fn entering_zoom<E>(
    el: E,
    id: impl Into<ElementId>,
    b: ZoomBox,
    m: Motion,
    cx: &App,
) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if ActiveTheme::reduce_motion(cx) {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(m.ms)).with_easing(move |t| m.curve.at(t)),
        move |el, delta| {
            // `scale` may be above 1.0: a modal panel settles down from 105%.
            let f = m.scale + (1.0 - m.scale) * delta;
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
            if let Some(p) = b.padding_top {
                el = el.pt(lerp(p, f));
            }
            if let Some(p) = b.padding_bottom {
                el = el.pb(lerp(p, f));
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

/// v3's `[data-exiting]`: `animate-out zoom-out-95 fade-out duration-150`.
///
/// The mirror of [`entering_zoom`] — the panel shrinks to `ZOOM_TO` and fades
/// as it leaves. It only has anything to animate because the component keeps
/// rendering for [`EXITING_MS`] after `isOpen` goes false; see
/// [`crate::util::overlay_phase`].
///
/// Returns `el` untouched under reduced motion, which is also what makes the
/// panel disappear immediately: with nothing to animate, the extra frames are
/// invisible.
pub fn exiting<E>(el: E, id: impl Into<ElementId>, b: ZoomBox, m: Motion, cx: &App) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if ActiveTheme::reduce_motion(cx) {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(m.ms)).with_easing(move |t| m.curve.at(t)),
        move |el, delta| {
            // `delta` runs 0 -> 1 over the exit, so the scale runs 1 -> m.scale.
            let f = 1.0 - (1.0 - m.scale) * delta;
            let mut el = el.opacity(1.0 - delta);
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
            if let Some(p) = b.padding_top {
                el = el.pt(lerp(p, f));
            }
            if let Some(p) = b.padding_bottom {
                el = el.pb(lerp(p, f));
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

/// v3's `transition-colors`: the background eases between two colours instead
/// of switching on the frame the pointer arrives.
///
/// gpui has no property transitions — `hover` swaps the style outright — so the
/// element keeps its own hover flag and a generation counter, and each change
/// starts a fresh animation that interpolates in OKLab. `colors` is the
/// `(idle, hovered)` pair — the two ends; everything else about the element is
/// untouched.
///
/// `interaction` is the hover source when the element already records one: a
/// `content` closure's `isHovered` needs the same enter/leave that drives this
/// fade, and gpui allows exactly one `on_hover` per element, so when the caller
/// wired `util::track_interaction` the fade reads the slot it keeps rather than
/// binding a second listener. `None` leaves the fade owning its own listener.
/// Either way exactly one `on_hover` is bound.
///
/// **The animated colour lives on an absolutely-positioned child fill, not the
/// element itself.** gpui keys element state by the *full* element-id path, and
/// `with_animation` restarts by changing its id — if the animation wrapped the
/// element, the id change would shift the element's path and reset every
/// listener latch on it, so hover-out was silently lost the moment the fade
/// wrapper appeared (and the button stayed in its hover colour). Here the
/// element keeps a constant id — a stable path, working hover listeners — and
/// only the fill's animation id moves; the fill has no listeners or hitbox to
/// lose. `round_corners` shapes the fill like the element itself (a group
/// member's corners are partial).
///
/// Returns the element with a plain `hover` swap under reduced motion, so the
/// state is still visible without motion.
pub fn hover_fade(
    el: gpui::Stateful<gpui::Div>,
    id: impl Into<ElementId>,
    colors: (gpui::Hsla, gpui::Hsla),
    interaction: Option<&crate::util::Interaction>,
    round_corners: impl Fn(gpui::Div) -> gpui::Div,
    window: &mut gpui::Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div> {
    let (idle, hovered) = colors;
    let id = id.into();
    if ActiveTheme::reduce_motion(cx) {
        return el.bg(idle).hover(move |s: StyleRefinement| s.bg(hovered));
    }

    let state = window.use_keyed_state(id.clone(), cx, |_, _| HoverFade::default());
    let mut current = *state.read(cx);

    // One hover listener for the whole element. `util::track_interaction` owns
    // `on_hover` when the interaction slot exists; the fade then reads the
    // hover bit it keeps and only has to notice when the bit *changed*, which
    // is the same generation bump the listener used to perform itself.
    // `relative()` makes the element the containing block the fill stretches
    // across; the resting colour is the element's own, and the fill overlays it
    // between generations.
    let mut el = el
        .relative()
        .bg(if current.hovered { hovered } else { idle });
    el = match interaction {
        Some(slot) => {
            let hovered_now = slot.read(cx).0;
            if hovered_now != current.hovered {
                current.hovered = hovered_now;
                current.generation = current.generation.wrapping_add(1);
                // The refresh that repaints this frame was already requested by
                // `track_interaction`'s handler; the new animation id starts
                // the transition here, and `with_animation` keeps its own
                // frames coming until it settles.
                state.update(cx, |s, _| *s = current);
            }
            el
        }
        None => {
            let held = state.clone();
            el.on_hover(move |over: &bool, _, cx| {
                let over = *over;
                held.update(cx, |s, cx| {
                    if s.hovered != over {
                        s.hovered = over;
                        // A new generation gives the fill's animation a new id,
                        // which is what restarts it mid-flight when the pointer
                        // turns around.
                        s.generation = s.generation.wrapping_add(1);
                        cx.notify();
                    }
                });
            })
        }
    };

    // Generation 0 is the first render: the resting colour on the element IS
    // the state, and there is nothing to ease from yet.
    if current.generation == 0 {
        return el;
    }
    let (from, to) = if current.hovered {
        (idle, hovered)
    } else {
        (hovered, idle)
    };
    // The fill sits under everything the caller adds afterwards: it exactly
    // covers the rounded element and carries only the colour transition, so the
    // element's own state — and its hover listeners — survive the id change.
    el.child(
        round_corners(gpui::div().absolute().inset_0()).with_animation(
            ElementId::Name(format!("{id:?}-fade-{}", current.generation).into()),
            gpui::Animation::new(Duration::from_millis(TRANSITION_MS)).with_easing(ease_out()),
            move |fill, delta| fill.bg(herogpui_core::mix_oklab(from, to, delta)),
        ),
    )
}

/// The hover flag and restart counter [`hover_fade`] keeps per element.
#[derive(Clone, Copy, Debug, Default)]
struct HoverFade {
    hovered: bool,
    generation: usize,
}

/// v3's `@keyframes caret-blink`: opaque at 0/70/100%, transparent at 20/50%.
///
/// Reproduced as a repeating 1s animation over the same stops, so a text caret
/// blinks the way it does on the web instead of sitting solid.
pub fn caret_blink<E>(el: E, id: impl Into<ElementId>, cx: &App) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if ActiveTheme::reduce_motion(cx) {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(1000)).repeat(),
        |el, delta| el.opacity(caret_opacity(delta)),
    )
    .into_any_element()
}

/// The `caret-blink` keyframe curve, linear between its stops.
fn caret_opacity(delta: f32) -> f32 {
    match delta {
        d if d < 0.20 => 1.0 - (d / 0.20),
        d if d < 0.50 => 0.0,
        d if d < 0.70 => (d - 0.50) / 0.20,
        _ => 1.0,
    }
}

/// Applies the v3 overlay entry animation: a 200ms ease-out fade.
///
/// The fade alone, for a panel with no metrics worth growing. Prefer
/// [`entering_zoom`], which adds v3's `zoom-in-90`. Returns `el` untouched when
/// the app has reduced motion enabled.
pub fn entering<E>(el: E, id: impl Into<ElementId>, m: Motion, cx: &App) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if ActiveTheme::reduce_motion(cx) {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(m.ms)).with_easing(move |t| m.curve.at(t)),
        Styled::opacity,
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
    m: Motion,
    cx: &App,
) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if ActiveTheme::reduce_motion(cx) {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(m.ms)).with_easing(move |t| m.curve.at(t)),
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

/// The mirror of [`entering_from`]: the panel slides back out to `edge`.
///
/// v3's drawer uses `slide-out-to-*` here, at the shorter exit duration.
pub fn exiting_to<E>(
    el: E,
    id: impl Into<ElementId>,
    edge: Edge,
    travel: gpui::Pixels,
    m: Motion,
    cx: &App,
) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    if ActiveTheme::reduce_motion(cx) {
        return el.into_any_element();
    }

    el.with_animation(
        id.into(),
        gpui::Animation::new(Duration::from_millis(m.ms)).with_easing(move |t| m.curve.at(t)),
        move |el, delta| {
            let gone = travel * delta;
            let el = el.opacity(1.0 - delta);
            match edge {
                Edge::Left => el.ml(-gone),
                Edge::Right => el.mr(-gone),
                Edge::Top => el.mt(-gone),
                Edge::Bottom => el.mb(-gone),
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
            assert!(
                (shrunk + inset * 2.0 - h).abs() < 1e-4,
                "footprint changed at {h}"
            );
        }
    }

    #[test]
    fn cubic_bezier_pins_its_endpoints_and_rises() {
        let out = ease_out();
        assert!(out(0.0).abs() < 1e-3);
        assert!((out(1.0) - 1.0).abs() < 1e-3);
        // ease-out leads: it is ahead of linear through the middle.
        assert!(out(0.5) > 0.5, "ease-out should lead at the midpoint");
        // and it never goes backwards
        let mut prev = 0.0;
        for i in 0..=20 {
            let v = out(i as f32 / 20.0);
            assert!(v >= prev - 1e-4, "ease-out dipped at {i}");
            prev = v;
        }
        let smooth = ease_smooth();
        assert!(smooth(0.0).abs() < 1e-3);
        assert!((smooth(1.0) - 1.0).abs() < 1e-3);
    }

    #[test]
    #[allow(clippy::float_cmp)] // the keyframe values are meant to be exact
    fn caret_blink_matches_its_keyframes() {
        // 0%, 70% and 100% are opaque; 20% and 50% are transparent.
        let at = |t: f32| caret_opacity(t);
        for (t, want) in [
            (0.0, 1.0),
            (0.20, 0.0),
            (0.35, 0.0),
            (0.50, 0.0),
            (0.70, 1.0),
            (1.0, 1.0),
        ] {
            assert!((at(t) - want).abs() < 1e-6, "{t} should be {want}");
        }
        // and it ramps rather than jumping
        assert!((caret_opacity(0.10) - 0.5).abs() < 1e-6);
        assert!((caret_opacity(0.60) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn progress_circle_spin_is_one_linear_turn() {
        assert!(progress_circle_spin_turn(0.0).abs() < 1e-6);
        assert!((progress_circle_spin_turn(0.25) - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert!((progress_circle_spin_turn(1.0) - std::f32::consts::TAU).abs() < 1e-6);

        let mut previous = 0.0;
        for step in 0..=20 {
            let rotation = progress_circle_spin_turn(step as f32 / 20.0);
            assert!(rotation >= previous, "spin moved backwards at step {step}");
            previous = rotation;
        }
    }

    #[test]
    #[allow(clippy::float_cmp)] // clamped to exactly zero, not to near-zero
    fn shrink_never_goes_negative() {
        assert!(f32::from(shrink(px(1.), px(4.))).abs() < 1e-6);
    }
}
