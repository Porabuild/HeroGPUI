//! Button — port of `@heroui/button` (v3).
//!
//! v3 replaced v2's `variant` x `color` matrix with a single emphasis scale:
//! `primary | secondary | tertiary | outline | ghost | danger | danger-soft`.
//! There is no `color` or `radius` prop, `isLoading` became `isPending`, and
//! v2's `startContent`/`endContent` slots are gone: icons are ordered
//! [`ParentElement`] children around the label.

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, Stateful, Styled, Window,
};
use herogpui_core::{Size, Variant};
use herogpui_theme::ActiveTheme;

use crate::util;

/// A press handler. `Arc` rather than `Box` because it is bound twice: the
/// pointer's `on_click` and the keyboard's Enter/Space both run it.
type OnPress = std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Which edge of a [`crate::button_group::ButtonGroup`] a button sits on.
///
/// `.button-group .button` is `rounded-none`; the first member takes
/// `rounded-s-3xl` and the last `rounded-e-3xl`, so a joined group has one
/// outer radius rather than a rounded box per member. The press scale is also
/// off inside a group (`.button-group .button:active { transform: none }`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupEdge {
    /// First member: the leading corners are round.
    Start,
    /// Between two others: square on both ends.
    Middle,
    /// Last member: the trailing corners are round.
    End,
    /// The only member, so it keeps the full radius.
    Only,
}

