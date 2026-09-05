//! Shared render helpers for HeroGPUI components.

use gpui::{App, BorrowAppContext, Div, Hsla, ParentElement, Pixels, Styled};
use herogpui_core::{FieldVariant, Prominence};
use herogpui_theme::ActiveTheme;

// Browser hosts register the bundled mono family before opening a window.
pub(crate) const MONO_FONT: &str = if cfg!(target_family = "wasm") {
    "JetBrains Mono"
} else {
    "Consolas"
};

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

/// `rounded-xs` — the small ProgressBar track.
pub fn micro_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.radius_xs())
}

/// Corner radius of a form field — `--field-radius`.
pub fn field_radius(cx: &App) -> Pixels {
    let layout = cx.layout();
    layout.capped(layout.field_radius)
}

/// `min(32px, --radius-3xl)` — cards, the table, and every floating panel
/// (modal, popover, toast, alert, dropdown). Surface is not on the list:
/// upstream `.surface` declares no radius.
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
                inset: false,
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

pub(crate) fn window_overlay(el: impl gpui::IntoElement, window: &gpui::Window) -> gpui::Deferred {
    use gpui::InteractiveElement;

    let viewport = window.viewport_size();
    floating(
        gpui::anchored()
            .position(gpui::point(gpui::px(0.), gpui::px(0.)))
            .child(
                gpui::div()
                    .relative()
                    .occlude()
                    .w(viewport.width)
                    .h(viewport.height)
                    .flex_shrink_0()
                    .child(el),
            ),
    )
}

/// The result of an explicit overlay dismissal attempt.
///
/// Only `Handled` consumes the event. A declined outside press continues to
/// the control under the pointer, matching React Aria's `useOverlay`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DismissResult {
    Handled,
    Declined,
}

#[derive(Default)]
struct OverlayStack {
    entries: Vec<gpui::WeakEntity<OverlayRegistration>>,
    next_order: u64,
}

impl gpui::Global for OverlayStack {}

#[derive(Clone)]
struct OverlayRegistration {
    window_id: gpui::WindowId,
    order: u64,
    phase: OverlayPhase,
    keep_exiting: bool,
    exit_generation: u64,
    escape_capture: Option<std::sync::Arc<dyn Fn(&mut gpui::Window, &mut App) -> DismissResult>>,
}

/// A direct handle to one registration returned by [`overlay_scope`].
///
/// The token is intentionally not inferred from an element or a render-local
/// variable. It becomes inert when its registration is unmounted.
#[derive(Clone)]
pub struct OverlayToken {
    registration: gpui::WeakEntity<OverlayRegistration>,
    window_id: gpui::WindowId,
}

fn ensure_overlay_stack(cx: &mut App) {
    if cx.try_global::<OverlayStack>().is_none() {
        cx.set_global(OverlayStack::default());
    }
}

fn prune_overlay_stack(stack: &mut OverlayStack, cx: &App) {
    stack.entries.retain(|entry| {
        let Some(entry) = entry.upgrade() else {
            return false;
        };
        entry.read(cx).phase == OverlayPhase::Open
    });
}

fn sync_overlay_stack(registration: &gpui::Entity<OverlayRegistration>, cx: &mut App) {
    ensure_overlay_stack(cx);
    let weak = registration.downgrade();
    let active = registration.read(cx).phase == OverlayPhase::Open;
    cx.update_global::<OverlayStack, _>(|stack, cx| {
        prune_overlay_stack(stack, cx);
        if active && !stack.entries.iter().any(|entry| entry == &weak) {
            stack.entries.push(weak);
        }
    });
}

fn is_topmost(token: &OverlayToken, cx: &mut App) -> bool {
    ensure_overlay_stack(cx);
    cx.update_global::<OverlayStack, _>(|stack, cx| {
        prune_overlay_stack(stack, cx);
        let Some(registration) = token.registration.upgrade() else {
            return false;
        };
        let state = registration.read(cx);
        if state.window_id != token.window_id || state.phase != OverlayPhase::Open {
            return false;
        }
        stack
            .entries
            .iter()
            .filter_map(|entry| entry.upgrade())
            .filter(|entry| entry.read(cx).window_id == token.window_id)
            .max_by_key(|entry| entry.read(cx).order)
            .is_some_and(|entry| entry == registration)
    })
}

