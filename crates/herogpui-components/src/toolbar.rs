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

        let mut el = div().flex().items_center().gap(gap);

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

        // `Inherits from React Aria Toolbar`: the arrows move between the
        // controls inside it. Those controls are the tab stops, so the arrows
        // ask gpui for the next and the previous one -- the difference from v3 is
        // at the ends, where React Aria stays inside the toolbar and this walks
        // on to whatever follows.
        el.on_key_down(
            |event: &KeyDownEvent, window, _| match event.keystroke.key.as_str() {
                "right" | "down" => window.focus_next(),
                "left" | "up" => window.focus_prev(),
                _ => {}
            },
        )
        .children(self.children)
    }
}
