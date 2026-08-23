//! ToggleButton & ToggleButtonGroup — port of `@heroui/toggle-button` v3.
//!
//! Toggle between selected/unselected. Supports all button variants, sizes,
//! icon-only, controlled `isSelected` and group selection modes.

use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, ElementId, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::{Orientation as SelectionOrientation, SelectionMode, Size};
use herogpui_theme::ActiveTheme;


// ---------------------------------------------------------------------------
// ToggleButton
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleVariant {
    #[default]
    Default,
    Ghost,
}

#[derive(IntoElement)]
pub struct ToggleButton {
    id: ElementId,
    /// Selection key inside a group. Defaults to the element id, so a group can
    /// namespace its ids without breaking selection.
    key: Option<SharedString>,
    label: Option<SharedString>,
    variant: ToggleVariant,
    size: Size,
    /// `isSelected` — `None` leaves the button holding the state, seeded from
    /// `defaultSelected`.
    is_selected: Option<bool>,
    default_selected: bool,
    is_icon_only: bool,
    is_disabled: bool,
    children: Vec<AnyElement>,
    on_press: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_change: Option<Box<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
}

impl ToggleButton {
    /// Overrides the selection key, which otherwise mirrors the element id.
    pub fn key(mut self, key: impl Into<SharedString>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// The member's key inside a [`ToggleButtonGroup`].
    pub fn selection_key(&self) -> SharedString {
        self.key.clone().unwrap_or_else(|| match &self.id {
            ElementId::Name(name) => name.clone(),
            other => SharedString::from(format!("{other:?}")),
        })
    }

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            key: None,
            label: None,
            variant: ToggleVariant::Default,
            size: Size::Md,
            is_selected: None,
            default_selected: false,
            is_icon_only: false,
            is_disabled: false,
            children: Vec::new(),
            on_press: None,
            on_change: None,
        }
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn variant(mut self, v: ToggleVariant) -> Self {
        self.variant = v;
        self
    }


    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }


    pub fn is_selected(mut self, v: bool) -> Self {
        self.is_selected = Some(v);
        self
    }

    /// `defaultSelected` — the uncontrolled initial state.
    ///
    /// Only consulted when `isSelected` is not supplied; the button then owns
    /// the state and toggles itself. A group always drives selection, so this
    /// is for a standalone toggle.
    pub fn default_selected(mut self, v: bool) -> Self {
        self.default_selected = v;
        self
    }

    pub fn is_icon_only(mut self, v: bool) -> Self {
        self.is_icon_only = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn child(mut self, el: impl IntoElement) -> Self {
        self.children.push(el.into_any_element());
        self
    }

    /// `onChange` — reports the selection the press moves to.
    ///
    /// Fires alongside [`ToggleButton::on_press`]; use whichever shape suits.
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_press(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_press = Some(Box::new(f));
        self
    }
}

impl ParentElement for ToggleButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ToggleButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_selected, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-selected", self.id).into()),
            self.is_selected,
            self.default_selected,
        );

        let sem = cx.colors().accent;
        let colors = cx.colors();
        let layout = cx.layout();

        let mut el = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .whitespace_nowrap()
            .flex_shrink_0()
            .border_1()
            .when(is_selected, |e| {
                e.bg(sem.color).text_color(sem.foreground).border_color(sem.color)
            })
            .when(!is_selected, |e| match self.variant {
                ToggleVariant::Default => e
                    .bg(colors.surface.background)
                    .text_color(colors.foreground)
                    .border_color(colors.separator),
                ToggleVariant::Ghost => e
                    .bg(gpui::transparent_black())
                    .text_color(colors.foreground)
                    .border_color(gpui::transparent_black()),
            });

        // sizing
        el = match self.size {
            Size::Sm => {
                if self.is_icon_only {
                    el.w(px(32.)).h(px(32.))
                } else {
                    el.px(px(12.)).h(px(32.)).gap(px(6.))
                }
                .text_size(px(12.))
            }
            Size::Md => {
                if self.is_icon_only {
                    el.w(px(40.)).h(px(40.))
                } else {
                    el.px(px(16.)).h(px(40.)).gap(px(8.))
                }
                .text_size(px(14.))
            }
            Size::Lg => {
                if self.is_icon_only {
                    el.w(px(48.)).h(px(48.))
                } else {
                    el.px(px(20.)).h(px(48.)).gap(px(8.))
                }
                .text_size(px(16.))
            }
        };

        el = el.rounded(crate::util::control_radius(cx));

        if self.is_disabled {
            el = el.opacity(layout.disabled_opacity);
        } else {
            let hover_bg = colors.default.color;
            el = el.cursor_pointer().hover(move |s| s.bg(hover_bg));
        }

        if let Some(label) = self.label {
            el = el.child(label.to_string());
        }
        el = el.children(self.children);

        if !self.is_disabled
            && (self.on_press.is_some() || self.on_change.is_some() || own.is_some())
        {
            let on_press = self.on_press;
            let on_change = self.on_change;
            let next = !is_selected;
            el = el.on_click(move |ev, w, cx| {
                // Uncontrolled: flip our own copy, or a standalone toggle could
                // never change.
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_press {
                    cb(ev, w, cx);
                }
                if let Some(cb) = &on_change {
                    cb(next, w, cx);
                }
            });
        }

        el
    }
}

