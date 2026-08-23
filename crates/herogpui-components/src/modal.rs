//! Modal — port of `@heroui/modal`.
//!
//! Render the returned element from your root view; it covers the window
//! with a dimmed backdrop and a centered panel when `is_open`.

use gpui::{
    prelude::*, px, AnyElement, App, ClickEvent, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::Backdrop;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// Modal width preset (`size`) — `xs | sm | md | lg | cover | full`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModalSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
    /// Nearly fills the viewport, keeping a margin.
    Cover,
    /// Fills the viewport edge to edge.
    Full,
}

impl ModalSize {
    pub const ALL: [ModalSize; 6] = [
        ModalSize::Xs,
        ModalSize::Sm,
        ModalSize::Md,
        ModalSize::Lg,
        ModalSize::Cover,
        ModalSize::Full,
    ];

    fn width(self) -> gpui::Pixels {
        match self {
            ModalSize::Xs => px(280.),
            ModalSize::Sm => px(360.),
            ModalSize::Md => px(480.),
            ModalSize::Lg => px(640.),
            ModalSize::Cover => px(900.),
            ModalSize::Full => px(1600.),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ModalSize::Xs => "Xs",
            ModalSize::Sm => "Sm",
            ModalSize::Md => "Md",
            ModalSize::Lg => "Lg",
            ModalSize::Cover => "Cover",
            ModalSize::Full => "Full",
        }
    }
}

/// Vertical placement (`placement`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModalPlacement {
    /// `"auto"` — centred on desktop; v3 only switches to a sheet on mobile.
    #[default]
    Auto,
    Center,
    Top,
    Bottom,
}

/// `scroll` — whether overflow scrolls inside the dialog or moves the whole
/// container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModalScroll {
    /// The body scrolls; the dialog stays put.
    #[default]
    Inside,
    /// The dialog grows and the surrounding container scrolls.
    Outside,
}

pub type OnClose = std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// `onOpenChange` — every overlay reports dismissal through this shape.
pub type OnOpenChange = std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// HeroUI Modal (controlled).
#[derive(IntoElement)]
pub struct Modal {
    is_open: bool,
    title: Option<SharedString>,
    size: ModalSize,
    backdrop: Backdrop,
    placement: ModalPlacement,
    is_dismissible: bool,
    is_keyboard_dismiss_disabled: bool,
    scroll: ModalScroll,
    on_open_change: Option<OnOpenChange>,
    hide_close_button: bool,
    body: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    on_close: Option<OnClose>,
}

