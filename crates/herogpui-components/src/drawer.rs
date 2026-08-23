//! Drawer — port of `@heroui/drawer`.
//!
//! Edge-anchored overlay panel. Render from your root view like [`Modal`].

use gpui::{
    prelude::*, px, AnyElement, App, ClickEvent, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use herogpui_core::Backdrop;

use crate::{
    icons,
    modal::{OnClose, OnOpenChange},
};

/// Which edge the drawer is anchored to (`placement`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerPlacement {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// The drawer panel's extent along its axis. v3 sets this in `drawer.css`
/// rather than with a prop, capped at 90% of the window by `max-w`/`max-h`.
const PANEL_EXTENT: gpui::Pixels = px(320.);

/// HeroUI Drawer (controlled).
#[derive(IntoElement)]
pub struct Drawer {
    /// Keys this dialog's own state; see [`Drawer::id`].
    id: gpui::ElementId,
    is_open: bool,
    placement: DrawerPlacement,
    is_dismissible: bool,
    backdrop: Backdrop,
    is_keyboard_dismiss_disabled: bool,
    on_open_change: Option<OnOpenChange>,
    hide_close_button: bool,
    title: Option<SharedString>,
    body: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    on_close: Option<OnClose>,
}

impl Drawer {
    /// The element id this dialog's state is keyed by.
    ///
    /// Not a v3 prop: gpui needs an explicit id, and the exit phase, the focus
    /// handle and the drag offset are all keyed by it. Two dialogs on screen
    /// with the same key share all three -- which is what
    /// `HEROGPUI_OPEN_OVERLAYS=1` puts on screen.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn new() -> Self {
        Self {
            id: gpui::ElementId::Name("drawer".into()),
            is_open: false,
            placement: DrawerPlacement::Right,
            is_dismissible: true,
            backdrop: Backdrop::Opaque,
            is_keyboard_dismiss_disabled: false,
            on_open_change: None,
            hide_close_button: false,
            title: None,
            body: Vec::new(),
            footer: Vec::new(),
            on_close: None,
        }
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = v;
        self
    }

    pub fn placement(mut self, p: DrawerPlacement) -> Self {
        self.placement = p;
        self
    }

    /// `isKeyboardDismissDisabled` — stops Escape from closing the drawer.
    pub fn is_keyboard_dismiss_disabled(mut self, v: bool) -> Self {
        self.is_keyboard_dismiss_disabled = v;
        self
    }

    /// `variant` on `Drawer.Backdrop` — the scrim style.
    pub fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// `onOpenChange` — fires with `false` on every dismissal path, alongside
    /// [`Drawer::on_close`].
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn is_dismissible(mut self, v: bool) -> Self {
        self.is_dismissible = v;
        self
    }

    pub fn hide_close_button(mut self, v: bool) -> Self {
        self.hide_close_button = v;
        self
    }

    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn footer_child(mut self, el: impl IntoElement) -> Self {
        self.footer.push(el.into_any_element());
        self
    }

    pub fn on_close(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(std::sync::Arc::new(f));
        self
    }
}

impl Default for Drawer {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Drawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for Drawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // v3 keeps a closing panel on screen for its `slide-out-to-*` run.
        let phase = crate::util::overlay_phase(
            window,
            cx,
            crate::modal::dialog_key(&self.id, "phase"),
            self.is_open,
        );
        if phase == crate::util::OverlayPhase::Closed {
            return gpui::div().into_any_element();
        }
        let exiting = phase == crate::util::OverlayPhase::Exiting;

        // Escape has to reach the overlay, and key events only travel to the
        // focused element and its ancestors. Claiming focus while nothing
        // inside holds it makes Escape work immediately; once a field inside
        // takes focus the event still bubbles up to here.
        let focus =
            window.use_keyed_state(crate::modal::dialog_key(&self.id, "focus"), cx, |_, cx| {
                cx.focus_handle()
            });
        let focus_handle = focus.read(cx).clone();
        if !focus_handle.contains_focused(window, cx) {
            window.focus(&focus_handle);
        }

        // A drag in progress: where it started along the dismissal axis, and how
        // far it has come. `use_keyed_state` takes `cx` mutably, so it precedes
        // the theme tokens.
        let drag =
            window.use_keyed_state(crate::modal::dialog_key(&self.id, "drag"), cx, |_, _| {
                None::<(f32, f32)>
            });
        let drag_now = *drag.read(cx);
        // How far the panel has been pulled toward its edge, which is what the
        // panel is offset by while the pointer is down.
        let drag_offset = drag_now.map_or(0.0, |(_, offset)| offset.max(0.0));

