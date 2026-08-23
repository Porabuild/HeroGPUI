//! Shared render helpers for HeroGPUI components.

use gpui::{App, Div, Hsla, Pixels, Styled};
use herogpui_core::{FieldVariant, Prominence};
use herogpui_theme::ActiveTheme;

/// The one height every v3 form field has. v3 removed `size` from the field
/// components (Input, Select, ComboBox, DateField, ...), keeping it only on the
/// nineteen where a scale is documented, so a field's metrics are constants
/// rather than a [`herogpui_core::Size`] lookup.
pub const FIELD_HEIGHT: Pixels = gpui::px(40.);
/// Type size inside a form field.
pub const FIELD_TEXT: Pixels = gpui::px(14.);
/// Glyph size for an icon inside a form field.
pub const FIELD_ICON: Pixels = gpui::px(16.);

/// Corner radius of a standard control (buttons, chips, menu items) —
/// `--radius-lg`.
pub fn control_radius(cx: &App) -> Pixels {
    cx.layout().radius_lg()
}

/// Corner radius of a form field — `--field-radius`.
pub fn field_radius(cx: &App) -> Pixels {
    cx.layout().field_radius
}

/// Corner radius of a container (cards, surfaces, panels) — `--radius-xl`.
pub fn container_radius(cx: &App) -> Pixels {
    cx.layout().radius_xl()
}

/// Background for a [`Prominence`] level. `Transparent` yields `None`.
pub fn prominence_bg(prominence: Prominence, cx: &App) -> Option<Hsla> {
    let colors = cx.colors();
    match prominence {
        Prominence::Transparent => None,
        Prominence::Default => Some(colors.surface.background),
        Prominence::Secondary => Some(colors.surface_secondary),
        Prominence::Tertiary => Some(colors.surface_tertiary),
    }
}

/// Applies the v3 field chrome: background, radius, border and — for
/// `primary` only — the `--field-shadow`.
pub fn apply_field_chrome(
    el: Div,
    variant: FieldVariant,
    is_invalid: bool,
    is_focused: bool,
    cx: &App,
) -> Div {
    let colors = cx.colors();
    let layout = cx.layout();

    let mut el = el.rounded(field_radius(cx)).bg(match variant {
        FieldVariant::Primary => colors.field.background,
        // Low-emphasis fields sit on a surface, so they borrow its next level.
        FieldVariant::Secondary => colors.surface_secondary,
    });

    if variant == FieldVariant::Primary && !layout.field_shadow.is_empty() {
        el = el.shadow(layout.field_shadow.clone());
    }

    // `--field-border-width` is 0 by default; an invalid or focused field draws
    // a ring regardless so the state is visible.
    if is_invalid {
        el = el.border(layout.border_width.max(gpui::px(1.))).border_color(colors.danger.color);
    } else if is_focused {
        el = el.border(layout.border_width.max(gpui::px(1.))).border_color(colors.focus);
    } else if layout.field_border_width > gpui::px(0.) {
        el = el
            .border(layout.field_border_width)
            .border_color(colors.field.border);
    }

    el
}

/// Lifts a floating panel above the rest of the page.
///
/// gpui paints in tree order, so an `absolute` panel is still overdrawn by any
/// later sibling — a `Select` list opened near the top of a page would be
/// painted over by the sections below it. `deferred` keeps the panel in the
/// layout tree but paints it after all of its ancestors, which is what every
/// floating surface needs.
pub fn floating(el: impl gpui::IntoElement) -> gpui::Deferred {
    gpui::deferred(el)
}

/// Wraps a callback for sharing between closures.
///
/// gpui callbacks take `&mut App` and therefore never leave the main thread;
/// `Arc` is used only because `Box<dyn Fn>` is not `Clone`. clippy's
/// `arc_with_non_send_sync` check is about cross-thread sharing, which cannot
/// happen here.
#[allow(clippy::arc_with_non_send_sync)]
pub fn shared<F: 'static>(f: F) -> std::sync::Arc<F> {
    std::sync::Arc::new(f)
}

/// An absolutely-positioned wrapper that places a floating panel for
/// `placement`, `offset` pixels clear of the trigger.
///
/// Every picker, dropdown and popover positions through here so they cannot
/// drift apart. The caller still has to hand the result to [`floating`] --
/// gpui paints in tree order, so `absolute` alone does not lift a panel above
/// later siblings.
pub fn placed_panel(
    placement: herogpui_core::Placement,
    offset: gpui::Pixels,
) -> gpui::Div {
    use herogpui_core::{Placement, PlacementAlign};

    let base = gpui::div().absolute();
    match placement {
        Placement::Left => base.right_full().top(gpui::px(0.)).mr(offset),
        Placement::Right => base.left_full().top(gpui::px(0.)).ml(offset),
        _ => {
            let base = if placement.is_above() {
                base.bottom_full().mb(offset)
            } else {
                base.top_full().mt(offset)
            };
            match placement.align() {
                PlacementAlign::Start => base.left(gpui::px(0.)),
                PlacementAlign::End => base.right(gpui::px(0.)),
                // gpui has no `translate`, so a centred panel is approximated by
                // stretching to the trigger's width and centring its content.
                PlacementAlign::Center => base
                    .left(gpui::px(0.))
                    .right(gpui::px(0.))
                    .flex()
                    .justify_center(),
            }
        }
    }
}