/// Gives an overlay a document-level Escape handler.
///
/// React Aria uses this for Tooltip because focus may sit anywhere else in the
/// document while hover keeps the tip open. [`app_focus_root`] invokes the
/// handler during capture, before a focused descendant can answer the key.
/// The newest open captured handler wins even when a non-capturing overlay was
/// registered later, matching the document listener's precedence.
pub fn capture_escape(
    token: &OverlayToken,
    handler: impl Fn(&mut gpui::Window, &mut App) -> DismissResult + 'static,
    cx: &mut App,
) {
    if let Some(registration) = token.registration.upgrade() {
        let handler = shared(handler);
        registration.update(cx, |state, _| state.escape_capture = Some(handler));
    }
}

fn dismiss_captured_escape(window: &mut gpui::Window, cx: &mut App) -> bool {
    let window_id = window.window_handle().window_id();
    ensure_overlay_stack(cx);
    let handler = cx.update_global::<OverlayStack, _>(|stack, cx| {
        prune_overlay_stack(stack, cx);
        stack
            .entries
            .iter()
            .filter_map(|entry| entry.upgrade())
            .filter(|entry| entry.read(cx).window_id == window_id)
            .filter_map(|entry| {
                let state = entry.read(cx);
                state
                    .escape_capture
                    .clone()
                    .map(|handler| (state.order, handler))
            })
            .max_by_key(|(order, _)| *order)
            .map(|(_, handler)| handler)
    });
    handler.is_some_and(|handler| handler(window, cx) == DismissResult::Handled)
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
    let el = dismiss_on_escape(el, move |window, cx| on_escape(window, cx));
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
    // Legacy helper: callers not yet migrated to `overlay_scope` retain their
    // old unconditional dismissal semantics, without render-local inference.
    el.on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
        if event.keystroke.key == "escape" {
            close(window, cx);
        }
    })
}

pub fn dismiss_on_escape_with_token<E: gpui::InteractiveElement>(
    el: E,
    token: OverlayToken,
    close: impl Fn(&mut gpui::Window, &mut App) -> DismissResult + 'static,
) -> E {
    el.on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
        if event.keystroke.key == "escape"
            && is_topmost(&token, cx)
            && close(window, cx) == DismissResult::Handled
        {
            cx.stop_propagation();
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
    // Legacy helper: callers not yet migrated to `overlay_scope` retain their
    // old unconditional dismissal semantics, without render-local inference.
    el.on_mouse_down_out(move |_, window, cx| {
        close(window, cx);
    })
}

pub fn dismiss_on_press_outside_with_token<E: gpui::InteractiveElement>(
    el: E,
    token: OverlayToken,
    close: impl Fn(&mut gpui::Window, &mut App) -> DismissResult + 'static,
) -> E {
    dismiss_on_press_outside_with_token_event(el, token, move |_, window, cx| close(window, cx))
}

/// The event-aware form of [`dismiss_on_press_outside_with_token`].
///
/// Compound surfaces such as a menu and its deferred submenu use the pointer
/// position to treat the union of both panels as inside, while the token still
/// prevents a lower overlay from answering the same press.
pub fn dismiss_on_press_outside_with_token_event<E: gpui::InteractiveElement>(
    el: E,
    token: OverlayToken,
    close: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut App) -> DismissResult + 'static,
) -> E {
    el.on_mouse_down_out(move |event, window, cx| {
        if is_topmost(&token, cx) && close(event, window, cx) == DismissResult::Handled {
            cx.stop_propagation();
        }
    })
}

/// A focus handle for a floating panel, focused as the panel opens.
///
/// A panel nothing focuses never sees a key, so this is what makes Escape and
/// the arrows work at all. It is not a tab stop: the panel is transient, and Tab
/// inside it should reach the controls it contains.
///
/// The open transition saves the previously focused handle and claims the
/// panel. The close transition restores that handle only when focus is still
/// inside the panel, so an intentional move elsewhere wins. Both transitions
/// are remembered in keyed state derived from `base`; a per-render flag would
/// forget ownership on the repaint caused by the opening press.
#[derive(Clone, Debug, Default)]
struct PanelFocusState {
    was_open: bool,
    restore: Option<gpui::WeakFocusHandle>,
}