        let colors = cx.colors();

        // header
        let mut header = gpui::div()
            .flex()
            .items_center()
            .justify_between()
            .px(px(16.))
            .py(px(14.));
        if let Some(title) = &self.title {
            header = header.child(
                gpui::div()
                    .text_size(px(16.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(title.to_string()),
            );
        } else {
            header = header.child("");
        }
        if !self.hide_close_button && self.is_dismissible {
            if let Some(on_close) = self.on_close.clone() {
                let mut btn = gpui::div()
                    .id("drawer-close")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(28.))
                    .rounded_full()
                    .cursor_pointer();
                btn = btn.hover(move |s| s.bg(colors.default.soft_hover()));
                btn = btn.on_click(move |ev, window, cx| on_close(ev, window, cx));
                header = header.child(
                    btn.child(
                        gpui::svg()
                            .size(px(14.))
                            .path(icons::CLOSE)
                            .text_color(colors.foreground),
                    ),
                );
            }
        }

        if self.is_dismissible {
            let axis = self.placement;
            let held_down = drag.clone();
            header = header.on_mouse_down(
                gpui::MouseButton::Left,
                move |ev: &gpui::MouseDownEvent, _window, cx| {
                    let at = drag_axis_position(axis, ev.position);
                    held_down.update(cx, |v, cx| {
                        *v = Some((at, 0.0));
                        cx.notify();
                    });
                },
            );
        }

        let mut panel = gpui::div()
            .flex()
            .flex_col()
            .bg(colors.overlay.background)
            .text_color(colors.foreground)
            .shadow(cx.layout().overlay_shadow.clone())
            .overflow_hidden()
            .child(header)
            .child(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .px(px(16.))
                    .pb(px(12.))
                    .text_size(px(14.))
                    .line_height(px(22.))
                    .children(self.body),
            );

        if !self.footer.is_empty() {
            panel = panel.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(12.))
                    .border_t_1()
                    .border_color(colors.separator)
                    .children(self.footer),
            );
        }

        // anchor to the requested edge, pulled out by however far the drag has
        // come: a div cannot be translated in this gpui, so the inset moves.
        let pulled = px(-drag_offset);
        let anchored = match self.placement {
            DrawerPlacement::Left => panel
                .w(PANEL_EXTENT)
                .max_w(gpui::relative(0.9))
                .h_full()
                .absolute()
                .left(pulled)
                .top(px(0.))
                .bottom(px(0.)),
            DrawerPlacement::Right => panel
                .w(PANEL_EXTENT)
                .max_w(gpui::relative(0.9))
                .h_full()
                .absolute()
                .right(pulled)
                .top(px(0.))
                .bottom(px(0.)),
            DrawerPlacement::Top => panel
                .h(PANEL_EXTENT)
                .max_h(gpui::relative(0.9))
                .w_full()
                .absolute()
                .top(pulled)
                .left(px(0.))
                .right(px(0.)),
            DrawerPlacement::Bottom => panel
                .h(PANEL_EXTENT)
                .max_h(gpui::relative(0.9))
                .w_full()
                .absolute()
                .bottom(pulled)
                .left(px(0.))
                .right(px(0.)),
        };

        // v3 backdrop variants; gpui has no backdrop-filter, so `Blur` renders a
        // lighter scrim than `Opaque` to keep the layering readable.
        let backdrop_bg = match self.backdrop {
            Backdrop::Opaque => colors.backdrop,
            Backdrop::Blur => colors.backdrop.alpha(colors.backdrop.a * 0.6),
            Backdrop::Transparent => gpui::transparent_black(),
        };
        // Every dismissal path reports through both callbacks.
        let dismiss: Option<OnClose> = match (self.on_close.clone(), self.on_open_change.clone()) {
            (None, None) => None,
            (close, open_change) => Some(crate::util::shared(
                move |ev: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if let Some(f) = &close {
                        f(ev, window, cx);
                    }
                    if let Some(f) = &open_change {
                        f(false, window, cx);
                    }
                },
            )),
        };

        // `ClickEvent::default()` is the Keyboard variant, so a caller
        // inspecting the event sees a keyboard activation, which is what this
        // is.
        let keyboard_dismiss = if self.is_keyboard_dismiss_disabled {
            None
        } else {
            dismiss.clone()
        };

        let mut overlay = gpui::div()
            .absolute()
            .inset_0()
            .track_focus(&focus_handle)
            .on_key_down(move |ev: &gpui::KeyDownEvent, window, cx| {
                if ev.keystroke.key == "escape" {
                    if let Some(f) = &keyboard_dismiss {
                        f(&ClickEvent::default(), window, cx);
                    }
                }
            });

        // `Drag to dismiss`: pull the panel toward its edge and let go. The
        // pointer leaves the header the moment it moves, so the move and the
        // release are watched here, on the overlay that covers the window --
        // the same reason a table's column resize is handled on the table.
        if self.is_dismissible {
            let axis = self.placement;
            let held_move = drag.clone();
            let held_up = drag;
            let release = dismiss.clone();
            overlay = overlay
                .on_mouse_move(move |ev: &gpui::MouseMoveEvent, _window, cx| {
                    let Some((start, offset)) = *held_move.read(cx) else {
                        return;
                    };
                    let at = drag_axis_position(axis, ev.position);
                    // Toward the edge is positive, whichever edge that is.
                    let next = match axis {
                        DrawerPlacement::Left | DrawerPlacement::Top => start - at,
                        DrawerPlacement::Right | DrawerPlacement::Bottom => at - start,
                    }
                    .max(0.0);
                    // Redraw only when the pull actually changes.
                    if (next - offset).abs() >= 1.0 {
                        held_move.update(cx, |v, cx| {
                            *v = Some((start, next));
                            cx.notify();
                        });
                    }
                })
                .on_mouse_up(gpui::MouseButton::Left, move |_, window, cx| {
                    let Some((_, offset)) = *held_up.read(cx) else {
                        return;
                    };
                    held_up.update(cx, |v, cx| {
                        *v = None;
                        cx.notify();
                    });
                    // A quarter of the panel is far enough to mean it; anything
                    // less springs back, which is what clearing the drag does.
                    if offset >= f32::from(PANEL_EXTENT) * 0.25 {
                        if let Some(f) = &release {
                            f(&ClickEvent::default(), window, cx);
                        }
                    }
                });
        }
        // v3 fades the backdrop in alongside the panel.
        match (self.is_dismissible && !exiting, dismiss.clone()) {
            (true, Some(on_close)) => {
                overlay = overlay.child(crate::anim::entering(
                    gpui::div()
                        .id("drawer-backdrop")
                        .absolute()
                        .inset_0()
                        .bg(backdrop_bg)
                        .on_click(move |ev, window, cx| on_close(ev, window, cx)),
                    "drawer-backdrop-anim",
                    crate::anim::Motion::BACKDROP_IN,
                    cx,
                ));
            }
            _ => {
                let scrim = gpui::div().absolute().inset_0().bg(backdrop_bg);
                overlay = overlay.child(if exiting {
                    crate::anim::exiting(
                        scrim,
                        "drawer-backdrop-out",
                        crate::anim::ZoomBox::default(),
                        crate::anim::Motion::BACKDROP_OUT,
                        cx,
                    )
                } else {
                    crate::anim::entering(
                        scrim,
                        "drawer-backdrop-anim",
                        crate::anim::Motion::BACKDROP_IN,
                        cx,
                    )
                });
            }
        }
        // Drawers slide in from the edge they are anchored to.
        let edge = match self.placement {
            DrawerPlacement::Left => crate::anim::Edge::Left,
            DrawerPlacement::Right => crate::anim::Edge::Right,
            DrawerPlacement::Top => crate::anim::Edge::Top,
            DrawerPlacement::Bottom => crate::anim::Edge::Bottom,
        };
        // A pulled panel is being placed by the drag, so it must not also be
        // running its entry animation from the same edge.
        overlay = overlay.child(if exiting {
            crate::anim::exiting_to(
                anchored,
                "drawer-panel-out",
                edge,
                PANEL_EXTENT,
                crate::anim::Motion::DRAWER_OUT,
                cx,
            )
        } else {
            crate::anim::entering_from(
                anchored,
                "drawer-panel",
                edge,
                PANEL_EXTENT,
                crate::anim::Motion::DRAWER_IN,
                cx,
            )
        });

        overlay.into_any_element()
    }
}

/// Where a pointer sits along the axis a drawer is dismissed on.
fn drag_axis_position(placement: DrawerPlacement, at: gpui::Point<gpui::Pixels>) -> f32 {
    match placement {
        DrawerPlacement::Left | DrawerPlacement::Right => f32::from(at.x),
        DrawerPlacement::Top | DrawerPlacement::Bottom => f32::from(at.y),
    }
}
