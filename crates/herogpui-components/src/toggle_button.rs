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
    /// Set by [`ToggleButtonGroup`]: which end of the group this member is,
    /// and whether the group stacks. `.toggle-button-group .toggle-button` is
    /// `rounded-none` with the outer radius on the first and last member.
    group_edge: Option<(crate::button::GroupEdge, bool)>,
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
            group_edge: None,
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

    /// Joins this toggle to a group edge. Internal: a caller reaches it by
    /// putting the toggle in a [`ToggleButtonGroup`].
    pub(crate) fn group_edge(mut self, edge: crate::button::GroupEdge, vertical: bool) -> Self {
        self.group_edge = Some((edge, vertical));
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
            ElementId::Name(format!("{:?}-selected", self.id).into()),
            self.is_selected,
            self.default_selected,
        );

        // `.toggle-button:focus-visible` is `status-focused`.
        let focus_handle = crate::util::tab_stop_handle(
            ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
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
                e.bg(sem.color)
                    .text_color(sem.foreground)
                    .border_color(sem.color)
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

        // sizing — kept in locals so the press geometry below scales exactly
        // what was applied here.
        // `.toggle-button` is `h-10 md:h-9` with `--sm` at `h-9 md:h-8` and
        // `--lg` at `h-11 md:h-10`: 32 / 36 / 40 on a desktop, the same pair as
        // `.button`. This had them a step too tall.
        let (height, pad_x, gap) = match self.size {
            Size::Sm => (px(32.), px(12.), px(6.)),
            Size::Md => (px(36.), px(16.), px(8.)),
            Size::Lg => (px(40.), px(20.), px(8.)),
        };
        let text = self.size.text_size();
        let line = self.size.line_height();
        let radius = crate::util::control_radius(cx);
        el = el.h(height).text_size(text).line_height(line);
        el = if self.is_icon_only {
            el.w(height)
        } else {
            el.px(pad_x).gap(gap)
        };

        el = crate::button::group_radius_any(el, self.group_edge, radius);

        if self.is_disabled {
            el = el.opacity(layout.disabled_opacity);
        } else {
            let hover_bg = colors.default.color;
            el = el.cursor_pointer().hover(move |s| s.bg(hover_bg));
            // v3 documents ToggleButton's pressed state as including the same
            // `scale(0.97)` transform as Button.
            el = crate::anim::pressed(
                el,
                crate::anim::PressBox {
                    height,
                    padding_x: (!self.is_icon_only).then_some(pad_x),
                    width: self.is_icon_only.then_some(height),
                    min_width: None,
                    text_size: text,
                    line_height: line,
                    gap,
                    radius,
                    shrink_x: true,
                },
                cx,
            );
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

        if self.is_disabled {
            return el;
        }
        crate::util::ring_if_focused(
            el.track_focus(&focus_handle),
            &focus_handle,
            true,
            Vec::new(),
            window,
            cx,
        )
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
    /// Whether a `ToggleButtonGroup.Separator` sits before each member after
    /// the first. v3 composes it as a child, and hides it when detached.
    separators: bool,
    is_vertical: bool,
    disallow_empty_selection: bool,
    full_width: bool,
    children: Vec<ToggleButton>,
    on_change: Option<std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
}

impl ToggleButtonGroup {
    /// `orientation` — lays the group out along the given axis.
    pub fn orientation(mut self, orientation: SelectionOrientation) -> Self {
        self.is_vertical = orientation == SelectionOrientation::Vertical;
        self
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
            separators: true,
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

    /// `ToggleButtonGroup.Separator` — the hairline between members.
    pub fn separators(mut self, v: bool) -> Self {
        self.separators = v;
        self
    }

    pub fn is_detached(mut self, v: bool) -> Self {
        self.is_detached = v;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn selected_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.selected = keys.into_iter().map(Into::into).collect();
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
        // `.toggle-button-group` is `inline-flex items-center justify-center
        // gap-0`; `--detached` is `gap-1` and restores each member's full
        // radius.
        let gap = if self.is_detached { px(4.) } else { px(0.) };
        let mut row = div()
            .flex()
            .items_center()
            .justify_center()
            .when(self.is_vertical, |r| r.flex_col())
            .when(!self.is_vertical, |r| r.flex_row())
            .gap(gap)
            .when(self.full_width, |r| r.w_full());
        let total = self.children.len();
        let separators = self.separators && !self.is_detached;
        let is_vertical = self.is_vertical;
        let full_width = self.full_width;
        // `.toggle-button-group__separator` is `bg-current opacity-15`, 1px by
        // half the member, one pixel before its leading edge. This used to be a
        // 20px line in `--separator` with a 2px margin, which is neither the
        // colour nor the geometry v3 draws.
        let separator_color = cx.colors().foreground.alpha(0.15);
        let separator_radius = crate::util::hairline_radius(cx);

        let selected = self.selected.clone();
        let mode = self.selection_mode;
        let disallow_empty = self.disallow_empty_selection;

        for (i, btn) in self.children.into_iter().enumerate() {
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

            // The edge decides which corners stay round, so it has to reach the
            // `ToggleButton` before it becomes an element.
            let edge = if self.is_detached || total <= 1 {
                crate::button::GroupEdge::Only
            } else if i == 0 {
                crate::button::GroupEdge::Start
            } else if i + 1 == total {
                crate::button::GroupEdge::End
            } else {
                crate::button::GroupEdge::Middle
            };
            let mut slot = div()
                .relative()
                .child(btn.group_edge(edge, is_vertical))
                .when(full_width, |sl| sl.flex_1());
            if separators && i > 0 {
                slot = slot.child(
                    div()
                        .absolute()
                        .bg(separator_color)
                        .rounded(separator_radius)
                        .map(|sep| {
                            if is_vertical {
                                sep.left(gpui::relative(0.25))
                                    .top(px(-1.))
                                    .w(gpui::relative(0.5))
                                    .h(px(1.))
                            } else {
                                sep.left(px(-1.))
                                    .top(gpui::relative(0.25))
                                    .w(px(1.))
                                    .h(gpui::relative(0.5))
                            }
                        }),
                );
            }
            row = row.child(slot);
        }

        row
    }
}

/// Separator element for ToggleButtonGroup — visual divider.
#[derive(IntoElement)]
pub struct ToggleSeparator;

impl RenderOnce for ToggleSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().w(px(1.)).h(px(20.)).bg(cx.colors().separator)
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