/// A stable handle for the control that opens a [`panel_focus`] scope.
///
/// Track this on the trigger wrapper. It is deliberately not a tab stop: the
/// trigger's own control owns that position, while this handle gives the panel
/// a stable restoration target even when the trigger is a rebuilt child.
pub fn panel_restore_focus(
    window: &mut gpui::Window,
    cx: &mut App,
    base: &str,
) -> gpui::FocusHandle {
    window
        .use_keyed_state(
            gpui::ElementId::Name(format!("{base}-panel-restore-focus").into()),
            cx,
            |_, cx| cx.focus_handle(),
        )
        .read(cx)
        .clone()
}

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
    let state = window.use_keyed_state(
        gpui::ElementId::Name(format!("{base}-panel-focus-state").into()),
        cx,
        |_, _| PanelFocusState::default(),
    );
    let current = state.read(cx).clone();

    if open && !current.was_open {
        let trigger = panel_restore_focus(window, cx, base);
        // `focused` is the actual control that opened the panel. The wrapper
        // is only a fallback for a programmatic open with no focused control;
        // restoring it when it merely contains the Button loses the Button's
        // own focus-visible state and keyboard activation.
        let restore = window
            .focused(cx)
            .filter(|focused| focused.tab_stop)
            .map(|focused| focused.downgrade())
            .or_else(|| Some(trigger.downgrade()));
        window.focus(&handle, cx);
        state.update(cx, |state, _| {
            state.was_open = true;
            state.restore = restore;
        });
    } else if !open && current.was_open {
        if handle.contains_focused(window, cx) {
            if let Some(restore) = current.restore.and_then(|handle| handle.upgrade()) {
                window.focus(&restore, cx);
            }
        }
        state.update(cx, |state, _| {
            state.was_open = false;
            state.restore = None;
        });
    }
    handle
}

/// Returns a stable, non-tab-stop scope that closes an open popover when focus
/// leaves its trigger-plus-panel subtree. Track it on their common root.
///
/// Active windows use `on_focus_out`. GPUI blanks that event's focus paths for
/// inactive windows, including its headless test platform, so a shared
/// render-time edge detects moves to an outside tab stop there. Both paths
/// consume `seen_inside`, preventing duplicate closes.
#[derive(Clone, Copy, Debug, Default)]
struct CloseOnBlurState {
    /// Whether the scope held the focus as of the last observed frame.
    seen_inside: bool,
}

#[derive(Clone)]
pub struct FocusLeave {
    handle: gpui::FocusHandle,
    subscription: gpui::Entity<Option<gpui::Subscription>>,
    state: gpui::Entity<CloseOnBlurState>,
}

impl FocusLeave {
    pub fn focus_handle(&self) -> gpui::FocusHandle {
        self.handle.clone()
    }

    /// Marks a departure as already handled by the event that caused it.
    pub fn consume(&self, cx: &mut App) {
        self.state.update(cx, |state, _| state.seen_inside = false);
        self.subscription.update(cx, |slot, _| *slot = None);
    }
}

pub fn close_on_blur(
    window: &mut gpui::Window,
    cx: &mut App,
    base: &str,
    open: bool,
    close: impl Fn(&mut gpui::Window, &mut App) + 'static,
) -> gpui::FocusHandle {
    on_focus_leave(window, cx, base, open, close).focus_handle()
}

