//! AlertDialog — port of `@heroui/alert-dialog` (v3).
//!
//! A modal for critical confirmations. Unlike [`Modal`](crate::modal::Modal) it
//! is not dismissible by clicking the backdrop, it always announces a title and
//! description, and it renders a confirm/cancel action pair.

use std::sync::Arc;

use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::{Backdrop, Color, Size, Variant};
use herogpui_theme::ActiveTheme;

use crate::{
    button::Button,
    icons,
    modal::{ModalPlacement, OnOpenChange},
    util,
};

/// AlertDialog width preset (`size`) — `xs | sm | md | lg | cover`.
///
/// There is no `full` here: a critical confirmation should never fill the
/// viewport edge to edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertDialogSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    Cover,
}

impl AlertDialogSize {
    pub const ALL: [AlertDialogSize; 5] = [
        AlertDialogSize::Xs,
        AlertDialogSize::Sm,
        AlertDialogSize::Md,
        AlertDialogSize::Lg,
        AlertDialogSize::Cover,
    ];

    fn width(self) -> gpui::Pixels {
        match self {
            AlertDialogSize::Xs => px(280.),
            AlertDialogSize::Sm => px(360.),
            AlertDialogSize::Md => px(440.),
            AlertDialogSize::Lg => px(560.),
            AlertDialogSize::Cover => px(800.),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AlertDialogSize::Xs => "Xs",
            AlertDialogSize::Sm => "Sm",
            AlertDialogSize::Md => "Md",
            AlertDialogSize::Lg => "Lg",
            AlertDialogSize::Cover => "Cover",
        }
    }
}

type OnAction = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI AlertDialog (controlled).
#[derive(IntoElement)]
pub struct AlertDialog {
    /// Keys this dialog's own state; see [`AlertDialog::id`].
    id: gpui::ElementId,
    is_open: bool,
    title: SharedString,
    description: Option<SharedString>,
    size: AlertDialogSize,
    backdrop: Backdrop,
    placement: ModalPlacement,
    /// `status` on `AlertDialog.Icon`. `None` renders no icon, matching a v3
    /// dialog that does not compose one.
    status: Option<Color>,
    is_dismissible: bool,
    is_keyboard_dismiss_disabled: bool,
    on_open_change: Option<OnOpenChange>,
    confirm_label: SharedString,
    cancel_label: SharedString,
    /// Renders the confirm button as destructive.
    is_destructive: bool,
    /// Disables the confirm button while an action is in flight.
    is_pending: bool,
    children: Vec<AnyElement>,
    on_confirm: Option<OnAction>,
    on_cancel: Option<OnAction>,
}

impl AlertDialog {
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

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: gpui::ElementId::Name("alert-dialog".into()),
            is_open: false,
            title: title.into(),
            description: None,
            size: AlertDialogSize::default(),
            backdrop: Backdrop::Opaque,
            placement: ModalPlacement::default(),
            status: None,
            // v3 defaults an alert dialog to non-dismissible: the user has to
            // pick one of the two actions.
            is_dismissible: false,
            // v3 defaults this to true for an alert dialog.
            is_keyboard_dismiss_disabled: true,
            on_open_change: None,
            confirm_label: "Confirm".into(),
            cancel_label: "Cancel".into(),
            is_destructive: false,
            is_pending: false,
            children: Vec::new(),
            on_confirm: None,
            on_cancel: None,
        }
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = v;
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn size(mut self, size: AlertDialogSize) -> Self {
        self.size = size;
        self
    }

    /// `placement` on `AlertDialog.Container`.
    pub fn placement(mut self, placement: ModalPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// `status` on `AlertDialog.Icon` — shows the status glyph in that colour.
    pub fn status(mut self, status: Color) -> Self {
        self.status = Some(status);
        self
    }

    /// `isKeyboardDismissDisabled` — Escape is disabled by default here, so
    /// pass `false` to allow it.
    pub fn is_keyboard_dismiss_disabled(mut self, v: bool) -> Self {
        self.is_keyboard_dismiss_disabled = v;
        self
    }

    /// `isDismissable` — allows dismissal by clicking the backdrop.
    pub fn is_dismissible(mut self, v: bool) -> Self {
        self.is_dismissible = v;
        self
    }

    /// `onOpenChange` — fires with `false` when the dialog is dismissed.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Arc::new(f));
        self
    }

    pub fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn confirm_label(mut self, text: impl Into<SharedString>) -> Self {
        self.confirm_label = text.into();
        self
    }

    pub fn cancel_label(mut self, text: impl Into<SharedString>) -> Self {
        self.cancel_label = text.into();
        self
    }

    pub fn is_destructive(mut self, v: bool) -> Self {
        self.is_destructive = v;
        self
    }

    pub fn is_pending(mut self, v: bool) -> Self {
        self.is_pending = v;
        self
    }

    pub fn on_confirm(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_confirm = Some(Arc::new(handler));
        self
    }

    pub fn on_cancel(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_cancel = Some(Arc::new(handler));
        self
    }
}

