//! Toolbar — port of `@heroui/toolbar`.
//!
//! A container for interactive controls with arrow-key navigation. Mirrors the
//! React API: `orientation` and `isAttached`.

use gpui::{
    div, px, AnyElement, App, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Pixels,
    RenderOnce, Styled, Window,
};
use herogpui_core::Orientation;
use herogpui_theme::ActiveTheme;

/// HeroUI Toolbar.
#[derive(IntoElement)]
pub struct Toolbar {
    orientation: Orientation,
    is_attached: bool,
    gap: Option<Pixels>,
    children: Vec<AnyElement>,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            is_attached: false,
            gap: None,
            children: Vec::new(),
        }
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `isAttached` — renders the toolbar as a fully-rounded surface that hugs
    /// its controls.
    pub fn is_attached(mut self, v: bool) -> Self {
        self.is_attached = v;
        self
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = Some(gap.into());
        self
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        // Attached toolbars sit tight against their controls; detached ones use
        // the wider 8px rhythm between groups.
        let gap = self
            .gap
            .unwrap_or(if self.is_attached { px(4.) } else { px(8.) });

        // `Inherits from React Aria Toolbar`: the arrows move between the
        // controls *inside* it and wrap at the ends; Tab leaves it. The
        // children are the tab stops, so the arrows step with gpui's
        // window-wide `focus_next`/`focus_prev` and then check whether the
        // focus is still inside the toolbar -- the same step-and-check
        // `util::trap_tab` uses to keep Tab in a dialog. A roving tab stop
        // would need the focused child to claim the toolbar's one handle, but
        // the children are opaque elements this component cannot reach into,
        // and wrapping each one to claim the handle would put the focus on the
        // wrapper rather than the control (breaking Enter on a focused button,
        // which gpui fires on the focused element). The scope handle is
        // deliberately *not* a tab stop, so the toolbar adds nothing to the
        // window's Tab order -- it only marks the subtree whose stops the
        // arrows may visit.
        let scope = cx.focus_handle();

        let mut el = div().key_context("Toolbar").flex().items_center().gap(gap);

        el = match self.orientation {
            Orientation::Horizontal => el.flex_row(),
            Orientation::Vertical => el.flex_col(),
        };

        if self.is_attached {
            el = el
                // `.toolbar--attached` is `p-1 rounded-3xl`.
                .p(px(4.))
                .rounded(crate::util::control_radius(cx))
                .bg(colors.surface_secondary)
                .border_1()
                .border_color(colors.border);
        }

        el = el.track_focus(&scope);
        // React Aria's `useToolbar` handles exactly the orientation's axis —
        // ArrowRight/ArrowLeft when horizontal, ArrowDown/ArrowUp when
        // vertical — and returns early for everything else. Resolve the key
        // first, then act, which is the same shape Tabs uses; only a key the
        // match actually consumed reaches `stop_propagation`, so Tab, the
        // cross-axis arrows, Home/End and an enclosing scroller still see
        // them.
        let vertical = self.orientation == Orientation::Vertical;
        el.on_key_down(move |event: &KeyDownEvent, window, cx| {
            let key = match (vertical, event.keystroke.key.as_str()) {
                (false, "right") | (true, "down") => "next",
                (false, "left") | (true, "up") => "prev",
                _ => return,
            };
            cx.stop_propagation();
            match key {
                // Next stop in the window's order; if that one left the
                // toolbar, the focus was on the last control and wraps to
                // the first (the step landed on the sibling that follows).
                "next" => {
                    window.focus_next();
                    if !scope.contains_focused(window, cx) {
                        window.focus(&scope);
                        window.focus_next();
                    }
                }
                // Same backwards, re-entering from the far end: walk
                // forward until the focus leaves the toolbar, then one
                // back, bounded so an empty toolbar cannot spin.
                "prev" => {
                    window.focus_prev();
                    if !scope.contains_focused(window, cx) {
                        window.focus(&scope);
                        for _ in 0..256 {
                            window.focus_next();
                            if !scope.contains_focused(window, cx) {
                                window.focus_prev();
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .children(self.children)
    }
}