// ---------------------------------------------------------------------------
// ToggleButtonGroup
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct ToggleButtonGroup {
    selected: Vec<SharedString>,
    selection_mode: SelectionMode,
    is_detached: bool,
    is_vertical: bool,
    disallow_empty_selection: bool,
    full_width: bool,
    children: Vec<ToggleButton>,
    on_change: Option<std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
}

impl ToggleButtonGroup {
    /// `orientation` — the v3 name for [`ToggleButtonGroup::is_vertical`].
    pub fn orientation(self, orientation: SelectionOrientation) -> Self {
        self.is_vertical(orientation == SelectionOrientation::Vertical)
    }

    /// `disallowEmptySelection` — keeps at least one member selected.
    pub fn disallow_empty_selection(mut self, v: bool) -> Self {
        self.disallow_empty_selection = v;
        self
    }

    /// `onSelectionChange` — the v3 name for [`ToggleButtonGroup::on_change`].
    pub fn on_selection_change(
        self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change(handler)
    }

    pub fn new() -> Self {
        Self {
            selected: Vec::new(),
            selection_mode: SelectionMode::Multiple,
            is_detached: false,
            is_vertical: false,
            disallow_empty_selection: false,
            full_width: false,
            children: Vec::new(),
            on_change: None,
        }
    }

    pub fn selection_mode(mut self, m: SelectionMode) -> Self {
        self.selection_mode = m;
        self
    }

    pub fn is_detached(mut self, v: bool) -> Self {
        self.is_detached = v;
        self
    }

    pub fn is_vertical(mut self, v: bool) -> Self {
        self.is_vertical = v;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.selected = keys.into_iter().map(|k| k.into()).collect();
        self
    }

    pub fn on_change(
        mut self,
        f: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn child_toggle(mut self, btn: ToggleButton) -> Self {
        self.children.push(btn);
        self
    }
}

impl Default for ToggleButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ToggleButtonGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // Direct AnyElement children are wrapped as generic — for typed ToggleButton use child_toggle
        let _ = elements;
    }
}

impl RenderOnce for ToggleButtonGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let gap = if self.is_detached { px(8.) } else { px(0.) };
        let mut row = div()
            .flex()
            .when(self.is_vertical, |r| r.flex_col())
            .when(!self.is_vertical, |r| r.flex_row())
            .gap(gap)
            .when(self.full_width, |r| r.w_full());

        let selected = self.selected.clone();
        let mode = self.selection_mode;
        let disallow_empty = self.disallow_empty_selection;

        for (i, btn) in self.children.into_iter().enumerate() {
            if !self.is_detached && i > 0 {
                // Separator line between attached toggles
                row = row.child(
                    div()
                        .w(px(1.))
                        .h(px(20.))
                        .bg(cx.colors().separator)
                        .mx(px(2.)),
                );
            }
            // Reflect the group's selection into the child, and let the child
            // report the next selection back through the group's callback.
            let key = btn.selection_key();
            let is_selected = selected.iter().any(|k| k == &key);
            let mut btn = btn.is_selected(is_selected);

            if let Some(on_change) = self.on_change.clone() {
                let current = selected.clone();
                let key = key.clone();
                btn = btn.on_press(move |_, window, cx| {
                    let next =
                        crate::selection::next_selection(&current, &key, mode, disallow_empty);
                    on_change(&next, window, cx);
                });
            }

            row = row.child(btn);
        }

        row
    }
}

/// Separator element for ToggleButtonGroup — visual divider.
#[derive(IntoElement)]
pub struct ToggleSeparator;

impl RenderOnce for ToggleSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .w(px(1.))
            .h(px(20.))
            .bg(cx.colors().separator)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_key_defaults_to_the_element_id() {
        // A group namespaces its child ids, so selection has to fall back to
        // the id when no explicit key is given.
        let plain = ToggleButton::new("bold");
        assert_eq!(plain.selection_key().as_ref(), "bold");
        let keyed = ToggleButton::new("grp-bold").key("bold");
        assert_eq!(keyed.selection_key().as_ref(), "bold");
    }
}