impl Modal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            title: None,
            size: ModalSize::Md,
            backdrop: Backdrop::Opaque,
            placement: ModalPlacement::Center,
            is_dismissible: true,
            is_keyboard_dismiss_disabled: false,
            scroll: ModalScroll::default(),
            on_open_change: None,
            hide_close_button: false,
            body: Vec::new(),
            footer: Vec::new(),
            on_close: None,
        }
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = v;
        self
    }

    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn size(mut self, s: ModalSize) -> Self {
        self.size = s;
        self
    }

    /// Whether clicking the backdrop closes the modal (`isDismissable`).
    pub fn is_dismissible(mut self, v: bool) -> Self {
        self.is_dismissible = v;
        self
    }

    pub fn hide_close_button(mut self, v: bool) -> Self {
        self.hide_close_button = v;
        self
    }

    pub fn backdrop(mut self, b: Backdrop) -> Self {
        self.backdrop = b;
        self
    }

    pub fn placement(mut self, p: ModalPlacement) -> Self {
        self.placement = p;
        self
    }

    /// `scroll` — `Inside` keeps the dialog fixed and scrolls its body;
    /// `Outside` lets the dialog grow and scrolls the container.
    pub fn scroll(mut self, scroll: ModalScroll) -> Self {
        self.scroll = scroll;
        self
    }

    /// `onOpenChange` — fires with `false` on every dismissal path, alongside
    /// [`Modal::on_close`].
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn is_keyboard_dismiss_disabled(mut self, v: bool) -> Self {
        self.is_keyboard_dismiss_disabled = v;
        self
    }

    /// Body content (ModalBody).
    /// Adds a child to the footer row (ModalFooter).
    pub fn footer_child(mut self, el: impl IntoElement) -> Self {
        self.footer.push(el.into_any_element());
        self
    }

    /// Shows a close button and enables backdrop dismissal (`onClose`).
    pub fn on_close(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(std::sync::Arc::new(f));
        self
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for Modal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // v3 keeps a closing panel on screen for its `[data-exiting]` run.
        let phase = crate::util::overlay_phase(window, cx, "modal-phase", self.is_open);
        if phase == crate::util::OverlayPhase::Closed {
            return gpui::div().into_any_element();
        }
        let exiting = phase == crate::util::OverlayPhase::Exiting;

        // Escape has to reach the overlay, and key events only travel to the
        // focused element and its ancestors. Claiming focus while nothing
        // inside holds it makes Escape work immediately; once a field inside
        // takes focus the event still bubbles up to here.
        let focus = window.use_keyed_state("modal-focus", cx, |_, cx| cx.focus_handle());
        let focus_handle = focus.read(cx).clone();
        if !focus_handle.contains_focused(window, cx) {
            window.focus(&focus_handle);
        }

        let colors = cx.colors();

        // Every dismissal path reports through both callbacks, so a caller can
        // use either without losing events.
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
        let backdrop_click: Option<OnClose> = dismiss.clone();
        // `ClickEvent::default()` is the Keyboard variant, so a caller
        // inspecting the event sees a keyboard activation, which is what this
        // is.
        let keyboard_dismiss = if self.is_keyboard_dismiss_disabled {
            None
        } else {
            dismiss.clone()
        };

        // Header: title (optional) + close button
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
            if let Some(on_close) = dismiss.clone() {
                let mut btn = gpui::div()
                    .id("modal-close")
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

        let panel = gpui::div()
            .relative()
            .flex()
            .flex_col()
            .w(self.size.width())
            .max_w(px(720.))
            .when(self.scroll == ModalScroll::Inside, |e| {
                e.max_h(gpui::relative(0.85))
            })
            .bg(colors.overlay.background)
            .text_color(colors.foreground)
            .rounded(crate::util::container_radius(cx))
            .shadow(cx.layout().overlay_shadow.clone())
            .overflow_hidden()
            .child(header)
            .child(
                gpui::div()
                    .id("modal-body")
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .px(px(16.))
                    .pb(px(12.))
                    .text_size(px(14.))
                    .line_height(px(22.))
                    // `Inside` scrolls the body; `Outside` lets it grow and
                    // scrolls the container instead.
                    .when(self.scroll == ModalScroll::Inside, |e| e.overflow_y_scroll())
                    .children(self.body),
            );

        // Footer
        let panel = if self.footer.is_empty() {
            panel
        } else {
            panel.child(
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
            )
        };

        // Backdrop — v3 variants: opaque / blur / transparent
        // gpui has no backdrop-filter, so `Blur` renders a lighter scrim than
        // `Opaque` to keep the layering readable.
        let backdrop_bg = match self.backdrop {
            Backdrop::Opaque => colors.backdrop,
            Backdrop::Blur => colors.backdrop.alpha(colors.backdrop.a * 0.6),
            Backdrop::Transparent => gpui::transparent_black(),
        };
        let mut overlay = gpui::div()
            // `overflow_y_scroll` needs a stateful element, so the id is set
            // unconditionally and only the overflow is conditional.
            .id("modal-scroll")
            .track_focus(&focus_handle)
            .on_key_down(move |ev: &gpui::KeyDownEvent, window, cx| {
                if ev.keystroke.key == "escape" {
                    if let Some(f) = &keyboard_dismiss {
                        f(&ClickEvent::default(), window, cx);
                    }
                }
            })
            .absolute()
            .inset_0()
            .flex()
            .when(self.scroll == ModalScroll::Outside, |e| e.overflow_y_scroll())
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
            });
        // v3 fades the backdrop in alongside the panel (`.backdrop[data-entering]`).
        match (self.is_dismissible && !exiting, backdrop_click.clone()) {
            (true, Some(on_close)) => {
                overlay = overlay.child(crate::anim::entering(
                    gpui::div()
                        .id("modal-backdrop")
                        .absolute()
                        .inset_0()
                        .bg(backdrop_bg)
                        .on_click(move |ev, window, cx| on_close(ev, window, cx)),
                    "modal-backdrop-anim",
                    crate::anim::Motion::BACKDROP_IN,
                    cx,
                ));
            }
            _ => {
                let scrim = gpui::div().absolute().inset_0().bg(backdrop_bg);
                overlay = overlay.child(if exiting {
                    crate::anim::exiting(
                        scrim,
                        "modal-backdrop-out",
                        crate::anim::ZoomBox::default(),
                        crate::anim::Motion::BACKDROP_OUT,
                        cx,
                    )
                } else {
                    crate::anim::entering(
                        scrim,
                        "modal-backdrop-anim",
                        crate::anim::Motion::BACKDROP_IN,
                        cx,
                    )
                });
            }
        }
        let zoom = crate::anim::ZoomBox {
            width: Some(self.size.width()),
            radius: Some(crate::util::container_radius(cx)),
            ..Default::default()
        };
        overlay = overlay.child(if exiting {
            crate::anim::exiting(
                panel,
                "modal-panel-out",
                zoom,
                crate::anim::Motion::PANEL_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                panel,
                "modal-panel",
                zoom,
                crate::anim::Motion::PANEL_IN,
                cx,
            )
        });

        overlay.into_any_element()
    }
}