/// HeroUI Button.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    /// v3's `children`-as-a-function: handed `{isHovered, isPressed, isFocused,
    /// isFocusVisible, isDisabled, isPending}` and drawn in place of the label.
    content: Option<std::sync::Arc<dyn Fn(util::InteractiveState) -> AnyElement + 'static>>,
    variant: Variant,
    variant_is_set: bool,
    size: Size,
    size_is_set: bool,
    full_width: bool,
    /// Set by [`Button::full_width`]. ButtonGroup context supplies width as a
    /// *default* (`button.tsx`: `finalFullWidth = fullWidth ??
    /// context.fullWidth`), so this flag is what keeps an explicit child
    /// `full_width(false)` from being overwritten by a full-width group.
    full_width_is_set: bool,
    is_icon_only: bool,
    /// Set by [`crate::button_group::ButtonGroup`]: which end of the group this
    /// button is, and whether the group stacks.
    group_edge: Option<(GroupEdge, bool)>,
    is_disabled: bool,
    is_disabled_is_set: bool,
    is_pending: bool,
    children: Vec<AnyElement>,
    on_press: Option<OnPress>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            content: None,
            variant: Variant::Primary,
            variant_is_set: false,
            size: Size::Md,
            size_is_set: false,
            full_width: false,
            full_width_is_set: false,
            is_icon_only: false,
            group_edge: None,
            is_disabled: false,
            is_disabled_is_set: false,
            is_pending: false,
            children: Vec::new(),
            on_press: None,
        }
    }

    /// v3's render function for a button's children, handed `isHovered`,
    /// `isPressed`, `isFocused`, `isFocusVisible`, `isDisabled` and `isPending`.
    ///
    /// The hover and the press are a frame behind the pointer: gpui reports both
    /// to a handler, so the render that draws them can only read what the last
    /// frame recorded. The button's own hover and press styling does not go
    /// through this -- it is applied by gpui in the same frame.
    pub fn content(
        mut self,
        render: impl Fn(util::InteractiveState) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self.variant_is_set = true;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self.size_is_set = true;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self.full_width_is_set = true;
        self
    }

    pub fn is_icon_only(mut self, v: bool) -> Self {
        self.is_icon_only = v;
        self
    }

    /// Joins this button to a group edge. Internal: a caller reaches it by
    /// putting the button in a [`crate::button_group::ButtonGroup`].
    pub(crate) fn group_edge(mut self, edge: GroupEdge, vertical: bool) -> Self {
        self.group_edge = Some((edge, vertical));
        self
    }

    /// Applies ButtonGroup context values only where the child did not set its
    /// own prop, matching React's direct-child context precedence.
    pub(crate) fn group_defaults(
        mut self,
        variant: Variant,
        size: Size,
        is_disabled: bool,
        full_width: bool,
    ) -> Self {
        if !self.variant_is_set {
            self.variant = variant;
        }
        if !self.size_is_set {
            self.size = size;
        }
        if !self.is_disabled_is_set {
            self.is_disabled = is_disabled;
        }
        if !self.full_width_is_set {
            self.full_width = full_width;
        }
        self
    }

    /// The member's resolved width after [`Self::group_defaults`]: an explicit
    /// child value when one was set, the group's `fullWidth` otherwise.
    pub(crate) fn is_full_width(&self) -> bool {
        self.full_width
    }

    /// The member's resolved variant after [`Self::group_defaults`]: an
    /// explicit child value when one was set, the group's otherwise.
    /// ButtonGroup reads it for the member's `bg-current` separator colour.
    pub(crate) fn resolved_variant(&self) -> Variant {
        self.variant
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self.is_disabled_is_set = true;
        self
    }

    /// `isPending` — blocks presses and hover while retaining the tab stop and focus ring.
    /// A `content` closure receives the pending state and owns any loading indicator.
    pub fn is_pending(mut self, v: bool) -> Self {
        self.is_pending = v;
        self
    }

    pub fn on_press(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press = Some(std::sync::Arc::new(handler));
        self
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Paints a button's fill, border, text and interaction states for `variant`.
///
/// Shared with `ButtonGroup`, which propagates the same variant to its members.
pub fn apply_button_variant(
    el: Stateful<Div>,
    variant: Variant,
    interactive: bool,
    cx: &App,
) -> Stateful<Div> {
    apply_variant(el, variant, interactive, true, cx)
}

/// The background pair `variant` eases between on hover, or `None` when the
/// variant has no background to ease.
///
/// Used by [`Button`] to run v3's `transition-colors` through
/// [`crate::anim::hover_fade`] instead of swapping the fill on one frame.
pub fn button_hover_colors(variant: Variant, cx: &App) -> Option<(gpui::Hsla, gpui::Hsla)> {
    let colors = cx.colors();
    match variant {
        Variant::Primary => Some((colors.accent.color, colors.accent.hover())),
        Variant::Secondary => Some((colors.default.color, colors.default.hover())),
        Variant::Tertiary => Some((colors.default.color, colors.default.hover())),
        Variant::Outline => Some((gpui::transparent_black(), colors.default.color.alpha(0.6))),
        Variant::Ghost => Some((gpui::transparent_black(), colors.default.color)),
        Variant::Danger => Some((colors.danger.color, colors.danger.hover())),
        Variant::DangerSoft => Some((colors.danger.soft(), colors.danger.soft_hover())),
    }
}

/// [`apply_button_variant`], with `hover_bg` off when the caller is going to
/// animate the background itself.
fn apply_variant(
    el: Stateful<Div>,
    variant: Variant,
    interactive: bool,
    hover_bg: bool,
    cx: &App,
) -> Stateful<Div> {
    let colors = cx.colors();
    let layout = cx.layout();

    match variant {
        Variant::Primary => {
            let base = colors.accent;
            let el = el.text_color(base.foreground);
            let el = if hover_bg { el.bg(base.color) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        // `secondary` is the neutral filled style: v3 maps the removed
        // `bg-secondary` token to `bg-default`.
        Variant::Secondary => {
            let base = colors.default;
            let el = el.text_color(colors.accent.soft_foreground(colors.foreground));
            let el = if hover_bg { el.bg(base.color) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Tertiary => {
            let base = colors.default;
            let fg = colors.foreground;
            let el = if hover_bg {
                el.bg(base.color).text_color(fg)
            } else {
                el.text_color(fg)
            };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Outline => {
            let base = colors.default;
            let el = el
                .border(layout.border_width)
                .border_color(colors.border)
                .text_color(base.foreground);
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.color.alpha(0.6))))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Ghost => {
            let base = colors.default;
            let el = el.text_color(base.foreground);
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.color)))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Danger => {
            let base = colors.danger;
            let el = el.text_color(base.foreground);
            let el = if hover_bg { el.bg(base.color) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::DangerSoft => {
            let base = colors.danger;
            let el = el.text_color(base.soft_foreground(colors.foreground));
            let el = if hover_bg { el.bg(base.soft()) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.soft_hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
    }
}

/// Button's own type and spacing ladder, from `button.css`.
///
/// Only three things move across the sizes. `.button` sets `px-4 gap-2 text-sm`
/// for every size; `.button--sm` narrows the padding to `px-3` and `.button--lg`
/// steps the type up to `text-base` — neither touches the gap, and `--sm` does
/// not touch the type. Reading a generic sm/md/lg ladder instead made the small
/// button's label a step too small and the large button's padding and gap a
/// step too wide.
fn button_metrics(size: Size) -> ButtonMetrics {
    let (text, line_height) = match size {
        // `text-sm` / `text-base`, with Tailwind's paired line heights.
        Size::Sm | Size::Md => (gpui::px(14.), gpui::px(20.)),
        Size::Lg => (gpui::px(16.), gpui::px(24.)),
    };
    ButtonMetrics {
        text,
        line_height,
        // `px-3` on `--sm`, `px-4` everywhere else.
        padding_x: match size {
            Size::Sm => gpui::px(12.),
            Size::Md | Size::Lg => gpui::px(16.),
        },
        // `gap-2`, never overridden.
        gap: gpui::px(8.),
    }
}

struct ButtonMetrics {
    text: gpui::Pixels,
    line_height: gpui::Pixels,
    padding_x: gpui::Pixels,
    gap: gpui::Pixels,
}

/// The text colour `variant` paints, for child svgs that cannot inherit
/// `text_color` from their parent.
pub fn button_foreground(variant: Variant, cx: &App) -> gpui::Hsla {
    let colors = cx.colors();
    match variant {
        Variant::Primary => colors.accent.foreground,
        Variant::Secondary => colors.accent.soft_foreground(colors.foreground),
        Variant::Tertiary => colors.foreground,
        Variant::Outline | Variant::Ghost => colors.default.foreground,
        Variant::Danger => colors.danger.foreground,
        Variant::DangerSoft => colors.danger.soft_foreground(colors.foreground),
    }
}

/// [`group_radius`] for any styled element — `ToggleButtonGroup` merges its
/// members' corners the same way `.button-group` does.
pub(crate) fn group_radius_any<T: Styled>(
    el: T,
    edge: Option<(GroupEdge, bool)>,
    radius: gpui::Pixels,
) -> T {
    let Some((edge, vertical)) = edge else {
        return el.rounded(radius);
    };
    match (edge, vertical) {
        (GroupEdge::Only, _) => el.rounded(radius),
        (GroupEdge::Start, false) => el.rounded_tl(radius).rounded_bl(radius),
        (GroupEdge::End, false) => el.rounded_tr(radius).rounded_br(radius),
        (GroupEdge::Start, true) => el.rounded_tl(radius).rounded_tr(radius),
        (GroupEdge::End, true) => el.rounded_bl(radius).rounded_br(radius),
        (GroupEdge::Middle, _) => el,
    }
}

/// The border sides an outline group member drops, in gpui's per-side order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct CollapsedSides {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

/// Which borders an outline member's group position collapses, read off the
/// pinned `button-group.css`: the horizontal sheet rows are
/// `:first-child { border-e-0 }`, `:last-child { border-s-0 }` and a middle
/// member (`:not(:first-child):not(:last-child)`) `border-x-0`; the vertical
/// sheet mirrors them into the block axis with `border-b-0`, `border-t-0` and
/// `border-y-0`. A lone member is `:first-child:last-child`, so both edge
/// rules apply at once and its whole stacking-axis border collapses. Pure so
/// every GroupEdge x orientation case can be table-tested against the pinned
/// stylesheet.
pub(crate) fn collapsed_border_sides(edge: GroupEdge, vertical: bool) -> CollapsedSides {
    match (edge, vertical) {
        (GroupEdge::Start, false) => CollapsedSides {
            right: true,
            ..Default::default()
        },
        (GroupEdge::End, false) => CollapsedSides {
            left: true,
            ..Default::default()
        },
        (GroupEdge::Middle | GroupEdge::Only, false) => CollapsedSides {
            left: true,
            right: true,
            ..Default::default()
        },
        (GroupEdge::Start, true) => CollapsedSides {
            bottom: true,
            ..Default::default()
        },
        (GroupEdge::End, true) => CollapsedSides {
            top: true,
            ..Default::default()
        },
        (GroupEdge::Middle | GroupEdge::Only, true) => CollapsedSides {
            top: true,
            bottom: true,
            ..Default::default()
        },
    }
}

/// Zeroes exactly the collapsed sides of an already-bordered element.
fn apply_collapsed_sides<T: Styled>(el: T, sides: CollapsedSides) -> T {
    let el = if sides.left { el.border_l_0() } else { el };
    let el = if sides.right { el.border_r_0() } else { el };
    let el = if sides.top { el.border_t_0() } else { el };
    if sides.bottom {
        el.border_b_0()
    } else {
        el
    }
}

/// Applies `radius` to only the corners a group edge leaves round.
fn group_radius(
    el: Stateful<Div>,
    edge: Option<(GroupEdge, bool)>,
    radius: gpui::Pixels,
) -> Stateful<Div> {
    let Some((edge, vertical)) = edge else {
        return el.rounded(radius);
    };
    match (edge, vertical) {
        (GroupEdge::Only, _) => el.rounded(radius),
        // Horizontal: the start edge rounds its left corners, the end edge its
        // right ones. Vertical: top and bottom.
        (GroupEdge::Start, false) => el.rounded_tl(radius).rounded_bl(radius),
        (GroupEdge::End, false) => el.rounded_tr(radius).rounded_br(radius),
        (GroupEdge::Start, true) => el.rounded_tl(radius).rounded_tr(radius),
        (GroupEdge::End, true) => el.rounded_bl(radius).rounded_br(radius),
        (GroupEdge::Middle, _) => el,
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // The handle that says whether this button holds the focus.
        // `use_keyed_state` takes `cx` mutably, so it precedes the tokens.
        let focus_handle = util::tab_stop_handle(
            ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        // The hover and press this button will report to a `content` closure.
        // Only tracked when one is set: the handlers cost a frame of state.
        let interaction = self.content.as_ref().map(|_| {
            util::interaction(
                ElementId::Name(format!("{:?}-interaction", self.id).into()),
                window,
                cx,
            )
        });
        let layout = cx.layout();
        // Copied out: `hover_fade` below takes `&mut App`, and holding the
        // `layout` borrow across it would be a second borrow of `cx`.
        let disabled_opacity = layout.disabled_opacity;
        let focusable = !self.is_disabled;
        let interactive = focusable && !self.is_pending;
        if !interactive {
            if let Some(slot) = &interaction {
                if *slot.read(cx) != (false, false) {
                    slot.update(cx, |state, _| *state = (false, false));
                }
            }
        }
        // v3's `transition-colors`: the fill eases rather than switching on the
        // frame the pointer arrives. The variant then leaves the background
        // alone so the two do not fight over it.
        let fade = interactive
            .then(|| button_hover_colors(self.variant, cx))
            .flatten();

        let metrics = button_metrics(self.size);
        let mut el = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            // `button.css` declares no `overflow`: a label too long for the
            // button spills, it is not cut. Clipping it here also gave the row
            // an automatic minimum size of zero, which let the label collapse
            // instead of overflowing.
            .whitespace_nowrap()
            .font_weight(gpui::FontWeight::MEDIUM)
            .map(|e| group_radius(e, self.group_edge, util::control_radius(cx)))
            .text_size(metrics.text)
            .line_height(metrics.line_height)
            .h(self.size.control_height());

        el = if self.is_icon_only {
            el.w(self.size.icon_control_size())
        } else {
            el.px(metrics.padding_x).gap(metrics.gap)
        };

        if self.full_width {
            el = el.w_full();
        }

        el = apply_variant(el, self.variant, interactive, fade.is_none(), cx);

        // `button-group.css` collapses the borders an outline member shows
        // toward its neighbours so a seam is the one composed separator
        // hairline rather than two borders. `collapsed_border_sides` holds
        // the per-case mapping; outside a group the full border stays.
        if self.variant == Variant::Outline {
            if let Some((edge, vertical)) = self.group_edge {
                el = apply_collapsed_sides(el, collapsed_border_sides(edge, vertical));
            }
        }

        // The fade's animated layer is glued under everything that follows: the
        // colour transition lives on an inset fill *inside* the button, so the
        // button's own element id — and with it the hover listener latch — never
        // moves when the fill's animation restarts (see `anim::hover_fade`).
        // The interaction slot is handed over when a `content` closure is set:
        // `track_interaction` then owns `on_hover`, and the fade reads the hover
        // bit the slot records instead of binding a second listener.
        if let Some(colors) = fade {
            let edge = self.group_edge;
            let radius = util::control_radius(cx);
            el = crate::anim::hover_fade(
                el,
                ElementId::Name(format!("{:?}-fade", self.id).into()),
                colors,
                interaction.as_ref(),
                move |fill| group_radius_any(fill, edge, radius),
                window,
                cx,
            );
        }

        // `.button:focus-visible` is `status-focused`: a 2px ring, offset from
        // the button by another in the background colour. A disabled button is
        // not a tab stop, which is what `pointer-events-none` amounts to here.
        if focusable {
            el = util::ring_if_focused(
                el.track_focus(&focus_handle),
                &focus_handle,
                true,
                Vec::new(),
                window,
                cx,
            );
        }

        if self.is_disabled || self.is_pending {
            el = el.opacity(disabled_opacity);
        }

        if let Some(render) = self.content.clone() {
            let (is_hovered, is_pressed) = if interactive {
                interaction
                    .as_ref()
                    .map(|slot| *slot.read(cx))
                    .unwrap_or_default()
            } else {
                (false, false)
            };
            let focused = focusable && focus_handle.is_focused(window);
            el = el.child(render(util::InteractiveState {
                is_hovered,
                is_pressed,
                is_focused: focused,
                is_focus_visible: focused && util::focus_visible(cx),
                is_selected: false,
                is_disabled: self.is_disabled,
                is_pending: self.is_pending,
                is_indeterminate: false,
            }));
        } else if let Some(label) = self.label {
            el = el.child(label.to_string());
        }
        if interactive {
            if let Some(slot) = &interaction {
                el = util::track_interaction(el, slot);
            }
        }
        el = el.children(self.children);

        // v3's `[data-pressed]` scale. Applied last so the press geometry sits
        // on top of whatever the variant did to padding.
        if interactive && self.group_edge.is_none() {
            let press_scale = match self.size {
                Size::Sm => crate::anim::PRESSED_SCALE_SUBTLE,
                Size::Md => crate::anim::PRESSED_SCALE,
                Size::Lg => crate::anim::PRESSED_SCALE_FIRM,
            };
            el = crate::anim::pressed(
                el,
                crate::anim::PressBox {
                    height: self.size.control_height(),
                    padding_x: (!self.is_icon_only).then_some(metrics.padding_x),
                    width: self.is_icon_only.then(|| self.size.icon_control_size()),
                    // v3's `.button` is `w-fit` with no minimum, so a press has
                    // no floor to scale.
                    min_width: None,
                    text_size: metrics.text,
                    line_height: metrics.line_height,
                    gap: metrics.gap,
                    radius: util::control_radius(cx),
                    shrink_x: !self.full_width,
                    scale: press_scale,
                },
                cx,
            );
        }

        if let Some(on_press) = self.on_press {
            if interactive {
                // gpui fires a *focused* element's click listeners on Enter and
                // Space with `ClickEvent::Keyboard`, which is React Aria's press
                // exactly -- so this one binding answers the pointer and the
                // keyboard, and the focus handle above is what switched the
                // second half on.
                el = el.on_click(move |ev: &ClickEvent, window, cx| on_press(ev, window, cx));
            }
        }

        // `fade` was consumed where the fill was added; the button is the
        // plain styled div either way.
        el.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_defaults_preserve_explicit_child_props() {
        let button = Button::new("override")
            .variant(Variant::Outline)
            .size(Size::Lg)
            .is_disabled(false)
            .full_width(false)
            .group_defaults(Variant::Secondary, Size::Sm, true, true);

        assert_eq!(button.variant, Variant::Outline);
        assert_eq!(button.size, Size::Lg);
        assert!(!button.is_disabled);
        assert!(!button.is_full_width());
    }

    #[test]
    fn group_defaults_fill_unset_child_props() {
        let button =
            Button::new("inherited").group_defaults(Variant::Secondary, Size::Sm, true, true);

        assert_eq!(button.variant, Variant::Secondary);
        assert_eq!(button.size, Size::Sm);
        assert!(button.is_disabled);
        assert!(button.is_full_width());
    }

    /// `button-group.css` outline collapse, one row per GroupEdge x
    /// orientation, each naming the pinned selector that demands it.
    #[test]
    fn outline_collapse_table_matches_pinned_css() {
        let cases = [
            (
                GroupEdge::Start,
                false,
                CollapsedSides { right: true, ..Default::default() },
                ".button-group--horizontal .button--outline:first-child { border-e-0 }",
            ),
            (
                GroupEdge::End,
                false,
                CollapsedSides { left: true, ..Default::default() },
                ".button-group--horizontal .button--outline:last-child { border-s-0 }",
            ),
            (
                GroupEdge::Middle,
                false,
                CollapsedSides { left: true, right: true, ..Default::default() },
                ".button-group--horizontal .button--outline:not(:first-child):not(:last-child) { border-x-0 }",
            ),
            (
                GroupEdge::Start,
                true,
                CollapsedSides { bottom: true, ..Default::default() },
                ".button-group--vertical .button--outline:first-child { border-b-0 }",
            ),
            (
                GroupEdge::End,
                true,
                CollapsedSides { top: true, ..Default::default() },
                ".button-group--vertical .button--outline:last-child { border-t-0 }",
            ),
            (
                GroupEdge::Middle,
                true,
                CollapsedSides { top: true, bottom: true, ..Default::default() },
                ".button-group--vertical .button--outline:not(:first-child):not(:last-child) { border-y-0 }",
            ),
        ];
        for (edge, vertical, expected, selector) in cases {
            assert_eq!(
                collapsed_border_sides(edge, vertical),
                expected,
                "`{selector}` must collapse exactly these borders"
            );
        }

        // A lone member is `:first-child:last-child`, so both edge rules
        // apply at once and its whole stacking-axis border collapses.
        assert_eq!(
            collapsed_border_sides(GroupEdge::Only, false),
            CollapsedSides {
                left: true,
                right: true,
                ..Default::default()
            },
            "a lone horizontal outline member matches :first-child:last-child, \
             so border-e-0 and border-s-0 both apply"
        );
        assert_eq!(
            collapsed_border_sides(GroupEdge::Only, true),
            CollapsedSides {
                top: true,
                bottom: true,
                ..Default::default()
            },
            "a lone vertical outline member matches :first-child:last-child, \
             so border-b-0 and border-t-0 both apply"
        );
    }

    /// `button.tsx`: `finalFullWidth = fullWidth ?? context.fullWidth` — the
    /// child value wins in both directions, and an unset child inherits the
    /// context in both directions.
    #[test]
    fn group_defaults_full_width_precedence() {
        let inherit_false =
            Button::new("inherit-false").group_defaults(Variant::Primary, Size::Md, false, false);
        let inherit_true =
            Button::new("inherit-true").group_defaults(Variant::Primary, Size::Md, false, true);
        let override_false = Button::new("override-false")
            .full_width(false)
            .group_defaults(Variant::Primary, Size::Md, false, true);
        let override_true = Button::new("override-true")
            .full_width(true)
            .group_defaults(Variant::Primary, Size::Md, false, false);

        assert!(!inherit_false.is_full_width());
        assert!(inherit_true.is_full_width());
        assert!(
            !override_false.is_full_width(),
            "an explicit child fullWidth=false must survive a full-width group context"
        );
        assert!(
            override_true.is_full_width(),
            "an explicit child fullWidth=true must survive a non-full group context"
        );
    }
}