/// Observes focus leaving a stable subtree while `active`.
pub fn on_focus_leave(
    window: &mut gpui::Window,
    cx: &mut App,
    base: &str,
    active: bool,
    leave: impl Fn(&mut gpui::Window, &mut App) + 'static,
) -> FocusLeave {
    let held_scope = window.use_keyed_state(
        gpui::ElementId::Name(format!("{base}-close-on-blur-scope").into()),
        cx,
        |_, cx| cx.focus_handle().tab_stop(false),
    );
    let scope = held_scope.read(cx).clone();
    // Storing the subscription is arming; dropping it is disarming. The
    // `Option` slot flips either way without subscribing twice.
    let subscription = window.use_keyed_state(
        gpui::ElementId::Name(format!("{base}-close-on-blur-subscription").into()),
        cx,
        |_, _| None::<gpui::Subscription>,
    );
    let state = window.use_keyed_state(
        gpui::ElementId::Name(format!("{base}-close-on-blur-state").into()),
        cx,
        |_, _| CloseOnBlurState::default(),
    );
    let armed = subscription.read(cx).is_some();
    // Both observation legs hand the same closer out; gpui runs single-threaded,
    // so an `Rc` shares it without asking the closure to be `Clone`.
    let leave = std::rc::Rc::new(leave);

    // The frame-end half (real, focused windows): the guard reads the shared
    // edge so a render that got there first leaves this nothing to do, and
    // firing also drops the subscription -- a transition owns its close once.
    if active && !armed {
        // The listener is owned by `subscription`; weak captures avoid a cycle
        // that would otherwise retain an unmounted open component forever.
        let disarmer = subscription.downgrade();
        let edge = state.downgrade();
        let leave = std::rc::Rc::clone(&leave);
        let listener = window.on_focus_out(&scope, cx, move |_, window, cx| {
            let Some(edge) = edge.upgrade() else {
                return;
            };
            let due = edge.read(cx).seen_inside;
            if let Some(disarmer) = disarmer.upgrade() {
                disarmer.update(cx, |slot, _| *slot = None);
            }
            if !due {
                return;
            }
            edge.update(cx, |state, _| state.seen_inside = false);
            leave(window, cx);
        });
        subscription.update(cx, |slot, _| *slot = Some(listener));
    }

    // The render half (everywhere else): whatever the observer APIs do, a
    // focus move always invalidates the window, so a move to another tab stop
    // can be observed on the next frame. Requiring a tab stop avoids treating
    // the app root's non-interactive recovery handle as a user departure.
    // Never-before-seen counts as absent, not departed: the first frames of a
    // freshly opened surface hold no focus yet.
    if active {
        if scope.contains_focused(window, cx) {
            state.update(cx, |state, _| state.seen_inside = true);
        } else if state.read(cx).seen_inside
            && window.focused(cx).is_some_and(|focused| focused.tab_stop)
        {
            state.update(cx, |state, _| state.seen_inside = false);
            subscription.update(cx, |slot, _| *slot = None);
            leave(window, cx);
        }
    } else {
        if state.read(cx).seen_inside {
            state.update(cx, |state, _| state.seen_inside = false);
        }
        if armed {
            subscription.update(cx, |slot, _| *slot = None);
        }
    }

    FocusLeave {
        handle: scope,
        subscription,
        state,
    }
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
        window.focus(handle, cx);
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
    exit_generation: u64,
}

/// Registers one overlay in the app-global, window-scoped dismissal stack.
///
/// The returned token is the only handle accepted by the explicit dismissal
/// helpers. An overlay receives a new order only when it transitions from
/// `Closed` or `Exiting` to `Open`; repainting an open overlay is stable.
pub fn overlay_scope(
    window: &mut gpui::Window,
    cx: &mut App,
    key: impl Into<gpui::ElementId>,
    is_open: bool,
    keep_exiting: bool,
) -> (OverlayPhase, OverlayToken) {
    overlay_scope_with_exit(
        window,
        cx,
        key,
        is_open,
        keep_exiting,
        crate::anim::EXITING_MS,
    )
}

/// Registers an overlay whose exit lifetime differs from the shared 100ms.
pub fn overlay_scope_with_exit(
    window: &mut gpui::Window,
    cx: &mut App,
    key: impl Into<gpui::ElementId>,
    is_open: bool,
    keep_exiting: bool,
    exit_ms: u64,
) -> (OverlayPhase, OverlayToken) {
    let key = key.into();
    let window_id = window.window_handle().window_id();
    let registration = window.use_keyed_state(key, cx, |_, _| OverlayRegistration {
        window_id,
        order: 0,
        phase: OverlayPhase::Closed,
        keep_exiting,
        exit_generation: 0,
        escape_capture: None,
    });
    let current = registration.read(cx).clone();

    let phase = if is_open {
        if current.phase != OverlayPhase::Open {
            ensure_overlay_stack(cx);
            let order = cx.update_global::<OverlayStack, _>(|stack, _| {
                stack.next_order = stack.next_order.saturating_add(1);
                stack.next_order
            });
            registration.update(cx, |state, _| {
                state.order = order;
                state.phase = OverlayPhase::Open;
                state.keep_exiting = keep_exiting;
            });
        }
        OverlayPhase::Open
    } else if current.phase == OverlayPhase::Open && keep_exiting {
        let exit_generation = current.exit_generation.saturating_add(1);
        registration.update(cx, |state, _| {
            state.phase = OverlayPhase::Exiting;
            state.keep_exiting = keep_exiting;
            state.exit_generation = exit_generation;
        });
        let held = registration.downgrade();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(exit_ms))
                .await;
            cx.update(|cx| {
                let _ = held.update(cx, |state, cx| {
                    if state.phase == OverlayPhase::Exiting
                        && state.exit_generation == exit_generation
                    {
                        state.phase = OverlayPhase::Closed;
                        cx.notify();
                    }
                });
            });
        })
        .detach();
        OverlayPhase::Exiting
    } else if current.phase == OverlayPhase::Open {
        registration.update(cx, |state, _| {
            state.phase = OverlayPhase::Closed;
            state.keep_exiting = keep_exiting;
        });
        OverlayPhase::Closed
    } else if current.phase == OverlayPhase::Exiting {
        OverlayPhase::Exiting
    } else {
        if current.keep_exiting != keep_exiting {
            registration.update(cx, |state, _| state.keep_exiting = keep_exiting);
        }
        OverlayPhase::Closed
    };

    sync_overlay_stack(&registration, cx);
    (
        phase,
        OverlayToken {
            registration: registration.downgrade(),
            window_id,
        },
    )
}

