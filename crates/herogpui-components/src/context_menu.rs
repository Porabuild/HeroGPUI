//! `ContextMenu` — a right-click menu over an area (HeroGPUI extension;
//! HeroUI v3 has no context menu, only the trigger-anchored `Dropdown`).
//!
//! It reuses the Dropdown's [`Menu`] panel unchanged — rows, sections,
//! submenus, typeahead, keyboard navigation, disabled keys, the enter/exit
//! motion, and the topmost-overlay Escape / outside-press arbitration — and
//! only replaces the trigger: a secondary-button press inside the wrapped
//! area opens the menu with its top-left corner at the pointer, flipping to
//! stay inside the window. Choosing an item, Escape or a press outside the
//! panel closes it.
//!
//! ```
//! use herogpui_components::{ContextMenu, MenuItem};
//! use gpui::{div, prelude::*, px};
//!
//! let menu = ContextMenu::new(
//!     "canvas-menu",
//!     div().size(px(240.)).child("Right-click here"),
//!     vec![
//!         MenuItem::new("copy", "Copy").shortcut("⌘C"),
//!         MenuItem::new("paste", "Paste").shortcut("⌘V"),
//!         MenuItem::Separator,
//!         MenuItem::new("delete", "Delete").danger(),
//!     ],
//! )
//! .on_action(|key, _window, _cx| println!("chose {key}"));
//! ```

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    px, size, AnyElement, App, Bounds, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Pixels, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::{element_id, Placement};

use crate::{Menu, MenuItem};

type Callback<T> = Arc<dyn Fn(&T, &mut Window, &mut App) + 'static>;

/// A secondary-click menu over `child`. See the [module docs](self).
#[derive(IntoElement)]
pub struct ContextMenu {
    id: gpui::ElementId,
    child: AnyElement,
    items: Vec<MenuItem>,
    disabled_keys: Vec<SharedString>,
    is_disabled: bool,
    on_action: Option<Callback<SharedString>>,
    on_open_change: Option<Callback<bool>>,
}

impl ContextMenu {
    /// A context menu with `items` over the `child` area. `id` keys the
    /// menu's state; give every instance its own.
    pub fn new(
        id: impl Into<gpui::ElementId>,
        child: impl IntoElement,
        items: Vec<MenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            items,
            disabled_keys: Vec::new(),
            is_disabled: false,
            on_action: None,
            on_open_change: None,
        }
    }

    /// Runs with the chosen item's key; the menu then closes.
    pub fn on_action(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Arc::new(f));
        self
    }

    /// Reports every open and close.
    pub fn on_open_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Arc::new(f));
        self
    }

    /// Items that render dimmed and cannot be chosen or focused.
    pub fn disabled_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.disabled_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// A disabled context menu ignores secondary clicks; the area still
    /// renders and receives every other event.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }
}

fn set_open(
    own: &Option<gpui::Entity<bool>>,
    on_open_change: &Option<Callback<bool>>,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(held) = own {
        held.update(cx, |v, cx| {
            *v = open;
            cx.notify();
        });
    }
    if let Some(cb) = on_open_change {
        cb(&open, window, cx);
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.id.clone();
        let (is_open, own) =
            crate::util::controlled(window, cx, element_id::scoped(&base, "open"), None, false);
        let (phase, overlay_token) = crate::util::overlay_scope(
            window,
            cx,
            element_id::scoped(&base, "phase"),
            is_open,
            true,
        );
        // The pointer position, as a zero-size anchor the shared popover
        // positioner places the panel against (offset 0, so the panel's
        // corner sits on the pointer).
        let anchor = window
            .use_keyed_state(element_id::scoped(&base, "anchor"), cx, |_, _| {
                Rc::new(Cell::new(None::<Bounds<Pixels>>))
            })
            .read(cx)
            .clone();
        let focus_first =
            window.use_keyed_state(element_id::scoped(&base, "focus-first"), cx, |_, _| false);

        let mut area = gpui::div()
            .id(element_id::scoped(&base, "area"))
            .child(self.child);
        if !self.is_disabled {
            let anchor = anchor.clone();
            let own = own.clone();
            let on_open_change = self.on_open_change.clone();
            area = area.on_mouse_down(MouseButton::Right, move |event, window, cx| {
                anchor.set(Some(Bounds::new(event.position, size(px(0.), px(0.)))));
                set_open(&own, &on_open_change, true, window, cx);
                cx.stop_propagation();
            });
        }

        let mut root = gpui::div().relative().child(area);
        if phase != crate::util::OverlayPhase::Closed {
            let mut menu = Menu::new(element_id::scoped(&base, "menu-content"), self.items)
                .id(element_id::scoped(&base, "menu"))
                .dropdown_composition()
                .focus_first(focus_first)
                .exiting(phase == crate::util::OverlayPhase::Exiting)
                .disabled_keys(self.disabled_keys)
                .overlay_token(overlay_token);
            if let Some(on_action) = self.on_action {
                menu = menu.on_action(move |key, window, cx| on_action(key, window, cx));
            }
            let on_open_change = self.on_open_change;
            menu = menu.on_dismiss(move |_refocus, window, cx| {
                set_open(&own, &on_open_change, false, window, cx);
            });
            root = root.child(crate::util::floating(
                crate::popover::popover_with_resolved_placement(
                    anchor,
                    Placement::BottomStart,
                    px(0.),
                    None,
                    menu.panel_debug_label("context-menu"),
                ),
            ));
        }
        root
    }
}
