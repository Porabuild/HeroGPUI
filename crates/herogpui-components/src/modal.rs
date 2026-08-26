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

    /// `max-w-xs` … `max-w-lg` from `.modal__dialog--*`, which is Tailwind's
    /// scale: 20rem, 24rem, 28rem, 32rem. `Cover` and `Full` are `w-full`
    /// instead, so the width comes from the container.
    fn max_width(self) -> Option<gpui::Pixels> {
        match self {
            ModalSize::Xs => Some(px(320.)),
            ModalSize::Sm => Some(px(384.)),
            ModalSize::Md => Some(px(448.)),
            ModalSize::Lg => Some(px(512.)),
            ModalSize::Cover | ModalSize::Full => None,
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
    /// Keys this dialog's own state; see [`Modal::id`].
    id: gpui::ElementId,
    is_open: bool,
    title: Option<SharedString>,
    icon: Option<SharedString>,
    icon_color: Option<herogpui_core::Color>,
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

/// `"<dialog id>-<part>"`, the key one dialog's piece of state lives under.
///
/// Shared by the three dialogs so they cannot spell it differently.
pub(crate) fn dialog_key(id: &gpui::ElementId, part: &str) -> gpui::ElementId {
    gpui::ElementId::Name(format!("{id:?}-{part}").into())
}

impl Modal {
    /// The element id this dialog's state is keyed by.
    ///
    /// Not a v3 prop: gpui needs an explicit id, and the phase, the focus handle
    /// and the drag offset are all keyed by it. Two dialogs on screen with the
    /// same key share all three.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn new() -> Self {
        Self {
            id: gpui::ElementId::Name("modal".into()),
            is_open: false,
            title: None,
            icon: None,
            icon_color: None,
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

    /// `Modal.Icon` — the glyph above the heading, drawn in a `size-10
    /// rounded-3xl` box. v3 composes it as a child part and tints it with a
    /// class (`bg-default text-foreground`); this takes the asset path.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    /// The role colour of that box. Absent is v3's own default, `bg-default`.
    pub fn icon_color(mut self, color: herogpui_core::Color) -> Self {
        self.icon_color = Some(color);
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
        let (phase, dismissal_token) = crate::util::overlay_scope(
            window,
            cx,
            dialog_key(&self.id, "phase"),
            self.is_open,
            true,
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
            window.use_keyed_state(dialog_key(&self.id, "focus"), cx, |_, cx| cx.focus_handle());
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
        // `ClickEvent::default()` is the Keyboard variant, so a caller
        // inspecting the event sees a keyboard activation, which is what this
        // is.
        let keyboard_dismiss = if self.is_keyboard_dismiss_disabled {
            None
        } else {
            dismiss.clone()
        };

        // `.modal__header` is `flex flex-col gap-3` and carries no padding of
        // its own: the dialog's `p-6` is the whole inset. The heading is
        // `text-base font-medium`.
        // `.modal__icon` is `size-10 rounded-3xl`, a child of the header above
        // the heading -- not a disc in the corner.
        let icon = self.icon.as_ref().map(|path| {
            let (bg, fg) = match self.icon_color {
                Some(color) => {
                    let role = cx.role(color);
                    (role.soft(), role.soft_foreground())
                }
                None => (colors.default.color, colors.foreground),
            };
            gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .size(px(40.))
                .rounded(crate::util::control_radius(cx))
                .bg(bg)
                .child(gpui::svg().size(px(20.)).path(path.clone()).text_color(fg))
        });
        let header = if self.title.is_some() || icon.is_some() {
            Some(
                gpui::div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .children(icon)
                    .when_some(self.title.as_ref(), |el, title| {
                        el.child(
                            gpui::div()
                                .text_size(px(16.))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(title.to_string()),
                        )
                    }),
            )
        } else {
            None
        };

        let has_header = header.is_some();
        let has_body = !self.body.is_empty();
        // `Inside` scrolls the body; the two heights below are how. Both are
        // absolute pixels on purpose: gpui resolves `relative()` against the
        // parent's *content box* (the overlay's viewport minus its 40px
        // padding), so every percentage this file tried landed short of the
        // window and left the body clipped. The panel's cap is the viewport
        // itself, so an overflowing Inside dialog spans the window edge to
        // edge. v3's `.modal__dialog--scroll-inside` (`max-h-full`) caps at
        // the container's content box and keeps a `p-10` margin of scrim on
        // all four sides; copying that (a 1000px cap here) parks the panel's
        // top at 40 and leaves the deepest revealed rows at the window's
        // bottom edge, where the long-body behaviour test drives presses that
        // must stay on the panel. The viewport cap is the closest arrangement
        // that keeps every control reachable by the body's scroll alone, and
        // it only differs from v3 in the overflow case -- a dialog whose
        // content fits is still content-sized and centred. The body's budget
        // is the cap minus the dialog's `p-6` inset; the header and the
        // footer claim their own space from the flex layout before the body's
        // max height ever binds.
        let scroll_inside = self.scroll == ModalScroll::Inside;
        let inside_body_max = window.viewport_size().height - px(48.);
        // `.modal__dialog`: `w-full` with a `max-w-*` per size, `p-6`, and the
        // floating-panel radius. `Full` drops the radius and the shadow.
        let full = self.size == ModalSize::Full;
        let panel = gpui::div()
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .when(
                self.scroll == ModalScroll::Outside
                    && matches!(
                        self.placement,
                        ModalPlacement::Center | ModalPlacement::Auto
                    ),
                gpui::Styled::my_auto,
            )
            .when(
                self.scroll == ModalScroll::Outside && self.placement == ModalPlacement::Bottom,
                gpui::Styled::mt_auto,
            )
            .when_some(self.size.max_width(), |e, w| e.max_w(w))
            .p(px(24.))
            .when(self.scroll == ModalScroll::Inside, |e| {
                e.max_h(window.viewport_size().height)
            })
            .bg(colors.overlay.background)
            .text_color(colors.foreground)
            .when(!full, |e| {
                e.rounded(crate::util::container_radius(cx))
                    .shadow(cx.layout().overlay_shadow.clone())
            })
            .overflow_hidden()
            .when_some(header, gpui::ParentElement::child)
            .when(has_body, |panel| {
                panel.child(
                    gpui::div()
                        .id("modal-body")
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        // `.modal__header + .modal__body` is `mt-2`.
                        .when(has_header, |b| b.mt(px(8.)))
                        .text_size(px(14.))
                        // `leading-[1.43]` on `text-sm`.
                        .line_height(px(20.))
                        .text_color(colors.muted)
                        // v3 spells the body `min-h-0 flex-1` and scrolls it
                        // inside `.modal__dialog--scroll-inside`'s max height.
                        // There is no equivalent here: a gpui scroll container
                        // in an auto-height flex column measures as *zero*, so
                        // that spelling made every default modal draw its
                        // heading and its footer with nothing between them.
                        // `Outside` keeps the working arrangement: the body is
                        // content-sized and the container scrolls. `Inside`
                        // caps the panel at the viewport above and scrolls the
                        // body itself within that budget, and the budget is a
                        // *max* height, so a header and a footer still sit
                        // between the body and the panel's edges.
                        .when(scroll_inside, |b| {
                            b.max_h(inside_body_max).overflow_y_scroll()
                        })
                        .children(self.body),
                )
            });

        // `.modal__footer` is `flex-row items-center justify-end gap-2` with no
        // border: the separator this used to draw is not in v3's sheet.
        let panel = if self.footer.is_empty() {
            panel
        } else {
            panel.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(8.))
                    // `+ .modal__footer` is `mt-5` after either sibling.
                    .when(has_header || has_body, |f| f.mt(px(20.)))
                    .children(self.footer),
            )
        };

        // `.modal__close-trigger` is `absolute end-4 top-4`, outside the header.
        let panel = match (
            self.hide_close_button || !self.is_dismissible,
            dismiss.clone(),
        ) {
            (false, Some(on_close)) => panel.child(
                gpui::div().absolute().top(px(16.)).right(px(16.)).child(
                    crate::close_button::CloseButton::new("modal-close")
                        .on_press(move |ev, window, cx| on_close(ev, window, cx)),
                ),
            ),
            _ => panel,
        };

        // Backdrop dismissal lives on the **panel**, not on the backdrop.
        // gpui has no hitbox occlusion, so a `on_click` on the full-window
        // backdrop fires for a press on the panel above it as well — the
        // close button reported every press twice. `on_mouse_down_out` reads
        // the element's own bounds instead of hit-testing, so `close` runs
        // exactly when the press landed outside this box, which is the
        // backdrop. `is_dismissible` gates it, and the exit phase gets none:
        // the dialog is already closing.
        let panel = if self.is_dismissible && !exiting {
            if let Some(on_close) = dismiss.clone() {
                crate::util::dismiss_on_press_outside_with_token(
                    panel,
                    dismissal_token.clone(),
                    move |window, cx| {
                        on_close(&ClickEvent::default(), window, cx);
                        crate::util::DismissResult::Handled
                    },
                )
            } else {
                panel
            }
        } else {
            panel
        };

        // Backdrop — v3 variants: opaque / blur / transparent
        // gpui has no backdrop-filter, so `Blur` renders a lighter scrim than
        // `Opaque` to keep the layering readable.
        let backdrop_bg = match self.backdrop {
            Backdrop::Opaque => colors.backdrop,
            Backdrop::Blur => colors.backdrop.alpha(colors.backdrop.a * 0.6),
            Backdrop::Transparent => gpui::transparent_black(),
        };
        // `Tab` cycles the dialog's own controls: v3 documents that, and gpui's
        // tab order is the whole window's, so the dialog has to keep it.
        let mut overlay = crate::util::trap_tab(
            gpui::div()
                // `overflow_y_scroll` needs a stateful element, so the id is set
                // unconditionally and only the overflow is conditional.
                .id("modal-scroll")
                .track_focus(&focus_handle),
            &focus_handle,
        )
        .absolute()
        .inset_0()
        .flex()
        // `.modal__container` is `p-4 sm:p-10`.
        .p(px(40.))
        // `Outside` scrolls here -- the dialog grows and this container moves.
        // `Inside` has the body's own scroller instead; keeping this one would
        // put two scroll containers under the pointer, and a wheel over the
        // body would move the whole dialog while the body scrolled beneath it
        // -- the scrim comes up under the pointer and the next press dismisses
        // the modal. See `.modal__body`'s comment.
        .when(self.scroll == ModalScroll::Outside, |e| {
            // v3 scrolls the top-aligned backdrop and positions the dialog
            // within it with auto margins. Centering an oversized flex child
            // gives it a negative origin that no scroll offset can reach.
            e.flex_col().items_center().justify_start().overflow_y_scroll()
        })
        .when(
            self.scroll == ModalScroll::Inside
                && matches!(
                    self.placement,
                    ModalPlacement::Center | ModalPlacement::Auto
                ),
            |e| e.items_center().justify_center(),
        )
        .when(
            self.scroll == ModalScroll::Inside && self.placement == ModalPlacement::Top,
            |e| e.items_start().justify_center(),
        )
        .when(
            self.scroll == ModalScroll::Inside
                && self.placement == ModalPlacement::Bottom,
            |e| e.items_end().justify_center(),
        );
        if let Some(on_escape) = keyboard_dismiss {
            overlay = crate::util::dismiss_on_escape_with_token(
                overlay,
                dismissal_token,
                move |window, cx| {
                    on_escape(&ClickEvent::default(), window, cx);
                    crate::util::DismissResult::Handled
                },
            );
        }
        // `.modal__backdrop` is a bare scrim — `--opaque`/`--blur`/
        // `--transparent` are the Backdrop enum above: it must look dimmed
        // but never grab presses, because the panel's `on_mouse_down_out`
        // owns backdrop dismissal (see above), which also keeps the press out
        // of the panel's own controls. v3 fades it in alongside the panel
        // (`.backdrop[data-entering]`).
        let scrim = gpui::div()
            .id("modal-backdrop")
            .absolute()
            .inset_0()
            .bg(backdrop_bg);
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
        let zoom = crate::anim::ZoomBox {
            // The zoom needs a width to scale geometrically; `Cover` and `Full`
            // have none of their own, so the container's is as close as it gets.
            width: self.size.max_width(),
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