/// Resolves `isOpen` into a phase that includes v3's `[data-exiting]`.
///
/// Legacy non-stack helper for components not yet migrated. New overlays must
/// use [`overlay_scope`] and pass its token to the explicit dismissal helpers.
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
    let key = key.into();
    let held = window.use_keyed_state(key, cx, |_, _| PhaseState::default());
    let current = *held.read(cx);

    if is_open {
        if !current.was_open {
            held.update(cx, |s, _| {
                s.was_open = true;
                s.exiting = false;
            });
        }
        OverlayPhase::Open
    } else if current.was_open {
        // Just closed: hold the panel for its exit, then drop it.
        let exit_generation = current.exit_generation.saturating_add(1);
        held.update(cx, |s, _| {
            s.was_open = false;
            s.exiting = true;
            s.exit_generation = exit_generation;
        });
        let held = held.clone();
        cx.spawn(async move |cx: &mut gpui::AsyncApp| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(crate::anim::EXITING_MS))
                .await;
            cx.update(|cx| {
                held.update(cx, |s, cx| {
                    if s.exiting && s.exit_generation == exit_generation {
                        s.exiting = false;
                        cx.notify();
                    }
                });
            });
        })
        .detach();
        OverlayPhase::Exiting
    } else if current.exiting {
        OverlayPhase::Exiting
    } else {
        OverlayPhase::Closed
    }
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

#[derive(Default)]
struct ActiveKeyboardPresses(Vec<(gpui::WindowId, String, gpui::WeakEntity<(bool, bool)>)>);
impl gpui::Global for ActiveKeyboardPresses {}

/// `[data-focus-visible]` — whether a focus ring should be showing.
///
/// A browser rings a control focused by the keyboard and not one focused by a
/// click; React Aria says the same thing with `data-focus-visible`, and 41 of
/// v3's stylesheets style that state. gpui reports *that* an element has focus
/// but not how the focus arrived, so the app root records which kind of input
/// was last seen and every ring in the tree reads it.
/// The shared focus portion of v3 field render props.
///
/// Fields draw their focus chrome from these three values, so content closures
/// receive them rather than re-deriving focus. Components with additional
/// render props embed the same values in their component-specific state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FieldFocus {
    /// `isFocused` — this control holds the keyboard.
    pub is_focused: bool,
    /// `isFocusWithin` — it or something inside it does.
    pub is_focus_within: bool,
    /// `isFocusVisible` — focused, and the last input was a key.
    pub is_focus_visible: bool,
}

/// v3's *value* render props, as one value.
///
/// `Select.Value`, `Autocomplete.Value` and `ComboBox.Value` all hand their
/// children a function and pass in `{defaultChildren, isPlaceholder,
/// selectedItems, selectedText}`. `defaultChildren` is what the slot would have
/// drawn, so a caller can wrap it instead of rebuilding it -- which is what v3's
/// own examples do (`if (isPlaceholder) return defaultChildren`).
pub struct SelectionValue<'a> {
    /// `selectedItems` — the chosen items' text. The order is the component's
    /// selection order: Select walks the collection, while ComboBox and
    /// Autocomplete follow their selection set's insertion order, the way
    /// pinned react-stately 3.49.0's `Set` iterates.
    pub selected_items: &'a [gpui::SharedString],
    /// Where those items sit in the collection, for a caller keyed by index,
    /// in the same order as `selected_items`.
    pub selected_indices: &'a [usize],
    /// The chosen items' keys, in selection insertion order, when the
    /// component's collection is keyed and those keys are distinct from the
    /// labels (`Autocomplete`, `ComboBox`). `Select` is index-keyed and
    /// carries no distinct key here. `selected_items` and `selected_indices`
    /// only contain entries for keys that currently resolve to collection
    /// items, so async-loaded or missing keys can make their lengths differ
    /// from `selected_keys`.
    pub selected_keys: Option<&'a [gpui::SharedString]>,
    /// `selectedText` — the same items joined. Select approximates v3's en-US
    /// list formatter; the other two use plain comma-space in this port.
    pub selected_text: &'a str,
    /// `isPlaceholder` — nothing is chosen, so the placeholder shows.
    pub is_placeholder: bool,
    /// `defaultChildren` — the element this slot would have drawn.
    pub default_children: gpui::AnyElement,
}