impl ParentElement for AlertDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for AlertDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // v3 keeps a closing panel on screen for its `[data-exiting]` run.
        let phase = util::overlay_phase(
            window,
            cx,
            crate::modal::dialog_key(&self.id, "phase"),
            self.is_open,
        );
        if phase == util::OverlayPhase::Closed {
            return div().into_any_element();
        }
        let exiting = phase == util::OverlayPhase::Exiting;

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

        let colors = cx.colors();
        let layout = cx.layout();

        let mut panel = div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(8.))
            .w(self.size.width())
            .max_w_full()
            .p(px(24.))
            .rounded(util::container_radius(cx))
            .bg(colors.overlay.background)
            // v3 gives a floating panel no border; dark mode's inset hairline is
            // what separates it from the page.
            .when_some(layout.overlay_hairline, |el, hairline| {
                el.border(layout.border_width).border_color(hairline)
            })
            .text_color(colors.overlay.foreground)
            .when(!layout.overlay_shadow.is_empty(), |e| {
                e.shadow(layout.overlay_shadow.clone())
            })
            .child(
                div()
                    .text_size(px(18.))
                    .line_height(px(25.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(self.title.to_string()),
            );

        if let Some(status) = self.status {
            let role = cx.role(status);
            // Soft disc behind the glyph, as in v3's Icon slot.
            panel = panel.child(
                div()
                    .absolute()
                    .top(px(24.))
                    .right(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(36.))
                    .rounded_full()
                    .bg(role.soft())
                    .child(
                        gpui::svg()
                            .size(px(18.))
                            .path(icons::ALERT_TRIANGLE)
                            // svg() never inherits text colour.
                            .text_color(role.color),
                    ),
            );
        }

        if let Some(description) = &self.description {
            panel = panel.child(
                div()
                    .text_size(px(14.))
                    .line_height(px(21.))
                    .text_color(colors.muted)
                    .child(description.to_string()),
            );
        }

        if !self.children.is_empty() {
            panel = panel.child(div().flex().flex_col().gap(px(8.)).children(self.children));
        }

        // Cancel first, confirm last — the destructive action is furthest from
        // the reading position.
        let mut actions = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap(px(8.))
            .mt(px(8.));

        let mut cancel = Button::new("alert-dialog-cancel")
            .label(self.cancel_label.clone())
            .variant(Variant::Tertiary)
            .size(Size::Md);
        if let Some(on_cancel) = self.on_cancel.clone() {
            cancel = cancel.on_press(move |ev, window, cx| on_cancel(ev, window, cx));
        }

        let mut confirm = Button::new("alert-dialog-confirm")
            .label(self.confirm_label.clone())
            .variant(if self.is_destructive {
                Variant::Danger
            } else {
                Variant::Primary
            })
            .size(Size::Md)
            .is_pending(self.is_pending);
        if let Some(on_confirm) = self.on_confirm.clone() {
            confirm = confirm.on_press(move |ev, window, cx| on_confirm(ev, window, cx));
        }

        actions = actions.child(cancel).child(confirm);
        panel = panel.child(actions);

        let backdrop_bg = match self.backdrop {
            Backdrop::Opaque => colors.backdrop,
            Backdrop::Blur => colors.backdrop.alpha(colors.backdrop.a * 0.6),
            Backdrop::Transparent => gpui::transparent_black(),
        };

        // Dismissal reports through both callbacks, so a caller can use either.
        let dismiss: Option<OnAction> = match (self.on_cancel.clone(), self.on_open_change.clone())
        {
            _ if !self.is_dismissible => None,
            (None, None) => None,
            (cancel, open_change) => Some(util::shared(
                move |ev: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if let Some(f) = &cancel {
                        f(ev, window, cx);
                    }
                    if let Some(f) = &open_change {
                        f(false, window, cx);
                    }
                },
            )),
        };

        let keyboard_dismiss: Option<OnAction> = if self.is_keyboard_dismiss_disabled {
            None
        } else {
            match (self.on_cancel.clone(), self.on_open_change.clone()) {
                (None, None) => None,
                (cancel, open_change) => Some(util::shared(
                    move |ev: &ClickEvent, window: &mut Window, cx: &mut App| {
                        if let Some(f) = &cancel {
                            f(ev, window, cx);
                        }
                        if let Some(f) = &open_change {
                            f(false, window, cx);
                        }
                    },
                )),
            }
        };

        let mut backdrop = div()
            .id("alert-dialog-backdrop")
            .absolute()
            .inset_0()
            .bg(backdrop_bg);
        if let Some(on_dismiss) = dismiss {
            backdrop = backdrop.on_click(move |ev, window, cx| on_dismiss(ev, window, cx));
        }
        // v3 fades the backdrop in alongside the panel, and out with it.
        let backdrop = if exiting {
            crate::anim::exiting(
                backdrop,
                "alert-dialog-backdrop-out",
                crate::anim::ZoomBox::default(),
                crate::anim::Motion::BACKDROP_OUT,
                cx,
            )
        } else {
            crate::anim::entering(
                backdrop,
                "alert-dialog-backdrop-anim",
                crate::anim::Motion::BACKDROP_IN,
                cx,
            )
        };

        div()
            .id("alert-dialog-root")
            .absolute()
            .inset_0()
            .flex()
            .track_focus(&focus_handle)
            .on_key_down(move |ev: &gpui::KeyDownEvent, window, cx| {
                if ev.keystroke.key == "escape" {
                    if let Some(f) = &keyboard_dismiss {
                        f(&ClickEvent::default(), window, cx);
                    }
                }
            })
            .when(
                matches!(
                    self.placement,
                    ModalPlacement::Center | ModalPlacement::Auto
                ),
                |e| e.items_center().justify_center(),
            )
            .when(self.placement == ModalPlacement::Top, |e| {
                e.items_start().justify_center().pt(px(32.))
            })
            .when(self.placement == ModalPlacement::Bottom, |e| {
                e.items_end().justify_center().pb(px(32.))
            })
            .child(backdrop)
            .child({
                let zoom = crate::anim::ZoomBox::panel(px(24.), util::container_radius(cx))
                    .padding_x(px(24.))
                    .sized(self.size.width());
                if exiting {
                    crate::anim::exiting(
                        panel,
                        "alert-dialog-panel-out",
                        zoom,
                        crate::anim::Motion::PANEL_OUT,
                        cx,
                    )
                } else {
                    crate::anim::entering_zoom(
                        panel,
                        "alert-dialog-panel",
                        zoom,
                        crate::anim::Motion::PANEL_IN,
                        cx,
                    )
                }
            })
            .into_any_element()
    }
}
