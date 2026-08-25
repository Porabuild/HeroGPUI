//! ButtonGroup — port of `@heroui/button-group` (v3).
//!
//! Joined buttons sharing one outer radius. `variant` and `size` are propagated
//! to the members; `outline` is not part of the group vocabulary, so building a
//! group from [`ButtonGroup::button`] keeps the variants valid by construction.

use gpui::{
    div, prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use herogpui_core::{Orientation, Size, Variant};

use crate::{button::Button, util};

/// HeroUI ButtonGroup.
#[derive(IntoElement)]
pub struct ButtonGroup {
    variant: Variant,
    size: Size,
    /// Whether a `ButtonGroup.Separator` is drawn before each member after the
    /// first. v3 composes it as a child of the member that follows it, so this
    /// is the slot flag rather than a documented prop. Defaults to none, as a
    /// group without `Separator` children draws no dividers.
    separators: bool,
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
            separators: false,
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

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    /// Adds a button that inherits the group's `variant` and `size`.
    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }

    /// `ButtonGroup.Separator` — the hairline before each member after the
    /// first.
    ///
    /// v3 composes it as a child of whichever member should show one, and
    /// `ButtonGroupRoot` synthesizes none of its own, so this port spells the
    /// composition as a flag. Defaults to false: a group only draws dividers
    /// when its example composes them.
    pub fn separators(mut self, v: bool) -> Self {
        self.separators = v;
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
        // `.button-group` is `inline-flex items-center justify-center gap-0`
        // and nothing else -- no border, no radius, no background. Each member
        // keeps its own fill, and the outer corners come from the first and last
        // of them. This used to draw a bordered, radius-clipped box with the
        // members overlapped by -1px, which is not a shape v3 has.
        let mut el = div().flex().items_center().justify_center();
        el = if vertical {
            el.flex_col()
        } else {
            el.flex_row()
        };
        if self.full_width {
            el = el.w_full();
        }

        let variant = self.variant;
        let size = self.size;
        let disabled = self.is_disabled;
        let inherited: Vec<Button> = self.buttons;
        let extra: Vec<AnyElement> = self.children;
        let total = inherited.len() + extra.len();

        // `.button-group__separator` is `bg-current opacity-15`, 1px by 50% of
        // the member, sitting one pixel before its leading edge.
        let separator_color = crate::button::button_foreground(variant, cx).alpha(0.15);
        let separator_radius = util::hairline_radius(cx);
        let separators = self.separators;

        let edge = move |i: usize| {
            if total <= 1 {
                crate::button::GroupEdge::Only
            } else if i == 0 {
                crate::button::GroupEdge::Start
            } else if i + 1 == total {
                crate::button::GroupEdge::End
            } else {
                crate::button::GroupEdge::Middle
            }
        };

        // The edge has to reach the `Button` before it is erased to an
        // `AnyElement`, so the members are built index-first.
        let mut wrapped: Vec<gpui::Div> = Vec::with_capacity(total);
        let styled = inherited.into_iter().enumerate().map(|(i, b)| {
            b.variant(variant)
                .size(size)
                .is_disabled(disabled)
                .group_edge(edge(i), vertical)
                .into_any_element()
        });
        for (i, child) in styled.chain(extra).enumerate() {
            let mut slot = div().relative().child(child);
            if self.full_width {
                slot = slot.flex_1();
            }
            if separators && i > 0 {
                slot = slot.child(
                    div()
                        .absolute()
                        .bg(separator_color)
                        .rounded(separator_radius)
                        .map(|s| {
                            if vertical {
                                s.left(gpui::relative(0.25))
                                    .top(px(-1.))
                                    .w(gpui::relative(0.5))
                                    .h(px(1.))
                            } else {
                                s.left(px(-1.))
                                    .top(gpui::relative(0.25))
                                    .w(px(1.))
                                    .h(gpui::relative(0.5))
                            }
                        }),
                );
            }
            wrapped.push(slot);
        }
        el.children(wrapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separators_are_explicit_composition() {
        assert!(
            !ButtonGroup::new().separators,
            "v3 groups without a Separator child must not synthesize dividers"
        );
        assert!(ButtonGroup::new().separators(true).separators);
    }
}