/// v3's interactive render props, as one value.
///
/// Every pressable control in v3 hands its children a function and passes these
/// in: `{isHovered, isPressed, isFocused, isFocusVisible, isSelected,
/// isDisabled}`; Button additionally supplies `isPending`. This port draws
/// each of those states itself, and a component
/// that also takes a content closure hands the same values over rather than
/// leaving a caller to re-derive them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InteractiveState {
    /// `isHovered` — the pointer is over the control. Known one frame late: gpui
    /// reports a hover to a *handler*, not to the render that draws it.
    pub is_hovered: bool,
    /// `isPressed` — the pointer or activation key is down, likewise one frame late.
    pub is_pressed: bool,
    /// `isFocused`
    pub is_focused: bool,
    /// `isFocusVisible` — focused *and* the last input was a key.
    pub is_focus_visible: bool,
    /// `isSelected` — for the controls where selection is a state.
    pub is_selected: bool,
    /// `isDisabled`
    pub is_disabled: bool,
    /// `isPending` — Button is waiting for an operation while remaining focusable.
    pub is_pending: bool,
    /// `isIndeterminate` — a multi-selection row where some but not all of the
    /// group's keys are chosen.
    pub is_indeterminate: bool,
}

/// Where a control keeps the hover and press it will report next frame.
pub type Interaction = gpui::Entity<(bool, bool)>;

fn has_active_keyboard_press(slot: &Interaction, window: &gpui::Window, cx: &App) -> bool {
    let weak = slot.downgrade();
    let window_id = window.window_handle().window_id();
    cx.try_global::<ActiveKeyboardPresses>()
        .is_some_and(|pressed| {
            pressed.0.iter().any(|(active_window, _, interaction)| {
                *active_window == window_id && interaction == &weak
            })
        })
}

pub(crate) fn begin_keyboard_press(
    slot: &Interaction,
    event: &gpui::KeyDownEvent,
    window: &gpui::Window,
    cx: &mut App,
) {
    if event.is_held || !matches!(event.keystroke.key.as_str(), "enter" | "space") {
        return;
    }
    let began = slot.update(cx, |state, cx| {
        if state.1 {
            false
        } else {
            state.1 = true;
            cx.notify();
            true
        }
    });
    if began {
        if cx.try_global::<ActiveKeyboardPresses>().is_none() {
            cx.set_global(ActiveKeyboardPresses::default());
        }
        let active = (
            window.window_handle().window_id(),
            event.keystroke.key.clone(),
            slot.downgrade(),
        );
        cx.update_global::<ActiveKeyboardPresses, _>(|pressed, _| {
            pressed
                .0
                .retain(|(_, _, interaction)| interaction.upgrade().is_some());
            pressed.0.push(active);
        });
    }
}

/// The keyed `(hovered, pressed)` slot for one control.
///
/// gpui tells a *handler* about a hover and a press; a render can only read what
/// the last frame recorded, which is why this is a piece of state rather than a
/// question asked during layout.
pub fn interaction(id: gpui::ElementId, window: &mut gpui::Window, cx: &mut App) -> Interaction {
    window.use_keyed_state(id, cx, |_, _| (false, false))
}

/// Wires the hover and press handlers that keep an [`Interaction`] current.
pub fn track_interaction<T>(el: T, slot: &Interaction) -> T
where
    T: gpui::StatefulInteractiveElement + ParentElement,
{
    track_interaction_on_mouse_down(el, slot, |_, _| {})
}

