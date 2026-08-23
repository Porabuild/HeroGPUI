//! ButtonGroup — port of `@heroui/button-group` (v3).
//!
//! Joined buttons sharing one outer radius. `variant` and `size` are propagated
//! to the members; `outline` is not part of the group vocabulary, so building a
//! group from [`ButtonGroup::button`] keeps the variants valid by construction.

use gpui::{
    div, prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use herogpui_core::{Orientation, Size, Variant};
use herogpui_theme::ActiveTheme;

use crate::{button::Button, util};

/// HeroUI ButtonGroup.
#[derive(IntoElement)]
pub struct ButtonGroup {
    variant: Variant,
    size: Size,
    disable_radius_merge: bool,
    hide_separator: bool,
    is_disabled: bool,
    orientation: Orientation,
    full_width: bool,
    /// Buttons added through [`ButtonGroup::button`], which inherit the group's
    /// variant and size.
    buttons: Vec<Button>,
    children: Vec<AnyElement>,
}

impl ButtonGroup {
    /// `isDisabled` — disables every member.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `orientation` — a vertical group stacks its buttons.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn new() -> Self {
        Self {
            variant: Variant::Primary,
            size: Size::Md,
            disable_radius_merge: false,
            hide_separator: false,
            is_disabled: false,
            orientation: Orientation::Horizontal,
            full_width: false,
            buttons: Vec::new(),
            children: Vec::new(),
        }
    }

    /// The variant every member inherits. `ButtonGroup` does not accept
    /// [`Variant::Outline`]; passing it falls back to [`Variant::Tertiary`],
    /// which is the closest group-legal style.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = if variant == Variant::Outline {
            Variant::Tertiary
        } else {
            variant
        };
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Keeps each button's own radius instead of merging into one shape.
    pub fn disable_radius_merge(mut self, v: bool) -> Self {
        self.disable_radius_merge = v;
        self
    }

    /// `hideSeparator` — removes the hairlines between members.
    pub fn hide_separator(mut self, v: bool) -> Self {
        self.hide_separator = v;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    /// Adds a button that inherits the group's `variant` and `size`.
    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ButtonGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let vertical = !self.orientation.is_horizontal();
        let mut el = div().flex().items_center();
        el = if vertical { el.flex_col() } else { el.flex_row() };

        if self.full_width {
            el = el.w_full();
        }

        if self.disable_radius_merge {
            el = el.gap(px(8.));
        } else {
            el = el
                .rounded(util::control_radius(cx))
                .overflow_hidden()
                .border(cx.layout().border_width)
                .border_color(cx.colors().border);
        }

        // Inherited buttons first, then any raw children.
        let variant = self.variant;
        let size = self.size;
        let disabled = self.is_disabled;
        let inherited = self
            .buttons
            .into_iter()
            .map(move |b| {
                b.variant(variant)
                    .size(size)
                    .is_disabled(disabled)
                    .into_any_element()
            });

        // Overlapping the 1px edges is what draws a single hairline between
        // members, so hiding the separator means not overlapping them.
        let overlap = !self.disable_radius_merge && !self.hide_separator;
        el.children(
            inherited
                .chain(self.children)
                .enumerate()
                .map(move |(i, child)| {
                    // Collapse the 1px seam between adjacent members.
                    div()
                        .when(overlap && i > 0 && !vertical, |d| d.ml(px(-1.)))
                        .when(overlap && i > 0 && vertical, |d| d.mt(px(-1.)))
                        .child(child)
                }),
        )
    }
}
