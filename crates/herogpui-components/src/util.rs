//! Shared render helpers for HeroGPUI components.

use gpui::{App, Div, Hsla, Pixels, Styled};
use herogpui_core::{FieldVariant, Prominence};
use herogpui_theme::ActiveTheme;

/// The one height every v3 form field has: `.date-input-group` and
/// `.color-input-group` are `h-9`, and `.input`'s `py-2` plus its line box comes
/// to the same 36px. v3 removed `size` from the field
/// components (Input, Select, ComboBox, DateField, ...), keeping it only on the
/// nineteen where a scale is documented, so a field's metrics are constants
/// rather than a [`herogpui_core::Size`] lookup.
pub const FIELD_HEIGHT: Pixels = gpui::px(36.);
/// Type size inside a form field.
pub const FIELD_TEXT: Pixels = gpui::px(14.);
/// Glyph size for an icon inside a form field.
pub const FIELD_ICON: Pixels = gpui::px(16.);

// v3 does not have one "control" radius: each component names its own step, and
// they span the whole scale. `design_audit.py` diffs these against the real
// stylesheets, so the mapping here is checked rather than asserted.

/// `rounded-3xl` — buttons, toggle buttons and avatars.
pub fn control_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_3xl())
}

/// `rounded-2xl` — chips, menu and list rows, the colour area.
pub fn soft_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_2xl())
}

/// `rounded-xl` — close buttons, tags, links, tooltips.
pub fn small_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_xl())
}

/// `rounded-md` — the checkbox control (`.checkbox__control` is
/// `size-4 rounded-md`).
pub fn mark_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_md())
}

/// `rounded-lg` — the keyboard key and the radio control, which v3 draws as a
/// rounded square rather than a circle.
pub fn key_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_lg())
}

/// `rounded-sm` — separators and skeletons, which are nearly square.
pub fn hairline_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_sm())
}

/// Corner radius of a form field — `--field-radius`.
pub fn field_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.field_radius)
}

/// `min(32px, --radius-3xl)` — cards, surfaces, and every floating panel
/// (modal, popover, toast, alert, dropdown).
pub fn container_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_3xl())
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
///
/// Generic over [`Styled`] so a field that needed an `.id()` first (and is
/// therefore a `Stateful<Div>`) can share it. Six components used to hand-roll
/// this, and every one of them filled the `secondary` variant with
/// `surface_secondary` instead of `--default`.
pub fn apply_field_chrome<T: Styled>(
    el: T,
    variant: FieldVariant,
    is_invalid: bool,
    is_focused: bool,
    cx: &App,
) -> T {
    let colors = cx.colors();
    let layout = cx.layout();

    let mut el = el.rounded(field_radius(cx)).bg(match variant {
        FieldVariant::Primary => colors.field.background,
        // `.input--secondary` sets `--input-bg: var(--default)` and drops the
        // shadow. This used to use `surface_secondary`, which is a different
        // token (oklch 95.24% vs 94% in light mode) and a shade too light.
        FieldVariant::Secondary => colors.default.color,
    });

    // `--field-border-width` is 0, so a field's states are *rings*, not borders:
    // `status-focused-field` is `ring-2 ring-focus` with no offset, and
    // `status-invalid-field` is a 1px danger outline that becomes a 2px danger
    // ring once the field takes focus. Both ride on the field's own shadow,
    // because `shadow()` replaces the list rather than adding to it.
    let mut shadows = if variant == FieldVariant::Primary {
        layout.field_shadow.clone()
    } else {
        Vec::new()
    };

    if is_invalid {
        if is_focused {
            shadows.push(gpui::BoxShadow {
                color: colors.danger.color,
                offset: gpui::point(gpui::px(0.), gpui::px(0.)),
                blur_radius: gpui::px(1.),
                spread_radius: gpui::px(2.),
            });
        } else {
            el = el
                .border(layout.border_width.max(gpui::px(1.)))
                .border_color(colors.danger.color);
        }
    } else if is_focused {
        shadows.extend(focus_ring_shadows(false, cx));
    } else if layout.field_border_width > gpui::px(0.) {
        el = el
            .border(layout.field_border_width)
            .border_color(colors.field.border);
    }

    if shadows.is_empty() {
        el
    } else {
        el.shadow(shadows)
    }
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

/// Closes a floating panel on Escape and on a press outside it.
///
/// No prop table asks for this: React Aria gives every popover-like surface
/// `useOverlay`, so v3 only documents dismissal where it is *configurable*
/// (`isDismissable` on a dialog backdrop). A panel that closes only through its
/// own trigger is the difference between a port that looks right and one that
/// works -- an open menu followed the page as it scrolled and stayed open
/// forever.
///
/// Attach this to the panel itself, not to a wrapper: `on_mouse_down_out` reads
/// the element's own bounds, and the wrapper an absolute panel sits in has none,
/// which would make every press inside the panel count as outside. Escape is a
/// key event, so it needs the focus to be inside the panel -- pair it with
/// [`panel_focus`] where nothing else there is focused.
pub fn dismissable<E: gpui::InteractiveElement>(
    el: E,
    close: impl Fn(&mut gpui::Window, &mut App) + 'static,
) -> E {
    let close = shared(close);
    let on_escape = close.clone();
    let el = el.on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
        if event.keystroke.key == "escape" {
            on_escape(window, cx);
        }
    });
    dismiss_on_press_outside(el, move |window, cx| close(window, cx))
}