/// Wires [`track_interaction`] and performs component-specific focus work in
/// the same mouse-down handler that records the press.
pub(crate) fn track_interaction_on_mouse_down<T>(
    el: T,
    slot: &Interaction,
    on_mouse_down: impl Fn(&mut gpui::Window, &mut App) + 'static,
) -> T
where
    T: gpui::StatefulInteractiveElement + ParentElement,
{
    let hover = slot.clone();
    let down = slot.clone();
    let up = slot.clone();
    let key_down = slot.clone();
    let key_up = slot.clone();
    let outside_up = slot.clone();
    // This listener is deliberately unconditional, not armed while the slot is
    // pressed: `Window::on_mouse_event` is `debug_assert_paint`-only, so a
    // press-armed registration could not exist until the frame *after* the
    // mouse down, and a release dispatched before that frame completes -- a
    // fast click, or a press dragged outside -- would be missed and leave the
    // slot stuck pressed. gpui 0.2.2 has no pointer capture; the per-frame
    // re-registration is what makes any outside release observable at all.
    let release = gpui::canvas(
        |bounds, _, _| bounds,
        move |_, _, window, _| {
            window.on_mouse_event(move |event: &gpui::MouseUpEvent, phase, window, cx| {
                if phase == gpui::DispatchPhase::Capture
                    && event.button == gpui::MouseButton::Left
                    && !has_active_keyboard_press(&outside_up, window, cx)
                {
                    outside_up.update(cx, |state, cx| {
                        if state.1 {
                            state.1 = false;
                            cx.notify();
                        }
                    });
                }
            });
        },
    )
    .absolute()
    .inset_0();
    el.on_hover(move |over, _, cx| {
        let over = *over;
        hover.update(cx, |state, cx| {
            if state.0 != over {
                state.0 = over;
                cx.notify();
            }
        });
    })
    .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
        down.update(cx, |state, cx| {
            if !state.1 {
                state.1 = true;
                cx.notify();
            }
        });
        on_mouse_down(window, cx);
    })
    .on_mouse_up(gpui::MouseButton::Left, move |_, window, cx| {
        if !has_active_keyboard_press(&up, window, cx) {
            up.update(cx, |state, cx| {
                if state.1 {
                    state.1 = false;
                    cx.notify();
                }
            });
        }
    })
    .on_key_down(move |event, window, cx| {
        begin_keyboard_press(&key_down, event, window, cx);
    })
    .on_key_up(move |event, _, cx| {
        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
            key_up.update(cx, |state, cx| {
                if state.1 {
                    state.1 = false;
                    cx.notify();
                }
            });
        }
    })
    .child(release)
}

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
        window.focus(&root, cx);
    }
    el.track_focus(&root)
        .capture_any_mouse_down(|_, _, cx| set_focus_visible(false, cx))
        .capture_key_down(|event, window, cx| {
            if event.keystroke.key == "escape" && dismiss_captured_escape(window, cx) {
                cx.stop_propagation();
            }
        })
        .on_key_down(|event, window, cx| {
            set_focus_visible(true, cx);
            if event.keystroke.key == "tab" {
                if event.keystroke.modifiers.shift {
                    window.focus_prev(cx);
                } else {
                    window.focus_next(cx);
                }
                cx.stop_propagation();
            }
        })
        .on_key_up(|event, window, cx| {
            if !matches!(event.keystroke.key.as_str(), "enter" | "space")
                || cx.try_global::<ActiveKeyboardPresses>().is_none()
            {
                return;
            }
            let window_id = window.window_handle().window_id();
            let key = event.keystroke.key.as_str();
            let interactions = cx.update_global::<ActiveKeyboardPresses, _>(|pressed, _| {
                let mut released = Vec::new();
                pressed
                    .0
                    .retain(|(active_window, active_key, interaction)| {
                        if *active_window == window_id && active_key == key {
                            released.push(interaction.clone());
                            false
                        } else {
                            interaction.upgrade().is_some()
                        }
                    });
                released
            });
            for interaction in interactions {
                if let Some(interaction) = interaction.upgrade() {
                    interaction.update(cx, |state, cx| {
                        if state.1 {
                            state.1 = false;
                            cx.notify();
                        }
                    });
                }
            }
        })
}

/// A focus handle the Tab key can reach, kept in the window's keyed state.
///
/// gpui registers a tab stop from the **handle's** own `tab_stop` flag; the
/// element's `tab_index` builder only configures a handle the element creates
/// for itself, which a component that has to read its own focus state cannot
/// use. Marking the handle is what makes `window.focus_next(cx)` see it.
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
        inset: false,
    }];
    if gap > gpui::px(0.) {
        shadows.push(gpui::BoxShadow {
            color: colors.background,
            offset: gpui::point(gpui::px(0.), gpui::px(0.)),
            blur_radius: blur,
            spread_radius: gap,
            inset: false,
        });
    }
    shadows
}

