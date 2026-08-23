//! Popover — port of `@heroui/popover`.

use gpui::{
    prelude::*, px, AnyElement, App, ClickEvent, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::icons;

/// `placement` on `Popover.Content`.
///
/// Shares the one placement vocabulary with the pickers and dropdown.
pub use herogpui_core::Placement as PopoverPlacement;

type OnOpenChange = std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// HeroUI Popover (controlled).
#[derive(IntoElement)]
pub struct Popover {
    /// Distinguishes this popover's uncontrolled state from its neighbours'.
    id: gpui::ElementId,
    trigger: AnyElement,
    /// `isOpen` — `None` leaves the component holding the state, seeded
    /// from `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: PopoverPlacement,
    title: Option<SharedString>,
    show_close_button: bool,
    offset: gpui::Pixels,
    should_flip: bool,
    on_open_change: Option<OnOpenChange>,
    children: Vec<AnyElement>,
}

impl Popover {
    pub fn new(trigger: impl IntoElement) -> Self {
        Self {
            id: gpui::ElementId::Name("popover".into()),
            trigger: trigger.into_any_element(),
            is_open: None,
            default_open: false,
            placement: PopoverPlacement::Bottom,
            offset: px(8.),
            should_flip: true,
            title: None,
            show_close_button: true,
            on_open_change: None,
            children: Vec::new(),
        }
    }

    /// Distinguishes this popover from its neighbours.
    ///
    /// Only matters in the uncontrolled mode, where the open flag lives in
    /// element state: two popovers sharing a key would open together.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }

    /// `defaultOpen` — the uncontrolled initial state.
    ///
    /// Only consulted when `is_open` is not supplied; the component then owns
    /// the flag and the trigger toggles it.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    /// `offset` — distance from the trigger, 8px in v3.
    pub fn offset(mut self, offset: impl Into<gpui::Pixels>) -> Self {
        self.offset = offset.into();
        self
    }

    /// `shouldFlip` — lets the panel reposition to stay inside the window.
    ///
    /// gpui slides the panel back into the viewport rather than mirroring it to
    /// the opposite side, which is the closest behaviour `anchored` offers.
    pub fn should_flip(mut self, v: bool) -> Self {
        self.should_flip = v;
        self
    }

    pub fn placement(mut self, p: PopoverPlacement) -> Self {
        self.placement = p;
        self
    }

    /// Optional bold header inside the panel.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn show_close_button(mut self, v: bool) -> Self {
        self.show_close_button = v;
        self
    }

    /// Toggle handler wired to the trigger click.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl ParentElement for Popover {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-open", self.id).into()),
            self.is_open,
            self.default_open,
        );
        // v3 keeps a closing panel on screen for its `[data-exiting]` run.
        // `overlay_phase` takes `cx` mutably too, so it goes here.
        let phase = crate::util::overlay_phase(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-popover-phase", self.id).into()),
            is_open,
        );
        let exiting = phase == crate::util::OverlayPhase::Exiting;
        // The panel has to hold the focus for Escape to reach it.
        // `use_keyed_state` takes `cx` mutably, so it precedes the theme.
        let panel_focus = crate::util::panel_focus(
            window,
            cx,
            &format!("{:?}", self.id),
            phase != crate::util::OverlayPhase::Closed,
        );
        let colors = cx.colors();
        let layout = cx.layout();

        let mut trigger_wrap = gpui::div()
            .id(gpui::ElementId::Name(
                format!("{:?}-trigger", self.id).into(),
            ))
            .flex()
            .cursor_pointer();
        if self.on_open_change.is_some() || open_own.is_some() {
            let on_open_change = self.on_open_change.clone();
            let own = open_own.clone();
            let open = is_open;
            trigger_wrap = trigger_wrap.on_click(move |_: &ClickEvent, window, cx| {
                // Uncontrolled: flip our own copy, or the trigger would be
                // inert without a caller handler.
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = !open;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_open_change {
                    cb(!open, window, cx);
                }
            });
        }

        let mut root = gpui::div()
            .relative()
            .flex()
            .flex_col()
            .items_start()
            .child(trigger_wrap.child(self.trigger));

        if phase == crate::util::OverlayPhase::Closed {
            return root;
        }

        // Panel
        let mut header_row = gpui::div().flex().items_center().justify_between();
        if let Some(title) = &self.title {
            header_row = header_row.child(
                gpui::div()
                    .text_size(px(14.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.foreground)
                    .child(title.to_string()),
            );
        } else {
            header_row = header_row.child("");
        }
        if self.show_close_button {
            let close_own = open_own.clone();
            if self.on_open_change.is_some() || close_own.is_some() {
                let on_open_change = self.on_open_change.clone();
                let mut btn = gpui::div()
                    .id("popover-close")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(24.))
                    .rounded_full()
                    .cursor_pointer();
                let hover_bg = colors.default.soft_hover();
                btn = btn.hover(move |s| s.bg(hover_bg));
                btn = btn.on_click(move |_, window, cx| {
                    // Uncontrolled: clear our own copy as well.
                    if let Some(held) = &close_own {
                        held.update(cx, |v, cx| {
                            *v = false;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_open_change {
                        cb(false, window, cx);
                    }
                });
                header_row = header_row.child(
                    btn.child(
                        gpui::svg()
                            .size(px(12.))
                            .path(icons::CLOSE)
                            .text_color(colors.muted),
                    ),
                );
            }
        }

        let mut panel = gpui::div()
            .w(px(260.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .px(px(16.))
            .py(px(16.))
            .bg(colors.overlay.background)
            .text_color(colors.surface.foreground)
            .rounded(crate::util::control_radius(cx))
            // v3 gives a floating panel no border: `.popover` and friends are
            // `bg-overlay shadow-overlay` and a radius, and dark mode's
            // inset hairline is what separates the panel from the page.
            .when_some(layout.overlay_hairline, |el, hairline| {
            el.border(layout.border_width).border_color(hairline)
            })
            .shadow(layout.overlay_shadow.clone());

        if self.title.is_some() || self.show_close_button {
            panel = panel.child(header_row);
        }
        panel = panel.children(self.children);

        // React Aria dismisses a popover on Escape and on a press outside it.
        let dismiss_own = open_own;
        let dismiss_cb = self.on_open_change.clone();
        let panel = crate::util::dismissable(panel.track_focus(&panel_focus), move |window, cx| {
            if let Some(held) = &dismiss_own {
                held.update(cx, |v, cx| {
                    *v = false;
                    cx.notify();
                });
            }
            if let Some(cb) = &dismiss_cb {
                cb(false, window, cx);
            }
        });

        let placed = crate::util::placed_panel(self.placement, self.offset);

        // v3 fades the panel in on `[data-entering]`.
        let zoom = crate::anim::ZoomBox::panel(px(12.), crate::util::control_radius(cx))
            .padding_x(px(14.))
            .sized(px(260.));
        let panel = if exiting {
            crate::anim::exiting(
                panel,
                "popover-panel-out",
                zoom,
                crate::anim::Motion::LIST_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                panel,
                "popover-panel",
                zoom,
                crate::anim::Motion::POPOVER_IN,
                cx,
            )
        };

        // `shouldFlip` lets the panel move to stay on screen. gpui's `anchored`
        // slides it back inside the window rather than mirroring it to the
        // opposite side, which is the closest primitive available.
        if self.should_flip {
            root = root.child(
                gpui::anchored()
                    .position_mode(gpui::AnchoredPositionMode::Local)
                    .snap_to_window()
                    .child(placed.child(panel)),
            );
        } else {
            root = root.child(placed.child(panel));
        }
        root
    }
}