/// Positions a trigger-width panel (Select, ComboBox, Autocomplete) for
/// `placement`.
///
/// These panels stretch to the trigger's width, so the start and end alignment
/// variants coincide and only the side differs.
pub fn placed_field_panel(
    placement: herogpui_core::Placement,
    offset: gpui::Pixels,
) -> gpui::Div {
    use herogpui_core::Placement;

    let base = gpui::div().absolute();
    match placement {
        Placement::Left => base.right_full().top(gpui::px(0.)).mr(offset),
        Placement::Right => base.left_full().top(gpui::px(0.)).ml(offset),
        p if p.is_above() => base
            .bottom_full()
            .left(gpui::px(0.))
            .right(gpui::px(0.))
            .mb(offset),
        _ => base
            .top_full()
            .left(gpui::px(0.))
            .right(gpui::px(0.))
            .mt(offset),
    }
}

/// Gives `handle` focus the first time this element renders, and never again.
///
/// This is `autoFocus`. The "first time" has to be remembered somewhere, so a
/// one-shot flag lives in element state keyed by `key`: without it the field
/// would steal focus back on every frame and the user could never leave it.
pub fn focus_once(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    key: impl Into<gpui::ElementId>,
    handle: &gpui::FocusHandle,
) {
    let done = window.use_keyed_state(key.into(), cx, |_, _| false);
    if !*done.read(cx) {
        window.focus(handle);
        done.update(cx, |d, _| *d = true);
    }
}

/// Which phase an overlay is in, so `[data-exiting]` has something to render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayPhase {
    /// Not rendered at all.
    #[default]
    Closed,
    /// Rendered, and animating in.
    Open,
    /// `isOpen` has gone false, but the panel is still on screen for its exit.
    Exiting,
}

/// What `overlay_phase` remembers between renders.
#[derive(Clone, Copy, Debug, Default)]
struct PhaseState {
    was_open: bool,
    exiting: bool,
}

/// Resolves `isOpen` into a phase that includes v3's `[data-exiting]`.
///
/// A `RenderOnce` component drops out of the tree the moment `isOpen` goes
/// false, which leaves an exit animation nothing to play. This keeps the panel
/// alive for [`crate::anim::EXITING_MS`] afterwards: the flip to closed starts a
/// timer, and until it fires the phase is `Exiting`.
///
/// Callers render nothing on `Closed`, [`crate::anim::entering_zoom`] on `Open`
/// and [`crate::anim::exiting`] on `Exiting`.
pub fn overlay_phase(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    key: impl Into<gpui::ElementId>,
    is_open: bool,
) -> OverlayPhase {
    let held = window.use_keyed_state(key.into(), cx, |_, _| PhaseState::default());
    let current = *held.read(cx);

    if is_open {
        if !current.was_open {
            held.update(cx, |s, _| {
                s.was_open = true;
                s.exiting = false;
            });
        }
        return OverlayPhase::Open;
    }

    if current.was_open {
        // Just closed: hold the panel for its exit, then drop it.
        held.update(cx, |s, _| {
            s.was_open = false;
            s.exiting = true;
        });
        let held = held.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(crate::anim::EXITING_MS))
                .await;
            let _ = cx.update(|cx| {
                held.update(cx, |s, cx| {
                    s.exiting = false;
                    cx.notify();
                });
            });
        })
        .detach();
        return OverlayPhase::Exiting;
    }

    if current.exiting {
        return OverlayPhase::Exiting;
    }
    OverlayPhase::Closed
}

/// Runs `apply` on the first render only.
///
/// This is how a `default*` prop seeds a caller-owned state entity: the entity
/// outlives any one render, so writing the default unconditionally would fight
/// the user on every frame. Keyed on `key`, so two components of the same kind
/// seed independently.
pub fn seed_once(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    key: impl Into<gpui::ElementId>,
    apply: impl FnOnce(&mut gpui::App),
) {
    let done = window.use_keyed_state(key.into(), cx, |_, _| false);
    if !*done.read(cx) {
        done.update(cx, |d, _| *d = true);
        apply(cx);
    }
}

/// Resolves a controlled prop against an uncontrolled default.
///
/// This is v3's `value` / `defaultValue` pair. When the caller supplies the
/// controlled value it owns the state and the setter is a no-op passthrough;
/// when it does not, the component keeps the value itself in element state,
/// seeded once from `default`.
///
/// Returns the value to render and, in the uncontrolled case, the entity to
/// write the next value into.
pub fn controlled<T>(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    key: impl Into<gpui::ElementId>,
    controlled: Option<T>,
    default: T,
) -> (T, Option<gpui::Entity<T>>)
where
    T: Clone + 'static,
{
    match controlled {
        // The caller drives it; nothing to remember.
        Some(v) => (v, None),
        None => {
            let held = window.use_keyed_state(key.into(), cx, move |_, _| default);
            let current = held.read(cx).clone();
            (current, Some(held))
        }
    }
}