/// Keeps Tab inside `scope`, which is v3's `Tab` cycles elements.
///
/// gpui's tab order is the window's: a tab group only *orders* its children, so
/// Tab walks straight out of a dialog and into the page behind it. There is no
/// way to enumerate one subtree's stops either, so the trap is done by moving
/// and checking: step, and if the focus left the scope, come back in from the
/// far end. Reversing means walking forward until the focus leaves and stepping
/// back once, which is bounded so a dialog with no stops of its own cannot spin.
///
/// The step has to be ours rather than the app root's, so this stops
/// propagation: `util::app_focus_root` binds Tab to `focus_next` on a listener
/// higher in the tree, and both firing would move twice.
pub fn trap_tab<T: gpui::InteractiveElement>(el: T, scope: &gpui::FocusHandle) -> T {
    let scope = scope.clone();
    el.on_key_down(move |event: &gpui::KeyDownEvent, window, cx| {
        if event.keystroke.key != "tab" {
            return;
        }
        // Stopping propagation also skips the root's `set_focus_visible`, and a
        // trapped Tab that moved the focus without turning the ring on looks
        // like it did nothing at all.
        cx.stop_propagation();
        set_focus_visible(true, cx);
        let back = event.keystroke.modifiers.shift;
        if back {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
        if scope.contains_focused(window, cx) {
            return;
        }
        // Out of the scope: re-enter from the other side.
        window.focus(&scope, cx);
        window.focus_next(cx);
        if !back {
            return;
        }
        // Backwards: walk to the last stop inside, then stop one short.
        for _ in 0..256 {
            window.focus_next(cx);
            if !scope.contains_focused(window, cx) {
                window.focus_prev(cx);
                return;
            }
        }
    })
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

#[cfg(test)]
mod overlay_stack_tests {
    use super::*;
    use gpui::AppContext;

    fn registration(
        cx: &mut gpui::TestAppContext,
        window_id: gpui::WindowId,
        order: u64,
    ) -> gpui::Entity<OverlayRegistration> {
        cx.new(|_| OverlayRegistration {
            window_id,
            order,
            phase: OverlayPhase::Open,
            keep_exiting: false,
            exit_generation: 0,
            escape_capture: None,
        })
    }

    #[gpui::test]
    fn sibling_order_is_stable_when_open_registrations_repaint(cx: &mut gpui::TestAppContext) {
        let outer = registration(cx, gpui::WindowId::from(1), 1);
        let inner = registration(cx, gpui::WindowId::from(1), 2);
        cx.update(|cx| {
            sync_overlay_stack(&outer, cx);
            sync_overlay_stack(&inner, cx);
            sync_overlay_stack(&outer, cx);
            sync_overlay_stack(&inner, cx);
            let stack = cx.global::<OverlayStack>();
            assert_eq!(stack.entries.len(), 2);
            assert_eq!(stack.entries[0].upgrade().unwrap().read(cx).order, 1);
            assert_eq!(stack.entries[1].upgrade().unwrap().read(cx).order, 2);
        });
    }

    #[gpui::test]
    fn topmost_registration_is_window_scoped(cx: &mut gpui::TestAppContext) {
        let first = registration(cx, gpui::WindowId::from(1), 1);
        let second = registration(cx, gpui::WindowId::from(2), 2);
        let first_token = OverlayToken {
            registration: first.downgrade(),
            window_id: gpui::WindowId::from(1),
        };
        let second_token = OverlayToken {
            registration: second.downgrade(),
            window_id: gpui::WindowId::from(2),
        };
        cx.update(|cx| {
            cx.set_global(OverlayStack {
                entries: vec![first.downgrade(), second.downgrade()],
                next_order: 2,
            });
            assert!(is_topmost(&first_token, cx));
            assert!(is_topmost(&second_token, cx));
        });
    }

    #[gpui::test]
    fn dead_registration_is_pruned_from_the_stack(cx: &mut gpui::TestAppContext) {
        let registration = registration(cx, gpui::WindowId::from(1), 1);
        let weak = registration.downgrade();
        cx.update(|cx| {
            cx.set_global(OverlayStack {
                entries: vec![weak.clone()],
                next_order: 1,
            });
        });
        drop(registration);
        cx.update(|cx| {
            cx.update_global::<OverlayStack, _>(|stack, cx| {
                prune_overlay_stack(stack, cx);
                assert!(stack.entries.is_empty());
            });
        });
    }
}