/// The Escape half of [`dismissable`], for a surface whose panel must *not*
/// hold the focus.
///
/// A key event goes to the focused element and bubbles to its ancestors, so a
/// panel that claims the focus silences the keyboard of everything inside it --
/// focusing a date picker's panel would have taken the arrows away from the
/// calendar grid. Attaching this to the component root instead lets the key
/// bubble up from whatever inside it does have the focus.
pub fn dismiss_on_escape<E: gpui::InteractiveElement>(
    el: E,
    close: impl Fn(&mut gpui::Window, &mut App) + 'static,
) -> E {
    el.on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
        if event.keystroke.key == "escape" {
            close(window, cx);
        }
    })
}

/// The outside-press half of [`dismissable`], for a surface whose Escape is
/// already part of a keyboard it owns -- a select and a combo box read Escape in
/// the same handler that reads the arrows, and binding it twice would close
/// twice.
pub fn dismiss_on_press_outside<E: gpui::InteractiveElement>(
    el: E,
    close: impl Fn(&mut gpui::Window, &mut App) + 'static,
) -> E {
    el.on_mouse_down_out(move |_, window, cx| close(window, cx))
}

/// A focus handle for a floating panel, focused as the panel opens.
///
/// A panel nothing focuses never sees a key, so this is what makes Escape and
/// the arrows work at all. It is not a tab stop: the panel is transient, and Tab
/// inside it should reach the controls it contains.
///
/// `open` is not optional. Claiming the focus while the panel is closed spends
/// the one-shot on a frame that draws nothing -- the popover's Escape did
/// nothing at all until this was gated, because the handle had already been
/// "focused" on the first closed render and the flag was set.
pub fn panel_focus(
    window: &mut gpui::Window,
    cx: &mut App,
    base: &str,
    open: bool,
) -> gpui::FocusHandle {
    let held = window.use_keyed_state(
        gpui::ElementId::Name(format!("{base}-panel-focus").into()),
        cx,
        |_, cx| cx.focus_handle(),
    );
    let handle = held.read(cx).clone();
    if open {
        focus_once(
            window,
            cx,
            gpui::ElementId::Name(format!("{base}-panel-autofocus").into()),
            &handle,
        );
    }
    handle
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
pub fn placed_panel(placement: herogpui_core::Placement, offset: Pixels) -> Div {
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
pub fn placed_field_panel(placement: herogpui_core::Placement, offset: Pixels) -> Div {
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
    cx: &mut App,
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
    cx: &mut App,
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
    cx: &mut App,
    key: impl Into<gpui::ElementId>,
    apply: impl FnOnce(&mut App),
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
    cx: &mut App,
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

// ---------------------------------------------------------------------------
// Focus rings (`status-focused`)
// ---------------------------------------------------------------------------

/// Whether the last input this app saw was a key.
struct FocusVisible(bool);
impl gpui::Global for FocusVisible {}

/// `[data-focus-visible]` — whether a focus ring should be showing.
///
/// A browser rings a control focused by the keyboard and not one focused by a
/// click; React Aria says the same thing with `data-focus-visible`, and 41 of
/// v3's stylesheets style that state. gpui reports *that* an element has focus
/// but not how the focus arrived, so the app root records which kind of input
/// was last seen and every ring in the tree reads it.
pub fn focus_visible(cx: &App) -> bool {
    cx.try_global::<FocusVisible>().is_some_and(|v| v.0)
}

pub fn set_focus_visible(visible: bool, cx: &mut App) {
    if focus_visible(cx) != visible {
        cx.set_global(FocusVisible(visible));
        cx.refresh_windows();
    }
}

/// Records keyboard-versus-pointer input, and moves the focus on Tab.
///
/// Put this on the app's root element once. Three things have to be true for a
/// focus ring to work at all, and this is where they are arranged:
///
/// - **The root holds the focus when nothing else does.** gpui delivers a key
///   event to the focused element and then up through its ancestors; with
///   nothing focused there is no chain, so the very first Tab would go nowhere.
/// - **Tab moves the focus.** In a browser the platform does this. Here the app
///   asks for it, and gpui walks the tab stops in tree order.
/// - **The kind of input is recorded**, because a ring shows for a keyboard
///   focus and not for a click. The mouse half runs in the capture phase, before
///   the press reaches whatever it landed on.
pub fn app_focus_root<T>(el: T, window: &mut gpui::Window, cx: &mut App) -> T
where
    T: gpui::InteractiveElement,
{
    let root = window
        .use_keyed_state(
            gpui::ElementId::Name("herogpui-focus-root".into()),
            cx,
            |_, cx| cx.focus_handle(),
        )
        .read(cx)
        .clone();
    if !root.contains_focused(window, cx) {
        window.focus(&root);
    }
    el.track_focus(&root)
        .capture_any_mouse_down(|_, _, cx| set_focus_visible(false, cx))
        .on_key_down(|event, window, cx| {
            set_focus_visible(true, cx);
            match event.keystroke.key.as_str() {
                "tab" if event.keystroke.modifiers.shift => window.focus_prev(),
                "tab" => window.focus_next(),
                _ => {}
            }
        })
}

/// A focus handle the Tab key can reach, kept in the window's keyed state.
///
/// gpui registers a tab stop from the **handle's** own `tab_stop` flag; the
/// element's `tab_index` builder only configures a handle the element creates
/// for itself, which a component that has to read its own focus state cannot
/// use. Marking the handle is what makes `window.focus_next()` see it.
pub fn tab_stop_handle(
    id: gpui::ElementId,
    window: &mut gpui::Window,
    cx: &mut App,
) -> gpui::FocusHandle {
    window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle().tab_stop(true))
        .read(cx)
        .clone()
}

/// The shadows that draw v3's focus ring.
///
/// `status-focused` is `ring-2 ring-focus` over a `ring-offset-2` in the
/// background colour: two rings, the inner one separating the accent from the
/// control. A ring costs no layout here because it is a shadow -- a border would
/// move the content inside it -- and they are painted largest first, since a
/// later shadow paints over an earlier one and that overlap is what carves the
/// gap.
///
/// **The blur cannot be zero.** gpui's shadow shader is a Gaussian integral: it
/// samples over `3 * blur_radius`, so a blur of zero integrates over nothing and
/// paints a completely transparent shadow -- which is why the first version of
/// this drew no ring at all. One pixel is the smallest blur that draws, and it
/// softens the ring's outer edge by about a pixel: the closest this gpui gets to
/// a crisp `ring-2`.
pub fn focus_ring_shadows(offset: bool, cx: &App) -> Vec<gpui::BoxShadow> {
    let colors = cx.colors();
    let layout = cx.layout();
    let ring = gpui::px(2.);
    let blur = gpui::px(1.);
    let gap = if offset {
        layout.ring_offset_width
    } else {
        gpui::px(0.)
    };
    let mut shadows = vec![gpui::BoxShadow {
        color: colors.focus,
        offset: gpui::point(gpui::px(0.), gpui::px(0.)),
        blur_radius: blur,
        spread_radius: gap + ring,
    }];
    if gap > gpui::px(0.) {
        shadows.push(gpui::BoxShadow {
            color: colors.background,
            offset: gpui::point(gpui::px(0.), gpui::px(0.)),
            blur_radius: blur,
            spread_radius: gap,
        });
    }
    shadows
}

/// v3's *inset* focus ring, as an overlay to hang inside the focused element.
///
/// A table is the exception to `status-focused`: `.table__cell` and
/// `.table__column` are `shadow-[inset_0_0_0_2px_var(--focus)]` with
/// `rounded-lg`, and a focused row draws the same ring split across its cells so
/// it reads as one continuous outline *inside* the row. An outset ring cannot
/// work there -- the next cell is flush against it, so a ring drawn outside is
/// either clipped or, on a transparent cell, bleeds through and fills it (a
/// focused column header came out solid accent).
///
/// gpui has no inset shadow and a border would move the content, so the ring is
/// an absolutely positioned child: it paints over the element and costs no
/// layout. The parent needs `.relative()`.
pub fn inset_focus_ring(cx: &App) -> Div {
    let colors = cx.colors();
    gpui::div()
        .absolute()
        .inset_0()
        .border_2()
        .border_color(colors.focus)
        .rounded(key_radius(cx))
}

/// Applies the focus ring on top of whatever the element already casts.
///
/// `base` is the element's own shadow list, because `shadow()` replaces rather
/// than adds: a focused field that dropped its `field_shadow` would flatten as
/// it took focus.
pub fn with_focus_ring<T: Styled>(
    el: T,
    focused: bool,
    offset: bool,
    base: Vec<gpui::BoxShadow>,
    cx: &App,
) -> T {
    if !focused {
        return if base.is_empty() { el } else { el.shadow(base) };
    }
    let mut all = base;
    all.extend(focus_ring_shadows(offset, cx));
    el.shadow(all)
}

/// Makes `el` a tab stop that rings when the keyboard focuses it.
///
/// The whole of `status-focused` in one call, for the common case: an element
/// that casts no shadow of its own and both takes the focus and shows the ring.
pub fn focusable<T>(
    el: T,
    id: gpui::ElementId,
    offset: bool,
    window: &mut gpui::Window,
    cx: &mut App,
) -> T
where
    T: Styled + gpui::InteractiveElement,
{
    let handle = tab_stop_handle(id, window, cx);
    ring_if_focused(
        el.track_focus(&handle),
        &handle,
        offset,
        Vec::new(),
        window,
        cx,
    )
}

/// The ring a control shows when it holds a keyboard focus.
///
/// The two conditions v3's selector has: the element is focused, *and* the focus
/// came from the keyboard.
pub fn ring_if_focused<T: Styled>(
    el: T,
    handle: &gpui::FocusHandle,
    offset: bool,
    base: Vec<gpui::BoxShadow>,
    window: &gpui::Window,
    cx: &App,
) -> T {
    let focused = handle.is_focused(window) && focus_visible(cx);
    with_focus_ring(el, focused, offset, base, cx)
}
